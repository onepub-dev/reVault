# Prepared guarded staging prerequisite

This is a **test-only (`cfg(test)`) shared-tree adapter** prerequisite for full-size
form texts and values. It connects bounded two-pass page preparation to the
existing typed-variable writer; it does not activate public forms or select a
format. Existing production secure-page entry points still generate fresh
nonces. Their common sealing/encoding body is refactored without changing wire
bytes, and affected production page/crypto/public tests pass.

The first pass encodes one guarded page, retains its exact size and stored digest,
and discards it. Immutable descriptors capture every sealing input, including
mode, archive/page/object identity, sequence, kind, key and nonce. One packed
guarded buffer retains full-payload fingerprints. The second pass obtains one
guarded payload from the callback, checks length and fingerprint in constant time
**before resealing**, and reproduces the exact page. Each protected page receives
a fresh nonce during preparation; reuse is private to reproducing that same
immutable page. Changed sources refuse before a second seal. No plaintext spool
or collection of all encoded pages is required.

Pre-write reservation, rebind and record-shape admission remain intact. First-pass
or preflight errors leave storage unchanged; second-pass callback failures use
the existing preparation/recovery protocol. An additional pre-write stored digest
and length check is retained for both prepared and existing ready-vector plans.
This adds work; unchanged vector CPU is not claimed. The prior resource ratios
remain tied to frozen `b6af42eb`, not this implementation.

The existing secure-page representation is uncompressed in every mode: plaintext
uses its raw body and protected pages use a normal-compression wrapper containing
one raw chunk. This work claims no compression gain. The existing experimental
4,096 prepared-page bound and other ownership/transaction caps remain explicit;
they do not qualify product capacity. Normal setters, read assembly, allocator
policy and the 8 MiB locked-memory limit are unchanged.

## Correctness evidence

Before formatting, all-mode prepared-page tests reproduce exact bytes, decode
independently, reject changed/short sources before resealing and cover composition
and capacity refusal. Segment tests pass 4/4. Prepared transaction tests cover
preflight failures before writes and both callback error/source change at either
segment, repeat recovery, retained selected values and erased/truncated abandoned
extents. The final focused run additionally checks nonce uniqueness across pages
and the unaffected neighbor file after recovery.

The complete variable module passes **8 tests, 2 ignored** serially, including
3,432 transaction cuts and 2,724 interrupted recoveries for value mutation and
1,512/504 for moves. Existing ready-vector add/replace/refusal and reused-write
controls pass, the latter covering 792 faults. Production crypto tests pass 1/1,
page tests 10/10, and the public variables/forms roundtrip passes all 16 modes.
Strict archive Clippy passes. Post-format verification at `edb07f36` passes the same complete variable module,
page 10/10, crypto 1/1, both ready-vector controls (including 792 faults), public
all-mode integration and strict Clippy. Exact logs and hashes are retained under
`post-format/`. The frozen aggregate probe is not rerun merely for formatting;
its pre-format source identity stays separate. Reproduction commands are in
`commands.txt`.

## Aggregate endpoint probe

Exactly **32 fresh processes** cover all 16 modes with one or twelve 1 MiB logical
values, without retries. Each uses a real FileStore, closes its writer, reopens
independently and compares every value through guarded reads. The fixture reuses
one guarded 1 MiB synthetic source for twelve distinct values; its temporary
ordinary synthetic construction buffer is intentional. This is not proof that
twelve simultaneous caller-owned SecretStrings fit under the same policy.

Twelve-value cases write 12 MiB logical data and 12,633,600–25,165,824 bytes of
stored payload, exceeding 8 MiB. Archives measure 13,238,272 bytes in modes 0–7
and 25,755,648 bytes in modes 8–15. All process endpoints stay within the unchanged
8,192 KiB limit: source construction reaches 1,352 KiB VmLck and the largest
observed endpoint is 2,788 KiB after reading. These are phase snapshots, **not
incremental peaks**, a concurrency guarantee, or a timing comparison. Retained
arenas and verification affect later endpoints. Earlier concurrent allocation
failures remain in the typed-variable evidence.

Mode bits are encryption=1, signing=2, compression=4, padding=8. Per-case metrics,
archive hashes and transcripts are retained under `aggregate/`. Frozen executable
identity, compiler versions, lockfile/source hashes and pre-format source patches
are in `probe-frozen/`; the binary and synthetic archives remain under
`/tmp/revault-prepared-staging-2026-10-09/` and are not committed. Frozen `.rs.txt`
snapshots preserve exact bytes while avoiding the formatting hook; manifest
source names remain provenance names. The probe predates only the final two
test-assertion additions, whose focused results are retained separately.

Forms still need typed metadata, full-size text segmentation, captured historical
field semantics, references and selected physical ownership. First establish the
actual public mixed-capture lifecycle as an isolated baseline. Public activation,
native recovery, complete archive performance/aging and compatibility remain open.
