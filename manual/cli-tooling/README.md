# CLI tooling

The reVault command-line program is installed as both `lockbox` and `lbx`. They are the same program; this manual uses the shorter `lbx` form.

Most Lockbox commands put the file first and the action second:

```bash
lbx secrets.lbox create
lbx secrets.lbox add ./notes.txt
lbx secrets.lbox list
```

Vault-wide operations start with `lbx vault`, while open-session controls start with `lbx session`.

Start with the [Quick start guide](quick-start-guide.md). For help at any level, add `--help`:

```bash
lbx --help
lbx vault --help
lbx secrets.lbox variable --help
```

Add `--verbose` to expose advanced command forms and options that are normally kept out of the way.

