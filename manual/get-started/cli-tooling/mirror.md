# Mirror

A mirror project maintains a one-way relationship between a host directory and a directory inside a Lockbox. The host directory is authoritative: an update adds new files, replaces changed files and removes managed files that disappeared from the selected host content.

{% hint style="warning" %}
Always inspect `mirror status` before an update. A mirror is synchronisation, not an append-only backup.
{% endhint %}

## Create a project

```bash
lbx backup.lbox mirror project create \
  --from ./project \
  --to /projects/project
```

`project` is the mirror's name. Creating it records the configuration but does not import files. Preview and apply the first update with:

```bash
lbx backup.lbox mirror project status
lbx backup.lbox mirror project update
```

The destination must normally be empty. `--adopt` allows the new project to take ownership of an existing non-empty directory, but still does not change its contents until an update.

Each project exclusively owns its complete destination subtree. Ordinary Lockbox file commands cannot change files there; use the corresponding `mirror` commands.

## Manage projects

```bash
lbx backup.lbox mirror projects
lbx backup.lbox mirror project info
lbx backup.lbox mirror project configure
lbx backup.lbox mirror project rebind
```

When a Lockbox contains exactly one mirror, you may omit its name. With more than one, supply the name before the operation.

If the host directory moves or is deliberately replaced, use `rebind` rather than silently accepting a different filesystem object at the old path.

`forget` removes the project's metadata but retains its Lockbox files. `delete` removes both the project and its complete managed directory. Inspect the proposed effect carefully before confirming either operation.

## Selection rules

Persistent include and exclude rules limit the source set:

```bash
lbx backup.lbox mirror project rule add include '**/*.rs'
lbx backup.lbox mirror project rule add exclude '*.tmp'
lbx backup.lbox mirror project rule list
```

An empty selected source could otherwise remove every managed file. reVault blocks that and unusually large deletions unless you explicitly allow them.

## Work inside a mirror

Mirror-scoped file commands respect the project's ownership:

```bash
lbx backup.lbox mirror project list --recursive
lbx backup.lbox mirror project cat docs/readme.md
lbx backup.lbox mirror project extract --to ./restore
lbx backup.lbox mirror project add ./notes.txt
lbx backup.lbox mirror project move notes.txt archive/notes.txt
lbx backup.lbox mirror project remove archive/notes.txt
```

Use `lbx backup.lbox mirror --help` and help on the individual subcommand for overwrite, recursive and safety options.
