//! Complete native Rust API. The public names remain `vault` and `lockbox`;
//! transport framing is only used at foreign-language boundaries.

pub mod lockbox {
    pub use revault_lockbox_api::*;
}

pub mod vault {
    pub use revault_vault_api::*;
}

pub use revault_lockbox_api::{
    ContactKeyPair, ContactPublicKey, Lockbox, OwnerSigningKeyPair, OwnerSigningPublicKey,
};
pub use revault_vault_api::VaultDirectory;
