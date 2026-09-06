use crate::command::{self, TaskResult};
use std::{fs, path::PathBuf, process::Command};

pub fn build() -> TaskResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../browser");
    command::run(
        Command::new("npm")
            .current_dir(&root)
            .args(["ci", "--ignore-scripts"]),
    )?;
    fs::create_dir_all(root.join("generated")).map_err(|e| e.to_string())?;
    command::run(
        Command::new(root.join("node_modules/.bin/pbjs"))
            .current_dir(&root)
            .args([
                "-t",
                "static-module",
                "-w",
                "es6",
                "--force-number",
                "-o",
                "generated/protocol.js",
                "../rust/revault_browser_protocol/proto/browser.proto",
            ]),
    )?;
    for (source, output, format) in [
        ("src/background.js", "chromium/background.js", "iife"),
        ("src/content.js", "chromium/content.js", "iife"),
        ("src/sdk.js", "dist/sdk.js", "esm"),
    ] {
        command::run(
            Command::new(root.join("node_modules/.bin/esbuild"))
                .current_dir(&root)
                .args([
                    source,
                    "--bundle",
                    "--platform=browser",
                    &format!("--format={format}"),
                    &format!("--outfile={output}"),
                ]),
        )?;
    }
    for file in ["background.js", "content.js"] {
        fs::copy(
            root.join("chromium").join(file),
            root.join("firefox").join(file),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn test() -> TaskResult {
    build()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../browser");
    command::run(
        Command::new(root.join("node_modules/.bin/esbuild"))
            .current_dir(&root)
            .args([
                "test/security.js",
                "--bundle",
                "--platform=node",
                "--format=esm",
                "--outfile=generated/security-tests.mjs",
            ]),
    )?;
    command::run(
        Command::new("node")
            .current_dir(root)
            .args(["--test", "generated/security-tests.mjs"]),
    )
}

pub fn generate_protocol() -> TaskResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../revault_browser_protocol");
    prost_build::Config::new()
        .out_dir(root.join("src"))
        .compile_protos(&[root.join("proto/browser.proto")], &[root.join("proto")])
        .map_err(|e| e.to_string())?;
    fs::rename(
        root.join("src/revault.browser.v1.rs"),
        root.join("src/wire.rs"),
    )
    .map_err(|e| e.to_string())
}
