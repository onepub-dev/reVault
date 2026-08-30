---
description: "Definitions for the terms used throughout reVault and its manual."
---

# Glossary

This glossary defines the vocabulary used by the reVault manual, command line
help, and API guides.

## AgentSession

`AgentSession` is the language binding API used to connect to and control the
[Session Agent](cli-tooling/revault-session-agent.md). Use it when Lockbox
access needs to be shared across processes or process restarts.

## Auto Open

An optional feature that stores the Vault passphrase in the
[platform credential store](#platform-credential-store). It is separate from
the Session Agent and its cached Lockbox keys.

## Contact

Another person's public Profile stored in your Vault. A Contact may be granted
access to a Lockbox but does not give you the person's private key.

## Content key

The random symmetric key that encrypts a Lockbox's private content. Profile
keys and Lockbox passwords provide authorised ways to open it. Most users do
not need to work with content keys directly.

## Credential

The general term for material that can provide access. A Profile key, a
remembered Lockbox password, and the Vault passphrase are different kinds of
credential. The manual names the specific kind whenever that distinction
affects what you must provide.

## Fingerprint

A compact representation of a public Profile used to compare keys over an
independent channel. Matching a fingerprint detects substitution; the channel
still determines whose key you trust it to be.

## Form

A typed collection of related fields. A Form definition describes its field
names and kinds; a Form record stores values for one instance.

## Key Sharing Service

The optional reVault service used to exchange Profile public keys. A key server
is one server running the Key Sharing Service.

## Lockbox

The portable `.lbox` file containing compressed, encrypted, and signed files,
variables, Forms, and access information. A Lockbox is the reVault archive; use
“archive” only when discussing archive formats in general.

## Lockbox password

An optional password that grants access to one Lockbox. Profiles are the normal
way to create and open Lockboxes. Password access is useful when the intended
recipient's Contact details are not available.

## Platform credential store

The operating system facility used by Auto Open to store the Vault passphrase.
The usual implementations are Credential Manager on Windows, Secret Service
on Linux, and Keychain on macOS.

## Profile

One of your named public and private key identities inside the Vault. A Profile
can create, sign, and open Lockboxes. The `default` Profile is created with the
Vault.

## Profile generation

One key generation in a Profile's history. Rotation creates a new active
generation while retaining older generations needed by existing Lockboxes.

## Secret

A variable or Form field given extra handling in memory and on the command line.
Secret marking reduces accidental exposure; it does not protect a value from
the program that receives it.

## Session Agent

The optional local process for one user that temporarily caches selected Lockbox
content keys and manages suspend protection for sensitive operations. It is
separate from the platform credential store used by Auto Open.

## Vault

The encrypted store reVault maintains on one device for Profiles, Contacts,
reusable Form definitions, and remembered Lockbox access. A Vault is not a
collection of Lockboxes and should not be synchronised as a live file between
devices.

## Vault passphrase

The secret that opens the Vault. It is distinct from a Lockbox password. Auto
Open can store the Vault passphrase in the platform credential store.
