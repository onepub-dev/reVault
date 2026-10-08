# Equivalent FSE-table arithmetic optimization

Date: 2026-10-02. Promoted locally after safety and paired performance checks.
The baseline is the accepted [checksum/wiping correction](../decoder-checksum-2026-10-02/README.md),
not the rejected overlapping-copy trial.

The pinned profile attributed dynamic table-building cost to `build_decoder`,
not predefined-table construction. The prior helper repeated power-of-two,
integer-division and distribution arithmetic for every state of a symbol.
For valid power-of-two tables it now uses:

```
next_state = probability + state_number
bits = log2(table_size) - floor(log2(next_state))
baseline = (next_state << bits) - table_size
```

The old arithmetic remains the fallback for zero/out-of-domain inputs. Table
entry ordering and values, codec format, checksum comparison, memory bounds and
wiping are unchanged; no cache or private-input retention was introduced.

## Validation

The exhaustive reference test compares **11,188,905** combinations across logs
1–12, every supported probability/state pair, plus zero/out-of-domain cases.
Vendor decoder/workspace tests and three targeted streaming tests pass. Release
reVault compression tests pass 20, format tests pass 241 with 7 ignored, and
strict Clippy passes. Main-worktree reruns pass after promotion; all six promoted
checksum/wipe/FSE source hashes match the isolated source before and after.
The exact FSE source and hash, test logs and compiler identities are retained here.

## Paired performance

Four cases each run 30 pairs after three warmups against the checksum-corrected
baseline, with identical archives, protection, units, one worker, CPU 2 and
fresh handles. Exact-byte/authentication smokes pass and all input/executable
hashes remain unchanged. Both readers use Rust 1.88.0. Full observations are in
[measurements/](measurements/) and the [metric table](fse-paired-results.tsv).

Ratios are optimized/baseline duration; smaller is better.

| Case | Read-only ratio, paired 95% interval | Total ratio, paired 95% interval |
| --- | --- | --- |
| Compressed 8 MiB plaintext | 0.955 [0.940, 0.967] | 0.957 [0.944, 0.968] |
| Compressed 8 MiB protected | 0.961 [0.952, 0.969] | 0.966 [0.959, 0.972] |
| Small mixed files | 0.913 [0.908, 0.919] | 0.932 [0.927, 0.937] |
| Raw control | 0.995 [0.988, 1.001] | 0.994 [0.989, 0.999] |

The measured compressed read benefit is approximately 3.9–8.7%, depending on
the corpus. RSS intervals include parity. Raw read shows no clear change, as
expected for a decoder-only optimization. These case-specific gains do not
establish full ZIP parity, A3/A4/A5 qualification or public/native activation.

The accepted binary's [bounded CPU profile](profiles/) has 601 samples and no
lost samples. Table-building self CPU share falls from the prior 9.74% to 3.99%,
supporting the proposed mechanism. These are sample proportions, not absolute
time reductions. Remaining leading costs are copying (20.8%), XXHash (17.8%),
sequence decoding (17.8%) and the checked FSE state update (5.16%). A subsequent
isolated inline-hint experiment targets that last call without changing its
operations or checks.
