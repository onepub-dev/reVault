#![deny(missing_docs)]

//! Storage and HTTP services for the reVault publish key server.

/// Persistent, bounded reciprocal invitation storage.
pub mod exchange_store;
/// Operating-system service installation helpers.
pub mod install;
/// Represents server.
pub mod server;
/// Represents server log.
pub mod server_log;
/// Represents store.
pub mod store;
