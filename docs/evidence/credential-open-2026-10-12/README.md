# Public API password-open memory — 2026-10-12

## Completed measurement

Retain bounded credential-directory reads on the existing locked storage handle.
The descriptive baseline completed 8 processes; the fixed comparison completed
56 (48 measured, 8 preparation) with identity, cold-archive, independent persisted
proof and output-drain gates passing. Sources and builds were fixed before each
batch; builds and tests did not overlap timing.

| Payload size | Median old → new, seconds | New/old elapsed ratio, 95% interval | Maximum old → new RSS, bytes |
| --- | --- | --- | --- |
| 64 MiB | 0.180 → 0.110 | 0.640 [0.611, 0.671] | 139,448,320 → 71,950,336 |
| 1 GiB | 1.065 → 0.110 | 0.109 [0.104, 0.115] | 1,146,363,904 → 71,946,240 |

Both aggregate and both round-stratum intervals favor the new path. Every new
measured process stayed below 100 MB. The 1 GiB case reduced elapsed time about
89% and maximum RSS about 94%; this is password API unlock plus a tiny proof,
not full extraction or a claim about normal CLI unlock. The detailed audit and
seeded paired bootstrap results are in [metrics-summary.json](raw/metrics-summary.json).

The change reuses the existing bounded directory reader and factors identical
ranked slot-unwrapping loops for bytes, Vault paths and already locked storage.
It preserves candidate ranking, credential errors, contact read-only intent and
the final authenticated archive open. No payload is accepted merely because a
slot unwrap succeeds. The same repair applies to contact-key API opens, but this
experiment does not measure contact performance. Fixed KDF costs remain included.

The frozen protocol follows; focused and batched integration validation are
recorded separately after measurement. Full format-4, candidate architecture,
many-file memory and release qualification remain open.

## Frozen protocol


This bounded experiment starts at `b6da783e`, after the retained native extraction
cache improvement. It measures `Lockbox::open(Password)` plus one tiny persisted
proof read and handle drop. The explicit CLI `open --password-env` already uses
bounded VaultOpen directory reads; ordinary cached-content-key CLI extraction is
also different. No CLI-unlock improvement is claimed from this API experiment.

Create two encrypted, unsigned, default-padded archives using the public default
CLI, with compression disabled: pinned random payloads of 64 MiB and 1 GiB plus
a tiny proof file. Credentials are synthetic and passed only in clean child
environments. Separate public CLI extraction must verify the complete payload
and proof bytes once before timing. Short isolated agent contexts must report
running through an actual public session roundtrip; stop only owned sessions.
No user configuration or credential store is consulted.

The evaluator opens through the public password API, writes `/proof.bin` to a
new output file, flushes/closes it and drops the archive. The runner independently
compares the persisted proof artifact, drains it and removes only that output.
GNU time measures the whole evaluator process, including startup, fixed Argon2
work, authentication/open, proof read and exit. Internal timing is secondary.
Whole-payload verification is setup-only, not claimed for the timed operation.

First run a descriptive old-code baseline: one excluded preparation and three
measured runs per size (8 processes). No runtime edits or overlapping builds/tests.
After evidence justifies a bounded-directory repair, freeze both old/new release
executables and run two preparations plus twelve measured rounds per size and
variant (56 processes). Alternate old/new order by `(round + caseIndex) % 2`.
Use twelve paired elapsed log ratios and a seeded 10,000-replicate bootstrap over
six adjacent two-round blocks, plus even/odd round views. No multiplicity correction.
Retain only if every measured new process uses less than 100,000,000 RSS bytes
(GNU time KiB ×1024), every aggregate elapsed upper 95% bound is at most 1.05,
and byte/identity gates pass. Report stratum uncertainty without changing counts.
This is a local password-path decision; contact performance is not measured.

Use CPU affinity 0–4, separate processes and cold owned-archive page-cache
admission (fincore resident bytes zero, at most three sync/fadvise attempts).
No internal reader-TID or physical-read admission claim applies. Every process
is bounded to 180 seconds plus 10-second kill grace; each batch has a 30-minute
cap that shortens commands to remaining time. Preserve failures and stop on
nonzero exit, timeout, changed identities, invalid metrics or output mismatch.
Freeze runner/protocol/source/binary/archive hashes before each batch and verify
them afterward. Keep all prior evidence and a combined 24 GiB owned-artifact cap,
including old corpus fixtures and the next temporary full verification output.

Only after measured retention, run focused direct file-backed password/contact
semantics, wrong credentials, mirror selection, authenticated refusal and access
intent checks. In-memory-only tests do not cover the changed locked-storage path.
Then perform the planned broader integration checkpoint for batched retained
runtime changes. No format change, weaker authentication or release is authorized.

## Evidence provenance

The 1,218 setup and per-process command logs are losslessly stored in
[command-evidence.tar.gz](raw/command-evidence.tar.gz). Every extracted file was
checked against [its SHA-256 manifest](raw/command-evidence.sha256); originals
remain in ignored local build storage. Extract from `raw/` to restore `commands/`.
Identity manifests, sample rows, summaries and validation logs remain directly
readable. [The tested source snapshot](raw/validation/pre-format-source.tar.gz),
[hashes](raw/validation/pre-format-source.sha256) and
[tracked diff](raw/validation/pre-format-tested.diff) record the pre-format epoch.
The tracked commit hook performs final formatting after validation.

## Validation outcome

Eleven focused release-mode credential/authentication tests passed, including
direct file-backed signed/unsigned password and contact opens, wrong credentials,
read-only authority, independent mirror fallback, both-mirror refusal, true
revocation, secure cache reads and authenticated-header rejection. Initial
test-only compile/transaction-fixture corrections are preserved with the passing
retry; no measured runtime change followed the paired comparison.

The batched integration attempted the complete default and native-block-layout
API library suites, with normal ignored tests and no filters or skips. Both hit
the declared 1,200-second bound (exit 124) while the same unchanged candidate
form-field recovery campaign was still running. Default/native logs contain
283/287 observed passes, 11 ignored each and no observed failure or panic, but
neither has a complete-suite summary. These are incomplete runs, not passes.
The initial wrong-directory invocation ran no tests and is also retained.
CLI creation options passed 3/3, password profiles 2/2 and the Vault API 30/30.
See [the result](raw/integration/result.json),
[partial accounting](raw/integration/partial-suite-accounting.txt) and adjacent
exact command/log/source-epoch evidence. No release qualification is claimed;
the continuing performance goal proceeds to the separately frozen many-file
extraction baseline.
