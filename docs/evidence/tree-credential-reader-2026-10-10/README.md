# Typed credential opening and dense conversion

The experimental typed-tree reader now opens password and hybrid-contact slots
directly. It requires the exact same authenticated publication before credential
unwrapping and after loading the typed catalogue. A valid successor using the same
key and records is still refused if a backend substitutes it during that operation.
The synthetic switching backend is read-only and both independent snapshots are
verified before testing the switch. Callers still need a stable snapshot/read lock;
this guard is not a replacement for that storage contract.

The first two focused tests failed because opening the typed catalogue needs
secure-memory allocations while the recovered key's global secure read guard was
active. The corrected reader ends that guard before decoding, retaining a bounded
32-byte zeroizing temporary for the duration of open, including failure paths.
The recovered key remains in secure memory, and persistent secret variable/form
values continue to use guarded storage. This is not a claim that all transient
cryptographic stack memory is locked. The initial failing log is retained.

Tests independently open all eight encrypted signing/compression/padding modes by
password and hybrid contact and read file bytes, filesystem metadata, normal and
65,537-byte secret variables, historical form values/labels and current definitions.
Unknown slots, wrong passwords and wrong owners are refused without changing the
source. Four encrypted mode combinations exercise the publication-switch refusal.

The dense-to-tree exporter also preserves bounded public slots. It authenticates
and decodes the source directory before destination writes, retains every wrapped
key and slot ID, and re-encodes the directory at generation one for the fresh output
lineage. Copying the old directory bytes verbatim would be incorrect when their
generation exceeds the new archive generation. Source lineage and bytes remain
unchanged; this export does not grant/revoke access or rotate content keys.

Conversion tests cover all eight encrypted modes, directories newer than generation
one, separate loss of either directory mirror, independent credential/content and
metadata reopen, and wrong-password/unknown-slot refusal. Losing both mirrors
refuses an empty output. Sixteen partial directory write failures (each mirror in
each mode) fail and truncate the owned output while preserving the source.

All checks use pinned Rust 1.88.0. Raw logs identify exact commands and outcomes.
This remains private experimental code: no public CLI writes this layout, so the
fixtures use internal writers and document the exceptional authenticated directory
generation manipulation. No public API, format selection or release is activated.

The 4 KiB directory/48-slot bounds, credential overflow and mutation, partial
temporary ownership, full API integration, two native recovery failures and the
complete CPU/RSS/aging/migration/interoperability gates remain open. The older
packed-C fresh exporter has a different access-root representation and still refuses
it; this conversion change applies only to authenticated shared dense sources.

Post-format checks at `5cd2064f` pass both typed credential tests, both dense
conversion tests (including 16 partial writes), and strict core Clippy. Earlier
controls also pass the single-traversal reader and all-mode filesystem lifecycle.

## Credential access after process death

The subsequent two-test process suite passes 72 child exits: eight encrypted
modes at six installation checkpoints and three complete-copy resume checkpoints.
The parent independently opens the surviving archive by both password and contact,
then checks every logical fixture family. Before rename the exact original survives;
after rename the direct successor survives with exact retained wrapper bytes.
Resume interrupted before rename can be retried and then matches the completed
candidate byte for byte. Strict Clippy passes.

Only synthetic public keys are handed to the child through files; the contact
private key remains in the parent process. The child arms its requested exit only
after fixture preparation, so preparatory compaction cannot consume the checkpoint.
This models abrupt process exit, not physical power loss, automatic temporary
ownership/cleanup, or platform-independent durability.
