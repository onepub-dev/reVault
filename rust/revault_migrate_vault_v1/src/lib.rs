#![deny(missing_docs)]

//! Historical native-format v1 exporter. This crate never imports or creates
//! current native vaults.

mod vault_v1;

pub use vault_v1::export_vault_v1;
