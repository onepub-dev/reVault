#!/usr/bin/env python3
"""Reciprocal frozen-version readers, outside timed sampling."""
import json
import os
from pathlib import Path
import subprocess
import sys

from run_fixed import TEST, marker, sha

root = Path(sys.argv[1]).resolve()
meta = json.loads((root / "metadata.json").read_text())
finished = json.loads((root / "finished.json").read_text())
assert finished["identities_stable"]
rows = [json.loads(line) for line in (root / "attempts.jsonl").read_text().splitlines()]
last = {(row["mode"], row["variant"]): row for row in rows if row["pair"] == 29}
out = root / "cross-readers"
out.mkdir(exist_ok=False)
results = []
for mode in range(16):
    for producer, reader in (("control", "candidate"), ("candidate", "control")):
        row = last[mode, producer]
        result = {"mode": mode, "producer": producer, "reader": reader, "success": False}
        if row["success"]:
            binary = Path(meta["binaries"][reader]["path"])
            assert sha(binary) == meta["binaries"][reader]["sha256"]
            archive = root / row["directory"] / "image.archive"
            assert sha(archive) == row["archive_sha256"]
            env = os.environ.copy()
            env.update(REVAULT_VARIABLE_MODE=str(mode), REVAULT_VARIABLE_IMAGE=str(archive), REVAULT_VARIABLE_VERIFY_ONLY="1")
            log = out / f"mode-{mode:02d}-{reader}-reads-{producer}.log"
            with log.open("wb") as stream:
                child = subprocess.run([str(binary), TEST, "--exact", "--ignored", "--nocapture", "--test-threads=1"], env=env, stdout=stream, stderr=subprocess.STDOUT)
            result.update(exit_code=child.returncode, verified=marker(log, "VARIABLE_RESOURCE_VERIFIED "))
            result["success"] = (child.returncode == 0 and result["verified"] == row["verified"] and sha(archive) == row["archive_sha256"])
        else:
            result["unavailable"] = "last declared producer failed; no substitute sample"
        results.append(result)
        (out / "results.json").write_text(json.dumps(results, indent=2) + "\n")
print(f"{sum(result['success'] for result in results)}/{len(results)} reciprocal frozen-version readers passed")
if not all(result["success"] for result in results):
    raise SystemExit(1)
