use crate::command::{self, TaskResult};

pub fn cli() -> TaskResult {
    install_path("revault_cli")?;
    install_path("revault_migrate_vault_v1")?;
    install_path("revault_migrate_vault_v2")?;
    install_path("revault_migrate_archive_v1")?;
    println!("Installed local `lockbox`, `lbx`, and Vault/Lockbox migration exporters.");
    Ok(())
}

fn install_path(path: &str) -> TaskResult {
    command::run(command::command("cargo").args(["install", "--locked", "--force", "--path", path]))
}
