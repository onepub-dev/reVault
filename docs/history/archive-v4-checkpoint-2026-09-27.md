> Historical handoff recorded on 2026-09-27 and moved from AGENTS.md on
> 2026-10-09. Branch heads, worktree changes, test results and outstanding work
> below describe that checkpoint, not the current repository. Recheck them
> before use. Moving this record does not resume the paused implementation.

## Archive v4 resume checkpoint — 2026-09-27

This is a handoff, not a replacement for the project goals or implementation plan.
The goal is **paused**, not complete: the user requested stopping when Codex usage
remaining fell below 70%; it reached **69%**. This documentation request does not
resume implementation. On a later explicit resume, recheck the applicable usage
limit and actual worktree state before proceeding.

### Workspaces and authoritative documents

- Archive worktree: `/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance`.
  Branch: `issue-310-zip-read-performance`. Last committed checkpoint: **`4744b635`**
  (`Fix #310: connect shared overflow roots to authenticated traversal and ownership`).
  Continue this branch; do not start over in the main checkout.
- Headless/D-Bus issue #322 worktree:
  `/home/bsutton/git/.codex.workspaces/revault-issue-322-headless-cli`.
  Branch: `issue-322-headless-cli`, HEAD **`d8dfa6cf`**, clean when checked.
- `/home/bsutton/git/revault` has unrelated/user changes, including this pre-existing
  modified `AGENTS.md`, documentation, release/versioning work and hook files.
  Preserve them. The archive worktree's older `AGENTS.md` lacks some of the current
  main-checkout release and hook instructions; retain the contracts above.
- Read these files **in the archive worktree**, whose plan/evidence are ahead of
  the main checkout: `manual/project-goals.md` (canonical G1–G7),
  `docs/archive_v4_plan.md` (current implementation authority),
  `docs/archive_v4_evaluation.md` (acceptance gates and evidence),
  `docs/documentation_map.md`, and `docs/decisions/001*`, `002*`, `003*`.
  `docs/archive_v4_history.md` holds historical checkpoints. No v4 format has been
  selected; proposed decisions and component tests are not release qualification.

### Exact uncommitted work

Two files are modified in the archive worktree, and were deliberately left
uncommitted when the usage stop condition was reached:

- `rust/revault_lockbox_api/src/file_format/preparation_journal/compact/session.rs`
- `rust/revault_lockbox_api/src/file_format/preparation_journal/compact/tests.rs`

They add an opt-in `OverflowSession` around the existing inline executor:

1. Validate capacity/base/physical length, then durably publish an empty active
   inline preparation before appending the two separate overflow copies.
2. Synchronize and verify both overflow copies before publishing their stubs.
3. After caller-verified cleanup, publish **both** empty active stubs before the
   caller erases the old overflow arena; refuse direct finish while still linked.
4. Keep existing `InlineSession::open` refusing overflow, so existing dense-image
   recovery cannot accidentally run this new cleanup path.

This is a preparation protocol component, **not connected archive mutation or
committed-graph recovery**. Callers still must authenticate the selected graph,
validate reservations, and prove cleanup before unlinking. Stub mirroring only
checks overflow availability; it does not repair arbitrary arena addresses from
journal references, which alone cannot authorize writes. The tests' abort helper
uses explicitly described logical free-space fixtures, not a public CLI or a
production recovery implementation. Do not mistake it for that missing integration.

Verification completed before pausing:

- Four focused tests pass: **696** staging/unlink/abort power-loss cases plus
  **864** interrupted-recovery cases; capacity reaches **2,048 reservations**.
- Entire format suite: **206 passed, 5 explicit probes ignored**.
- Strict Clippy passes for library, tests and benches.
- Logs: `/tmp/revault-overflow-session-tests.log`,
  `/tmp/revault-overflow-session-format-tests.log`,
  `/tmp/revault-overflow-session-clippy.log`.
- No owned verification process remains running. These changes have **not** gone
  through the formatting hook or post-format verification; their new evidence has
  **not** yet been added to the plan/evidence documents.

