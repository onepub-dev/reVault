//! Isolated exporter for archive/container format 3.
#![allow(dead_code, unused_imports)]
mod archive_command;
#[path = "archive_reader/lib.rs"]
mod archive_reader;
#[path = "export_archive/lib.rs"]
mod export_archive;
#[path = "vault_reader/lib.rs"]
mod vault_reader;
/// Runs the isolated format-3 archive export command.
pub fn run_archive() {
    archive_command::main();
}
mod vault_command;
/// Runs the isolated format-3 vault export command.
pub fn run_vault() {
    vault_command::main();
}
