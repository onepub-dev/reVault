# Bounded preparation from authenticated selected payloads

This test-only (`cfg(test)`) shared-tree prerequisite prepares replacement form
texts from selected stored payloads without cloning the archive or retaining all
values. It is connected to a typed replacement fixture, not yet to public field
setters or cross-record secret upgrades. Those must preserve the separately
established public baseline before activation. No format has been selected.

SelectedSource has a private validating factory. Normal private publication and
complete index/ownership authentication precede exact old extent admission; each
(start,length,digest) must match a selected payload. Duplicates, stale base,
metadata/free/pending sources and oversized lists refuse. Admission runs before
reclaimed-range scans, so invalid capabilities cannot cause preparation-source
reads. Valid capabilities still require the existing reclaimed-zero validation
before return. This is admission ordering, not reduced integrity.

The reference-only read view bounds every operation to old sealed length and one
admitted extent. It refuses allocating read_at and mutation; Clone copies only
references. Capability membership does not authenticate content or independently
prove read_at_into destination provenance. The reviewed renderer uses guarded
read_at_secure/direct read_at_into, then validates stored digest and full page
kind/identity/context/ordinal/length. Stable source storage remains a caller
precondition. There is no generic Storage.clone shortcut or ordinary plaintext
spool; clone-trap and allocating-read guards qualify the connected path.

prepare_stored first validates one complete source text in guarded memory, drops
it, and prepares one segment at a time. The segment reader preserves complete page
checks; whole-text reading retains full UTF-8/control validation, while segments
may split UTF-8. Immutable source/destination layouts and a wipe-on-drop key drive
second-pass rendering. Prepared fingerprints bind complete destination payload
and seal context before any second seal. The existing secure-page representation
remains uncompressed. Repeated per-part structural validation is extra bounded
work, not a measured performance improvement.

An additive prepared writer plan revalidates base and source membership against
its own fresh selected graph before writes. Old source extents cannot be staging
reservations and remain readable until publication and authorized retirement.
with_page receives the Append wrapper, and each scoped read/inner RefCell borrow
ends before rendering, writes and secure readback. Existing append position,
reservation, pre-write length/digest and ownership checks remain. Existing ready
vector and borrowed-source prepared paths retain their behavior.

## Correctness evidence

Four final focused tests and three form-segment tests pass; strict Clippy for
library/tests/benches passes. All-mode connected replacement increments each text's
revision while preserving UUID/context/content, independently reads/streams every
record and erases retired old pages. Separate recontextualization copies a normal
name into a secret-value destination context, preserving source bytes/extents and
retiring only the old target. Empty, split-UTF8 and exact1MiB values pass.
Invalid capability/view operations refuse before payload-source reads/writes;
valid capabilities refuse reclaimed corruption. Stale writer plans invoke neither
renderer nor persistent operation, though ordinary authenticated graph/reclaimed
reads remain permitted. Renderer I/O, changed-source digest and destination
fingerprint failures before the first payload or after one staged payload preserve old selected state after
recovery. Deliberately corrupted old source bytes are restored only before that
transaction-recovery assertion: no source repair is claimed.

The representative eight-mode transaction matrix passes 3,288 transaction cuts
and 1,020 interrupted recoveries. It includes signed plaintext and
protected unsigned. All storage operations are cut at zero/97/full prefixes and
both modeled sync persistence outcomes, followed by normal/repeated and selected
interrupted/resumed recovery. Complete old/new text revision sets, logical
contents, file/variable neighbors, retired-source erasure and abandoned-tail
cleanup are checked. This is modeled returned-failure evidence, not OS process
termination or independent region-loss qualification.

## Fixed aggregate scope

Exactly 16 fresh Linux processes passed, one per mode without retries, using
a frozen binary/source/runner and unchanged inherited8MiB memlock/CPU affinity.
Each constructs the existing13,631,586-byte FileStore snapshot, drops writer and
caller objects, reopens a writer and replaces all form texts from admitted stored
sources through the clone-trap path. One synthetic caller secret initially supplies
eight distinct1MiB fields; normal metadata is ordinary input. Replacement does not
retain those caller objects or assemble every stored secret simultaneously.

The after_selected_source endpoint is captured before explicit retired-zero audit.
Writer is closed before an independent FileStore reader handle opens within the
same fresh process, not a separate verifier process. Every large normal text and
secret field is independently read, then every streamed event is verified one at
a time. VmLck/VmRSS are phase endpoints; VmHWM includes fixture setup and retained
arenas. No incremental peak, arbitrary concurrency, full-object getter, CPU ratio,
aging or complete-format acceptance follows. Report archive growth explicitly;
retired free space is not silently compacted away. Mode bits are protected1,
signed2,compression4,padding8; form pages remain uncompressed in every mode.


All final outcomes have one expected marker and exit zero. Frozen binary
`677b5520b4b3d01c56207202a3625e3dc637e6160a3f882963397760e379497b`
and runner `f9d254996d5ce1f79c1b0a852f9da62a3012bc2d3c70a39b59d96ceb4873abc7`
remain unchanged before/after; all archive hashes/sizes match retained outcomes.
The completed batch was recovered after an agent restart; no attempt was rerun.
Inherited CPU affinity was CPUs 0–15 and memlock soft/hard limits were 8 MiB.

| Endpoint | VmLck KiB | VmRSS KiB |
| --- | --- | --- |
| After selected-source replacement | 3,812–4,076 | 21,244–25,584 |
| After independent reads | 3,812–4,076 | 19,388–26,240 |
| After streamed salvage | 3,812–4,076 | 21,308–26,304 |

Whole-process VmHWM was 77,456–78,780 KiB, including setup. Archive sizes grew
from 14,548,992 to 28,311,552 bytes unpadded and from 29,818,880 to 58,916,864
bytes padded while old payloads were erased. This is not an aging/space pass.

## Retained provenance and checks

`final-qualified/` retains final focused/segment/Clippy logs, all 16 probe logs,
`outcomes.json`, exact runner, compiler/manifests/source hashes and frozen diff.
New Rust snapshots use `.rs.txt` to preserve bytes without triggering formatting.
Binary/archive files remain at the original `/tmp/revault-selected-source-*`
locations and are not checked into Git. Retained hashes identify them.
`qualified/` retains the expensive fault matrix and unchanged prepared/vector/
variable lifecycle controls. That source predates late-failure and
recontextualization test additions; production prerequisite code is unchanged.
`initial/` preserves the missing-import compile error; `qualified/clippy.log`
preserves the test-only unnecessary-borrow lint, corrected in the final checks.
Earlier passing iterations remain under `initial-rerun/` and `controls/`.

Commands use pinned Rust 1.88.0, release `external-source` library tests with
`--test-threads=1 --nocapture`; strict Clippy uses `--features external-source
--lib --tests --benches -- -D warnings`. Exact commands are in the logs/manifest.
Formatting has not yet run; the tracked hook and affected post-format verification
will be recorded separately. Existing 4,096-entry and preparation limits remain
experimental caps. Public field mutation, architecture selection and complete
format/resource/recovery qualification remain outstanding.
