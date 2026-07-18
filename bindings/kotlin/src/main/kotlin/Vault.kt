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

/** Main entry point for all lockbox, key, local-vault, agent, and platform operations. */
typealias Vault = com.onepub.revault.Revault
/** Owned, mutable view of one encrypted lockbox archive. */
typealias Lockbox = com.onepub.revault.Revault.Lockbox
/** Runtime cache and worker tuning for opening or creating lockboxes. */
typealias LockboxOptions = com.onepub.revault.Revault.LockboxOptions
/** Owned contact key pair used to decrypt received content keys. */
typealias ContactKeyPair = com.onepub.revault.Revault.ContactKeyPair
/** Shareable contact public key used to encrypt a recipient content key. */
typealias ContactPublicKey = com.onepub.revault.Revault.ContactPublicKey
/** Owned encrypted content-key envelope for one contact recipient. */
typealias WrappedContactKey = com.onepub.revault.Revault.WrappedContactKey
/** Owned signing key pair used to authorize mutable lockbox commits. */
typealias SigningKeyPair = com.onepub.revault.Revault.SigningKeyPair
/** Public key used to verify owner-authorized lockbox commits. */
typealias SigningPublicKey = com.onepub.revault.Revault.SigningPublicKey
/** Writable, password-protected local metadata vault. */
typealias VaultDirectory = com.onepub.revault.Revault.VaultDirectory
/** Read-only metadata view that never loads an owner signing key. */
typealias ReadOnlyVaultDirectory = com.onepub.revault.Revault.ReadOnlyVaultDirectory
/** Owned registration for an operation that currently requires secret access. */
typealias AgentActivity = com.onepub.revault.Revault.AgentActivity
/** High-level workflow for local metadata and remembered lockboxes. */
typealias LocalVault = com.onepub.revault.Revault.LocalVault

/** One historical generation of a vault profile's contact keys. */
typealias ProfileGeneration = revault.bindings.RevaultBindings.ProfileGeneration
/** Versioned key-generation history for one named vault profile. */
typealias ProfileHistory = revault.bindings.RevaultBindings.ProfileHistory
