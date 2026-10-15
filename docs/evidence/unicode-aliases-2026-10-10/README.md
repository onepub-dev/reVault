# Unicode Vault alias validation

Issue: [#328](https://github.com/onepub-dev/reVault/issues/328).
Implementation base: performance branch `dc93b097`.

This change belongs to the format-4 development line. It keeps the existing
`/lockbox_aliases/*.lbla` collection, `LBLA` payload version 1, and Vault
structure version 3. It does not qualify the unfinished archive format for
release. Automatic alias creation is ported as a prerequisite.

## Naming and persistence

The shared Vault API pins ICU normalization and property data to 2.3.0
(Unicode 17.0.0), with the existing Lockbox path normalizer pinned to 0.1.25.
The [manual](../../../manual/protect-and-share/the-vault/lockbox-aliases.md)
specifies the grammar, NFC behavior, UTF-8 byte limit and collision policy.

Focused local checks passed for naming boundaries, writable/read-only persisted
records, alias replacement and removal, backup/restore, filename repair,
collision preservation, helpers and completion prefixes. These use synthetic
Vaults and Lockboxes; they do not operate on personal data.

After reconciling performance commit `f57a12ca` (compact decoded-leaf cache),
the combined source passed all 21 CLI alias tests, both naming tests, and the
persisted Unicode/legacy alias API test. The old-reader baseline was supplied
for that run. Uncommitted work in the performance worktree was not imported.
The subsequent committed performance checkpoint `4cf76ea7` was also reconciled;
all 21 CLI alias tests passed again, including the supplied format-3 baseline.

The old-reader check uses a separately built format-3 CLI from main commit
`77cfd919eddb0e9344b990283b2baf990108bca3`, not a simulated reader or a released
binary. It must refuse the actual format-4 Vault, report found version 4 and
supported version 3, explain that a supporting newer build is required, and
leave the complete Vault bytes unchanged. The current client then reads the
Unicode alias and its target value. CI supplies this baseline explicitly via
`REVAULT_ALIAS_BASELINE_BIN`; without that variable the optional local baseline
test does not run its assertions.

## Native shell checks

The executable Dart/DCLI driver `tools/unicode_shell_argv.dart` checks shell
token preservation for unquoted alias names, `--alias NAME`, and `a@NAME`. Each
required shell receives 100 arguments. Missing shells and changed argument bytes
fail the driver. The local run used Linux with `C.UTF-8`:

| Shell | Version | Shell token result |
| --- | --- | --- |
| Bash | 5.3.9 | Passed |
| Zsh | 5.9 | Passed |
| Fish | 4.2.1 | Passed |
| PowerShell | 7.6.6 | Passed |
| Elvish | 0.21.0 | Passed |

The driver uses shell functions or `printf`; native child-process argument
handling is checked separately by the Rust CLI tests below. The driver writes
OS, locale, versions, code points and expected/actual UTF-8
bytes to `target/unicode-shell-argv-results.json`. CI retains that report as an
artifact. Linux PowerShell results do not establish Windows behavior.

`rust/revault_cli/tests/unicode_shell_cli.rs` additionally exercises persisted
CLI operations and the completion protocol through native shells. It sources
the generated registration and invokes Bash's registered function, Fish's
`complete -C`, PowerShell's `TabExpansion2`, Zsh's registered provider, and
Elvish's actual editor completion registry. The Zsh probe captures the
`_describe` candidate handoff; it does not test interactive menu rendering.
The Elvish probe supplies isolated terminal input to initialize its editor.

All five local native CLI and registered-completion probes passed, including
composed and decomposed prefixes. Fish preserves the raw typed prefix in its
insertion text because its completion engine filters against that spelling.
The test checks both NFC-equivalent identity and the exact preserved prefix,
then uses the returned candidate unquoted in a new CLI invocation and compares
the retrieved content bytes. Other shells return the canonical candidate.

The workflow runs all five shells on Linux and macOS, and native
PowerShell/Elvish plus Git Bash on Windows. The [Windows job on `e5142d07`](https://github.com/onepub-dev/reVault/actions/runs/38038388985/job/114173534422)
passed both the native CLI/registered-completion matrix and the actual
format-3 reader refusal test. Its environment was Microsoft Windows
10.0.26100, Git Bash 5.3.15(2), PowerShell 7.6.6 with `en-US` culture and UTF-8
input/output, and native Elvish built from pinned v0.21.0. All three passed
composed/decomposed completion checks.

The [macOS job on `e5142d07`](https://github.com/onepub-dev/reVault/actions/runs/38038388985/job/114173534316)
passed all five native CLI/registered-completion probes and the format-3 reader
refusal check. It used macOS 26.6.2 on arm64, `en_US.UTF-8`, Bash 3.2.57(1),
Zsh 5.9, Fish 4.9.2, PowerShell 7.6.5 (en-US culture, UTF-8 input/output),
and native Elvish built from pinned v0.21.0.

The first Linux CI attempt stopped in the token driver because its default
PowerShell executable name was `powershell` rather than `pwsh`. The corrected
default was verified locally without an executable override (100 arguments).
The next Linux run passed token preservation but exposed an insecure ambient
Zsh completion directory: `compinit -D` aborted without an interactive terminal,
leaving no registered provider. The harness now uses `compinit -i -D` to ignore
insecure ambient directories, checks initialization and registration explicitly,
and reports stderr. A local reproduction with world-writable `/tmp` in `FPATH`
passed both strict NFC/NFD completion assertions after this fix; focused Clippy
also passed.

The final [Linux job on `e5142d07`](https://github.com/onepub-dev/reVault/actions/runs/38038388985/job/114173534217)
passed the token driver, all five native CLI/registered-completion probes
(including NFC/NFD Zsh completion), and the actual format-3 reader refusal
check. It used Ubuntu 24.04.5 with `C.UTF-8`, Bash 5.2.21, Zsh 5.9,
Fish 3.7.0, PowerShell 7.6.6 (UTF-8 input/output), and Elvish 0.21.0.
The complete [three-platform Unicode workflow](https://github.com/onepub-dev/reVault/actions/runs/38038388985)
passed on `e5142d07`, which includes performance checkpoint `4cf76ea7`.

## Broader branch checks

This is not an all-green format-4 CI claim. The broader Rust workflow reports
three failures also present on feature base `dc93b097`, confirmed against
[baseline run 38028150229](https://github.com/onepub-dev/reVault/actions/runs/38028150229):

- [Migration fixture comparison](https://github.com/onepub-dev/reVault/actions/runs/38028150229/job/114143300702):
  `every_retained_native_version_migrates_to_current`.
- [Recovery diagnosis](https://github.com/onepub-dev/reVault/actions/runs/38028150229/job/114143300728):
  `doctor_recover_detects_and_completes_interrupted_cleanup` expects
  `state: cleanup required`.
- [Windows completion setup](https://github.com/onepub-dev/reVault/actions/runs/38028150229/job/114143300820):
  the pre-existing Bash registration test invokes WSL without an installed
  distribution. The new native Windows matrix explicitly selects Git Bash.

These remain separate from Unicode alias validation and from eventual format-4
release qualification.
