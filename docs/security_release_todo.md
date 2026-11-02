# Security release TODO

Status on 2026-11-02: CLI 0.6.0 and 0.6.1 and most binding packages are public.
The first fixed Rust versions in the advisory mappings are also public and
not yanked. All eight advisories remain private drafts pending coordinated
closeout. Ruby 0.6.0 gems were empty; public 0.6.1 gems contain the validated
payloads, but Linux artifacts require glibc 2.39 and fail on Debian 12.
Bindings 0.6.2 is being qualified to restore the Debian 12 baseline and update
active zip/rustls dependencies. Rust 0.6.1 publication stopped at a missing
crates.io trusted-publisher entry; the final corrected publication will use
0.6.2, preserving the partial 0.6.1 record. The operator has configured the
new consolidated crates.io publisher. Candidate `41361552039` failed qualification; #349 repairs its identified
prerequisite, lint and CLI coverage failures. Candidate `41392531422` was cancelled after an API-review manifest omission;
its native Linux jobs and strict Clippy passed. The omission is corrected.
Candidate `41393746205` was cancelled after the Android 16 KiB Flutter
acceptance job timed out. #350 tracks diagnosis; focused non-publishing run
`41400976269` passed, but replacement candidate `41401952839` again stalled
after Flutter APK installation. Corrected combined diagnostic `41409573929`
passed twice on fresh runners. #350 remains open because the original launch
stall is not conclusively explained. A fresh successful full candidate is
required before publication.
Advisory release guidance is updated but all eight advisories remain private.
See the [November 2 checkpoint](history/release-closeout-2026-11-02.md).

Historical status on 2026-10-25: eight findings from the October security review had
private draft GitHub security advisories. The fixes are committed on
`security/advisory-fixes-2026-10` at `8e93aaebebf3517bdc3a5282e4bceda6dff771c1`.
They are integrated into local `main`, with additional Profile backup hardening
and regression coverage at `458d077c`. Integrated main was pushed at `ec0d9030`
to start the authorized release pipeline. Corrected migration helpers
`revault_migrate_v4` 0.1.1 and
`revault_migrate_v2` 0.1.1 are published;
the fixed CLI/bindings and private
advisories are not yet published. The affected
Rust packages passed offline locked `cargo check --all-targets` and Clippy with
warnings denied; those checks do not establish release readiness. The private
advisories are the source of truth for finding details. Keep them private until
coordinated disclosure.
The [local 0.6.0 integration checkpoint](history/release-0.6.0-integration-2026-10-24.md)
records checks completed so far and the remaining release gates.

Use the normal [release process](release_process.md) and the compatibility
contract in [AGENTS.md](../AGENTS.md). This list tracks the additional security
work for that release. Publication and production deployment require explicit
authorization. The user authorized package publication and coordinated advisory
disclosure on October 25. Production deployment still requires separate
authorization. See the [October 25 review](history/security-release-review-2026-10-25.md).

## Before package publication

- [x] Review each fix and its private advisory together. Confirm that the
  impact, severity, affected versions, and proposed mitigation still match the
  code. Check whether each published crate, CLI, or binding actually exposes
  the affected behavior; do not mark an unaffected package as vulnerable.
- [x] Merge the intended fixes into local `main`.
- [x] Prepare and validate an exact release candidate from `main`, including
  focused security regressions where
  needed, installed package checks, and the applicable cross-language and
  within-line compatibility checks. Record the candidate revision and results.
- [x] Assign new, never-before-published versions to every affected public
  package. Keep the CLI and bindings in the correct archive/Vault compatibility
  line. Record, for each advisory, the last affected version and first patched
  version of every relevant ecosystem package.
- [x] Prepare security entries in the relevant package changelogs and release
  notes. Give users the upgrade path and link to the advisory once public;
  avoid publishing exploit details ahead of the coordinated release.
- [x] Review the key-server deployment plan separately. A patched crate does
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
