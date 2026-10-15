# Candidate file updates and read-only salvage

Candidate C now implements additions, replacements, removals and explicit salvage
as an internal file adapter. This is correctness evidence, not a production
integration, architecture selection or performance qualification. No public CLI
can create this experimental format; internal archive mutation/corruption tests
are necessary for that reason.

## Updates and source consistency

Updates preflight seekable source streams, hashing their bytes without collecting
whole files. Unchanged inputs and absent removals return the original storage
without starting preparation: bytes, journal and publication generation are
identical, even if a different candidate extent unit was requested. Duplicate
input paths and simultaneous removal/replacement of one path are refused.

Changed inputs are rewound, streamed into bounded independently protected packs,
and checked against preflight length and hash, including a final EOF read. Same-
length changes, growth, truncation and late source I/O failure explicitly abort
prepared allocations. This prevents a changed source from publishing a different
snapshot from the one selected in preflight. The current adapter receives a vector
of seekable readers; source-handle budgeting and public CLI source orchestration
are not yet integrated.

A replacement preserves logical object identity but replaces all its chunk
memberships. Any shared old pack is retired completely after copying and verifying
surviving fragments into replacement packs. Additions, replacements and removals
publish together. No deleted bytes remain inside a retained neighbour allocation.
Creation, normal open, update and salvage share a 32 MiB aggregate path-byte limit,
100,000 file limit and the existing one-million index-record limit. These are
explicit input limits, not proof of the proposed 256 MiB RSS budget.

## Read-only salvage

Salvage selects the newest authenticated publication, then walks only membership
proven by that selected index. It does not require unrelated allocation maps,
padding or payload to remain intact. One intact metadata mirror suffices. Missing
authenticated descendant pages are reported with their unavailable membership
count; losing both root copies fails closed. It never falls back to an older tree
or authorizes a file from scanned payload bytes.

Each recovered file must have its authenticated header, contiguous chunk ordinals,
matching object/offset/length descriptors, independently authenticated stored
fragments and matching whole-file hash. A damaged fragment invalidates that file,
not an intact neighbour sharing its physical pack. A chunk without its file header
is reported as an orphan, never emitted as a file. Missing membership counts are
not misrepresented as a known number of lost files.

The sink receives one file at a time. It must stage output, discard incomplete
files and retain the batch until the salvage call succeeds. Any I/O, key or sink
error invalidates the operation, including previously finished staged files.
Recovery is read-only and assumes stable storage under a reader lock/snapshot.
Signed-plaintext ordinary open still performs eager verification; salvage is an
explicit separate path, not a relaxation of normal-open behavior.

## Validation

Focused tests cover all 16 protection/codec/padding combinations; additions,
larger/smaller/empty replacements, removals, independent reopen and exact bytes;
source changes and failures after physical preparation; missing paths and true
no-change repeats; and conflicting inputs rejected without archive writes.

Every storage-mutation failure in four representative modes selects a complete
old/new file set and verifies ownership and physical reclamation. The update sweep
covers 174 failure cases, in addition to the existing 174 deletion cases. The
aging test runs 100 changed updates plus 100 no-change repeats in each of four
modes, checking neighbour bytes, reclaimed ranges and a stable high-water mark.

Fresh salvage tests cover shared-fragment damage in eight protection/codec modes,
primary-only and both-copy metadata loss across a 700-file tree, damaged padding
and allocation maps, truncated tail metadata, absent authorization, wrong keys,
sink failure and deleted/unpublished non-resurrection. Recovered bytes are checked
against independent expected content, and source bytes remain unchanged.

The complete default suite passes 450 unit tests, all executed integrations and
ten doctests (nine unit tests and one integration probe ignored). The format suite
passes 133 tests; a subsequently added partial-file/empty-file salvage regression
also passes. Strict all-targets native/external-source Clippy passes. Post-hook
validation passes all 134 format tests and strict Clippy; logs are retained. Lower-level power-loss and
process-death suites continue to cover the journal/allocator. This checkpoint does
not claim packed-file process-death, public CLI, complete public record/access,
compaction or release qualification. The two existing production-native recovery
failures remain unresolved. Next measure the declared packed/unpacked comparison,
100k-file resources and lifecycle costs; keep failed ZIP targets visible.
