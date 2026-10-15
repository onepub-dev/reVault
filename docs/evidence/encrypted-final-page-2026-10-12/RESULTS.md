# Direct encrypted final-page assembly

The fixed trial passed all retention gates: 56 verified public write lifecycles,
including 48 measured and eight excluded preparation lifecycles, completed in
726.41 seconds. Each operation created, opened and added a pinned 1 GiB source
through the public CLI with ChaCha20-Poly1305 encryption and owner signing.
Fresh paired controls were used; the preceding descriptive baseline was not
reused as a control ledger.

| Corpus | Old → new median elapsed | Paired elapsed ratio, 95% interval | Old → new median RSS | Old → new maximum RSS |
| --- | ---: | ---: | ---: | ---: |
| Source/Zstd | 3.995 → 3.415 s | 0.86175 [0.85276, 0.87033] | 98,592,768 → 84,531,200 B | 108,277,760 → 96,727,040 B |
| Random | 8.34 → 5.06 s | 0.60393 [0.59915, 0.61023] | 78,630,912 → 78,456,832 B | 93,974,528 → 78,733,312 B |

Compressed elapsed improved about 14% and median RSS about 14%; random elapsed
improved about 40%, with median RSS essentially unchanged. Both round strata
also had elapsed upper bounds below one. Both maximum-RSS ratios stayed below
1.05, and compressed median RSS passed the prespecified 0.90 ratio threshold.
The maxima shown also include preparation runs. Candidate subprocess peaks were
below 100,000,000 bytes in these cases, but the metric excludes the persistent
agent and does not establish total process-tree, many-file or universal memory
acceptance. No new ZIP comparison was performed.

Elapsed is the sum of create, explicit open and add subprocesses. RSS is their
maximum peak, not their sum. Synthetic Vault initialization and agent startup
were excluded; per-command Vault/KDF and open costs remained included. Archive
drain was measured separately. Rounded drain values do not establish zero work
or crash durability. All samples passed independent public extraction, exact
bytes/tree/mode checks, cold-source admission, identity verification and cleanup.
Archive lengths were unchanged across variants: 183,224,704 bytes for source/Zstd
and 1,074,224,512 bytes for random input. Nonces and signatures differ naturally.

## Change and scope

Normal-security decoded-page flush now serializes a raw encrypted body directly
into the final zeroizing page allocation and encrypts its body slice in place.
It preserves key derivation, fresh random nonces, authenticated fields, tag size,
header checksum, padding, object framing and signing. Cleartext, compressed
metadata, secure single-object and native already-encoded paths are unchanged.
Checked extent calculations precede allocation; ownership remains zeroizing
through errors until successful transfer. No archive-format change is intended.

The baseline binary is `9ab29f4cf50054ed9c816ae6e1bc464b8ff053fb0a2e83775a2d8da5a85f9312`;
the measured candidate is `4142dd043db7ccd608f5eaccdf3d9673b76b5879fb0a729e2527207c0924d4d6`.
The original 786-entry tracked-Rust manifest was captured before and after the
candidate build. The baseline-matched 779-entry Rust plus root Cargo manifest/
lock audit was retrospective, with exactly two changed runtime files; additional
Cargo/toolchain/config identities were frozen separately. The old mutable Cargo
output was deliberately replaced by the candidate build; preserved reference
binaries and original build evidence remained pinned. Exact measured source is
saved as `.rs.txt` under `raw/measured-source` so formatting cannot change it.

## Qualification

All 39 corrected-source focused test executions passed: default/native page
filters (14 each), default/native page-buffer filters (three each), pressure
commit/abort and injected storage failure, native packed deletion, and two public
CLI lifecycle checks. Current-source portable WASM release generation passed,
followed by the password regression and setter lifecycle (invalid mode,
Bytes(8192), Auto, commit/free/reopen and byte equality). Only the tracked WASM
binary changed; JavaScript/types and build-source identities remained unchanged.
The generated binary SHA-256 is
`95c1e064ea9e1b28c4c8d655aaf8ae57ead69a82e3ff5726e7d0270148bd0533`.
Exact commands, executed lifecycle source and terminal logs are in `raw/wasm`.

The first page filter passed 13 tests and
failed the new differential test because the fixture used a padded physical
length that was not a multiple of 1,024 bytes. The full legacy byte comparison
had already passed before the decoder rejected that invalid extent. Only the
test fixture was corrected to a valid rounded extent; production and measured
runtime were unchanged. The original failure is preserved alongside the retry.
No additional benchmark samples were taken. Source identities remained unchanged
during the corrected validation matrix. No extra full suite is claimed for this
experiment; the preceding complete allocator integration predates this change.

Measured and qualified source hashes identify pre-format epochs. The tracked
commit hook may change formatting; those hashes are not claimed to identify the
formatted commit. Historical baseline preparation scripts, including failed
attempts, are preserved byte-for-byte as `.dart.txt` with a relocation manifest
in the original baseline's `raw/preparation` directory.
