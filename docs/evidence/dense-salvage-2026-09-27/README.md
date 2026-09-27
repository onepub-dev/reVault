# Dense file-image recovery and damage locality

Date: 2026-09-27. Based on `0165a70e`; the containing commit adds read-only
salvage to the experimental shared-control file image. No production format or
performance gate is selected by this result.

Salvage authenticates the selected publication and mirrored private catalogue.
It uses only that catalogue's current file membership and independently verifies
stored fragments, AEAD, bounded decompression and complete file hashes. Unrelated
preparation records, public wrappers and pack padding do not prevent recovery.
Normal open retains those checks, including eager signed-plaintext verification.
Both paths refuse external control roots that the fresh file-only ownership graph
cannot represent. This is not an external-metadata or mutation implementation.

A file with any damaged or missing fragment is incomplete: the sink must discard
its entire staged contents. An I/O, key, authority or sink failure invalidates the
entire recovery batch. Missing both authoritative catalogue copies is fatal before
output; salvage cannot invent a missing-file count or scan retired generations.
The caller holds a stable source snapshot/read lock. Recovery does not write.

## Validation

[172 format tests pass](format-tests.log), with five explicit probes ignored;
[strict Clippy passes](clippy.log). Internal tests are necessary because there is
no public CLI writer for this experimental profile. New cases cover eight
protection/compression modes, isolated fragment corruption, damaged padding and
both preparation stubs, loss of both catalogues, truncated payload, whole-file
prefix discard, wrong credentials/owner, late I/O and sink errors. Source bytes
remain unchanged. Explicit-key-only images correctly reject password bootstrap
when they contain no wrapped key slot.

The [damage-locality stress case](region-loss.log) uses 512 highly compressible
4 KiB files with constant-byte contents. It is distinct from the retained primary
size corpus. Destroying the first 64 KiB payload region makes 64 files incomplete
in C (256 KiB logical contents), but 512 files incomplete in the dense image
(2 MiB). Changing one stored byte makes just one dense file incomplete; the other
511 files recover with exact bytes. Independent fragment protection limits
isolated corruption; it does not limit the logical loss from a whole physical
region. Neither layout replicates file payload.

This is an explicit size/recovery trade-off requiring a format decision. The
per-fragment decode bound cannot stand in for a damage-locality bound. Retained
primary corpus comparisons are recorded separately after freezing the binary.
Mutation, process-death qualification, larger metadata, recovery throughput and
the complete resource matrix remain outstanding.
