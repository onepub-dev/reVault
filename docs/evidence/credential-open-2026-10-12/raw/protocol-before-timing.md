# Public API password-open memory — 2026-10-12

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
