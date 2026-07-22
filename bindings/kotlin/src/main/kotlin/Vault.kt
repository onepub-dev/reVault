/**
 * Idiomatic Kotlin names for encrypted reVault lockboxes, keys, and local
 * vault metadata.
 *
 * [Vault] is the main entry point. Close owned handles promptly and use the
 * callback-scoped secret APIs to avoid retaining plaintext. See the
 * [repository README](https://github.com/onepub-dev/reVault#readme) for
 * installation, security guidance, and complete examples.
 */
package com.onepub.revault.kotlin

/** Primary API used to open lockboxes and manage keys, metadata, and local services. */
typealias Vault = com.onepub.revault.Revault
/** An open encrypted archive containing files, variables, secrets, and forms. */
typealias Lockbox = com.onepub.revault.Revault.Lockbox
/** Memory and CPU settings applied when creating or opening a lockbox. */
typealias LockboxOptions = com.onepub.revault.Revault.LockboxOptions
/** A profile's contact-encryption identity used to decrypt keys addressed to it. */
typealias ContactKeyPair = com.onepub.revault.Revault.ContactKeyPair
/** A recipient's shareable encryption identity used when granting access. */
typealias ContactPublicKey = com.onepub.revault.Revault.ContactPublicKey
/** A content key encrypted for one contact and recoverable by its matching key pair. */
typealias WrappedContactKey = com.onepub.revault.Revault.WrappedContactKey
/** A lockbox owner's signing identity used to authorize mutable revisions. */
typealias SigningKeyPair = com.onepub.revault.Revault.SigningKeyPair
/** The public identity readers use to verify owner-authorized revisions. */
typealias SigningPublicKey = com.onepub.revault.Revault.SigningPublicKey
/** Password-protected storage for profile keys, contacts, forms, backups, and lockbox paths. */
typealias VaultDirectory = com.onepub.revault.Revault.VaultDirectory
/** A metadata view for discovery that never loads an owner signing key. */
typealias ReadOnlyVaultDirectory = com.onepub.revault.Revault.ReadOnlyVaultDirectory
/** A token kept alive while an operation needs secrets cached by the agent. */
typealias AgentActivity = com.onepub.revault.Revault.AgentActivity
/** A session that opens lockboxes by host path, caches passwords, and closes local files. */
typealias LocalVault = com.onepub.revault.Revault.LocalVault

/** One historical generation of a vault profile's contact keys. */
typealias ProfileGeneration = com.onepub.revault.ProfileGeneration
/** Versioned key-generation history for one named vault profile. */
typealias ProfileHistory = com.onepub.revault.ProfileHistory
