# Shared-control physical ownership checkpoint

Date: 2026-09-27. Based on `07fdf6b8`. The containing commit adds physical graph
validation to the existing dense-image experiment. It does not activate a writer
or select a format.

The graph accounts for every byte through the authenticated sealed length. Each
control bank distinguishes publication/preparation, public key slots and private
catalogue slots. Live payload and mirrored roots cannot alias controls, each other
or free/pending intervals. Missing, overlapping, duplicate and overflowing claims
are refused. Unused public slots in fresh images are explicitly free and checked
for zero bytes during normal open; they are not hidden in a broad fixed prefix.
Read-only salvage still recovers authenticated files independently of those bytes.

The transition checker derives obligations from old and new complete graphs:
new writes must use previously vacant space or a durably reserved append;
retired live allocations must become pending; pending storage must be erased before
reuse; and changing pending to free requires erasure proof. In-place changes to
live metadata or payload are refused. Both publication copies must become durable
before any retirement. A shorter sealed length is refused: relocating metadata
inline does not itself make tail truncation safe.

The checker does not execute these operations or grant erase authority. Its
catalogue/extent inputs must already be authenticated by the selected root. Current
production integration is absent; the only reader integration is the test-only
fresh dense file image. Graph transition fixtures use logical authenticated-input
assumptions, not a persisted changing catalogue. The next step is to encode these
states in the catalogue and connect compact-journal publication/recovery, then
exercise real byte-level interruptions. Journal overflow and descendant metadata
pages remain unsupported rather than implicitly treated as fixed/unowned space.

The component remains bounded to 8,192 claims and one private/public root pair;
this is an experiment limit, not an accepted product capacity reduction. No CPU,
RSS, aging or release qualification follows from this checkpoint.


Validation: [177 format tests pass](format-tests.log), with five explicit probes
ignored, and [strict Clippy passes](clippy.log). Four graph tests cover complete
ownership, inline/external/inline obligations, early erasure and publication-link
refusals, and free/pending byte distinctions. A fifth test uses actual signed
plaintext and encrypted images: nonzero unused public slots fail normal open while
both files remain independently recoverable. Internal fixtures are necessary because
no public CLI writes the experimental shared-control profile.


## Descendant metadata ownership component

The graph now has an explicit descendant-metadata claim kind. A new
`derive_with_descendants` input accepts page pairs enumerated from an authenticated
selected catalogue traversal. It checks complete physical coverage, no overlap
with payload, control or vacant bytes, nonempty bounded pages, checked offsets,
and copies wholly contained in different 64 KiB failure regions. Duplicate page
ownership is refused even when both references have the same digest.

Descendant pages participate in the existing COW proof: changed live bytes cannot
be rewritten in place, retirement must become pending after publication, and early
freeing is refused. The narrow private-root tail-retirement proof requires all
descendant claims to remain unchanged; it cannot discard descendant pages to
shorten the file. This preserves that proof's existing scope while overflow is
being integrated.

This component does not authenticate caller-supplied references or traverse a
wire catalogue. The caller must provide the selected authenticated traversal and
verify page contents; journal reservations and durable publication still control
writes and erasure. Current image codecs continue to call the empty-descendant
path. The overflow reader/writer, journal overflow and segmented public records
remain unimplemented. The 8,192-claim experimental bound is unchanged.

The internal tests use explicit graph fixtures because no public CLI writes this
experimental layout. They check valid ownership, missing/duplicate/aliased claims,
same-region copies, boundary crossing, offset overflow, excessive/empty pages,
retirement obligations, early freeing, in-place substitution and attempted removal
through the private-root-only tail proof.

Validation: [focused descendant tests](descendant-tests.log), the
[199-test format regression suite](descendant-format-tests.log) (five explicit
probes ignored), and [strict Clippy](descendant-clippy.log) all pass.
