---
description: "Inspect damage and recover readable entries from a Lockbox."
---

# Recover a damaged Lockbox

Recovery scans authenticated pages rather than trusting only the fixed header or current table of contents. It can therefore recover complete, path-bearing entries when some index or data pages are damaged.

{% hint style="warning" %}
Work from a copy whenever possible. Keep the original unchanged until the recovered Lockbox has been opened, inspected and backed up.
{% endhint %}

## Preview recovery

```bash
lbx damaged.lbox recover --dry-run
```

The report identifies readable, partial and unrecoverable material without writing an output file. For automation:

```bash
lbx damaged.lbox recover --dry-run --format json
```

## Write a recovered Lockbox

```bash
lbx damaged.lbox recover --output recovered.lbox
```

If `--output` is omitted, reVault writes a sibling named like `damaged.recovered.lbox`. It refuses to replace an existing output unless you pass `--overwrite`.

Open the result and verify important entries:

```bash
lbx recovered.lbox open
lbx recovered.lbox list --recursive
lbx recovered.lbox extract --to ./recovery-check
```

Recovery writes only complete entries whose metadata can still be associated with a valid Lockbox path. A surviving name with missing data is reported rather than padded with invented bytes.

## Interrupted redaction cleanup

An interrupted transaction may leave a Lockbox in a special state that requires its cleanup to be sealed before ordinary reads or writes continue. The error explicitly asks for:

```bash
lbx affected.lbox recover --transaction
```

This resumes in-place transaction/redaction cleanup using the same credentials. It is different from salvage recovery and should be used only when reVault reports that specific condition.

## What recovery cannot do

Recovery cannot reconstruct overwritten or cryptographically unauthentic data, invent a lost content key, or bypass a lost Vault/Profile/password. If all usable key slots or required Profile keys are gone, the encrypted pages remain inaccessible.
