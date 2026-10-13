# Historical migration decoder remediation

Status: the historical decoder remediation passes its focused qualification. Nine decoder tests, three selected historical migration tests and all three ordinary retained-fixture tests pass; the retained fixture generator is intentionally ignored. All five packages pass all-target Clippy with warnings denied. All three lockfiles pass the clean official advisory audit. This is a tested source checkpoint, not a qualified replacement release candidate. No reVault release has been published.

The clean official RustSec database at `d8325cf4f578e2df4cc5b98a30e9f46b3b88e9f0` identified [RUSTSEC-2026-0344](https://rustsec.org/advisories/RUSTSEC-2026-0344.html) in ruzstd 0.8.3. The affected dependency is used by the historical format-1 migration exporters through published historical archive/Vault APIs. It is not the decoder used by the current format-4 core. The earlier audit against a modified local advisory database is retained as invalid qualification evidence; the official clean-database result takes precedence.

The fix preserves five separate historical implementations rather than consolidating versions or interpreting historical archives with the current API:

| Compatibility package | Published source |
| --- | --- |
| `revault_archive_v1_initial_reader` | `revault_lockbox_api 0.0.2` |
| `revault_archive_v1_legacy_reader` | `revault_lockbox_api 0.0.3` |
| `revault_archive_v1_reader` | `revault_lockbox_api 0.0.6` |
| `revault_vault_v1_reader` | `revault_vault_api 0.0.2` |
| `revault_vault_v2_container_v1_reader` | `revault_vault_api 0.0.6` |

Each original `.crate` was checked against its checksum in the recorded Cargo lock before extraction. The new package's `PROVENANCE.json` records that checksum. Both Vault source trees remain unchanged. Each archive source tree changes `compression.rs` for explicit window admission and three ordinary compatibility tests. The reader derived from 0.0.6 also changes one format string in `lockbox/mirrors.rs` to inline argument capture, with identical emitted text and control flow, to satisfy the required Clippy gate. Original licenses remain intact; original READMEs follow an explicit historical-package scope notice. Original external examples, tests and benchmarks are omitted from these dedicated reader packages. Explicit focused library tests and retained public migration tests supply the qualification; disabled automatic library tests are not counted as passed tests.

The decoder dependency becomes ruzstd 0.9.1, which supplies the upstream block-output bound. The old `decode_all_to_vec` interface already refuses output beyond its vector capacity; it was not an unbounded output-growth path. That interface and its concatenated-frame behavior are preserved. The explicit window cap is 100 × 1024 × 1024 bytes, matching the old decoder's reset limit and now also checked for the initial frame. Historical first-frame windows larger than this previously escaped that reset check and are now refused. This admission limit is neither a total-process memory limit nor evidence that migration stays below the product's decimal 100 MB target.

The three tests per archive reader cover a normal 300 KB multi-block round trip and insufficient-output refusal; two concatenated ordinary frames and insufficient-output refusal; and a valid empty 1 KiB-window frame followed by refusal of the equivalent 128 MiB-window header. They do not allocate an oversized frame or execute vulnerability reproduction corpora.

Current routing replaces all affected production and development historical aliases. Publication ordering places compatibility archive readers before their Vault readers, and all five before the historical exporters. No historical fixtures are regenerated or changed. Supported retained export/import and owner-admission behavior must pass before the release candidate is replaced.

Installer bookkeeping failures were preserved: the first stopped on a package/lib manifest-name ambiguity before writing a manifest, and the next failed while attempting to move its partial source directory across filesystems. The completed installer moved that task-owned partial directory to ignored same-filesystem evidence, then installed from verified archives. Neither failed attempt executed a decoder or changed retained fixture bytes.

## Qualification and preserved failures

[Decoder results](focused-decoder-tests.json) record three passes per archive reader, nine total. The first run's two new fixture encodes requested level 3, which maps to an encoder mode explicitly unimplemented in both historical ruzstd 0.8.3 and patched 0.9.1; that [failure is preserved](initial-compiler-and-fixture-failure.json). Only the new test fixture encodes changed to supported level 1. Production historical encoding behavior was not changed. The initial Clippy invocation also had a command-line selection error; subsequent per-package checks found the single format-string warning. [Three original passes](initial-clippy.json) and [two corrected passes](corrected-clippy.json) close the five-package lint gate without allowances.

[Migration results](migration-tests.json) record the three selected format-1 migration lifecycles and the complete ordinary retained target (3 passed, 1 ignored generator). All 21 historical files retained their exact hashes. No fixture was regenerated. [Package inventory](package-inventory.json) confirms all required source, README, license and provenance files appear in each of the five Cargo package lists (80 files per archive reader; 23 per Vault reader). This is list-only qualification, not package build, registry consumer or publication evidence.

[The clean audit after remediation](clean-audit-after-remediation.json) records zero findings for workspace, facade and fuzz locks, with unchanged lock hashes and clean official DB revision. [Manifest reconciliation](audit-manifest-reconciliation.md) explains the different JSON versus sha256sum text manifest digests: all 1,328 path/hash pairs are identical, and both audit outputs contain 1,309 advisories. The earlier count of 1,308 was a reporting error, not changed database content.

The tests ran on uncommitted changes over candidate 7cb57d5; source snapshots and lock epochs are explicit in each result. The tracked commit hook may format copied source; final post-hook provenance must distinguish those formatting deltas before claiming byte identity. The replacement immutable candidate still needs applicable platform/package gates and final resource/comparison qualification. New compatibility crates must be published in dependency order before their exporters. No old audit-failed candidate may be promoted.
