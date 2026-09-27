# Fresh dense file-image experiment

Date: 2026-09-27. Based on `a491c5fe`. The containing commit connects the
whole-layout model to a persisted, readable file-only shared-control image. It
remains test-only: no production writer, public adapter, mutation or release
format is activated. This is a fresh comparison artifact, not a migration API.

## What now works

The builder authenticates the source candidate's publication, streams existing
independently protected fragments into denser packs, and writes a compact catalogue
inside the mirrored private control envelopes. Descriptor binding fields are
reconstructed without changing fragment AEAD. The idle compact preparation record
binds the new shared publication. Both metadata copies are checked and synchronized
before publication; publication copies are written with intervening syncs.

The bounded catalogue parser checks canonical integers, paths/order/identities,
file/chunk lengths and codec limits, complete contiguous pack ownership, no overlap
or unreferenced packs, padding sizes and exact sealed length. Fragment decode stays
independent even when aggregate logical bytes in one physical pack exceed the
restart bound. Normal open verifies protected padding; signed plaintext also
verifies all file bytes eagerly. Range reads verify each stored fragment and its
AEAD/codec before invoking the caller's callback. Decoded metadata is cached for
this stable-source session; no global metadata traversal is needed per file read.

The existing candidate's payload floor stays unchanged. An explicit shared-control
codec admits payload above its 128 KiB prefix; the old codec still rejects those
lower addresses. This does not permit data to overlap shared controls.

Fresh reopen supports explicit-key access and password/Contact bootstrap through
the shared profile. A password-to-file read is tested end to end. The source owner
cannot be substituted, source publication is rechecked around construction, and
unsupported source access trees are refused before writing. Provided fixture key
slots are newly constructed; existing grants/labels are not translated.

The builder requires an empty owned destination. It verifies source and staged
content, publishes, then independently reopens/verifies the result. Any returned
failure erases, synchronizes and truncates that destination; cleanup failures are
reported. A nonempty destination is untouched. The source remains byte-identical.
The caller must hold a stable source snapshot/read lock; path installation is not
implemented by this experiment.

## Validation

From `rust/`, before the formatting hook:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib file_format
cargo clippy -p revault_lockbox_api --all-targets --features external-source,native-block-layout -- -D warnings
```

[169 format tests pass](format-tests.log), five explicit probes remain ignored;
[strict Clippy passes](clippy.log). Five new tests include all 16 protection,
compression and padding combinations; empty/multichunk/small files; fresh reopen
and cross-chunk ranges; separate complete loss of either control region; eager
signed-plaintext and lazy protected corruption behavior; invalid ranges; malformed
catalogue prefixes/integers/kinds/permissions; sink failures; owner/access-tree
refusals; and password-based file reading.

[All 44 injected destination mutation failures](failure-cases.log) clear the fresh
output and preserve source bytes. These are returned-error tests, not a new
power-loss or process-death qualification. Internal fixtures are necessary because
no public CLI creates this experimental profile. The original candidate's existing
crash/recovery tests remain in the passing suite; their coverage is not transferred
to this new writer by implication.

## Limits and next integration

The current catalogue is limited to 1,024 files, 4,096 fragments and 64 KiB decoded
metadata fitting a 48 KiB private envelope. Unsupported overflow is refused. The
stored permission value is the cost model's explicit synthetic `0644`; source C
has no permission field. Directory/link, variable/form, mirror ownership, real
permission and access-label semantics are absent. Generation restarts at one in
a distinct experimental profile; this is not an accepted migration/history policy.

Only fresh images with an initial idle journal are accepted. Incremental mutation,
reservation overflow transitions, graph-derived retirement, recovery/salvage and
compaction/installation are not connected. The catalogue cannot yet encode free,
pending or external metadata state. Those protocols and the full public record
model must precede format selection and a new performance claim.

A `dense-create` resource-probe phase builds C as a temporary source, writes a
separate `dense.lbox`, reopens it and compares every file byte with the source
corpus. It intentionally reports size and verification only: its conversion/staging
work is not a comparable creation workload, and no CPU/RSS/time result is claimed.
Run retained cases with a frozen committed binary; preserve old controls and record
actual image sizes separately from the earlier projections.
