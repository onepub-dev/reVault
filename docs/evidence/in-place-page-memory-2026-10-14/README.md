# In-place owned-page decryption — 2026-10-14

Status: 53 focused tests, all-target Clippy and the fixed two-mode resource screen pass. Full release qualification remains pending.
The [prior owned-page correction](../owned-page-memory-2026-10-14/README.md) passed
its plaintext 10,000-file screen with a conditional CLI-plus-helper bound of
93,937,664 bytes. The encrypted owner-signed case stopped at extraction: CLI RSS
alone reached 114,122,752 bytes, above the unchanged 100,000,000-byte limit. The
command succeeded but independent protected-output verification was not completed.
Cleanup, owned-root removal and source identity checks passed. That failed screen
is retained, and no statistical comparison or release pass is claimed.

Source review showed that allocating AEAD decryption still held the original
ciphertext while creating an entire plaintext body. Dropping ciphertext afterward
could not remove that overlap. The existing secure decoder already uses the same
library's detached in-place authentication primitive.

Owned normal-page input now promotes its entire allocation to a wiping owner
before mutation and authenticates/decrypts the stored body in place. It retains a
checked plaintext range excluding the authentication tag, without shrinking or
reallocating. The key derivation, nonce, associated data and error semantics remain
the same. Borrowed decoding and the secure-page path retain their existing behavior.
There is no format, policy, cache-limit, corpus or concurrency change.

Focused tests compare allocating/in-place authentication and failures for changed
ciphertext, tag, key, associated data, nonce and truncated input. Plaintext and
encrypted page tests prove exact original pointer and capacity retention across
padding modes. Existing postauthentication parser refusal and final-owner wiping
tests remain, with full-capacity failure wiping observed while storage is live.
Crypto 2, page 18, buffer 3, cache 20 and extraction 10 tests pass, as does all-target
Clippy with warnings denied. Input manifests match before and after validation.

[Sanitized results](results.json) preserve the exact prior failure and focused
scope. The fresh uninstrumented checkpoint `4bca4fa3f280` then passed the same
two-mode screen, once per mode, with 10,000 files, four workers and unchanged
100,000,000-byte admission. Both public lifecycles independently verified persisted
bytes and modes, stopped their owned helper and removed private state. Source and
binary identity checks passed afterward.

The plaintext CLI peak was 84,877,312 bytes and its conditional CLI-plus-helper
upper bound was 93,962,240 bytes. The encrypted owner-signed CLI peak was
87,568,384 bytes and its bound was 96,677,888 bytes. Setup and control commands are
included. These are bounded workload results, not a universal memory guarantee or
a statistical comparison. [Sanitized resource evidence](resource-screen.json)
records identities and scope. All earlier failed attempts remain unchanged.

An immutable replacement candidate, required platform/binding validation, a fresh
explicit mutation-fuzz run, and the remaining fixed comparison gates are still
required before publication.
