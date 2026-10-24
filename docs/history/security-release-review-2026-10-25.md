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
| GHSA-6wg9-rg68-69v6 | revault_lockbox_api, revault_vault_api | 0.0.49, 0.0.50 | 0.0.52, 0.0.53 |
| GHSA-hm5c-7xh9-3wgc | revault_cli | 0.5.1 | 0.6.0 |
| GHSA-xww5-725p-f6hc | revault_vault_api | 0.0.50 | 0.0.53 |
| GHSA-48j6-c3j4-83m3 | revault_vault_api | 0.0.50 | 0.0.53 |
| GHSA-7jxm-8453-75wf | revault_cli, revault_lockbox_api | 0.5.1, 0.0.49 | 0.6.0, 0.0.52 |
| GHSA-rgp7-g6f6-jx3f | revault_publish_protocol | 0.0.43 | 0.0.46 |
| GHSA-gx4w-7f7m-jr43 | revault_key_server | 0.0.45 | 0.0.48 |
| GHSA-xcpq-38j3-f3wj | revault_key_server | 0.0.45 | 0.0.48 |

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
