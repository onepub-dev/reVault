---
description: "Inspect damage and recover readable entries from a Lockbox."
---

# Recover a damaged Lockbox

`doctor recover` first checks for interrupted format-3 transaction cleanup. If cleanup is pending, it resumes that work in place. Otherwise it uses salvage recovery, scanning authenticated pages to recover complete, path-bearing entries when some index or data pages are damaged. The source must exist; recovery never recreates a missing Lockbox.

{% hint style="warning" %}
Work from a copy whenever possible. Keep the original unchanged until the recovered Lockbox has been opened, inspected and backed up.
{% endhint %}

## Preview recovery

Recovery can unlock a closed Lockbox from its surviving key directory using
Profile keys or passwords available through the Vault. You do not need a
successful normal `open` before recovery. If the key directory is damaged too,
recovery needs an already cached content key or an explicitly supplied recovery
key; it cannot reconstruct a lost key.

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

## Recover an older Lockbox after migration fails

The current CLI can recover format-2 Lockboxes directly. You do not need to
install an older CLI. Preview recovery, then supply a separate output path:

```bash
lbx old-secrets.lbox doctor recover --dry-run
lbx old-secrets.lbox doctor recover --output old-secrets.recovered.lbox
lbx old-secrets.recovered.lbox open
lbx old-secrets.recovered.lbox list --recursive
```

The recovered copy uses format 3 and retains usable access slots from the
surviving key directory. Check its contents before choosing whether to replace
the original. Recovery can omit damaged entries; a successful recovery does not
mean that every original entry survived. Format-2 recovery requires a separate
output and never repairs the original in place.

This route supports formats 2 and 3. It does not add recovery for format 1 or
newer unsupported formats. Keep the damaged source and any backups when a
compatible recovery reader is unavailable.

## Interrupted transactions

Format 3 retains cleanup after publication. It does not provide the deferred v4 protocol's durable rollback of unpublished allocations or interrupted tail truncation. A failed operation can leave unpublished physical storage even when the previous logical contents remain authoritative.

If a transaction published its new logical state before cleanup was interrupted, reVault must roll that cleanup forward. It cannot roll back because the new state is already authoritative.

Write-capable opens detect this authenticated state and finish the required recovery automatically. Callers that explicitly request a read-only open remain non-mutating and receive a recovery-required result. `doctor recover` detects the same state and completes it in place; there is no separate transaction option to choose.

Use `--dry-run` to see which operation was detected without changing the Lockbox. If no interrupted transaction is pending, `doctor recover` uses salvage recovery instead.

## What recovery cannot do

Recovery cannot reconstruct overwritten or cryptographically unauthentic data, invent a lost content key, or bypass a lost Vault/Profile/password. If all usable key slots or required Profile keys are gone, the encrypted pages remain inaccessible.
