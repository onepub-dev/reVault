use crate::command::{self, TaskResult};

pub fn dev_tools(args: &[String]) -> TaskResult {
    if args.len() == 1 && matches!(args[0].as_str(), "-h" | "--help") {
        println!(
            "Usage: cargo xtask dev-tools\n\n\
             Builds and installs revault-tool, the repository developer command hub.\n\
             revault-tool handles release preparation/publication, binding generation\n\
             and checks, conformance tests, and repository development tasks under\n\
             `revault-tool dev`."
        );
        return Ok(());
    }
    if !args.is_empty() {
        return Err("dev-tools does not accept arguments".to_owned());
    }
    install_path("revault_tooling")?;
    println!("Installed local `revault-tool`; run `revault-tool --help` to view its commands.");
    Ok(())
}

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
