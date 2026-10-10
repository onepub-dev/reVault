# Stored-byte hash cost experiment

The selective-read profile attributes most raw full-stream time to the existing domain-and-length-bound SHA-256 commitment. This isolated experiment compares that hash cost with BLAKE3 without changing archive code, archive bytes, authentication policy, dependencies or worker count.

## Method and results

Rust 1.88.0 on the recorded x86-64 host; `sha2 =0.11.0` and `blake3 =1.8.7` with only `std,pure` features. Each fresh process hashes the same 64 MiB random fixture in independent units, with an algorithm-specific domain and encoded length for each unit. Input loading and independently assembled one-shot validation occur outside timing. Each case has three warmup pairs and 30 measured pairs, alternating algorithm order on CPU 2 with one thread. A fixed-seed 10,000-resample paired log-ratio bootstrap supplies intervals. This measures repeated domain-bound fragment hashing, not a single whole-file hash.

| Unit | SHA-256 median (ms) | BLAKE3 median (ms) | BLAKE3/SHA-256 time [95% CI] |
| --- | --- | --- | --- |
| 4 KiB | 32.968 | 65.298 | 1.9817 [1.9745, 1.9875] |
| 64 KiB | 32.066 | 23.574 | 0.7456 [0.7372, 0.7556] |
| 256 KiB | 32.031 | 20.343 | 0.6403 [0.6313, 0.6518] |

All 198 sample rows verify their digest aggregates. Fresh before/after hashes match for the input, binary, manifest, lockfile, source and runner. Strict Clippy and both initial smoke runs pass. Source, runner, lockfile, feature/build evidence, [summary](raw/summary.json), [samples](raw/samples.jsonl) and identity records are retained. Initial sandbox launches failed because of the resident compiler/read-only Flutter cache; the successful fixed batch used the direct Dart SDK and authorized dcli launcher. Those launcher errors did not produce benchmark samples.

The verbose build selects Rust SSE2/SSE4.1/AVX2 implementations, with no Rayon dependency. The Rust `cc` crate remains a build dependency; selecting a Rust implementation for this host does not establish all-platform dependency qualification. BLAKE3's official [pinned manifest](https://github.com/BLAKE3-team/BLAKE3/blob/1.8.7/Cargo.toml) describes `pure` as an undocumented testing feature that may change. This pinned experiment is not a production dependency strategy.

## Decision and limits

Retain SHA-256 in the archive implementation for now. BLAKE3 is a plausible large-fragment experiment, but the 4 KiB cost regresses almost twofold and this is not an archive-level result. No security or compatibility decision follows from these timings alone. A future proposal must separately specify domain separation and algorithm/version binding, distinguish page and payload commitments, preserve verification before exposure, qualify supported platforms and dependency policy, and compare complete archive reads against the current reader and ZIP. It must address historical formats and the compatibility-line contract before any writer can emit different commitments. No Vault/mtime trust, checksum omission, CRC substitution or hidden worker increase is proposed.

These results do not establish G11 parity, encrypted/compressed performance, cold storage performance or performance on a different CPU. Increasing fragment size would also change small-range read amplification and needs its own experiment.
