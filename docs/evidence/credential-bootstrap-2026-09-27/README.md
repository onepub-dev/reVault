# Credential bootstrap component experiment

Date: 2026-09-27. Based on `ace45796`; the commit containing this evidence adds
[test-only bootstrap](../../../rust/revault_lockbox_api/src/file_format/publication_anchor/bootstrap.rs)
and its regression tests. This is not an activated archive format or public API.

## Result

A bounded public directory of existing password and hybrid Contact key wrappers
breaks the private-index bootstrap cycle. The directory contains wrapper IDs and
existing algorithm material, not private file names, access labels or owner
signing keys. Both directory copies are committed by publication digests and the
fixtures place copies in separate 64 KiB failure regions.

Signed mode verifies the pinned owner's publication before reading the directory.
Unsigned encrypted mode first inspects bounded, checksummed publication structure,
unwraps a candidate key, then authenticates publication with that key and requires
the authenticated commitment to match the initially selected generation. Neither
path reads private metadata during bootstrap. Tests subsequently use the returned
key to decrypt a real private index and compare its contents.

When one slot has a new key generation and its peer retains the previous one,
old-password unwrap failure cannot fall back to the previous generation. This is
not protection against an attacker replacing the entire archive with an old
snapshot, or destroying every surviving indication of a newer publication.
Unsigned bootstrap deliberately refuses a forged higher checksummed generation
whose MAC fails, even if an older slot can authenticate. This denial-of-service
tradeoff prevents credential failure from becoming permission to roll backward.

Signed bootstrap authenticates the owner-approved wrapper; private-object AEAD
still checks whether its unwrapped key actually decrypts those objects. Bootstrap
success therefore does not claim that damaged or truncated private contents are
readable. Ordinary I/O errors propagate instead of selecting another generation.
The caller must hold a stable source snapshot/read lock throughout the operation.

## Validation

From `rust/`, before formatting:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib publication_anchor
cargo test -p revault_lockbox_api --release --features external-source --lib file_format
cargo clippy -p revault_lockbox_api --all-targets --features external-source,native-block-layout -- -D warnings
```

- [Publication suite](publication-tests.log): 21 passed, including seven new
  bootstrap tests and retained publication process-death/torn-write checks.
- [Format suite](format-tests.log): 150 passed, five explicitly ignored probes.
  The separately recorded mixed-aging qualification remains failed; this run does
  not reclassify it.
- [Strict Clippy](clippy.log): passed across both archive backends.

New tests cover password/Contact access, wrong credentials/owner/slot, missing
owner pinning, separate private-index decryption, directory mirror loss, prepared
but unpublished credentials, rekeying with mixed publication generations,
malformed directory identity/generation/IDs/padding, overflow refusal, forged
unsigned publication, source preservation and injected read failures. A guarded
storage refuses any private read, write, truncate or sync during bootstrap and
checks public reads are no larger than 8 KiB. Internal fixtures are necessary
because no public CLI creates this experimental layout.

Initial test development exposed a fixture error (passing a signer for unsigned
publication); the fixture was corrected without relaxing production checks.

## Remaining integration

The 4 KiB directory explicitly refuses overflow and more than 48 slots; these are
component bounds, not a proposed reduction in supported access capacity. The
shared-control layout still needs authenticated overflow, placement/ownership,
credential mutation and rekey publication under the allocator/journal, secure
retirement, recovery and compaction. The current key tree remains unsuitable for
public wrappers. No archive size, performance, release or full-format gate is
passed by this component. Existing publication wire bytes and independent crypto
vectors are unchanged by factoring structural parsing from authentication.
