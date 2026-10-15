#!/usr/bin/env python3
"""Fixed, sequential fresh-process resource comparison; no retries."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time

TEST = "file_format::candidate_files::tests::tree_tests::variables::resource::typed_variable_resource_probe"


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def marker(log, prefix):
    found = []
    for line in log.read_text().splitlines():
        if prefix in line:
            found.append(json.loads(line.split(prefix, 1)[1]))
    return found


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--control", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    binaries = {name: getattr(args, name).resolve() for name in ("control", "candidate")}
    uname = os.uname()
    metadata = {
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "runner_sha256": sha(__file__),
        "analysis_sha256": sha(Path(__file__).with_name("analyze_fixed.py")),
        "cross_reader_sha256": sha(Path(__file__).with_name("cross_verify.py")),
        "binaries": {name: {"path": str(path), "sha256": sha(path)} for name, path in binaries.items()},
        "host": dict(zip(("sysname", "nodename", "release", "version", "machine"), uname)),
        "memlock_bytes": resource.getrlimit(resource.RLIMIT_MEMLOCK),
        "inherited_cpu_affinity": sorted(os.sched_getaffinity(0)),
        "python": sys.version,
        "protocol": {"modes": 16, "warmups_per_variant_mode": 1, "pairs_per_mode": 30,
                     "pair_order": "control first for even pair; candidate first for odd pair", "retries": 0},
        "corpus_sha256": {chr(byte): hashlib.sha256(bytes([byte]) * 1048576).hexdigest() for byte in (97, 98)},
    }
    (args.output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    records_path = args.output / "attempts.jsonl"
    with records_path.open("x", buffering=1) as records:
        for mode in range(16):
            for pair in range(-1, 30):
                order = ("control", "candidate") if pair < 0 or pair % 2 == 0 else ("candidate", "control")
                for variant in order:
                    name = f"mode-{mode:02d}-{'warmup' if pair < 0 else f'pair-{pair:02d}'}-{variant}"
                    folder = args.output / name
                    folder.mkdir()
                    archive = folder / "image.archive"
                    env = os.environ.copy()
                    env.pop("REVAULT_VARIABLE_VERIFY_ONLY", None)
                    env["REVAULT_VARIABLE_MODE"] = str(mode)
                    env["REVAULT_VARIABLE_IMAGE"] = str(archive.resolve())
                    command = [str(binaries[variant]), TEST, "--exact", "--ignored", "--nocapture", "--test-threads=1"]
                    log = folder / "producer.log"
                    with log.open("wb") as output:
                        result = subprocess.run(command, env=env, stdout=output, stderr=subprocess.STDOUT)
                    row = {"mode": mode, "pair": pair, "variant": variant, "warmup": pair < 0,
                           "producer_exit": result.returncode, "directory": name,
                           "phases": marker(log, "VARIABLE_RESOURCE_PHASE "),
                           "completed": marker(log, "VARIABLE_RESOURCE "), "success": False}
                    if result.returncode == 0 and len(row["completed"]) == 1:
                        verify_log = folder / "verify.log"
                        env["REVAULT_VARIABLE_VERIFY_ONLY"] = "1"
                        with verify_log.open("wb") as output:
                            verified = subprocess.run(command, env=env, stdout=output, stderr=subprocess.STDOUT)
                        row["verifier_exit"] = verified.returncode
                        row["verified"] = marker(verify_log, "VARIABLE_RESOURCE_VERIFIED ")
                        if verified.returncode == 0 and len(row["verified"]) == 1:
                            row["archive_sha256"] = sha(archive)
                            expected = row["completed"][0]["replaced"]
                            expected_phases = ["create", "no_change", "replace", "open_and_read"]
                            complete_phases = row["completed"][0]["phases"]
                            row["success"] = ([p["phase"] for p in row["phases"]] == expected_phases
                                              and row["phases"] == complete_phases
                                              and row["verified"][0] == expected
                                              and row["archive_sha256"] == expected["archive_sha256"])
                    records.write(json.dumps(row, sort_keys=True) + "\n")
                    records.flush()
                    os.fsync(records.fileno())
                    if row["success"] and pair != 29:
                        archive.unlink()
                        archive.with_suffix(".public").unlink()
                    print(f"{name}: {'PASS' if row['success'] else 'FAIL'}", flush=True)
    final_hashes = {name: sha(path) for name, path in binaries.items()}
    stable = all(final_hashes[name] == metadata["binaries"][name]["sha256"] for name in binaries) and sha(__file__) == metadata["runner_sha256"]
    (args.output / "finished.json").write_text(json.dumps({"finished_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "final_binary_sha256": final_hashes, "final_runner_sha256": sha(__file__), "identities_stable": stable}) + "\n")
    if not stable:
        raise SystemExit("Binary/runner identity changed: fixed-batch stability claim refused")


if __name__ == "__main__":
    main()
