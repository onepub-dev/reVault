---
description: How to manage contacts in your vault
---

# Contacts

Contacts are designed to make it easy to share a lockbox with someone you know.

The reVault Vault holds a list of trusted contacts. You establish the list of trusted contacts by using the `lockbox vault identity publish/lockbox contact receive` commands with an associate.

An associate publishes their public key (the public part of their master key, owned by one of their identities, held in their vault) and you receive it. After going through a process that ensures that you can trust the public key, it is added into your contact list.

Read the [sharing](sharing.md) section for details on populating your contact list by exchanging keys.

Once a public key is in your contact list you can add the contact to any lockbox, send the contact the lockbox and they will be able to unlock it using their installed version of reVault.

You can managed your list of contacts via the reVault cli command line tools:

* lockbox vault contact receive - received a public key after the owner publishes the key using the \`lockbox vault identity publish\` command and add the public key as a contact.
* lockbox vault contact import - add a contact by importing their public key from a file and entering their digital fingerprint.
* lockbox vault contact list - list the set of know contacts and their trust state.
* lockbox vault contact remove - remove a contact from the contact list.



To receive a contact from an associated read the section on [sharing](sharing.md).

## Revoking Contacts/Keys

A Contact represents an email address and the public key of the publisher.

If that publisher leaves the organisation or can nolonger be trusted it is impossible to remove their access from lockboxes they have in their possession - assume they have the private key squirrelled away.

As a receiver of a public Key/Contact the best you can do is delete the contact from your vault and remove the access to any lockboxes you have control over.

