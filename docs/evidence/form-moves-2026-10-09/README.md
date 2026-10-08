# Atomic selected form record moves

This test-only (`cfg(test)`) shared-tree adapter moves selected form records in a
single metadata transaction. It preserves stable record UUID, every segmented
text UUID/revision/context/extent, captured contents and exact definition
reference. Form context excludes path, so no payload page is copied, rewritten or
retired. Public activation, field/revision mutation and format selection remain
unfinished.

The complete canonical mapping is validated before any persistent write. Duplicate
sources, missing sources, duplicate destinations and unrelated occupied targets
refuse; cycles, swaps, chains and self-moves are valid. Empty/self-only operations
still authenticate and verify selected contents, then return unchanged without
allocation/publication changes. Missing ordinary parent directories are created
only in the candidate catalogue and published atomically with the move. Refusals
leave archive bytes unchanged even if an earlier proposed target needed parents.
Successful namespace behavior preserves the public baseline: files/forms may share
a path, direct file parents are accepted and form ancestors do not conflict.

The zero-page prepared transaction retains full selected ownership validation.
Tests reject allocating read_at over form/variable secure-page spans and any write
or truncation affecting live payload spans. Guarded read_at_into provenance also
relies on reviewed storage paths; ordinary file verification keeps its existing
wipe-on-drop path. Metadata-only describes persisted changes, not skipped content
authentication. Original payload bytes are independently compared after moves
and recovery.

## Verification scope

The two move tests pass, including **1,488 transaction fault cases** and
**504 interrupted recoveries**. Existing snapshot, guarded admission and normal
deletion/salvage controls pass, as does strict Clippy. All 16 modes independently reopen and reconstruct captured
values/references, compare exact UUID-to-path/layout identity and retain files,
variables and definitions. The sequence covers swaps, chains, new parents,
same-path file/form, direct file parents and form ancestors. Refusals/no-change
preserve image bytes; corrupted selected content refuses even an empty request.
Either control-bank loss exposes current paths only through selected salvage.

The eight-mode fault matrix includes signed plaintext and protected unsigned.
A swap plus a third record moved under newly created /new/deep directories is cut
at every storage operation with zero/97/full write prefixes and both modeled sync
persistence outcomes. Every recovered result is exactly the old or new record
mapping AND parent directory set. Contents, payload identity/bytes and abandoned
metadata-tail cleanup are checked after normal, repeated and interrupted/resumed
recovery. This is modeled returned-failure evidence, not OS process death.

No resource benchmark is rerun for this metadata-only change, and no performance,
concurrency or format-wide capacity claim follows. Earlier form aggregate and
borrowed-variable ratios remain tied to their original source variants.

Checks use pinned Rust 1.88.0, release tests with external-source and serialized
test execution, followed by strict Clippy for library/tests/benches. Source
identities and complete logs are retained in `initial/` and `qualified/`. Exact
new-source snapshots use `.rs.txt` to avoid formatting frozen evidence.
Post-format results will be retained separately after the tracked hook.
