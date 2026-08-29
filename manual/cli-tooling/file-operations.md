---
description: "Read, organise, describe and remove files inside a Lockbox."
---

# File operations

Lockbox paths use `/` separators on every host platform. Missing parent directories are created when files are added, moved or extracted.

## Read and organise

```bash
lbx project.lbox list --recursive
lbx project.lbox cat /docs/readme.md
lbx project.lbox move /draft.txt /docs/final.txt
```

`cat` writes file bytes to standard output, making it suitable for a pipeline or redirection. Avoid using it for secret material when the terminal, pipeline or logs are not trusted.

Extract one file or the complete archive:

```bash
lbx project.lbox extract /docs/readme.md ./README.md
lbx project.lbox extract --to ./restored
```

Existing destinations are not silently replaced. Use the command's explicit `--overwrite` option after reviewing the target.

## Remove entries

```bash
lbx project.lbox remove /draft.txt
lbx project.lbox remove --recursive /old/
lbx project.lbox rm '**/*.tmp'
```

Quote globs so that the shell does not expand them against host files. reVault asks for confirmation where an operation is broad; `--force` bypasses that prompt.

Deleting or replacing an entry updates the current Lockbox view and performs redaction cleanup of superseded storage. It is not a remote-delete mechanism: other Lockbox copies remain unchanged.

## Add an encrypted description

Descriptions record purpose without becoming part of the public header:

```bash
lbx project.lbox description set 'Project Atlas deployment material'
lbx project.lbox description get
lbx project.lbox description clear
```

Use `--interactive`, `--stdin`, `--file` or `--from-env` when the text should not appear in shell history.
