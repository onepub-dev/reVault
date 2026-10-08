# Atomic selected form field mutation

This test-only (`cfg(test)`) shared-tree adapter sets borrowed normal/secret
FormValue inputs and connects cross-record secret upgrades to authenticated
selected-source preparation. It does not activate the public API or select a
format. Public history baseline and compatibility rules remain authoritative.

The setter uses existing effective-kind validation before constructing an upgrade
candidate. Secret input to a normal URL/date/etc field validates as Secret, matching
the public upgrade behavior. A historical secret capture may receive normal input
when latest schema declares normal; the normal-input refusal follows latest kind.
These fixtures import historical snapshots and do not exercise a new schema-edit
API. All accepted mutations and byte-noops retain full selected verification.

A normal-to-secret upgrade adds a checked new definition revision with fresh text
identities; all old definition revisions stay selected. Every same-type record
advances its definition reference/alias, including absent captures. Matching
captures adopt current label/kind; old normal values convert through guarded
selected sources, old secret bytes survive, and the target caller override is
planned directly. Removed/other captures and unrelated types stay unchanged.
Ordinary setting updates only target reference/capture, matching public semantics.

Record identity is stable. Existing changed label/value IDs retain checked text
revision counters. Equal bytes do not justify reuse across a kind/context or
sensitivity change; equal context/content permits exact layout reuse. Label-only
changes preserve value extents. Ref/alias-only updates use a zero-page transaction.
Verified identical updates are physical noops, an explicit adapter optimization
beyond the public baseline's logical repeated-set contract. Counter exhaustion
refuses without persistent writes; valid maximum-revision text still permits a noop.

Preparation retains metadata descriptors, per-page guarded fingerprints and the
borrowed caller; source extents are admitted before first preparation. Mixed
callbacks read only selected old sources or the immutable caller. Each source
scope ends before reseal/write/readback; no archive clone or plaintext temporary
spool is used. Old source pages remain readable until publication and authorized
retirement. Existing row/graph/preparation limits are experimental; no new aggregate
1MiB cap, allocator change or broader capacity claim is introduced.

## Checks and retained failures

Final checks pass: five field controls (one aggregate probe ignored), three form
segment tests, three public form-history tests and strict Clippy for library,
tests and benches. Initial all-mode mixed-history test passed. A strengthened
control run passed label/reference behavior but failed the test helper's whole-
archive SecureVec identity scan with
`SecurityLimitExceeded("weakened secure memory allocation is disabled")`.
That helper was replaced by a length-bound SHA-256 scan using guarded64KiB chunks;
no setter or memory policy change was made. Archive size at that failing allocation
was not logged, so this result supplies no setter capacity inference. The revised
three-control group passed with strict Clippy. The authenticated maximum-text
revision control also passed; the upgrade matrix passed 2,232 transaction cuts
and 672 interrupted recoveries. Preserve all source variants/logs.

Final controls also cover label-only updates preserving value extents, zero-page
reference/alias transitions, equal-byte sensitivity/context changes, invalid Secret
input before upgrade, exact1MiB/split UTF-8, and corruption refusal on a nominal noop.
The separate move/noop/delete/salvage fixture exercises secret value replacement,
not a schema upgrade. Archive identity assertions use the length-bound streamed hash.

The recovery matrix models returned failures, not OS process death. Successful
baseline explicitly checks target override, absent-field retention and normal
neighbor conversion before using exact old/new public objects as its atomicity
oracle. Stable existing identities, unchanged extents, retired erasure, abandoned
new spans/tail, repeated/interrupted recovery and file/variable neighbors are checked.

The fixed aggregate protocol separately defines nine distinct1MiB normal values,
one borrowed secret override and eight selected-source neighbor conversions.
Setup callers are dropped before mutation; override stays borrowed through writer
return and is dropped before independent reader opens. No full collection of secret
values is assembled. Independent FileStore handle/scoped reads and streamed events
verify membership and bytes. Exactly 16 fresh processes passed once per mode, with no retries; all markers,
mode identities, test results and archive hashes/sizes were independently checked. No timing ratio, incremental peak, concurrency/aging/fullgetter,
released compatibility or full-format acceptance follows.


## Fixed aggregate results

Logical selected text totals 9,437,479 bytes. All 16 modes complete under inherited
8,388,608-byte soft/hard memlock and CPU affinity 0–15. The attempt journal contains
exactly 32 ordered started/completed entries. Frozen binary
`64f24e011de60a056ca436b3027383a1372009b08892a0b86c94058dc563f807`
and runner `c3068c0fd7776deac2120da6ce3f60e5c7130996aa5143dcce87347addd6b330`
retain identical before/after hashes.

| Endpoint | VmLck KiB | VmRSS KiB |
| --- | --- | --- |
| Immediately after upgrade | 3,812–4,140 | 22,164–22,924 |
| After independent reads | 3,812–4,140 | 22,168–22,964 |
| After streamed salvage | 3,812–4,140 | 22,292–23,092 |

Whole-process VmHWM is 78,116–79,540 KiB, including setup. Retained arenas and
caller lifetimes are explicit in the protocol; these endpoints are not incremental
peaks. Unpadded archives grow from 10,878,976 to 20,381,696 bytes; padded archives
from 22,347,776 to 42,795,008 bytes. Retired pages are erased, not compacted away.
Mode bits are protected1, signed2, compression4, padding8; form pages remain
uncompressed in all modes. There is no new CPU/size/aging acceptance claim.

## Reproducibility and remaining scope

`final-qualified/` retains exact commands/logs, outcomes, durable attempt journal,
compiler/manifests/source hashes, frozen tracked diff, untracked `.rs.txt` snapshots
and runner. Snapshot suffixes preserve Rust source bytes outside formatting hooks.
Frozen binaries and archive images remain at their original `/tmp/revault-form-fields-*`
paths and are not committed. `initial/`, `controls/`, `bounded-controls/` and
`recovery/` retain their original outcomes/source identities; no failure was erased.
The setter implementation is unchanged across these variants; later changes add
or strengthen fixtures and bound the test identity scan.

Tests use pinned Rust 1.88.0, release `external-source` with serial test execution.
Strict Clippy uses `--features external-source --lib --tests --benches -- -D warnings`.
The expensive 2,232/672 matrix predates only the added coexistence/aggregate fixtures;
final affected checks cover the frozen aggregate source. Hook formatting and
post-format checks remain to be recorded separately.

Next: establish public definition/revision/resolution and empty-record creation
baseline, then connect those semantics without public activation. Historical
capture admission and stricter migration-import policy remain distinct. Architecture
selection, broader public APIs, native recovery, process-death/region-loss, resource
and compatibility qualification remain open.
