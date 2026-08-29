# mirror

reVault lets you mirror a directory into an Lockbox and then update the lockbox to reflect any changes in the host directory.

You may mirror multiple host directories into a single lockbox.

\`lbx \<lockbox> mirror create \[project] --from \<host path> \[--to \<lockbox path>]

If --to is omitted then the project is created in the root of the archive and no further mirrors may be added.&#x20;

If the mirror is not in the root of the archive then you may also store additional (non-mirrored) files in the archive using the standard archive file commands such as 'lbx add note.txt'. You can not use the standard file commands to manipulate files in a mirror project folder, you need to user the equivalent mirror file commands \`lbx mirror file add note.txt'.



## Mirror projects

Mirror projects maintain a persistent, one-way relationship between a host directory and a directory inside a lockbox.

The host directory is authoritative. A mirror update can:

* add new host files to the lockbox;
* replace changed lockbox files;
* create directories;
* remove lockbox files that are absent from the selected host content.

A project exclusively owns its lockbox destination. Ordinary file commands cannot modify that directory; use the project- scoped mirror file commands instead.

### Command structure

```
lbx mirror []
```

For example:&#x20;

```
--to /projects/project project create
--from ./project
--to /projects/project
```

lbx backup.lbox mirror project status lbx backup.lbox mirror project update

may be omitted from project-specific commands when the lockbox contains exactly one mirror project:

lbx backup.lbox mirror status lbx backup.lbox mirror update

If multiple projects exist, the name is required. The create command always requires an explicit name.

### Ownership rules

Each mirror project exclusively owns its complete destination subtree.

For example, a project targeting /projects/alpha:

* may add, replace, move, or remove entries below /projects/alpha;
* cannot affect /projects/beta, /imported, or /notes.txt;
* prevents ordinary file commands from changing /projects/alpha;
* cannot overlap another mirror project's destination.

A project targeting / owns the complete file namespace and prevents any other mirror project from being created.

Variables, forms, access records, and other metadata use separate namespaces and are not affected by mirror ownership.

### Creating a project

```
lbx mirror create
--from <HOST_DIRECTORY>
--to <LOCKBOX_DIRECTORY>
[--adopt]
```

Example:

```
lbx backup.lbox mirror project create
--from ./project
--to /projects/project
```

Creating a project stores its configuration but does not copy any files. Run status to inspect the first update, followed by update to apply it.

The host path may be relative when supplied:

```
lbx backup.lbox mirror project create
--from ../project
--to /projects/project
```

reVault resolves and stores its canonical absolute path. Where available, it also stores a filesystem identity. On Unix this is the directory's device and inode pair, allowing replacement of the directory at the same path to be detected.

#### --adopt

By default, the destination must be empty.

Use --adopt to let the project take ownership of an existing non-empty lockbox directory:

```
lbx backup.lbox mirror project create
--from ./project
--to /projects/project
--adopt
```

Adoption does not immediately change the existing files. Run status to see how the host content differs before running update.

### Listing projects

```
lbx mirror projects [--format <table|tsv|json>]
```

Examples:

lbx backup.lbox mirror projects lbx backup.lbox mirror projects --format json

This command lists each project's:

* name;
* canonical host source;
* lockbox destination;
* missing-file policy.

mirror projects lists all projects and does not accept a project name.

### Showing project information

```
lbx mirror [] info [--format <table|tsv|json>]
```

Examples:

lbx backup.lbox mirror project info lbx backup.lbox mirror project info --format json

The output includes:

* project name;
* canonical host source;
* managed lockbox destination;
* missing-file policy;
* stored host filesystem identity;
* include rules;
* exclude rules.

### Previewing an update

```
lbx mirror [] status
[--format <table|tsv|json>]
```

Examples:

lbx backup.lbox mirror project status lbx backup.lbox mirror project status --format json

status calculates the complete update plan without changing the lockbox. It reports:

* files to add;
* files to replace;
* directories to create;
* files or directories to remove;
* unchanged files;
* safety conditions that would block an update.

status is the only preview command. There is no update --dry-run.

Before applying an update, reVault calculates the plan again. A previously displayed status is therefore informative rather than a reusable or stale plan.

### Applying an update

```
lbx mirror [] update
[--force]
[--allow-empty]
[--allow-large-delete]
```

Example:

lbx backup.lbox mirror project update

The update is displayed before reVault asks for confirmation.

#### --force

Apply the freshly calculated update without prompting:

```
lbx backup.lbox mirror project update --force
```

The source, filesystem identity, and deletion safety checks still apply.

#### --allow-empty

By default, an empty selected host source cannot remove existing managed files.

After inspecting status, explicitly allow it with:

```
lbx backup.lbox mirror project update
--allow-empty
```

This includes cases where the host directory contains files but none are selected by the project's rules.

#### --allow-large-delete

By default, an update cannot remove more than half of the managed files.

After inspecting status, explicitly allow it with:

```
lbx backup.lbox mirror project update
--allow-large-delete
```

Both overrides may be required:

```
lbx backup.lbox mirror project update
--allow-empty
--allow-large-delete
```

### Configuring missing-file behaviour

```
lbx mirror [] configure
--missing-files <remove|retain>
```

The missing-file policy controls what happens to archive files that are absent from the selected host content.

#### Remove missing files

```
lbx backup.lbox mirror project configure
--missing-files remove
```

This is the default and provides an exact mirror:

* files removed from the host are removed from the project;
* files no longer selected by include or exclude rules are removed;
* manually added archive-only files are removed by the next update.

#### Retain missing files

lbx backup.lbox mirror project configure\
\--missing-files retain

This preserves archive-only files while continuing to add and replace selected host files.

Rules still determine which host files are imported. The retain setting only changes how archive-only files are treated.

### Rebinding the host directory

```
lbx mirror [] rebind
--from <HOST_DIRECTORY>
[--force]
```

Use rebind when the configured host directory has moved or has been replaced.

```
lbx backup.lbox mirror project rebind
--from ../moved-project
```

reVault shows the old and new paths and asks for confirmation.

To rebind without prompting:

```
lbx backup.lbox mirror project rebind
--from ../moved-project
--force
```

Rebinding changes the stored host path and filesystem identity. It does not update project files. Run status afterward.

### Selection rules

Mirror rules are stored inside the encrypted lockbox and travel with it.

Rules use source-relative paths or globs. Forward slashes are used on every platform.

Examples:

README.md src/\*\* \*\*/_.rs .tmp target/_

Rule behavior:

* an empty include list selects all host paths;
* when include rules exist, only matching files are selected;
* exclude rules remove matching paths from the selected set;
* exclusions always win;
* rules are evaluated relative to the configured host directory.

#### Listing rules

```
lbx mirror [] rule list
[include|exclude]
[--format <table|tsv|json>]
```

List all rules:

```
lbx backup.lbox mirror project rule list
```

List only include rules:

```
lbx backup.lbox mirror project rule list include
```

List only exclude rules:

```
lbx backup.lbox mirror project rule list exclude
```

rules is an alias for rule:

```
lbx backup.lbox mirror project rules list
```

#### Adding rules

```
lbx mirror [] rule add
<include|exclude>
```

Add one include rule:

```
lbx backup.lbox mirror project rule add include '**/*.rs'
```

Add several include rules:

```
lbx backup.lbox mirror project rule add include
README.md
'src/'
'docs//*.md'
```

Add exclude rules:

```
lbx backup.lbox mirror project rule add exclude
'target/'
'*.tmp'
'.git/'
```

Quote patterns when necessary so the shell does not expand them before they reach reVault.

#### Removing rules

```
lbx mirror [] rule remove
<include|exclude>
...
```

Example:

lbx backup.lbox mirror project rule remove exclude '\*.tmp'

Remove several stored rules:

```
lbx backup.lbox mirror project rule remove include
'docs/**'
README.md
```

rm is an alias for rule remove:

lbx backup.lbox mirror project rule rm exclude '\*.tmp'

The supplied patterns must exactly match stored rule values.

#### Clearing rules

```
lbx mirror [] rule clear
<include|exclude|all>
```

Clear all include rules:

lbx backup.lbox mirror project rule clear include

After clearing the include rules, all paths are selected unless excluded.

Clear all exclude rules:

```
lbx backup.lbox mirror project rule clear exclude
```

Clear every rule:

```
lbx backup.lbox mirror project rule clear all
```

### Project-scoped file commands

Ordinary file commands cannot modify a mirror project's managed directory:

```
lbx backup.lbox add ./manual.txt
--to /projects/project/manual.txt
```

Use the corresponding mirror command instead:

```
lbx backup.lbox mirror project add ./manual.txt
--to manual.txt
```

All stored paths used by mirror file commands are relative to the project's destination.

Direct changes may be reversed by the next update. Run mirror status after using add, remove, or move.

### Adding files directly

```
lbx mirror [] add
[--recursive]
[--to <PROJECT_PATH>]
[--overwrite]
[--include ]...
[--exclude ]...
...
```

Add one file to the project root:

```
lbx backup.lbox mirror project add ./notes.txt
```

Add it under another name:

```
lbx backup.lbox mirror project add ./notes.txt
--to docs/readme.txt
```

Add multiple files to a directory:

```
lbx backup.lbox mirror project add ./*.json
--to config/
```

When adding multiple sources, --to must end in /.

#### Recursive add

```
lbx backup.lbox mirror project add
--recursive
./docs
--to documentation/
```

Only one directory source may be supplied to a recursive add.

#### Temporary include and exclude filters

```
lbx backup.lbox mirror project add
--recursive
./source
--include '/*.rs'
--exclude 'target/'
```

These filters apply only to this add operation. They do not change the mirror project's persistent rules.

#### Overwriting existing files

Existing files are protected by default:

```
lbx backup.lbox mirror project add ./notes.txt
--to notes.txt
--overwrite
```

In verbose mode, --jobs \<auto|1|N> controls import worker concurrency.

### Listing project files

```
lbx mirror [] list
[--recursive]
[--format <table|tsv|json>]
[<PATH_OR_GLOB>]
```

List the project root:

```
lbx backup.lbox mirror project list
```

List one directory:

```
lbx backup.lbox mirror project list docs
```

List recursively:

```
lbx backup.lbox mirror project list --recursive
```

List matching files:

```
lbx backup.lbox mirror project list '**/*.json'
```

Produce JSON output:

```
lbx backup.lbox mirror project list
--recursive
--format json
```

Options:

* -R, --recursive — list entries below child directories;
* \--format table — human-readable table output;
* \--format tsv — tab-separated output;
* \--format json — JSON output.

ls is an alias for list:

lbx backup.lbox mirror project ls -R

### Reading a project file

```
lbx mirror [] cat <PROJECT_PATH>
```

Print a stored file:

lbx backup.lbox mirror project cat docs/readme.md

Redirect it to another command or file:

```
lbx backup.lbox mirror project cat config/settings.json | 
jq lbx backup.lbox mirror project cat docs/readme.md > README.md
```

cat does not modify the lockbox.

### Extracting project files

#### Extract one file

```
lbx mirror [] extract
[--overwrite]
<PROJECT_PATH>
<HOST_DESTINATION>
```

Example:

```
lbx backup.lbox mirror project extract
docs/readme.md
./README.md
```

Overwrite an existing host file:

```
lbx backup.lbox mirror project extract
docs/readme.md
./README.md
--overwrite
```

#### Extract the complete project

```
lbx mirror [] extract
--to <HOST_DIRECTORY>
[--overwrite]
[--restore-symlinks]
[--restore-permissions]
```

Example:

```
lbx backup.lbox mirror project extract
--to ./restored-project
```

Options:

* \--overwrite — replace existing host paths;
* \--restore-symlinks — restore stored symbolic links; symlinks are skipped by default;
* \--restore-permissions — apply stored permission bits where supported.

Extraction safety limits for file count, individual file size, and total output size still apply.

### Removing project files

```
lbx mirror [] remove
[--recursive]
[--force]
<PATH_OR_GLOB>...
```

Remove one file:

lbx backup.lbox mirror project remove notes.txt

Remove several files:

```
lbx backup.lbox mirror project remove
notes.txt
config/old.json
```

Remove files matching a root-level glob:

```
lbx backup.lbox mirror project remove '*.json'
```

A quoted \*.json matches files in the project root. Use \*\* for recursive matching:

```
lbx backup.lbox mirror project remove '**/*.json'
```

Removing a directory requires --recursive:

```
lbx backup.lbox mirror project remove
--recursive
old-docs
```

Short forms:

lbx backup.lbox mirror project rm -r old-docs lbx backup.lbox mirror project rm -R old-docs

Use --force to remove matched entries without confirmation:

```
lbx backup.lbox mirror project remove
--force
'*.tmp'
```

The command validates the complete set before changing the lockbox.

### Moving or renaming project files

```
lbx mirror [] move
```

Rename a file:

```
lbx backup.lbox mirror project move
notes.txt
archived-notes.txt
```

Move it into a directory:

```
lbx backup.lbox mirror project move
notes.txt
archive/notes.txt
```

The source and destination must both remain inside the project's managed directory. Existing destinations are not overwritten implicitly.

Aliases:

```
lbx backup.lbox mirror project mv notes.txt archive/notes.txt 
lbx backup.lbox mirror project rename notes.txt archived-notes.txt
```

### Forgetting a project

```
lbx mirror [] forget [--force]
```

forget removes the project definition but retains all its files:

```
lbx backup.lbox mirror project forget
```

After confirmation:

* the mirror project no longer exists;
* its former destination becomes an ordinary lockbox directory;
* ordinary file commands may modify the retained files.

Skip confirmation with:

```
lbx backup.lbox mirror project forget --force
```

### Deleting a project

```
lbx mirror [] delete [--force]
```

delete removes both:

* the project definition;
* the complete managed file subtree.

lbx backup.lbox mirror project delete

Skip confirmation with:

lbx backup.lbox mirror project delete --force

For a project targeting /, deletion removes the complete lockbox file namespace while leaving variables, forms, access records, and other separate metadata intact.

### Stored project metadata

Each project is stored as an encrypted, dot-prefixed normal variable:

/.revault/mirrors/

The stored record includes:

* project name;
* canonical host source path;
* lockbox destination;
* include rules;
* exclude rules;
* missing-file policy;
* host filesystem identity, where available.

Dot-prefixed variables:

* do not appear in ordinary variable listings;
* appear with variable list --all;
* are never included in variable exports;
* can be inspected with an exact variable get.

Mirror metadata uses existing encrypted variable storage and does not change the lockbox archive format.



