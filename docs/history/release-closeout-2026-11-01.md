# Release closeout checkpoint — November 1, 2026

## Publication state

The original security-fix candidate `40125442175` passed at
`fde1df438cdbf3367f70600106299165ca13e161`. CLI and bindings 0.6.0 are public,
but the Ruby 0.6.0 gems were empty. CLI 0.6.1 was published separately.

Bindings 0.6.1 candidate `40147665599` passed at
`0cf906d9b555c0e1a560f58b0c20d7af0676ad62`. Promotion `41301091889` and its
publication run `41301155785` completed most registries but failed Rust
publication. Adding the bindings workflow as a trusted crates.io publisher
allowed `revault_lockbox_api` 0.0.59 to publish; the retry then stopped at
`revault_publish_protocol` 0.0.53. Preserve this partial immutable release.
Remaining Rust publication is superseded by the corrected 0.6.2 candidate.

All six public Ruby 0.6.1 gem digests match their frozen publication payloads.
The Linux gem installs and passes persisted-content checks on Ubuntu 24.04
(glibc 2.39), but fails on the repository's Debian 12 consumer (glibc 2.36).
No published artifact or tag was replaced.

The public C++ SDK and matching 0.6.1 source build and install successfully.
A separate installed consumer passes its core Lockbox, key, Vault, persistence,
and restore checks. The platform secret-store phase did not pass in the local
consumer environment; this is not a claim of full platform qualification.

## Integrated corrections

- #338 authenticates closed current-format archives during repeated migration.
  Its two focused public-CLI tests passed with an isolated build directory.
- #339 provides readable publication output with explicit JSON/key-value modes.
  The actual loopback publication E2E passed without a socket-related skip.
- #340 documents upgrading legacy clients to 0.6.1 after format-5 migration.
- #341 exposes independent persisted suspend controls in the existing config.
  Five new headless `lbx` E2Es, two existing headless E2Es, one configuration
  unit test, and the installed-binary test target passed. CI now uses installed
  `lbx` in the minimal Linux container.
- #209 adds sixteen language guides; the manual checker passed all 56 pages.
  GitBook synchronized revision `dLYHNtuA1kYYCL2iLnwO`.
- #343 builds both Linux architectures on Debian 12 and checks the exact frozen
  Ruby gem on Bookworm before promotion. Local x86-64 positive persistence and
  negative public-0.6.1 load checks passed. ARM64 remains a candidate-CI gate.
- #344 pins published CLI 0.6.0 for bidirectional archive/Vault compatibility,
  while retaining published 0.5.0 as a separate migration baseline. Both public
  binaries passed checksums and source attestations; both lifecycle tests and
  the release declaration unit test passed.
- The Unicode gate now accurately identifies its pinned reader as format 4 and
  verifies rejection of format 5, unchanged Vault bytes, and readable content.
- Active dependencies now use zip 9.0.5 and rustls 0.23.46. Release-tooling tests
  (84), protocol tests (33), and core all-target compilation passed. Retained
  historical/evidence lockfiles were not rewritten. PR #336 was closed because
  it proposed replacing a frozen historical exporter's reader dependency.

## Current candidate and remaining gates

Candidate `41305470747` was cancelled before publication to correct the
same-line compatibility gate. Its unpublished 0.6.2 version was reused for a
new immutable candidate, without moving a release tag.

Candidate `41306873278` qualifies bindings 0.6.2 from main revision
`9b0665e3c5dbcdd6ac717a23b20a07e7fa5acf39`. At this checkpoint it is running,
not qualified or published. Its ref is
`release-candidates/9b0665e3c5db-1793536261874982874`.

Before publication, require the successful sealed candidate, corrected
crates.io publisher configuration, and unchanged artifact provenance. After
publication, verify the public packages and update/publish the eight prepared
advisories. Their first-fixed Rust/CLI/binding versions remain the original
security-fix releases; do not replace these mappings with the newest patch.

The previously completed live two-party exchange smoke does not prove a
particular deployed server version or completion of pre-upgrade token/claim
mitigations. No production deployment was performed in this closeout work.

Seven merged issue branches (209, 217, 338–341, 343) and their clean worktrees
were removed. Exact temporary deletion exclusions were removed afterward;
the repository-wide deletion/non-fast-forward protections are active again.
