use crate::command::{self, TaskResult};
use std::fs;
use std::path::PathBuf;

const DEFAULT_IMAGE: &str = "rust:1.88-bullseye";
const DEFAULT_OUTPUT: &str = "target/portable/x86_64-unknown-linux-gnu-glibc-2.31";

/// Builds the CLI in a pinned Linux userspace so the resulting binary has a
/// conservative glibc baseline. The container is a build-time dependency only;
/// neither Docker nor Rust is required on the destination machine.
pub fn cli(args: &[String]) -> TaskResult {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!(
            "Usage: cargo xtask build-cli [--image IMAGE] [--output DIRECTORY]\n\n\
             Builds lockbox and lbx in Docker. IMAGE defaults to\n\
             {DEFAULT_IMAGE}. The output directory defaults to\n\
             {DEFAULT_OUTPUT}."
        );
        return Ok(());
    }

    let mut image = DEFAULT_IMAGE.to_owned();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        let option = &args[index];
        match option.as_str() {
            "--image" => {
                index += 1;
                image = args
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "missing value for --image".to_owned())?;
            }
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "missing value for --output".to_owned())?,
                ));
            }
            other => return Err(format!("unknown build-cli option {other:?}")),
        }
        index += 1;
    }

    if !command::exists_on_path("docker") {
        return Err("build-cli requires Docker (the Docker daemon must be running)".to_owned());
    }

    let root = command::workspace_root()?;
    let uid = command::output_lossy(command::command("id").arg("-u"))?;
    let gid = command::output_lossy(command::command("id").arg("-g"))?;
    let build_command = format!(
        "apt-get update && apt-get install -y --no-install-recommends pkg-config libdbus-1-dev && cargo build --locked --release --package revault_cli && chown -R {uid}:{gid} target"
    );
    let mut build = command::command("docker");
    build.args([
        "run",
        "--rm",
        "--volume",
        &format!("{}:/src", root.display()),
        "--workdir",
        "/src",
        &image,
        "sh",
        "-c",
        &build_command,
    ]);
    command::run(&mut build)?;

    let built = root.join("target/release");
    let destination = output.unwrap_or_else(|| root.join(DEFAULT_OUTPUT));
    fs::create_dir_all(&destination).map_err(to_string)?;
    for binary in ["lockbox", "lbx"] {
        let source = built.join(binary);
        command::require_file(&source)?;
        let destination_file = destination.join(binary);
        fs::copy(&source, &destination_file).map_err(to_string)?;
        println!("created {}", destination_file.display());
    }
    println!("build image: {image}");
    Ok(())
}

fn to_string(error: std::io::Error) -> String {
    error.to_string()
}
