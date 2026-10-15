# Split-inline-slot feasibility result

Date: 2026-09-27. **Not adopted.** The prototype is preserved as a
[patch](prototype.patch) against `525fb831`, SHA-256 `2de7d06c4cb0c2d1799528c97b047556f131459fa44b64b05c0c6d548991a96f`.
It is excluded from the active implementation branch. The current writer retains
the tested bounded metadata protocol and its recorded size failure.

The variant divides each bank's 48 KiB private area into two 24 KiB slots for
smaller catalogues, with a full-slot/external fallback. It preserves the 64 KiB
decoded bound and adds no in-place overwrites. The prototype passes
[190 format tests](format-tests.log), five explicit probes ignored, and
[strict Clippy](clippy.log). Coverage includes both slot geometries, power loss,
recovery interruption, role/size bounds, multi-claim allocation, password bootstrap,
real-file reopen and catalogue growth/shrinkage.

That correctness result does not address the primary size objective. Retained
whole-catalogue encoding already measures:

| Retained 512 × 4 KiB corpus | Encoded catalogue bytes | 24 KiB physical slot |
| --- | --- | --- |
| [Plaintext](../whole-layout-cost-model-2026-09-27/small-plain.json) | 29,385 | Does not fit, even before envelope overhead |
| [Encrypted/signed](../whole-layout-cost-model-2026-09-27/small-protected.json) | 37,733 | Does not fit, even before envelope overhead |

Both would take the external fallback, retaining the prior 448 KiB mutable-size
result. No new primary-corpus measurement or performance improvement is claimed.
Further slot-size tuning cannot create room for two complete copies of these
catalogues in one 48 KiB private area.

## Next whole-layout comparison

Evaluate a paged private catalogue with copy-on-write at the affected page, not
at the whole catalogue. Budget the authenticated root, complete stored leaf pages,
publication/signature bytes, journal and public wrappers, free/pending state, and
temporary old/new pages together. Use retained real records and actual compression,
AEAD overhead and allocation rounding before implementing another writer.

A small encrypted root embedded in each publication is one hypothesis, not a
selected encoding. Owner publications currently store both hybrid public keys and
signatures; their space must be measured, not assumed available. Root ownership
must avoid self-referential commitments. New leaves must be durable before either
publication, and both publications must be durable before erasing retired leaves.
A delta/history overlay retaining old private names or secrets is not an acceptable
substitute for physical retirement. Larger metadata, typed public records and the
full recovery/performance matrix remain requirements.