When resuming, review this diff and its caller preconditions, retain the evidence
in the documentation, then commit through the tracked hook and rerun affected
checks after formatting. Hook path is `/home/bsutton/git/revault/.githooks`.
Run Cargo from the archive worktree's `rust/` directory. The completed commands were:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib overflow_session -- --nocapture
cargo test -p revault_lockbox_api --release --features external-source --lib file_format::
cargo clippy -p revault_lockbox_api --features external-source --lib --tests --benches -- -D warnings
```

### What is already implemented

The experimental shared-control layout now has authenticated publication and
credential bootstrap, physical ownership proofs, file/range reads, metadata COW,
resumable metadata-tail retirement, canonical directories/symlink targets/public
permissions, and node-aware salvage. Salvage preserves current membership and
refuses to resurrect deleted directories or stale targets after bank loss.

`d6f895fd` adds distinct descendant-metadata ownership and retirement proofs.
`4744b635` connects a private `RV4TRE01` root manifest to the existing authenticated
index, bounded traversal, allocation records and full physical ownership checks.
Its fixtures exceed inline capacity and pass all 16 archive modes, including loss
of either control bank or either copy of every metadata page. The reader exposes
**raw records**, not complete public file/variable/form semantics. Its committed
baseline passed 202 format tests and post-format tests/Clippy. See:

- `docs/evidence/shared-tree-reader-2026-09-27/README.md`
- `docs/evidence/shared-ownership-2026-09-27/README.md`
- `docs/evidence/typed-filesystem-metadata-2026-09-27/README.md`
- `docs/evidence/metadata-tail-retirement-2026-09-27/README.md`

### Remaining implementation and qualification

1. Connect overflow preparation to the selected authenticated ownership graph,
   the overflow writer and committed/aborted recovery. Keep arenas readable until
   unlinking is durable; validate every erasure/repair against ownership. Add
   interrupted recovery, process-death and independent region-loss coverage.
2. Implement inline-to-overflow growth, updates/replacements/deletions and return
   to inline, including descendant retirement and compaction/installation. Retain
   the small inline path. Current limits (64 KiB decoded dense catalogue, 1,024
   dense files/nodes, 4,096 overflow page pairs/allocation records and 8,192 graph
   claims) are experiment bounds, not accepted reductions in product capacity.
3. Integrate typed file payload mutation, public recursive directory and symlink
   operations, mirror ownership/adoption rules, normal/secret variables, forms
   and revision/reference validation, access mutation/overflow and public APIs.
   Public variables support 1 MiB values; the reused index permits 49,152 bytes
   per value. Use authenticated segmentation and preserve secure-page/scoped
   secret access; ordinary wipe-on-drop index values are not sufficient proof.
4. Qualify the **same complete implementation** for CPU, incremental RSS, archive
   size, lifecycle aging, recovery and compaction. Keep the plan's acceptance gates
   and retained controls; do not infer full-format success from component passes.
5. Resolve the remaining native recovery failures and dependency release blocker,
   complete CLI/binding/Vault compatibility and migration checks, finish #322
   cross-platform/release integration, and only then select/release a format.
   Follow the release compatibility contract above; no version bump or release
   readiness follows from this checkpoint.

### Evidence and unresolved failures to preserve

- Shared small plaintext reads measure **0.626× ZIP**, upper 95% **0.654**.
  Large plaintext reads still take **2.85–3.36× ZIP**; full A3/A4/A5 are unqualified.
  Raw profiles attribute about **92% of sampled user CPU to hardware SHA-256**.
  Do not return to catalogue/buffer micro-tuning or silently weaken integrity,
  change hash policy or change worker policy to obtain a benchmark pass.
- Both small archives finish 100 metadata edits at **320 KiB**, temporarily
  **448 KiB**. This passes a metadata-size subcase, not mixed payload aging.
- Retained C mixed aging preserves contents/erasure across 4,000 operations but
  the unpadded case grows at cycle **421** because of aligned free-space
  fragmentation. Do not hide that failure with compaction.
- Denser physical packing worsens whole-region damage locality: primary small
  fixtures lose 190 plaintext / 176 protected files versus C's 64; the highly
  compressible stress case loses 512 versus 64. One-byte corruption loses one
  file. This trade-off still requires an explicit decision.
- Native failures remain: `recovery_skips_truncated_tail_and_keeps_prior_intact_files`
  and `native_file_writer_commits_and_reopens_multiframe_files_in_all_modes`.
  Do not weaken their assertions or owner authentication to pass them.
- The vendored `zstd-complete` 0.2.0 Huffman fix is a release blocker: workspace
  patches are not inherited by published dependent crates. Resolve the published
  dependency/packaging route and verify an isolated package before release.
- Preserve retained `/tmp/revault-*` frozen binaries, corpora, profiles and logs;
  manifests/hashes and scope are recorded under `docs/evidence/`. Do not relabel
  earlier file-only measurements as current typed/overflow-format performance.

#322's separate branch uses direct async Secret Service/zbus calls with a total
five-second deadline, including bus authentication and prompts; locked auto-open
remains noninteractive. Linux private-bus, headless CLI and clean Debian install
checks passed without a normal CLI dependency on `libdbus-sys`. Actual macOS/Windows
credential-service testing, binding packages and release/CI integration remain.
Prior push/label operations were rejected by automatic approval review; no remote
publication is established by these local checkpoints. Revalidate authorization
rather than assuming the local commits were pushed or the issues closed.
