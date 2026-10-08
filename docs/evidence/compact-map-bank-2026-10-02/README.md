# Bounded unpadded allocation-map banks

Date: 2026-10-02. Local, uncommitted change on top of the
[shared-tree transition checkpoint](../shared-tree-transitions-2026-10-02/README.md).
The [source manifest](source.sha256) pins allocation_map.rs and its unchanged
allocation/aging tests; the preceding checkpoint pins the other changed sources.

## Change and safety boundary

For a provably single-node allocation map, use the existing authenticated index's
fixed-record encoded-size bound instead of reserving two full 64 KiB banks.
The arena retains 64 KiB separation between copies plus the required copy bytes.
ArenaWriter checks the primary capacity and consequently the mirrored copy's
capacity before writing. Padded maps still require their full region sizes;
multi-node maps keep the prior layout. The arena encoding and authenticated
ownership rules do not change. No compaction, workload substitution, page-size
search or assertion relaxation is involved.

This is distinct from the rejected early-allocation hold trial. Only
allocation_map.rs changes in this tranche (+19/-8 lines); tests are unchanged.

## Validation

From rust/, the economical test agent ran the unchanged mixed-aging test in
release mode with external-source and explicit --ignored --nocapture, then the
allocation_map:: tests, file_format:: regressions and strict library/test/bench
Clippy with external-source. See logs for exact executable and results.

- [Mixed aging](aging.log): one qualification passed, four modes times 1,000
  operations, preserving content, erasure, unchanged-repeat and size assertions.
- [Allocation tests](allocation-tests.log): 13 passed, one manual probe ignored;
  includes mutation/power-loss and metadata-repair checks.
- [Format regression](format-tests.log): 220 passed, six manual probes ignored.
- [Strict Clippy](clippy.log): passed.

## Space result and limits

| Mode | First-100-cycle high water | Completed size at cycle 1,000 |
| --- | ---: | ---: |
| 0 | 1,507,328 | 1,507,328 |
| 1 | 1,114,112 | 1,114,112 |
| 2 | 1,114,112 | 1,114,112 |
| 3 (unpadded) | 1,507,859 | 1,507,859 |

The unpadded historical control had a 1,245,184-byte early high water and grew
to 1,376,256 bytes at cycle 421. The new geometry passes the unchanged stability
assertion but retains MORE total storage than that old final size. Padded results
are unchanged. Do not describe this as improved total space efficiency or a full
A5 pass. No paired CPU/read/write performance comparison was performed.

Both native recovery failures remain separate blockers. This change does not
activate a format, alter integrity/padding defaults, or complete typed/public
record integration. No commit, formatting or production publication occurred.
