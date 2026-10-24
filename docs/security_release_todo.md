# Security release TODO

Status on 2026-10-24: eight findings from the October security review have
private draft GitHub security advisories. The fixes are committed on
`security/advisory-fixes-2026-10` at `8e93aaebebf3517bdc3a5282e4bceda6dff771c1`.
They are integrated into local `main` at `7667ab4f`, but that revision has not
been pushed, released, or disclosed. The affected
Rust packages passed offline locked `cargo check --all-targets` and Clippy with
warnings denied; those checks do not establish release readiness. The private
advisories are the source of truth for finding details. Keep them private until
coordinated disclosure.

Use the normal [release process](release_process.md) and the compatibility
contract in [AGENTS.md](../AGENTS.md). This list tracks the additional security
work for that release. Publication and production deployment require explicit
authorization.

## Before package publication

- [ ] Review each fix and its private advisory together. Confirm that the
  impact, severity, affected versions, and proposed mitigation still match the
  code. Check whether each published crate, CLI, or binding actually exposes
  the affected behavior; do not mark an unaffected package as vulnerable.
- [x] Merge the intended fixes into local `main`.
- [ ] Prepare and validate an exact release candidate from `main`, including
  focused security regressions where
  needed, installed package checks, and the applicable cross-language and
  within-line compatibility checks. Record the candidate revision and results.
- [ ] Assign new, never-before-published versions to every affected public
  package. Keep the CLI and bindings in the correct archive/Vault compatibility
  line. Record, for each advisory, the last affected version and first patched
  version of every relevant ecosystem package.
- [ ] Prepare security entries in the relevant package changelogs and release
  notes. Give users the upgrade path and link to the advisory once public;
  avoid publishing exploit details ahead of the coordinated release.
- [ ] Review the key-server deployment plan separately. A patched crate does
  not remediate a running server. Determine whether the private advisories call
  for credential or token invalidation or other incident response.

## Publication and disclosure

- [ ] With release authorization, publish the validated packages from `main`
  to their applicable registries and verify that the exact versions and
  artifacts are available. Deploy and verify the patched key server with
  separate production deployment authorization.
- [ ] Update all eight private GitHub advisories with accurate affected and
  patched package versions, impact, mitigations, and public release links.
  Include binding packages only where their published versions are affected.
- [ ] Publish the advisories promptly after the fixes are available. Check that
  the public records contain the correct package names and version ranges so
  dependency alerts can identify affected users. Request CVE identifiers only
  if useful; they are optional.
- [ ] Publish the prepared changelogs and release notes with links to the
  advisories. Make any necessary direct notice to users of the hosted key
  server or other affected deployments.
- [ ] After public disclosure, consider RustSec submissions for qualifying
  crates, using the published GHSA identifiers as aliases. RustSec submissions
  are public; do not submit them while an advisory is still embargoed.

## Closeout

- [ ] Verify installed versions from each relevant registry and the deployed
  server version. Check advisory package mappings and fixed-version links after
  GitHub processes the records. Record any failed or partial publication and
  resume only unpublished work; never overwrite a public version or tag.
- [ ] Mark each advisory resolved only when its applicable package or deployed
  service is actually patched and users have a public upgrade or mitigation
  path. Keep any unresolved exposure visible in the advisory.
