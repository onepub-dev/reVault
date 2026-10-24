# Security release review, October 25, 2026

This is a local preparation record, not evidence of publication. The user has
authorized package publication and coordinated disclosure. Production deployment
has not been authorized. Keep the eight draft advisories private until patched
packages are available.

## Integrated changes and validation

Full reciprocal exchange (#226) is integrated on main at `820a2686`; `7eb7db82`
records integration of its original branch history. Binding conformance fixes
and resolved Rust facade lockfiles are committed at `c7dedec0`. Linux checks
cover the language facades, native ABI and WebAssembly; TypeScript's installed
package reports all 240 operations, including exchange open/execute/free.
This does not replace candidate platform and installed-package qualification.

Security fixes from `8e93aaeb` are integrated. `458d077c` additionally streams
Profile exports into private temporary files and bounds Profile restoration to
8 MiB. Owned plaintext buffers are wiping buffers from allocation, including
partial-read, invalid UTF-8 and parse error paths. A failed export preserves a
previous backup.

Focused security tests pass for Vault sidecar symlink/hardlink and unsafe-parent
refusal, whole-Vault backup private permissions and output-link refusal,
malformed/truncated/overflowing/trailing backup input preserving the live Vault,
Profile backup/restore lifecycles, directory symlink exclusion, held host-file
import refusal, HTTP header/body bounds and slow-drip total deadlines.
Clippy with warnings denied passes for CLI, core, Vault and publish protocol,
including all targets. Current-format doctor and future-format refusal checks
pass after updating their stale format-4 fixtures to format 5 (`3001ccce`).

## Version review still to finalize

The planned public compatibility line is CLI/bindings 0.6.0 (archive format 5,
Vault structure 3). Candidate preparation must assign fresh internal package
versions and its actual output must be checked against registries.

| Private advisory | Direct Rust package exposure | Last published version reviewed | Expected first patched version, subject to candidate preparation |
| --- | --- | --- | --- |
| GHSA-6wg9-rg68-69v6 | revault_lockbox_api, revault_vault_api | 0.0.49, 0.0.50 | 0.0.57, 0.0.58 |
| GHSA-hm5c-7xh9-3wgc | revault_cli | 0.5.1 | 0.6.0 |
| GHSA-xww5-725p-f6hc | revault_vault_api | 0.0.50 | 0.0.58 |
| GHSA-48j6-c3j4-83m3 | revault_vault_api | 0.0.50 | 0.0.58 |
| GHSA-7jxm-8453-75wf | revault_cli, revault_lockbox_api | 0.5.1, 0.0.49 | 0.6.0, 0.0.57 |
| GHSA-rgp7-g6f6-jx3f | revault_publish_protocol | 0.0.43 | 0.0.51 |
| GHSA-gx4w-7f7m-jr43 | revault_key_server | 0.0.45 | 0.0.53 |
| GHSA-xcpq-38j3-f3wj | revault_key_server | 0.0.45 | 0.0.53 |

Downloaded crates.io 0.0.1 packages for these five crate names identify source
`0894fc1b00bbfd014427e523241d436506de0d64`. That source already contains the
Profile backup buffers, unbounded Vault backup read, permissive backup create,
path-following file import, sidecar open and plain HTTP read paths, and the
publisher verification URL. However, the first server has an operational
per-peer limiter. Commit `602ffd7c` removes that limiter and declares server
0.0.7. Downloaded published 0.0.6 contains `allow_anonymous_request` and its
per-peer limiter; published 0.0.7 omits it and calls `store.handle` without a peer.
The rate-limit advisory's affected range therefore starts at 0.0.7. Do not use
a blanket earliest-version range for all findings.

The foreign ABI exposes `vault_backup_default` and `vault_restore_default` from
the first binding release tag, `revault-api-v0.1.0`. Packaged native carriers
therefore expose the backup permissions and unbounded restore findings in
binding versions >=0.1.0, <0.6.0. Hosted WebAssembly delegates these calls to its
native Node host. Its default-directory Vault constructor makes the directory
private, so this is not evidence of exposure to the explicit-file sidecar
precondition. Published Vault 0.0.4 has no explicit-file constructors; 0.0.5
has them. The direct Vault sidecar range starts at 0.0.5; the public core
Vault-scope lock primitive is present from 0.0.1.

Conversely, most foreign file imports read caller-side bytes rather
than calling Rust `add_file_from_path`; do not infer exposure from a similar
method name. Registry queries on October 25 report CLI and Rust/npm/Python/
Ruby/NuGet/Go/Swift/PHP/Java/Kotlin bindings at 0.5.1 and Dart at 0.5.2.
LuaRocks and platform carrier registry status remain to be verified. Final
ecosystem ranges and registry versions remain a release gate.

For the two whole-Vault backup advisories, use the native package names from
`bindings/release/registries.tsv`: npm `@onepub-dev/revault-api` (and hosted
`@onepub-dev/revault-api-wasm`), PyPI `revault-api`, Maven
`dev.onepub:revault-api` and `dev.onepub:revault-api-kotlin`, NuGet `Revault.Api`,
pub `revault_api`, RubyGems `revault_api`, Composer `onepub/revault-api`, and
Go `github.com/onepub-dev/revault-api`. Native archives, Lua rocks and Swift
packages also require replacement even where an advisory ecosystem cannot
represent their distribution. Rust consumers receive the underlying crate
advisories through their dependency graph.

## Key-server operational response

Package publication alone does not patch running servers. Before an authorized
deployment, inventory every active relay, its version, proxy configuration,
replication peers, watchdogs and configured maximum receive lifetime. Preserve
access logs and state for incident investigation without copying bearer tokens
into release records.

Fence the old relay from clients and replication before promoting a replacement:
remove it from load balancers, block inbound public and replication traffic,
stop its service and disable automatic restarts, or power it off through the
infrastructure control plane. Confirm the fence from outside that host. The
operator-facing `fencing-help` command documents these options; a health-check
failure alone does not prove fencing.

Restarting every old process clears pending verification tokens: the token map
is memory-only and initialized empty in `PublishStore::open`. Previously
verified publication entries are persisted and replicated. Restarting does not
invalidate their verification status. Treat pre-upgrade verification claims as
untrusted and require independent fingerprint verification. Arrange authenticated
revocation and reissue where deletion credentials are available, or keep affected
entries inaccessible until their recorded expiration has passed. Use actual
record expirations/configured limits, not the default two-hour lifetime. Do not
delete arbitrary store files or reset replication counters to clear claims.

After deploying the patched version, verify public responses contain no email
verification link, ordinary email delivery still works, and per-peer limits
produce refusals. The application uses socket peer addresses, not arbitrary
forwarding headers; a reverse proxy needs its own client-aware limits. Notify
affected operators/users with the upgrade and independent-verification guidance
after disclosure. No production actions have been performed by this review.

## Remaining gates

Candidate `40100479912` at `30100cd0` was cancelled during its driver build:
review found that #226's new Vault-to-protocol dependency required publishing
the protocol before Vault. The corrected order passes a new regression against
normal, build and target-specific workspace dependencies and tooling Clippy.
No candidate package or release tag was published. The replacement preparation
advances internal package patch versions again; unused candidate versions are
not public releases and must not be named as the first patched versions.

Replacement `40101739845` at `a5350ec3` was also cancelled during its driver
build after finding the same new dependency missing from staged Rust binding
sources. Assembly, package-test copies and registry patches, foundation
publication, and bindings-only version selection now share one ordered source
package list. A regression checks the facade and transitive normal/build/target
path dependencies and publication order. Both focused tooling tests and tooling
Clippy pass. Neither cancelled candidate reached package publication or tagging.

An isolated staged Rust facade resolves all four dependency crates from the
staging tree with `cargo check --offline --locked`. Packaging with candidate
path patches, extracting the generated crate and running its public Rust
consumer passed archive/Vault creation, reopening and self-interoperability.
The staged core suite passed nine cases initially; its remaining recovery case
passed after correcting an obsolete format-4 magic assertion to format 5.
The staged Vault suite passed 29/29 with private fixtures. Its initial failure
was a test-created directory mode of 0775 under umask 0002, correctly refused
by the hardened lock code. Manual test fixture creation now sets Unix mode
0700 explicitly rather than relying on the process umask.
The affected standalone, read-only credential/process-lock, unversioned-Vault
and key-import tests then passed with the ambient umask left at 0002. The
production unsafe-directory rejection was not relaxed.

Candidate `40104070983` at `0a8fae7a` failed the actual registry-exporter
migration gate before platform jobs or publication. Five migration cases passed;
two successful migrations had stale format-4 backup-name assertions, and damaged
format-2 recovery exposed a real dependency gap: published `revault_migrate_v2`
0.1.0 lacks recovery-directory protocol 1. The implementation introduced in
`224ad22b` had never received a new exporter package version. The correction
publishes that implementation as 0.1.1 and pins both format-2 exporter binaries
to it. Capability validation explicitly requires the recovery-directory protocol.
Test exporter caches now derive their versions from the same release registry.
Backup assertions derive their target archive format from the current constant;
Vault structure-version expectations remain unchanged.

Package comparison found incidental `toml_edit` dependency drift. The workspace
lock restores its published historical version, 0.25.13+spec-1.1.0, so the v2
exporter's frozen dependency graph is preserved. The new package must pass
isolated package qualification before publication, followed by the mandatory
exact-version crates.io migration gate in the replacement candidate.

Isolated v2 0.1.1 qualification passed. The packaged manifest and lockfile
differ from the published 0.1.0 package only in the exporter version. Installing
the extracted crate with `cargo install --offline --locked` produced both
binaries with the required capabilities. Public CLI damaged-format-2 recovery
and closed-format-2 migration tests passed against those installed binaries,
checking original credentials, persisted content and backups. The bulk migration
repeat/failure lifecycle, exact capability contract and CLI/tooling Clippy also
passed. Qualification artifacts are under `/tmp/revault-migrate-v2-*`.

Support workflow `40106435018` passed from main `571a6d65` and published
`revault_migrate_v2` 0.1.1. The crates.io API confirms it is non-yanked with
checksum `73c8b077141c8688b32afbc882fd33426e142339f2a16eba1eb5418b5c3db375`.
Existing support package versions were skipped. The next candidate must install
this exact public version and exercise migration before CLI publication.

Candidate `40106736673` at `a438d974` passed the exact registry dependency
and public CLI migration gate, then exposed platform/source-check failures.
The format-4 helper's non-Unix lock path referred to an unqualified
`OpenOptions`; Windows compilation requires `std::fs::OpenOptions`. This is
packaged as v4 helper 0.1.1, without overwriting published 0.1.0. Its isolated
package manifest and lockfile differ from the published package only in its
own version; Windows GNU target checking, installed capabilities and the
public CLI migration compatibility test pass. The workspace lock returns to
the v4 package's original `toml_edit` 0.25.15 for its publication. Published v2
0.1.1 retains its own immutable 0.25.13 dependency lock; the candidate registry
gate installs each exact published package with its lockfile.

The server protocol test had incorrectly treated the topology tail as publish
payload. It now decodes the envelope and explicitly asserts that no verification
URL is returned. That regression and server Clippy pass. Source checks also
identified one Rust line-wrap and two Dart formatting corrections; these do not
change runtime behavior. Remaining matrix results must be collected before
the replacement candidate is prepared.

Commit `e2e6e1e4` includes those corrections. The exact required-package Rust
formatting check, Dart 3.10-language formatting check and CLI/tooling Clippy
pass. Support workflow `40108908315` published `revault_migrate_v4` 0.1.1 from
that main revision. The registry confirms non-yanked checksum
`f115a6432d24e1e45274c83e586541304852f490b2738db60dcaa30380a7774b`.
The downloaded package's VCS revision matches the source and its lockfile
differs from published v4 0.1.0 only in its own package version.

Installed WebAssembly conformance then detected a stale checked-in generated
binary: current sources included item-description operations, but the assembled
npm package rejected them. Candidate assembly now builds WebAssembly from the
exact source with the lockfile-matched generator, tests its complete hosted
operation inventory and persisted password lifecycle, and passes that artifact
to registry package assembly. Installed-package checks consume the same package
tree that publication freezes. Source-only WASM success did not establish that
the shipped binary was current.
The new build script and Dart analysis pass; the generated runtime operation
inventory/password lifecycle and installed npm conformance pass, covering 240
operations and loader modes. Validation used the cached Node runtime and an
isolated D-Bus session. Its disposable npm 12 consumer explicitly allowed the
required Koffi install script. The tested generated files refresh the tracked
development artifact; every candidate still rebuilds from its own source.

Linux CLI checks also exposed two remaining format-4 fixtures. The future-header
refusal test now uses the format-5 marker and a future container version, covering
past/current/future journal targets. The interrupted bulk migration fixture uses
the current archive format and the separate current Vault structure constant.
Both focused migration regressions and CLI Clippy pass after these corrections.

Candidate `40111659958` at `ff3ff649` passed the registry migration gate,
current-source WebAssembly build/package assembly and the installed Linux matrix
of 16 APIs and 480 interoperability paths. Source lint then found C# exchange
whitespace, corrected in `c12ad5dc`. A local sweep of all later lint stages found
and corrected two Python import-placement issues, one stale native-binding
future-format assertion, and the missing publish-protocol override in the
source Rust package check. These require another candidate revision.

The local sweep passed C# formatting/build without warnings, JavaScript syntax,
both strict TypeScript checks, Python Ruff, PHP/Ruby/Lua syntax and lint, Swift
build/test, and npm/NuGet/Composer/Ruby/LuaRocks/Python/Rust package validation.
File lifecycle checks passed for Python (6), PHP (18), Ruby (18), Lua (16),
JavaScript (3 with 1 skipped) and Dart (5 with 1 skipped). All 15 native-binding
tests are evidenced passing across the original run and two focused reruns:
the socket test required the normal IPC-enabled environment, and the future
fixture assertion now matches its explicit format-6 input and current support
constant. Bindings Clippy also passes. These local checks do not replace the
new exact-source candidate gate.

Candidate `40111659958` also exceeded the Linux CLI job's 60-minute limit:
the migration test binary started 24 tests but reported no completion before
cancellation. Every migration fixture invokes a nested Cargo exporter build;
the E2E driver concurrently starts twelve Cargo test shards sharing caches.
A bounded local run progressed through migration tests without reproducing a
deadlock, so the precise remote wait remains unproven. The driver now builds
exporters before starting concurrent shards and supplies Cargo's actual artifact
directory to the fixtures. Explicit registry-installed exporter overrides remain
unchanged. This removes the nested build from the concurrent test phase without
raising the timeout or weakening migration checks.

Candidate `40115768614` passed its CLI test binaries and reached the command
coverage gate, which found seven exchange commands absent from the run.
`exchange_cli` already exercises reciprocal exchange, both-direction archive
content, verification, cancellation and forgetting; it was missing from the
driver's shard list. The support shard now includes it, with explicit TTL option
coverage and the same bounded child-process capture as the other CLI tests.

Local qualification under umask `0002` also found that new standalone-Vault
parent directories inherited group-write permission and were then rejected by
the sidecar security check. The lock now creates missing parents with mode
`0700`; existing writable parents remain rejected without permission changes.
Standalone-Vault coverage verifies nested parent permissions and credential
persistence after reopening. The beget tests use private fixture roots while
leaving creation of their missing nested output directory to the public CLI.
These follow-up changes require a new exact-source candidate.

Follow-up validation passes under the local `0002` umask: the sidecar
security-boundary test, nested standalone-Vault creation/reopen, and both beget
CLI lifecycles. The full current CLI E2E contract passes with realistic coverage
of 137 commands, including rejection paths for 56. All seven xtask tests and
Clippy for xtask, core, Vault and CLI pass with warnings denied. The next
exact-source candidate remains required.

Candidate `40121959977` passed the corrected Linux CLI E2E gate. Meanwhile, the
retained full core unit run from `40106736673` completed after 3h44m with 731
passed, one failed and 20 ignored. The remaining failure was
`nested_vault_guard_retains_lock_after_outer_guard_drops`: it placed its Vault
sidecar directly under `/tmp`, which the security boundary correctly refuses.
The fixture now creates an owned `0700` directory, retaining both assertions
that the nested handle keeps the lock and that its final drop releases it.
This test-only correction requires another exact-source candidate; passing
checks from earlier revisions do not establish its release readiness.
All seven archive-lock unit tests and core Clippy pass after the correction.
The previously unreached core tail also passes locally: seven doc tests and all
14 integration targets except the public API suite owned by installed-package
qualification. The CI-only endian case remains ignored in this local run;
feature-specific external-source tests remain owned by their separate CI job.

The integrated #226 branch was deleted locally and remotely after checking both
tips were ancestors of remote main. Its clean worktree is retained detached.
Only its exact ref was temporarily excluded from deletion protection, and the
original ruleset was read back and verified restored. The browser integration,
phone approval and WASM-unlock design branches remain intentional holdbacks.

Main `ec0d9030fae8c7aa6db7e6e0facbd30495139bde` was pushed to begin the
authorized pipeline. Dependency workflow `40099928113` first failed because
Trusted Publishing cannot create a new crate. The existing Cargo owner
credential then published only `revault_migrate_v4` 0.1.0 from that same main.
Its registry checksum is
`9e5b3d9502f5e937e99116a3353055f9cb74faa8d62fd0fd00c4df4d9d10e993`;
it is non-yanked. Trusted Publishing was configured and read back for
`onepub-dev/reVault`, workflow `revault_cli-v-release.yml`, environment `release`.
The dependency workflow's second attempt passed, verifying all required versions
are present; existing immutable versions were skipped. All eight private advisory
drafts now carry reviewed affected ranges, reserved patched versions and the
focused regression evidence. Their reserved versions must match the final
candidate and be available before disclosure. The fixed CLI/bindings have not yet
been published and the advisories remain drafts.

Local packaged format-4 migration passed: `cargo package` produced the crate,
the extracted package was installed with `cargo install --offline --locked`,
and both binary capability contracts matched. The release-mode
`published_cli_and_candidate_read_and_update_each_others_archives_and_vaults`
test passed using a baseline CLI built from `revault_cli-v0.5.0` and the installed
package binaries. It verified preserved profile credentials, remembered archive,
payload, variable, exact original backup bytes and persisted subsequent edits.
Artifacts are under `/tmp/revault-v4-qualification-*`. This is package qualification,
not the required crates.io installation check.

The bootstrap script now includes v4, with a passing tooling regression requiring
every CLI exporter in that publication allowlist. Dart analysis also passes.
For these follow-up changes, bounded-reader and oversized-input regressions own
resource validation; no new throughput or RSS claim is made. Network lock-scope
and replication tests cover the affected exchange concurrency behavior. Candidate
platform correctness remains required.

Publish the qualified dependency, then install exact published exporter versions
and exercise their public CLI migration
lifecycles. Prepare and seal the integrated candidate from main, including
cross-platform, installed-package and compatibility checks. Verify final package
ranges, publish immutable artifacts in dependency order, verify registries, and
then update and disclose the eight advisories. Record actual candidate and
publication results rather than converting local test success into release
readiness.
