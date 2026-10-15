import hashlib, json, os, pathlib, re, resource, subprocess, sys
root = pathlib.Path('/tmp/revault-form-fields-2026-10-09-final-qualified')
binary = root / 'frozen' / 'revault_lockbox_api-tests'
runner = pathlib.Path(__file__)
outdir = root / 'aggregate'
outdir.mkdir(parents=True, exist_ok=True)
if any(outdir.iterdir()) or (root / "outcomes.json").exists():
    raise RuntimeError("refuse to overwrite/retry any existing aggregate attempts")
start_binary = hashlib.sha256(binary.read_bytes()).hexdigest()
start_runner = hashlib.sha256(runner.read_bytes()).hexdigest()
affinity = sorted(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else None
memlock = resource.getrlimit(resource.RLIMIT_MEMLOCK)
records = []
for mode in range(16):
    image = outdir / f'mode-{mode:02}.lbox'
    log = outdir / f'mode-{mode:02}.log'
    env = os.environ.copy()
    env['REVAULT_FORM_MODE'] = str(mode)
    env['REVAULT_FORM_IMAGE'] = str(image)
    cmd = [str(binary), 'typed_form_field_upgrade_aggregate_probe', '--ignored', '--test-threads=1', '--nocapture']
    with (root / 'attempts.jsonl').open('a') as journal:
        journal.write(json.dumps({'mode': mode, 'status': 'started'}) + '\n')
        journal.flush()
        os.fsync(journal.fileno())
    with log.open('xb') as stream:
        proc = subprocess.run(cmd, cwd='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust', env=env, stdout=stream, stderr=subprocess.STDOUT)
    output = log.read_text(errors='replace')
    markers = re.findall(r'FORM_FIELD_AGGREGATE (\{[^\n]*\})', output)
    marker_data = [json.loads(m) for m in markers]
    result_ok = bool(re.search(r'test result: ok\. 1 passed; 0 failed;', output))
    archive_hash = hashlib.sha256(image.read_bytes()).hexdigest() if image.is_file() else None
    rec = {
        'mode': mode, 'command': cmd, 'image': str(image), 'log': str(log),
        'exit_code': proc.returncode, 'test_result_ok': result_ok,
        'marker_count': len(marker_data), 'marker_mode_ok': len(marker_data) == 1 and marker_data[0].get('mode') == mode, 'marker': marker_data[0] if len(marker_data) == 1 else marker_data,
        'archive_sha256': archive_hash, 'archive_bytes': image.stat().st_size if image.is_file() else None,
    }
    records.append(rec)
    with (root / 'attempts.jsonl').open('a') as journal:
        journal.write(json.dumps({'mode': mode, 'status': 'completed', 'exit_code': proc.returncode}) + '\n')
        journal.flush()
        os.fsync(journal.fileno())
    print(json.dumps({'mode': mode, 'exit_code': proc.returncode, 'test_result_ok': result_ok, 'marker_count': len(marker_data), 'archive_bytes': rec['archive_bytes']}), flush=True)
end_binary = hashlib.sha256(binary.read_bytes()).hexdigest()
end_runner = hashlib.sha256(runner.read_bytes()).hexdigest()
summary = {
    'binary': str(binary), 'binary_sha256_start': start_binary, 'binary_sha256_end': end_binary,
    'binary_stable': start_binary == end_binary, 'runner': str(runner),
    'runner_sha256_start': start_runner, 'runner_sha256_end': end_runner,
    'runner_stable': start_runner == end_runner, 'cpu_affinity': affinity,
    'rlimit_memlock': {'soft': memlock[0], 'hard': memlock[1]}, 'cases': records,
}
(root / 'outcomes.json').write_text(json.dumps(summary, indent=2) + '\n')
if not summary['binary_stable'] or not summary['runner_stable']:
    print('FATAL: frozen binary or runner hash changed during run', file=sys.stderr)
    sys.exit(2)
if any(r['exit_code'] != 0 or not r['test_result_ok'] or r['marker_count'] != 1 or not r['marker_mode_ok'] or r['archive_sha256'] is None for r in records):
    sys.exit(1)
