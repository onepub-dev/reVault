import hashlib
import ctypes
import ctypes.util
import json
import math
import os
from pathlib import Path
import random
import statistics
import subprocess

root = Path(__file__).resolve().parent
output = root / "measurements"
output.mkdir(exist_ok=False)
affinity = sorted(os.sched_getaffinity(0))
cpu = 2 if 2 in affinity else affinity[0]
os.sched_setaffinity(0, {cpu})
cases = [("small-owned", "small", "owned", 256),
         ("medium-owned", "medium", "owned", 64),
         ("large-owned", "large", "owned", 4),
         ("medium-static", "medium", "static", 64)]

def inventory():
    paths = [root / name for name in ("baseline", "candidate", "src/main.rs", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "run.py")]
    paths += sorted((root / "corpus").iterdir())
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

before = inventory()
library_name = ctypes.util.find_library("zstd")
assert library_name, "independent libzstd decoder is required before timing"
library = ctypes.CDLL(library_name)
library.ZSTD_decompress.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
library.ZSTD_decompress.restype = ctypes.c_size_t
library.ZSTD_isError.argtypes = [ctypes.c_size_t]
library.ZSTD_isError.restype = ctypes.c_uint
library.ZSTD_versionString.restype = ctypes.c_char_p
for encoded_path in sorted((root / "corpus").glob("*.zst")):
    encoded = encoded_path.read_bytes()
    raw = encoded_path.with_suffix(".raw").read_bytes()
    destination = ctypes.create_string_buffer(len(raw))
    length = library.ZSTD_decompress(destination, len(raw), encoded, len(encoded))
    assert not library.ZSTD_isError(length) and length == len(raw)
    assert destination.raw == raw, "persisted corpus failed independent decode"
(output / "independent-corpus-verification.json").write_text(json.dumps({"library": library_name, "version": library.ZSTD_versionString().decode(), "verified": True, "cases": 3}) + "\n")
(output / "inventory-before.json").write_text(json.dumps(before, indent=2) + "\n")
(output / "scope.json").write_text(json.dumps({
    "cpu": cpu, "affinity_before": affinity, "pairs": 30, "warmup_pairs": 3,
    "cases": cases, "bootstrap_seed": 3101008, "bootstrap_samples": 10000,
    "scope": "fresh decoder lifecycle including owned release; caller output allocation/erasure and final caller-owned scratch erasure excluded; process RSS is whole-lifetime high water, not incremental decoder memory",
    "timing": "sum of per-decode lifetimes; output byte comparison outside each timer; encoded/raw inputs loaded before timing; each sample uses a fresh process",
    "host": dict(zip(("sysname", "nodename", "release", "version", "machine"), os.uname())), "load_before": os.getloadavg(),
}, indent=2) + "\n")

summary = []
for label, corpus, mode, repetitions in cases:
    samples = []
    for index in range(-3, 30):
        order = ("baseline", "candidate") if index % 2 == 0 else ("candidate", "baseline")
        pair = {"index": index, "warmup": index < 0, "order": order}
        for executable in order:
            record = json.loads(subprocess.check_output([str(root / executable), "sample", str(root / "corpus"), corpus, mode, str(repetitions)], text=True))
            pair[executable] = record
        samples.append(pair)
        with (output / (label + ".jsonl")).open("a") as target:
            target.write(json.dumps(pair) + "\n")
    measured = [sample for sample in samples if not sample["warmup"]]
    logs = [math.log(sample["candidate"]["seconds"] / sample["baseline"]["seconds"]) for sample in measured]
    rng = random.Random(3101008)
    bootstraps = sorted(math.exp(statistics.mean(rng.choices(logs, k=len(logs)))) for _ in range(10000))
    summary.append({"case": label, "ratio": math.exp(statistics.mean(logs)), "lower95": bootstraps[250], "upper95": bootstraps[9749],
                    "baseline_median_seconds": statistics.median(sample["baseline"]["seconds"] / repetitions for sample in measured),
                    "candidate_median_seconds": statistics.median(sample["candidate"]["seconds"] / repetitions for sample in measured),
                    "baseline_median_peak_rss_kib": statistics.median(sample["baseline"]["rss_kib"] for sample in measured),
                    "candidate_median_peak_rss_kib": statistics.median(sample["candidate"]["rss_kib"] for sample in measured),
                    "window_bytes": measured[0]["baseline"]["window_bytes"]})
after = inventory()
assert before == after, "source, executable or corpus changed during measurements"
(output / "inventory-after.json").write_text(json.dumps(after, indent=2) + "\n")
(output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
(output / "load-after.json").write_text(json.dumps(os.getloadavg()) + "\n")
print(json.dumps(summary, indent=2))
