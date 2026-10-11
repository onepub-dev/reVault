# Native v4 security implementation

Status: security implementation and scoped integration checkpoint completed.
This is not release qualification. Performance remains paused
until a new user instruction resumes it. The original audit logs and reproduction
patch are historical evidence, not tests rerun to demonstrate vulnerable acceptance.

## Implemented boundaries

Native CommitRoot/CommitAuth V3 authenticate selected graph references and access
metadata. TOC, variable, form, free-space and redaction pages carry commitments to
canonical decoded pages and their physical extents. Commitments include archive
and mode, page identity/sequence, and ordered object kind/identity/length/payload.
Readers verify the exact returned page on both cache and storage paths. Writers
derive commitments from trusted staged state, retaining them through incremental
tree updates rather than rereading storage or scanning all content.

Symlinks bind their exact object contents. File descriptors bind frame contents;
native block range reads retain their existing authenticated descriptor checks.
Packed relocation verifies the objects being copied against live authenticated
references before erasure. Inventory, retirement and cleanup consume authenticated
extents instead of trusting fresh public page headers.

Control records use canonical raw single-object framing and exact physical size.
Every accepted noninitial archive requires its current root/auth pair and sequence.
Empty state also authenticates. Public directory contents are authenticated after
credential bootstrap. Directory parsing has a 1 MiB bound per selected copy and
Argon2 parameters are fixed; there is no separate total derivation-attempt budget.

Native recovery uses the selected authenticated root's reachable membership,
including an empty TOC. It refuses current-state restoration without that authority.
Bounded header repair must verify the original tag; scanned old records cannot
silently become current. Candidate detached-proof recovery is separately qualified.
Whole-archive rollback still requires external freshness state.

Password creation defaults to unsigned. Explicit owner signing remains supported,
and credential additions/removals preserve signing mode. New file read/write modes
separate access intent from signer presence while retaining legacy open semantics.
The facades propagate optional explicit access; existing content-key-only native
wrappers retain their previous credential scope.

## Migration provenance

The reproducible [exporter builder](../../../tools/build_experimental_v4_exporter.dart)
pins historical source `a7a9f4d85b0fa716bf9e402de5f29ff5a6f61e9b`, its toolchain and
resolved dependencies, including the archived local `zstd-complete` patch. The
standalone adapter adds only its own package to the historical dependency graph.
Both exporters use private source copies and emit verified migration artifacts;
the current reader never downgrades its integrity interpretation.

The public CLI experimental-v4 migration test passed. It creates historical Vault
and signed password archive state with compressed content and variables, stops the
historical session, migrates through explicit exporters in a separate current
session, checks persisted bytes and signing, and verifies originals remain unchanged.
Old-v4 no-op migration without the exporter refuses; current-format repeat is a
byte-preserving no-op. This validates the pinned checkpoint's encoding, not every
experimental v4 revision. Imported commitments protect future state; migration
cannot retroactively establish authenticity of already compromised source content.

## Validation record

Focused canonical page/object/cache, control framing, graph lifecycle, access,
relocation and recovery checks passed before application to the active worktree.
Current public CLI default tests passed: vaultless sender/recipient lifecycle,
password-profile recipient writes, and explicit owner-signed creation. Current
native binding release build and WASM source target check passed. Python file
lifecycle passed 6 tests; Node passed 3 with one read-only-mount test skipped;
Dart passed 5 with one skipped. The supported format-3-to-current migration
integration passed, including retained content/metadata, owner authority and
interrupted import handling. This is not the full bidirectional released-0.4
interoperability matrix.

