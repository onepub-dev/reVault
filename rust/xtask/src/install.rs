use crate::command::{self, TaskResult};

pub fn cli() -> TaskResult {
    command::run(command::command("cargo").args([
        "install",
        "--locked",
        "--force",
        "--path",
        "revault_cli",
    ]))?;
    println!("Installed the local `lockbox` and `lbx` executables.");
    Ok(())
}
