#!/usr/bin/env python3
"""Deterministic paired bootstrap; endpoint memory is not peak/incremental RSS."""
import json
import math
from pathlib import Path
import random
import statistics
import sys

ROOT = Path(sys.argv[1])
ROWS = [json.loads(line) for line in (ROOT / "attempts.jsonl").read_text().splitlines()]
FINISHED = json.loads((ROOT / "finished.json").read_text())
if not FINISHED.get("identities_stable"):
    raise SystemExit("Cannot analyze an unstable batch")
expected = {(mode, pair, variant) for mode in range(16) for pair in range(-1, 30) for variant in ("control", "candidate")}
actual = [(row["mode"], row["pair"], row["variant"]) for row in ROWS]
if len(actual) != 992 or set(actual) != expected or any(row["warmup"] != (row["pair"] == -1) for row in ROWS):
    raise SystemExit("Expected exactly 992 distinct declared attempts including 32 warmups")
PHASES = ("create", "no_change", "replace", "open_and_read")
SEED = 31020261009
BOOTSTRAPS = 10000


def summary(values):
    return {"min": min(values), "median": statistics.median(values), "max": max(values), "mean": statistics.mean(values)}


def ratio(values, seed):
    logs = [math.log(value) for value in values]
    rng = random.Random(seed)
    means = sorted(math.exp(statistics.mean(rng.choices(logs, k=len(logs)))) for _ in range(BOOTSTRAPS))
    return {"geomean": math.exp(statistics.mean(logs)), "ci95": [means[249], means[9749]]}


out = {"seed": SEED, "bootstrap_resamples": BOOTSTRAPS, "ratio_direction": "candidate / control",
       "attempt_count": len(ROWS), "failures": [row for row in ROWS if not row["success"]], "cases": []}
for mode in range(16):
    selected = {(row["pair"], row["variant"]): row for row in ROWS if row["mode"] == mode and not row["warmup"]}
    if len(selected) != 60:
        raise SystemExit(f"incomplete fixed batch for mode {mode}: {len(selected)}/60")
    valid = [pair for pair in range(30) if all(selected[pair, variant]["success"] for variant in ("control", "candidate"))]
    for index, phase in enumerate(PHASES):
        case = {"mode": mode, "phase": phase, "complete_pairs": len(valid), "excluded_pairs": [pair for pair in range(30) if pair not in valid]}
        data = {variant: [{p["phase"]: p for p in selected[pair, variant]["completed"][0]["phases"]}[phase] for pair in valid] for variant in ("control", "candidate")}
        if valid:
            for metric in ("cpu_seconds", "wall_seconds"):
                values = [b[metric] / a[metric] for a, b in zip(data["control"], data["candidate"])]
                case[metric] = ratio(values, SEED + mode * 100 + index * 2 + (metric == "wall_seconds"))
                case[metric]["control"] = summary([p[metric] for p in data["control"]])
                case[metric]["candidate"] = summary([p[metric] for p in data["candidate"]])
            for metric in ("vm_lck_kib", "vm_rss_kib", "vm_hwm_kib"):
                case[metric] = {variant: {endpoint: summary([p[endpoint][metric] for p in data[variant]]) for endpoint in ("before", "after")} for variant in ("control", "candidate")}
                case[metric]["paired_after_difference"] = summary([b["after"][metric] - a["after"][metric] for a, b in zip(data["control"], data["candidate"])])
        out["cases"].append(case)
(ROOT / "summary.json").write_text(json.dumps(out, indent=2) + "\n")
lines = ["Mode bits: 1 = encryption, 2 = owner signing, 4 = compression, 8 = padding. Unset bits disable that option.", "",
         "| Mode | Phase | Pairs | CPU ratio (95% CI) | Wall ratio (95% CI) | Median locked KiB control → candidate |", "| --- | --- | --- | --- | --- | --- |"]
for case in out["cases"]:
    def interval(key):
        value = case[key]
        return f"{value['geomean']:.3f} ({value['ci95'][0]:.3f}–{value['ci95'][1]:.3f})"
    if case["complete_pairs"]:
        locked = case["vm_lck_kib"]
        lines.append(f"| {case['mode']} | {case['phase']} | {case['complete_pairs']} | {interval('cpu_seconds')} | {interval('wall_seconds')} | {locked['control']['after']['median']:g} → {locked['candidate']['after']['median']:g} |")
    else:
        lines.append(f"| {case['mode']} | {case['phase']} | 0 | — | — | — |")
(ROOT / "summary.md").write_text("\n".join(lines) + "\n")
