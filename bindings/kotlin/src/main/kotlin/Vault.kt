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

/** Runtime loader. Loading does not open a vault or lockbox. */
typealias Revault = com.onepub.revault.Revault
/** Persistent encrypted local store for profiles, keys, contacts, and metadata. */
typealias Vault = com.onepub.revault.Vault
/** Metadata-only view that cannot mutate or load private signing material. */
typealias ReadOnlyVault = com.onepub.revault.ReadOnlyVault
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
/** A profile signing identity used to authorize mutable lockbox revisions. */
typealias ProfileSigningKeyPair = com.onepub.revault.Revault.ProfileSigningKeyPair
/** The public profile identity readers use to verify authorized revisions. */
typealias ProfileSigningPublicKey = com.onepub.revault.Revault.ProfileSigningPublicKey
/** A token kept alive while an operation needs secrets cached by the agent. */
typealias AgentActivity = com.onepub.revault.Revault.AgentActivity

/** Explicit controller for the optional single session-agent process. */
typealias AgentSession = com.onepub.revault.AgentSession
/** Closed cache policy values. */
typealias CacheMode = com.onepub.revault.CacheMode
/** Closed I/O workload values. */
typealias WorkloadProfile = com.onepub.revault.WorkloadProfile
/** Closed worker scheduling values. */
typealias WorkerPolicy = com.onepub.revault.WorkerPolicy

/** One historical generation of a vault profile's contact keys. */
typealias ProfileGeneration = com.onepub.revault.ProfileGeneration
/** Versioned key-generation history for one named vault profile. */
typealias ProfileHistory = com.onepub.revault.ProfileHistory
