use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("fake cargo failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("install") {
        return Err("expected cargo install".into());
    }
    let root = option(&args, "--root").ok_or("cargo install omitted --root")?;
    let source = PathBuf::from(
        env::var_os("FAKE_EXPORTER_SOURCE").ok_or("FAKE_EXPORTER_SOURCE is required")?,
    );
    let name = source
        .file_name()
        .ok_or("exporter source has no file name")?;
    let destination = PathBuf::from(root).join("bin").join(name);
    fs::create_dir_all(destination.parent().unwrap())?;
    fs::copy(&source, &destination)?;

    println!("   Compiling revault_migrate_vault_v1 v0.0.1");
    println!("   Installed package `revault_migrate_vault_v1 v0.0.1`");
    eprintln!(
        "warning: be sure to add `{}` to your PATH to be able to run the installed binaries",
        destination.parent().unwrap().display()
    );
    Ok(())
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
