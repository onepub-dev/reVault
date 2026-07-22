#![deny(missing_docs)]
//! Complete native Rust API for reVault lockboxes and local vaults.
//!
//! Use [`lockbox`] for portable encrypted archives and [`vault`] for local key,
//! contact, form, platform-keyring, and session-agent operations. This package
//! re-exports the native core directly; transport framing exists only at the
//! foreign-language boundary.
//!
//! See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
//! for installation, security guidance, and examples.

/// Portable encrypted archive API.
pub mod lockbox {
    pub use revault_lockbox_api::*;
}

/// Local vault, keyring, and session-agent API.
pub mod vault {
    pub use revault_vault_api::*;
}

pub use revault_lockbox_api::{
    ContactKeyPair, ContactPublicKey, Lockbox, OwnerSigningKeyPair, OwnerSigningPublicKey,
};
pub use revault_vault_api::VaultDirectory;
