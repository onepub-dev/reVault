---
description: "Inspect damage and recover readable entries from a Lockbox."
---

# Recover a damaged Lockbox

`doctor recover` first checks for an interrupted authenticated transaction. If one exists, it resumes rollback, cleanup or truncation in place. Otherwise it uses salvage recovery, scanning authenticated pages to recover complete, path-bearing entries when some index or data pages are damaged.

{% hint style="warning" %}
Work from a copy whenever possible. Keep the original unchanged until the recovered Lockbox has been opened, inspected and backed up.
{% endhint %}

## Preview recovery

```bash
lbx damaged.lbox doctor recover --dry-run
```

The report identifies pending authenticated transaction recovery or, for salvage, readable, partial and unrecoverable material without writing an output file. For automation:

```bash
lbx damaged.lbox doctor recover --dry-run --format json
```

## Salvage into a recovered Lockbox

```bash
lbx damaged.lbox doctor recover --output recovered.lbox
```

For salvage, if `--output` is omitted, reVault writes a sibling named like `damaged.recovered.lbox`. It refuses to replace an existing output unless you pass `--overwrite`.

Open the result and verify important entries:

```bash
lbx recovered.lbox open
lbx recovered.lbox list --recursive
lbx recovered.lbox extract --to ./recovery-check
```

Recovery writes only complete entries whose metadata can still be associated with a valid Lockbox path. A surviving name with missing data is reported rather than padded with invented bytes.

## Interrupted transactions

Before a replacement commit is published, recovery rolls back to the previous committed state: it zeros reserved reusable ranges and truncates appended preparation data. Once published, it finishes cleanup. An interrupted free-tail truncation is completed without changing logical contents.

If a transaction published its new logical state before cleanup was interrupted, reVault must roll that cleanup forward. It cannot roll back because the new state is already authoritative.

Write-capable opens detect this authenticated state and finish the required recovery automatically. Callers that explicitly request a read-only open remain non-mutating and receive a recovery-required result. `doctor recover` detects the same state and completes it in place; there is no separate transaction option to choose.

Use `--dry-run` to see which operation was detected without changing the Lockbox. If no interrupted transaction is pending, `doctor recover` uses salvage recovery instead.

## What recovery cannot do

Recovery cannot reconstruct overwritten or cryptographically unauthentic data, invent a lost content key, or bypass a lost Vault/Profile/password. If all usable key slots or required Profile keys are gone, the encrypted pages remain inaccessible.
