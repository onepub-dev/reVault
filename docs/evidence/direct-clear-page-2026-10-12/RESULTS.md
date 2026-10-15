# Direct raw cleartext page serialization: retained

The fixed 112-operation campaign passed every frozen performance and correctness
gate in 1051.541 seconds. There were 96 measured processes and 16 excluded
preparations. All persisted bytes/modes, constant archive sizes, cold-source
admission, identities and owned-agent cleanup checks passed. The implementation
is retained after 37 passing focused test executions and successful current-source
portable WASM regeneration and lifecycle checks.

| Input / workflow | Elapsed ratio [95% interval] | Median seconds, old → new | Median RSS ratio | Maximum RSS, old → new (MB) |
| --- | ---: | ---: | ---: | ---: |
| source-zstd-1gib / single-file-interactive | 0.7602 [0.7244, 0.8019] | 2.160 → 1.650 | 0.8025 | 108.87 → 89.69 |
| source-zstd-1gib / one-file-directory-bulkimport | 0.7538 [0.7325, 0.7748] | 2.165 → 1.650 | 0.7495 | 99.75 → 79.45 |
| random-raw-1gib / single-file-interactive | 0.4828 [0.4650, 0.4952] | 6.525 → 3.190 | 0.7977 | 87.64 → 71.31 |
| random-raw-1gib / one-file-directory-bulkimport | 0.5270 [0.5073, 0.5562] | 6.550 → 3.360 | 0.8294 | 71.25 → 62.64 |

Ratios are new/old. MB means 1,000,000 bytes. The analysis uses six adjacent
two-round paired log-ratio blocks and 10,000 seeded bootstrap draws. All four
aggregate and both round-stratum elapsed upper bounds were below 1.0. All
maximum-RSS ratios were below 1.0, comfortably satisfying the 1.05 guards.
The named compressed Interactive median-RSS ratio 0.802476 met the required
at-most-0.95 usefulness gate.

Measured elapsed reductions were 24.0% and 24.6% for compressible Interactive
and BulkImport, and 51.7% and 47.3% for their raw counterparts. Median RSS fell
19.8%, 25.1%, 20.2% and 17.1%, respectively. Every measured candidate run in these
four plaintext one-GiB add cohorts was below 100 MB; the largest observed peak
was 89.69 MB. This closes that threshold only for this measured scope. It does
not establish a universal 100 MB bound for encrypted archives, many-file jobs,
other workloads or combined helper processes. Whole-process GNU time RSS here
excludes the separately running session agent.

## Change and compatibility scope

Eligible raw cleartext pages are serialized directly into their final allocation:
existing page header, body checksum, raw-body header, object stream and zero
padding. This removes the body/stored-body intermediates as well as the earlier
object-stream copies. Checked full size and u32 conversion precede allocation;
unpadded extents must match exactly. Shared serializer/header helpers preserve
the existing wire definition. A ZeroizingBytes owner wipes partial plaintext on
error/unwind and transfers the final Vec only after successful completion.

The existing cleartext decision and body-compression predicate select the fast
path. Compressed pages and encrypted page bodies keep their existing codec/AEAD
path. Cleartext KeyDirectory pages are eligible even in encrypted archives,
but no encrypted-write performance gain is measured or claimed here. No worker,
cache, compression, signature, format or release-line policy changes.

The preceding reserve and direct-body trials remain rejected evidence. This
larger ownership change passed a fresh fixed comparison against retained normal
CLI 08bf, not against either rejected trial. There were no outcome-dependent
extra samples or relaxed gates. This is not a new ZIP comparison and does not
replace the separate public ZIP reports or release qualification.

## Reproduction and validation status

The unchanged [protocol](README.md), [full summary](raw/metrics-summary.json),
[raw rows](raw/paired-samples.jsonl), [completion](raw/paired-complete.json) and
[identities](raw/paired-identities.json) preserve the exact decision. The original
48.82-second release-mode build log, patch, base revision and source/binary
manifests are in `raw/build-new`. `measured-page.rs.txt` preserves the exact
pre-test source epoch; later test additions and hook formatting are separate.

Baseline development CLI:
`08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`.
Measured candidate:
`42c6d6bd0250bab488b614f151555bd131eca3293107ae5e8bf2b71bc4712d21`.
All output was independently extracted by the normal CLI and compared byte for
byte with pinned input, including default stored mode 0600. Archive sizes were
183204224 / 183209344 / 1074206080 / 1074206080 bytes for the four table rows,
constant across both variants.

All 11 release-mode focused filters passed, totaling 37 test executions (some
filters overlap): default/native page tests and buffer wiping, cache pressure,
injected storage failures, packed survivors, secure neighbors, multiframe reopen
and two public CLI lifecycles. Exact commands, counts and logs are in
`raw/focused`. The independent full-page reference checks normal and secure
objects, empty streams, headers, checksums, padding and invalid extents.

The current-source WASM release build completed in 1m47s; wasm-bindgen 0.2.128
regenerated the artifact. Cached Node v24.19.0 passed the password lifecycle
(1/1) and the setter/invalid-input/Auto/commit/free/reopen lifecycle. JavaScript
and declarations were byte-identical; only the WASM binary changed. Source
manifests matched before and after building. Commands, lifecycle source and
logs are in `raw/wasm`; the build emitted 89 existing Vault API warnings.
The generated WASM SHA-256 is
`222f8daf135c72ec224353acc3e6435f8b5afe0cf8de4065c04e7ed391d7c0cf`.

Verbose command evidence is preserved losslessly in
`raw/command-evidence.tar.gz`, with individual hashes and verified extraction
recorded beside it. Measured and qualified pre-format source snapshots remain
separate from final hook formatting.

Earlier full API attempts remain incomplete at their documented timeout. The
next integration checkpoint will revisit those bounds using the completed
isolated campaign evidence; these focused checks are not a full-suite pass.
