import hashlib, json, os, pathlib, re, resource, subprocess, sys
root = pathlib.Path('/tmp/revault-form-lifecycle-2026-10-09-final-qualified')
binary = root / 'frozen' / 'revault_lockbox_api-tests'
outdir = root / 'aggregate'
outdir.mkdir(parents=True, exist_ok=True)
start_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
records = []
base = ['typed_form_aggregate_guarded_snapshot_probe', '--ignored', '--test-threads=1', '--nocapture']
for mode in range(16):
    image = outdir / f'mode-{mode:02}.lbox'
    log = outdir / f'mode-{mode:02}.log'
    env = os.environ.copy()
    env['REVAULT_FORM_MODE'] = str(mode)
    env['REVAULT_FORM_IMAGE'] = str(image)
    cmd = [str(binary), *base]
    with log.open('wb') as stream:
        proc = subprocess.run(cmd, cwd='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust', env=env, stdout=stream, stderr=subprocess.STDOUT)
    content = log.read_text(errors='replace')
    matches = re.findall(r'FORM_AGGREGATE (\{[^\n]*\})', content)
    marker = json.loads(matches[-1]) if matches else None
    image_digest = hashlib.sha256(image.read_bytes()).hexdigest() if image.exists() and image.is_file() else None
    rec = {'mode': mode, 'command': cmd, 'image': str(image), 'log': str(log), 'exit_code': proc.returncode, 'marker': marker, 'archive_sha256': image_digest, 'archive_bytes': image.stat().st_size if image.exists() else None, 'locked_memory_limit_bytes': resource.getrlimit(resource.RLIMIT_MEMLOCK)[0]}
    records.append(rec)
    print(json.dumps({'mode': mode, 'exit_code': proc.returncode, 'marker_seen': marker is not None, 'archive_bytes': rec['archive_bytes']}), flush=True)
end_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
summary = {'binary': str(binary), 'binary_sha256_start': start_hash, 'binary_sha256_end': end_hash, 'binary_stable': start_hash == end_hash, 'locked_memory_limit_bytes': resource.getrlimit(resource.RLIMIT_MEMLOCK)[0], 'cases': records}
(root / 'outcomes.json').write_text(json.dumps(summary, indent=2) + '\n')
if not summary['binary_stable']:
    print('FATAL: frozen binary hash changed during run', file=sys.stderr)
    sys.exit(2)
if any(r['exit_code'] != 0 or r['marker'] is None or r['archive_sha256'] is None for r in records):
    sys.exit(1)
