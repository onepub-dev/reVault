---
description: "Definitions for the terms used throughout reVault and its manual."
---

# Glossary

## Agent session

A time-limited, in-memory entry held by the [Session Agent](cli-tooling/revault-session-agent.md). It contains a Lockbox content key, not an open file handle.

## Auto Open

An optional feature that stores the Vault passphrase in the operating system's secure credential store. It is separate from an agent session.

## Contact

Another person's public Profile stored in your Vault. A Contact may be granted access to a Lockbox but does not give you the person's private key.

## Content key

The random symmetric key that encrypts a Lockbox's private pages. Password and Profile key slots provide different authorised ways to unwrap it.

## Fingerprint

A compact representation of a public Profile used to compare keys over an independent channel. Matching a fingerprint detects substitution; the channel still determines whose key you trust it to be.

## Form

A typed collection of related fields. A Form definition describes its field names and kinds; a Form record stores values for one instance.

## Lockbox

The portable `.lbox` encrypted archive containing files, variables, Forms and access records. `lbx` is the short command name; it is not another name for the archive.

## Profile

One of your named public/private key identities inside the Vault. A Profile can create, sign and open Lockboxes. The `default` Profile is created with the Vault.

## Profile generation

One key generation in a Profile's history. Rotation creates a new active generation while retaining older generations needed by existing Lockboxes.

## Secret

A variable or Form field given extra in-memory and command-line handling. Secret marking reduces accidental exposure; it does not protect a value from the program that receives it.

## Session Agent

The local, per-user process that caches selected content keys in memory and manages suspend protection for sensitive operations.

## Vault

The encrypted, device-local store for Profiles, Contacts, reusable Form definitions and remembered Lockbox access. A Vault is not a Lockbox collection and should not be synchronised as a live file between machines.
