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
