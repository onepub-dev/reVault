# Release process

The CLI and public language bindings are released from validated commits on
`main`. Feature branches are development workspaces, not release sources.
Publishing requires explicit authorization. Never replace an already-public
version, tag, or artifact with different code.

An immutable candidate reference may retain an exact commit already pushed to
`main`. It must not contain separate, unmerged release changes. Before tagging
or promoting, verify that the recorded candidate commit belongs to remote
`main`; the branch may have advanced since the candidate was prepared.

## Versions and packages

The public compatibility line is `0.minor` before 1.0 and the major version from
1.0 onward. CLI and binding patches may advance independently within a line.
`0.4.x` identifies archive format 3; `0.5.x` identifies archive format 4.
Validate requested public versions against the declared archive and Vault line
before preparation and publication, including single-component releases.
Internal implementation package versions do not identify the archive format.

Historical packages are selected by the source archive/container version:
`revault_migrate_v1`, `revault_migrate_v2`, and `revault_migrate_v3`. Each contains
archive and Vault export commands and bundled historical reader code. Container
1 includes Vault structures 1 and 2; container 2 includes structure 2; container
3 includes structure 3. Preserve every supported combination and the original
source provenance and licenses.

Do not publish separate test-fixture or historical reader packages. Mark test
packages `publish = false`, use path-only development dependencies where
appropriate, and inspect normalized package manifests for unpublished runtime
dependencies. Historical exporter versions and dependency graphs remain frozen
unless their code or packaging actually changes.

## Check ownership

| Check | When and why |
| --- | --- |
| Source checks, workspace MSRV, focused regressions | Validate the integrated candidate. Run each identical suite once; retain separate facade workspaces where the workspace check does not cover them. |
| Core and CLI correctness | Required for each candidate. Installed-package jobs may own a suite when they execute the same tests against the actual distributed implementation. |
| Native and language package installation | Required for affected distributed packages and platforms. Validates packaging, loading, ABI, and executable conformance beyond source compilation. |
| Cross-language and within-line compatibility | Validate persisted archive/Vault interoperability, including older readers consuming newer output. |
| Historical migrations and recovery | Run each supported source path with packaged exporters. Reopen migrated or recovered artifacts through public commands and verify contents, credentials, metadata, backups, and source preservation. |
| Exact public exporter dependencies | Install the exact CLI-selected package versions from crates.io in an isolated environment and verify their capability contracts. Local builds and mocked installers are insufficient. |
| Focused benchmark/resource checks | Run when relevant storage, allocation, compression, concurrency, dependencies, or platform behavior changes. Verify output correctness outside timed measurements. |
| Full performance observation campaigns | Explicit qualification or performance work. Repeated ZIP/PGP sampling, bootstrap analyses, profiling and broad RSS campaigns are not routine requirements for every release. |

Installed-package and platform tests are not duplicates merely because they
exercise operations that also have source tests. They validate different
artifacts and environments. Conversely, do not run a local full suite and the
same candidate full suite solely as ceremony. Use focused local checks while
developing and one broad integration checkpoint.

Record which performance-sensitive paths changed and which focused observations
are warranted. Reuse historical observations only with their original scope,
revision, workload, and limitations stated. Preserve failed and incomplete
results. Skipping an unnecessary repeat is not a claim that a performance goal
has passed.

## Preparation and publication order

1. Merge and review the intended changes on `main`. Preserve unrelated local
   edits separately from the release revision. Validate the compatibility line
   and package inventory.
2. For new or changed historical exporters, validate their packaged migration
   lifecycles before publishing those dependencies from `main`. Then verify
   installation of the exact published versions. A new CLI must not become
   available before its migration dependencies can be installed.
3. Prepare the exact candidate, run its required checks, and retain its driver,
   packages, source identity, and artifact hashes. Dependency-only publication
   does not imply that the CLI or bindings are qualified.
4. Promote the successful candidate's immutable artifacts. Verify the retained
   source and artifact provenance; do not rebuild or repeat the preflight during
   promotion. A code change requires a new candidate.
5. Verify the registry versions and installed products. For a partial failure,
   retry only unpublished work or an identical artifact where the registry
   permits it. Never overwrite an already-public version.

One release operator owns publication and monitoring. Wait on one blocking CI
watcher rather than repeatedly polling or starting competing publication runs.

For dependency-only publication, dispatch the existing crates.io Trusted
Publishing workflow at the exact reviewed commit on `main`:

```bash
gh workflow run revault-cli-crates.yml --ref main \
  -f mode=bootstrap -f source_sha=<full-main-commit-sha> -f publish=true
```

This mode publishes only `revault_secure_memory`, `revault_migration_format`,
and the three historical exporter packages, in dependency order. It does not
publish the CLI or current API libraries. Omit `publish=true` for package dry
runs. Existing non-yanked versions are skipped, so a retry resumes after a
partial failure. The selected SHA must match the workflow's checked-out main
revision; a moved branch requires selecting the new reviewed revision.

Existing packages use their configured crates.io Trusted Publishing identity.
A new crate's first publication may require configured owner credentials before
Trusted Publishing can be registered for this workflow. If that first upload
is rejected, publish only the missing new package through the authorized owner
credential path, configure its trusted publisher, and retry the dependency
workflow. Keep existing packages' Trusted Publishing configuration intact.

## Retiring public crates

Publish and verify replacement packages first. Then publish an intentional final
metadata/documentation release of each retired crate from `main`, with a clear
deprecation notice in its description and README, the replacement package name,
and instructions for users. Keep these retirement publications separate from
the normal package list so test fixtures and retired crates cannot resume
routine publication.

The explicit retirement allowlist is `rust/retired-packages.json`. Keep the
fixture source manifests at their frozen versions with publication disabled.
Stage terminal releases separately from committed source, applying only the
reviewed terminal version and registry dependency normalization. Never change
the fixture dependency graph merely to publish a deprecation notice.

Run `.github/scripts/stage_retired_crates.dart` from `main` after committing the
reviewed retirement recipe and notices. It stages committed sources under
`/tmp` and writes provenance; it does not publish. Verify replacement registry
availability and each staged package before intentionally publishing the
allowlisted terminal versions.

After the new stable CLI release and all replacements are publicly available,
publish the allowlisted terminal versions through the same trusted workflow:

```bash
gh workflow run revault-cli-crates.yml --ref main \
  -f mode=retirement -f source_sha=<full-main-commit-sha> -f publish=true
```

The retirement mode checks replacement availability, stages committed source
with the helper above, and publishes only the terminal versions in the allowlist.
It requires the current stable CLI version in both GitHub Releases and crates.io.
Omit `publish=true` for dry runs; retrying skips terminal versions already public.

Crates.io does not display Cargo's maintenance-status badge; use the README and
description for the visible notice. See the
[Cargo manifest reference](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section).
Do not yank historical releases merely to simplify the registry listing: old
CLIs and frozen exporters may still resolve those exact dependencies.
