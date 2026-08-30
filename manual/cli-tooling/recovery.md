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
lbx damaged.lbox doctor recover --dry-run
```

The report identifies pending authenticated cleanup or, for salvage, readable, partial and unrecoverable material without writing an output file. For automation:

```bash
lbx damaged.lbox doctor recover --dry-run --format json
```

## Write a recovered Lockbox

```bash
lbx damaged.lbox doctor recover --output recovered.lbox
```

If `--output` is omitted, reVault writes a sibling named like `damaged.recovered.lbox`. It refuses to replace an existing output unless you pass `--overwrite`.

Open the result and verify important entries:

```bash
lbx recovered.lbox open
lbx recovered.lbox list --recursive
lbx recovered.lbox extract --to ./recovery-check
```

Recovery writes only complete entries whose metadata can still be associated with a valid Lockbox path. A surviving name with missing data is reported rather than padded with invented bytes.

## Interrupted cleanup

If a transaction published its new logical state before cleanup was interrupted, reVault must roll that cleanup forward. It cannot roll back because the new state is already authoritative.

Write-capable opens detect this authenticated state and finish cleanup automatically. Callers that explicitly request a read-only open remain non-mutating and receive a recovery-required result. `doctor recover` detects the same state and completes it in place; there is no separate transaction option to choose.

Use `--dry-run` to see which operation was detected without changing the Lockbox. If no interrupted cleanup is pending, `doctor recover` uses salvage recovery instead.

## What recovery cannot do

Recovery cannot reconstruct overwritten or cryptographically unauthentic data, invent a lost content key, or bypass a lost Vault/Profile/password. If all usable key slots or required Profile keys are gone, the encrypted pages remain inaccessible.
