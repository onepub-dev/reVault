# Selective reader function profile

The user resumed performance work without the earlier usage cutoff. This diagnostic on `dc93b097` separates publication admission, format lookup, selected lookup work and payload verification. Temporary source instrumentation was removed after the measurements; restoring the saved originals produced an empty Rust diff against the committed baseline before the subsequent cache trial.

## Observations

Rust 1.88.0, CPU 2, warm OS cache, fresh archive handle per observation in a repeated-process diagnostic. These figures include nested timers/storage instrumentation and are not fresh-process ZIP comparisons. First and second selected reads share the same handle; the second reuses authenticated metadata, but still reloads and authenticates payloads.

| Case | Observations | Publication / format lookup (ms) | First / repeated selected read (ms) |
| --- | --- | --- | --- |
| 8 MiB raw, 4 KiB range | 100 | 0.123 / 0.130 | 0.050 / 0.049 |
| 64 MiB raw, 4 KiB range | 20 | 0.119 / 0.187 | 0.371 / 0.045 |
| 8 MiB compressed, 4 KiB range | 100 | 0.091 / 0.069 | 0.114 / 0.090 |
| 512 small compressed files, ranges | 100 | 0.098 / 0.147 | 2.766 / 2.636 |
| 64 MiB raw, full stream | 20 | 0.102 / 0.157 | 42.677 / 42.288 |

The 8 MiB fixture's single index leaf is already cached by the format lookup. The 64 MiB range must fetch additional index pages on its first read; its repeat approaches the 8 MiB payload-only cost. During opening, index decoding takes about 81 µs for raw 8 MiB and 99 µs for raw 64 MiB, alongside 46/83 µs of verified page reads. This supports testing a compact leaf representation instead of allocating individual buffers for every key/value on a fetched page. Every page still needs authentication and structural validation before selected data is returned.

Full raw 64 MiB reading spends about 32.18 ms in stored-byte SHA-256 commitments inside 40.31 ms of payload loading and 42.68 ms total read time. The checksum is nested in payload loading; storage-read time (~8.0 ms) is also nested there. Index decoding is only ~0.15 ms in that full read. A cache change cannot eliminate the bulk-read cryptographic cost, and this evidence does not authorize weakening or bypassing hashes. Compressed reads have a different payload/decompression cost profile.

## Validation and evidence limits

Strict Clippy, four selective-read tests and the full authenticated-index module (18 passed, one ignored manual probe) pass on the instrumented source. Every recorded case verifies full contents independently and checks its archive SHA before/after. Source and executable hashes are fresh before/after and stable. Raw observations, [summary](raw/summary.json), [stream summary](raw/raw64m-stream-summary.json), [audit](raw/audit.json), source snapshots, patch and executable scripts are retained; binaries and fixture payloads are excluded. Rust snapshots use `.rs.txt` so commit formatting cannot alter measured bytes.

Two collection mistakes are preserved: an initial raw8 run completed 100 observations but its collector missed the test runner prefix before the JSON, so it repeated raw8 during the corrected collection; the duplicate setup output is excluded from the summary. The collector applied its 20-observation raw64 setting to both range and stream, so the range has 20 observations, not the requested 100. There was no timing-based selection or rerun.

The initial fixture hash maps in `summary.json` are empty because its walker did not follow fixture symlinks. A later dereferencing audit matches every fixture input against the retained comparison's before/after identities and the diagnostic's archive digests. That later audit is not a fresh per-run before/after hash record for all fixture inputs. The archive digest checks and full-content verification within each diagnostic did run. No stronger fixture claim is inferred from the empty maps.

These measurements identify follow-up work; they do not establish a speedup, ZIP parity, public CLI memory qualification or release readiness.
