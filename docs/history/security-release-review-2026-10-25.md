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
| GHSA-6wg9-rg68-69v6 | revault_lockbox_api, revault_vault_api | 0.0.49, 0.0.50 | 0.0.50, 0.0.51 |
| GHSA-hm5c-7xh9-3wgc | revault_cli | 0.5.1 | 0.6.0 |
| GHSA-xww5-725p-f6hc | revault_vault_api | 0.0.50 | 0.0.51 |
| GHSA-48j6-c3j4-83m3 | revault_vault_api | 0.0.50 | 0.0.51 |
| GHSA-7jxm-8453-75wf | revault_cli, revault_lockbox_api | 0.5.1, 0.0.49 | 0.6.0, 0.0.50 |
| GHSA-rgp7-g6f6-jx3f | revault_publish_protocol | 0.0.43 | 0.0.44 |
| GHSA-gx4w-7f7m-jr43 | revault_key_server | 0.0.45 | 0.0.46 |
| GHSA-xcpq-38j3-f3wj | revault_key_server | 0.0.45 | 0.0.46 |

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

Validate packaged format-4 migration before publishing the new exporter. Then
install exact published exporter versions and exercise their public CLI migration
lifecycles. Prepare and seal the integrated candidate from main, including
cross-platform, installed-package and compatibility checks. Verify final package
ranges, publish immutable artifacts in dependency order, verify registries, and
then update and disclose the eight advisories. Record actual candidate and
publication results rather than converting local test success into release
readiness.
