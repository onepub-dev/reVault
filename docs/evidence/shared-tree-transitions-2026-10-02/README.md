# Authenticated raw-tree transitions and retained-region reuse

Date: 2026-10-02. Local uncommitted checkpoint on
`issue-310-zip-read-performance`, based on `4744b635d2adcd72a9d5d4b617d625f61ef6983f`.
The [source manifest](source.sha256) identifies all 12 changed/new Rust files;
paths are relative to the repository root. No formatting, commit, publication,
production activation or format selection is implied.

## Implemented scope

The test-only shared tree now has graph-authenticated abort recovery, selected
publication roll-forward, and bounded raw-record COW replacement. The writer
validates ordered records and signing authority before mutation, preserves
payload/key ownership, and leaves unchanged repeats byte-identical. Recovery
selects the actual committed generation, never a caller-selected old snapshot.

Reused metadata regions must be fully covered by authenticated, verified-zero
vacant claims. Adjacent short pending claims and free tails retain their separate
erasure obligations. Journal arenas can also be reused: a durable inline record
reserves both arena slots before writing them. Unlinking both stubs retains two
inline reservations until arena erasure completes, preserving cleanup authority
through interrupted recovery. No old journal reservation authorizes erasing new
committed live data. Selected dependencies/publication are mirrored before
retirement; all modes and independent copy loss are exercised.

This remains a bounded raw-record adapter with a 64 MiB input admission limit,
not an accepted public capacity change. Typed files/variables/forms, complete
inline-to-overflow and return-to-inline integration, public mutation, compaction,
migration and full architecture qualification remain unfinished. Existing dense
inline behavior is retained. Original pending journal work was preserved and
extended; unrelated worktrees and modifications were not discarded.

## Final source-aligned validation

Commands from `rust/`, run sequentially by the economical test agent:

```text
cargo test -p revault_lockbox_api --release --features native-block-layout,external-source --lib authenticated_tree_update -- --nocapture
cargo test -p revault_lockbox_api --release --features external-source --lib file_format::
cargo clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
```

- [Focused writer tests](writer-tests.log): 6 passed, including 3,168 writer
  power-loss cases, 3,360 reused-arena abort cases across all 16 modes, and 240
  late large-writer cleanup faults across two modes.
- [Format regression](format-tests.log): 220 passed, 6 ignored.
- [Strict Clippy](clippy.log): passed.
- Independent reopen verifies old-or-new membership and preserved payload bytes;
  malformed input and invalid authority/ownership refuse without modification.

The six ignored tests are explicitly invoked manual observations: allocator
aging, 100k-entry index resources, candidate-file resources, mixed payload aging,
journal resources, and committed-overflow resources. No existing test was newly
disabled. The committed-overflow resource probe passed separately at the preceding
checkpoint; it is not a fresh final-source performance measurement. Mixed aging
is an explicit failed qualification, not hidden by the ordinary suite.

## Space and resource observations

All 16 modes pass 40 small metadata cycles, stabilizing at 720,896 bytes from
458,752 bytes. The 6,144-record, 1,024-byte-value case starts at 10,813,440 bytes,
finishes its first rewrite at 13,565,952 bytes, and stabilizes at 26,542,080 bytes
over eight additional cycles. This is real retained growth, not compaction or
100/1,000-cycle whole-format qualification. Tests also exercise actual preparation
overflow rather than only fitting all reservations inline.

The [serial existing-binary resource observation](resource.log) reruns the small
all-mode 40-cycle case: 3.88 seconds user CPU, 0.01 seconds system CPU, 10.13
seconds elapsed and 11,000 KB process peak RSS. Setup, mutation and reopen checks
are included. HMB Flutter tests were concurrently active: host isolation is not
established. These are descriptive observations, not paired speedup evidence,
incremental-above-empty RSS qualification, or revised ZIP-read results.

## Failed experiments and blockers

This is the transition checkpoint's result, before the separately recorded
[compact map-bank fix](../compact-map-bank-2026-10-02/README.md), which subsequently
passes the unchanged aging stability gate with a higher retained-size trade-off.
The rejected experiment below remains rejected and is not the passing change.

The C metadata-arena hold hypothesis attempted to preserve an aligned reusable
map span before other allocations consumed it. The unchanged 4,000-operation
qualification [failed](rejected-map-hold-aging.log): mode 3 grew from its
1,124,773-byte first-100-cycle high water to 1,137,688 bytes through five
2,583-byte additions at cycles 111, 161, 221, 331 and 341. The [trial patch](rejected-map-hold.patch)
was removed completely; `allocation_map.rs` has no working diff. Its separate
[source hash](rejected-map-hold-source.sha256) and logs preserve provenance.
Do not label the smaller growth a pass, change the assertion, or hide it with
periodic compaction. The September 27 control evidence remains authoritative for
that earlier implementation.

Both known native failures reproduce on this checkpoint with
`native-block-layout,external-source`:

- [Truncated-tail recovery](native-truncated-tail.log): zero recovered files
  versus three required.
- [Signed multiframe recovery](native-multiframe.log): two partial files versus
  one required; intact-neighbor recovery remains incorrect.

The native signed snapshot depends on normal open and a whole-content digest;
its commit root references TOC offsets rather than independently authenticated
membership children. Independent signed salvage therefore needs membership
integration, not weakened signatures or digest checks. No recovery assertions
were relaxed. Full A1/A2/A3/A4/A5 qualification remains open or failed as recorded
in the evaluation scorecard; the earlier ZIP timings describe earlier file-only
snapshots, not this writer.
