#![deny(missing_docs)]

//! Historical native-format v2 exporter. This crate never imports or creates
//! current native vaults.

mod vault_v2;

pub use vault_v2::export_vault_v2;