Java compilation and Javadoc passed with cached Gradle 9.3.1/JDK 26 targeting
Java release 22; the exact CI JDK 22 was unavailable. C++ compilation could not
run because CMake and FlatBuffers C++ headers were absent. PHP, Ruby, Lua, Go,
.NET and Swift runtime qualification was unavailable in this environment.
The checked-in WASM module was rebuilt from current runtime sources with the
matching `wasm-bindgen-cli` 0.2.128. Its portable password lifecycle passed: create,
wrong-password refusal, independent reopen, no-change commit, replacement,
removal, variable update and persisted byte verification, including disposal.
The initial runtime test exposed an unsupported `std::time::Instant` call in
elapsed-time instrumentation. Portable WASM now reports zero for those optional
metrics; native timing retains the same clock and behavior. The final generated
module SHA-256 is
`861b629299a68a3e343909e8258f0e89fb2115e9cf1d8aa93ac454b933a88331`.
Runtime sources and manifests were unchanged across its final build and test.
This local artifact check does not authorize package publication.
Distribution version declarations are not yet reconciled: this branch has
CLI/Dart `0.5.0-dev.1`, JS/WASM `0.2.0` and native-bindings Cargo `0.2.23`.
The regenerated WASM is an unreleased format-4 branch artifact, not a compatible
`0.2.x` release. Package-version and full interoperability qualification remain
release work.

The binding review checker initially reported 41 unmapped names. The two new Rust
file access/signing entrypoints now map deliberately to `lockbox_file`. Every other
reported name exists in pre-security HEAD; no blanket exclusions were added. This
baseline debt remains distinct from current facade runtime validation.

Strict native Clippy passed for the lockbox library with native block layout,
and for CLI/bindings including all targets. A separate WASM-target strict-Clippy
check reports four preexisting `drop_non_drop` diagnostics on `std::fs::File`
close-before-rename paths (`extraction.rs` and `files.rs`). The portable target's
File type has no destructor. These existing native filesystem paths were not
changed or suppressed to claim a clean WASM lint run; WASM build and actual
portable lifecycle validation passed independently.

The first broad native run was cancelled with exit 130 after `/tmp` quota failures;
it has no final suite result and is not a pass. Subsequent checks use a dedicated
healthy temporary directory. The healthy release-mode all-library attempt reached
its 30-minute bound with exit 124: observed output contained 303 passes, three
failures and 11 ignored tests, but no final summary. The three failures were two
directory-inclusive recovery count expectations and a test passing a padded
candidate directory envelope directly to the strict native payload decoder. Each
corrected test subsequently passed in an exact release-mode rerun.

That attempt completed the candidate form-definition and field-upgrade fault
campaigns and 72 credential-install, 96 path-install and 48 resume process-death
cases. It stopped during candidate form-deletion fault qualification. Exhaustive
candidate fault qualification remains incomplete; this is not an all-v4 suite pass.
The final broad native command was
`cargo test -p revault_lockbox_api --lib --release --features native-block-layout -- --test-threads=2 --nocapture --skip file_format::candidate_files`.
It excludes that historical candidate bulk suite, retaining targeted bootstrap/refusal and
access-export checks for the candidate boundaries changed by this repair.

That native integration command completed with 514 passed, seven failed, nine
ignored and 176 filtered out. The seven failures exposed obsolete codec fixtures
or default-policy assumptions: a mistyped endian fixture constant, three page
fixtures inconsistent with canonical control framing, a historical auth-version
fixture, an implicit-owner password fixture and TOC offsets predating its
authenticated routing envelope. Corrections preserve the production validators;
all seven exact follow-up release-mode tests passed (one test per filter). Their
logs are retained separately rather than rewriting the broad run as a pass.

Earlier exporter attempts that regenerated dependency
versions or used registry `zstd-complete` are superseded, not migration evidence.
The [validation epoch](validation/epoch.json) indexes 77 retained logs, including
failed, cancelled and superseded attempts. The
[source manifest](validation/tested-source-sha256.txt) and
[source snapshot](validation/tested-source.tar.gz) preserve the 81 changed source
files before commit-hook formatting, against the recorded base revision. They
describe the tested pre-format epoch, not final formatted file hashes. Historical
exporter provenance is retained alongside them. This checkpoint resolves the
observed native security failures and records migration and available binding
checks; exhaustive candidate fault qualification, unavailable platform runtimes,
baseline lint/API-review debt and release interoperability remain open. No
performance or release gate is closed by these functional results.
