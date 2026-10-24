//! Isolated format-4 archive and Vault exporter.
#![allow(dead_code, unused_imports)]
mod archive_command;
#[path = "archive_reader/lib.rs"]
pub mod archive_reader;
#[path = "export_archive/lib.rs"]
pub mod export_archive;
mod vault_command;
#[path = "vault_reader/lib.rs"]
pub mod vault_reader;
/// Run the format-4 archive exporter.
pub fn run_archive() {
    archive_command::main();
}
/// Run the format-4 Vault exporter.
pub fn run_vault() {
    vault_command::main();
}
