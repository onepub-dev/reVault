# Bounded vector staging for typed file fragments

The typed reader now stages fragment descriptors in a bounded vector and joins
them through a file-ID index. It avoids building and removing a tree-map entry
for every fragment. The authenticated index supplies ordered records; each file
still requires consecutive ordinals from zero and exactly its declared count.
Missing, duplicate, extra, orphan and misbound records refuse before exposing a
catalogue. File identities, logical spans, pack addresses, physical coverage and
complete tree/ownership validation remain checked. Admission caps are unchanged.

## Fixed paired diagnostic

Baseline `d6ae8e90`, candidate plus the [exact patch](raw/source.patch), Rust 1.88.0,
CPU 2, four existing file-backed fixtures. Three warmup pairs and 30 measured
pairs each; alternating before/after order; 100 warm opens per fresh process.
The paired unit is mean elapsed time per open in that process. Observer overhead
is included; full-content verification before/after and object drop are excluded.
Fixed-seed 10,000-resample paired log-ratio bootstrap, no adaptive reruns.

| Fixture | Candidate/control [95% CI] | Mean control/candidate per-open time (ms) |
| --- | --- | --- |
| 512 × 4 KiB compressible | 0.89974 [0.87283, 0.93284] | 0.914173 / 0.822633 |
| 8 MiB random raw | 0.93392 [0.93014, 0.93780] | 0.299599 / 0.279808 |
| 8 MiB patterned compressed | 0.94120 [0.91476, 0.96536] | 0.244970 / 0.230522 |
| 64 MiB random raw | 0.92068 [0.88140, 0.96308] | 1.382691 / 1.277467 |

All four intervals indicate faster opening (about 10.0%, 6.6%, 5.9% and 7.9%).
Every process verified; all frozen hashes stayed stable. This is a repeated-open
diagnostic, not a whole-read or ZIP-parity claim. Absolute times from distinct
batches are not before/after controls. Total public CLI memory is not measured
here; removing map entries is not itself evidence of a lower peak RSS.

## Validation

The new decoder regression covers nine malformed record classes across all 16
modes, with a separate read of the unchanged real archive after each refusal.
It injects malformed in-memory records into the private decoder; the public CLI
cannot construct this experimental representation. It does not claim that those
malformed records were independently authenticated or persisted.

The focused traversal test, both credential-open tests, filesystem growth,
deletion, permissions and copy-loss lifecycle across all 16 modes, strict core
Clippy and the optimized build pass. The decoder regression covers missing and
duplicate fragments, orphan identity, nonzero initial ordinal, wrong logical
offset/length, wrong pack address, duplicate file identity and incomplete count.

The [runner report](raw/README.txt), [summary](raw/paired/summary.json), exact patch,
Dart script, logs and frozen source/executable/fixture identities are retained.
`raw/paired/<case>/process-samples.jsonl.gz` contains all 26,400 raw open
observations across 264 processes (including warmups), with lossless compression
verified during retention. Duplicate stdout/stderr, binaries and fixture payloads
are excluded. All build/test work finished before measured runs.

Next rerun the predeclared six-case whole-file/range ZIP comparison on the combined
accepted changes and retain CPU/RSS as well as elapsed times. The earlier ZIP
baseline fails all six cases. Public activation, scale, recovery, migration and
complete-format qualification remain incomplete.
