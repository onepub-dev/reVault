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

The workflow runs all five shells on Linux and native PowerShell/Elvish plus
Git Bash on Windows. Windows results are pending completion of the CI run.
