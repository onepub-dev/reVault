//! Isolated exporter for archive/container format 1.
#![allow(dead_code, unused_imports)]
mod archive_command;
#[path = "archive_initial/lib.rs"]
mod archive_initial;
#[path = "archive_reader/lib.rs"]
mod archive_reader;
#[path = "export_archive/lib.rs"]
mod export_archive;
#[path = "export_vault/lib.rs"]
mod export_vault;
#[path = "vault_initial/lib.rs"]
mod vault_initial;
#[path = "vault_reader/lib.rs"]
mod vault_reader;
/// Runs the isolated format-1 archive export command.
pub fn run_archive() {
    archive_command::main();
}
mod vault_command;
/// Runs the isolated format-1 vault export command.
pub fn run_vault() {
    vault_command::main();
}
