---
description: "Understand reVault's security boundaries and handle decrypted material safely."
---

# Keeping secrets a secret

Encryption protects a closed Lockbox. The moment you use a secret, a trusted process must see its clear value. Good practice limits which process sees it, how long it remains available, and what traces it leaves.

## What a Lockbox protects

Lockbox page bodies use authenticated encryption. Password slots stretch a passphrase before using it to unwrap the content key, and Profile/Contact access wraps that content key for the authorised public key. Signed commits allow readers to detect unauthorised modification by someone who lacks the owner's signing keys.

File contents, paths, symlink targets, variables, Forms, permissions and the Lockbox description are stored inside encrypted pages. Compression occurs before encryption when it is useful.

The file necessarily exposes limited framing information, including its format/version, physical size, page boundaries, public Lockbox identifier and public key-slot material needed to find a way to unwrap the content key. Key slots must not contain Contact names, email addresses or private file metadata.

{% hint style="info" %}
Encryption hides content, not the existence, approximate size, location or access time of the Lockbox file. Filesystem, backup and network observers may still learn those facts.
{% endhint %}

## Memory protection

Secret buffers are cleared when their owners are dropped and reVault attempts to lock sensitive memory where the operating system allows it. Memory locking is best effort: operating-system limits, containers and WebAssembly can prevent it. Zeroisation remains the dependable baseline, and a privileged debugger or compromised process may still read live plaintext.

Use secret variables and secret Form fields to prevent values being supplied as command-line arguments or exported with ordinary variables:

```bash
lbx secrets.lbox variable set API_TOKEN --secret --interactive
```

Prefer interactive entry for a person at a terminal. Other inputs have trade-offs:

* standard input can expose a value to another pipeline process;
* a source file can remain on disk or in backups;
* an environment variable can be inherited by children or captured in diagnostics; and
* immutable language strings may leave copies that cannot be reliably wiped.

Never put credentials in command arguments, source code, shell history, crash reports or logs.

## Session Agent and Auto Open

The Session Agent caches selected content keys in user-scoped memory. Closing a session clears the cached key, and suspend handling clears cached keys and can stop sensitive work.

Auto Open is a different trust decision: it stores the Vault passphrase in the operating system's secure credential store. On platforms without per-use user presence, a process already running as your unlocked desktop user may be able to retrieve that credential and reopen the Vault.

Closing the Session Agent is therefore not an authentication boundary while Auto Open remains available. Lock the desktop when unattended and disable Auto Open on shared, high-risk or unattended systems where that access is unacceptable.

## Sharing and revocation

Verify a Contact fingerprint through an independent channel before granting access. A key-sharing server proves, at most, that someone controlled an email inbox during publication; it does not prove legal identity, employment or authority.

Revoking a Contact changes future access to Lockbox copies you update. It cannot delete a Lockbox, key or plaintext already copied by that Contact. Replace exposed credentials and distribute newly protected material when actual revocation is required.

Profile rotation creates a new active key generation, but older generations may be retained to open historical Lockboxes. Treat an unexpected key replacement as a security event rather than accepting it automatically.

## Deletion and recovery

When an entry is removed or replaced, reVault rewrites any current objects sharing its old page and zeros the superseded physical page before publishing the new view. Interrupted redaction cleanup is sealed through the transaction-recovery path.

This protects against ordinary recovery of deleted current content from the same updated Lockbox. It cannot erase older Lockbox copies, filesystem snapshots, cloud versions, backups, swap already written by the operating system, or plaintext copied by another program.

Recovery authenticates intact pages and never invents missing bytes. It cannot recover overwritten data, a lost content key, or Profile keys that no longer exist.

## Backups and automation

Keep the Vault passphrase separate from its encrypted backup. Profile recovery files contain private key material and need independent protection. Test restoration rather than assuming a copied file is usable.

For CI/CD:

* use a separate Profile and Lockbox per environment;
* give each runner only the secrets it needs;
* inject each secret as late as possible;
* disable command tracing around secret operations;
* prevent secret-bearing build artefacts and logs; and
* prefer short-lived provider credentials where available.

reVault is a portable encrypted store, not a central policy engine. A managed secret service may be a better choice when you require dynamic credentials, central audit, online revocation or workload identity.

## Security checklist

* Keep reVault, the operating system and secret-consuming applications current.
* Use unique credentials with minimum permissions.
* Lock the desktop and review Auto Open.
* Back up and test the Vault and critical Profiles.
* Verify Contacts independently.
* Rotate credentials immediately after suspected exposure.
* Preserve a damaged Lockbox before attempting recovery.
* Review diagnostic output before sharing it.

Report a suspected vulnerability through the repository's [security policy](https://github.com/onepub-dev/reVault/security/policy), not a public issue.
