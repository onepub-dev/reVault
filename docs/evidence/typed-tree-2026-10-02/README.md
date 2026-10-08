# Typed shared-tree membership and read-only salvage

Date: 2026-10-02. Uncommitted experimental work based on `4744b635`, following
the [raw transition/reuse](../shared-tree-transitions-2026-10-02/README.md) and
[allocation-map aging](../compact-map-bank-2026-10-02/README.md) checkpoints.
The [source manifest](source.sha256) pins the ten files in this tranche and
supersedes earlier hashes for overlapping paths; the earlier manifests retain
their historical validation scope. No native-format activation is implied.

## Implemented scope

The candidate tree image stores bounded indexed records for files, fragments,
physical packs and filesystem nodes, reusing existing canonical path/permission
validation and data codecs. Catalogue payload claims must exactly equal the
selected ownership graph. Metadata updates validate complete typed semantics
before calling the raw authenticated tree writer and preserve existing payloads.
The tree has an explicit experimental 100,000-node bound and 32 MiB aggregate
path bound; existing dense limits and public capacity guarantees are unchanged.

Fresh export verifies and preserves its dense source. It requires an empty
destination and refuses access-slot translation; it is not an in-place migration,
an inline/tree transition or a complete credential adapter.

Read-only salvage authenticates selected publication and membership, then uses
the existing filesystem sink and recovery codec to verify files independently.
Missing selected membership is fatal; no scan of older generations substitutes
for it. Deleted nodes remain absent. Callers retain the stable snapshot and stage
sink output until success. Normal signed-plaintext opening still performs eager
full-content verification: successful partial salvage does not weaken normal open.

## Validation

From rust/, the economical runner used release external-source tests filtered
to typed_tree, then file_format::, followed by strict external-source library,
test and benchmark Clippy. See retained logs for executable and result details.

- [Focused tests](tests.log): 5 passed.
- [Format regression](format-tests.log): 225 passed, 6 manual probes ignored.
- [Strict Clippy](clippy.log): passed.
- All 16 modes exercise file bytes, metadata growth to 2,504 nodes beyond 64 KiB,
  deletion, permissions, symlink targets, unchanged repeats and metadata-copy loss.
- Invalid parent/path/aggregate bounds and selected pack-claim mismatch are refused;
  the aggregate-path refusal check is inside an existing test, not a sixth test.
- 1,128 interrupted-growth cases cover four modes, with complete selected metadata
  independently reopened. This is not an all-16-mode exhaustive fault claim.
- Two-file salvage across all 16 modes covers corrupt-neighbour payloads,
  truncation of a tail metadata copy, either control bank lost, and failure before
  sink output when both selected root copies are lost after a node deletion.

The lifecycle fixture completes at 1,114,112 bytes in every mode. No paired
performance, CPU/RSS, cold-I/O or complete lifecycle-aging observation was run for
this adapter. Prior file-only ZIP comparisons cannot be relabelled as this result.

## Remaining boundaries

In-place dense inline/tree growth and return, payload mutation, variables/forms,
access-slot mutation, public/native integration, migration and full qualification
remain unfinished. Both historical native recovery failures still describe the
old native encoding. Safely activating shared membership there requires the
publication/compatibility contract in decision 001; this test-only proof does not
select that format. No commit, formatting or production publication occurred.
