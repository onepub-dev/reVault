# In-place typed inline/tree transitions

Date: 2026-10-02. Local uncommitted candidate following the
[typed-tree and salvage checkpoint](../typed-tree-2026-10-02/README.md).
The [nine-file manifest](source.sha256) overrides earlier checkpoint hashes only
for the listed paths. Earlier logs retain their source-specific scope.

## Connected transition

Automatic metadata routing retains the small dense representation when it fits,
grows to the authenticated tree when required, and permits return to inline.
Growth authenticates the dense base and gives alignment/staging bytes preparation
ownership. Recovery dispatches from selected publication to a complete dense or
tree state. Return preflights dense capacity, preserves exact payload/key claims,
and retires authenticated descendants explicitly before reclaiming the tail.

The existing narrow metadata-tail proof still refuses descendant retirement.
A separate tree-tail proof requires no replacement descendants, exact payload/key
ownership and explicit retirement of every old descendant. Its prior negative
test remains, with positive retirement and payload-substitution refusal coverage.
The shorter selected publication is durably mirrored before erasure/truncation.

This is experimental filesystem metadata integration, not payload mutation,
variables/forms, complete credentials, public native activation or format selection.

## Validation

- [Initial focused transition run](transition-tests.log): 7 passed, with 4,572
  transition fault cases. This preceded the final automatic-routing refinement.
- [Final routing test](routing-tests.log): 1 passed, all-mode small-dense retention.
- [Final format regression](format-tests.log): 228 passed, 6 manual probes ignored;
  includes the transition checks on final source.
- [Strict Clippy](clippy.log): passed.

Runs used release external-source library tests and strict external-source
library/test/bench Clippy. No existing recovery assertion was weakened. The
all-mode lifecycle and bounded interruption cases independently reopen selected
filesystem contents and preserve source payload bytes.

## Size and resource observation

The [serial resource run](resource.log) used the existing release test binary:

```text
file_format::candidate_files::tests::tree_tests::typed_tree_inline_growth_and_return_reclaim_tail_all_modes --nocapture --test-threads=1
```

All 16 modes reached an observed completed tree size of 1,114,112 bytes and
returned to exactly their initial inline size: 131,079 or 131,107 bytes unpadded,
196,608 bytes padded. The observed peak is over completed operations, NOT the
maximum temporary allocation during a transaction. Full transient accounting and
mixed payload aging are separate qualifications.

The process used 1.57 seconds user CPU, 0.06 seconds system CPU, 1.64 seconds
elapsed and 72,316 KB peak RSS, including setup and reopen checks. Host load was
2.68/2.26/2.04; HMB collectors and a lockbox agent were present, with no other
Cargo process reported. This single resource observation is not an isolated,
paired speedup measurement or incremental-above-empty RSS gate.

No formatting, commit, push or production deployment occurred. Both old native
recovery failures remain tied to the unactivated native membership protocol;
the earlier mixed-aging fix retains its documented total-space trade-off.
