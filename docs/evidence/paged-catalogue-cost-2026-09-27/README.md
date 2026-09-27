# Paged private catalogue cost model

Date: 2026-09-27. Based on `2473beea`. This is a read-only feasibility comparison,
not a persisted encoding or a format decision. The prior split-slot variant was
[not adopted](../inline-slot-feasibility-2026-09-27/README.md) because its primary
catalogues cannot fit two complete generations in the private control area.

## Whole-layout hypothesis

Keep the two 64 KiB control banks and the existing preparation/public-wrapper
reservations. Store private file records and physical-pack descriptors in separate
leaf pages, with at most 32 records / 16 KiB decoded bytes per leaf. File records
preserve identities, names, permission bits, file hashes and every fragment's codec,
bounds and stored hash. Pack records preserve their bounds, digest and padding hash.
The model uses the retained source's real records and compression setting, actual
AEAD/nonce/tag bytes when encrypted, and 256-byte allocation rounding.

Budget a 2,016-byte encrypted/private root inside each existing 8 KiB publication,
starting at byte 6,144 and ending before its 32-byte checksum. Measure actual owner
public keys and hybrid signatures (or the MAC) from the authenticated retained
publication. They must fit before that root; spare space is not assumed. The root
contains typed child references/range keys, membership counts and free/pending
intervals. Large pack tables are leaves rather than copied into the embedded root.

Each leaf has matching copies in separate banks. The fresh model lays leaves
contiguously in each 48 KiB private area. For every file leaf, it measures a real
permission change and budgets coexistence of the complete old leaf and the new
page. The edited root also budgets both old-leaf pending intervals and both
remaining free intervals. Every measured leaf and root frame is independently
round-tripped through AEAD and bounded decompression. Source archives are read-only;
the resource probe separately compares their complete hashes before and after.

`inline_geometry_fits` means only that this fresh/one-edit byte budget fits the
private pools, embedded root and actual publication authentication region. It is
not whole-format eligibility, an aging result or a throughput measurement.

## Required protocol work if the budget fits

This placement requires a distinct experimental publication encoding: the existing
profile requires unused publication bytes to be zero. Root ciphertext is created
first and committed by the authenticated prefix, avoiding a self-referential hash.
New leaf copies must be durable before either new publication; both publications
must be durable before retiring the old leaf copies. Root replacement happens as
part of the already-mirrored publication, not an in-place private-page rewrite.

The model has no parser for a persisted page tree, allocator, crash recovery,
metadata overflow hierarchy, typed public records or migration path. It does not
qualify arbitrary renames, additions/removals, payload edits, fragmentation under
aging or a metadata-compaction/shrink protocol. Its source admission still inherits
the existing flat model's 1,024-file / 4,096-fragment and 64 KiB catalogue limits;
oversized individual file records need a separate fragment-index representation.
Those limits are not proposed product limits. No A3/A4/A5 pass is claimed.

Two [focused tests pass](tests.log): all 16 protection/compression/padding modes
preserve source bytes and budget actual publication authentication, and raw 8 MiB
plain/protected cases keep multi-page pack tables out of the small root. Retained
corpus measurements follow only after freezing the containing commit.

[Strict Clippy passes](clippy.log) for the frozen-source candidate.

## Frozen 32-record baseline and declared follow-up

[Five retained-corpus results](baseline-32/batch.json) use commit `951c43d3`,
executable SHA-256 `501ec0baad29607bcdf6ce86eef23e10f708ddd460b2421b69b1b6f87f0f51ec`.
All source hashes remain unchanged and all measured frames round-trip.

| Corpus | Live leaves/bank | Largest staged leaf | Peak/bank | Root maximum | Fits |
| --- | ---: | ---: | ---: | ---: | --- |
| Small plain | 49,408 | 3,072 | 52,480 | 824 | No |
| Small encrypted/signed | 49,408 | 3,072 | 52,480 | 840 | No |
| 8 MiB raw plain | 17,152 | 5,888 | 23,040 | 479 | Yes |
| 8 MiB compressed plain | 1,792 | 1,536 | 3,328 | 226 | Yes |
| 8 MiB compressed encrypted/signed | 1,792 | 1,536 | 3,328 | 242 | Yes |

The primary small corpus exceeds each 49,152-byte private pool by 3,328 bytes.
The embedded root fits, including the actual 5,379-byte hybrid authentication
block. This does not justify a writer. There is no small-input compression cutoff
in `encode_with_compression`; independent leaf compression and allocation rounding
must be measured together with the reserve needed to replace a leaf.

Before further implementation, compare the finite set of **16, 32, 64 and 128
records per leaf**, keeping the 16 KiB decoded bound, 256-byte allocation quantum,
all fields and hashes, protection settings and retained corpora unchanged. Measure
unrounded frame bytes and decoded bytes as well as total live/staged allocation.
This is a page-granularity comparison, not permission to relax size or security
requirements. Preserve this failed baseline regardless of the comparison outcome.

## Page-granularity result: do not implement this inline layout

The [frozen comparison](granularity/batch.json) uses commit `0ee379d0`, executable
SHA-256 `e494241ea2e93189d90ca1695ce1d42817f6def1031b11be46b5f4c811ba3a09`.
All five source archives remain byte-identical. The three focused tests and strict
Clippy pass again after the formatting hook (see `matrix-postformat-*.log`).

| Record cap | Small plain peak/bank | Small protected peak/bank | Plain live, before rounding | Protected live, before rounding |
| --- | ---: | ---: | ---: | ---: |
| 16 | 59,392 | 59,392 | 50,273 | 50,843 |
| 32 | 52,480 | 52,480 | 47,346 | 47,653 |
| 64 | 53,248 | 53,248 | 46,422 | 46,546 |
| 128 | 58,368 | 58,624 | 46,216 | 46,272 |

Every variant exceeds the 49,152-byte private pool on both primary small cases.
The larger-page compression saving is smaller than its increased replacement
reserve. The root fits in every case. All variants fit the three 8 MiB cases,
but that cannot compensate for a failing primary case. No persistent page writer
will be built around this failed inline coexistence budget.

The next experiment addresses **publication and reclaimable tail ownership**:
retain the full bounded catalogue, stage replacements externally, then copy the
selected catalogue back inline and publish a shorter sealed length before wiping
and truncating the retired external tail. This trades temporary write/space cost
for a bounded completed archive; those costs must be measured, not assumed fast.
A smaller published length is valid only after proving that the removed suffix
contains retired metadata/vacancies, all payload/key claims are unchanged, and
both replacement publications are durable before any suffix erasure. Interrupted
cleanup must resume without requiring erased old metadata or losing selected
publication authority. Do not simply truncate an archive whose selected anchor
still seals the tail. This is a distinct protocol hypothesis, not adoption.
