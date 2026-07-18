use crate::Result;
use clap::{Args, Subcommand};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum BindingsCommand {
    /// Verify every ABI operation is present in every generated surface.
    Check(Check),
    /// Regenerate Protobuf models through pinned ecosystem generators.
    GenerateProtobuf(GenerateProtobuf),
}

#[derive(Args)]
pub struct Check {
    #[arg(long, default_value = ".")]
    repository: PathBuf,
}

#[derive(Args)]
pub struct GenerateProtobuf {
    #[arg(long, default_value = ".")]
    repository: PathBuf,
}

pub fn run(command: BindingsCommand) -> Result {
    match command {
        BindingsCommand::Check(args) => check(&args.repository),
        BindingsCommand::GenerateProtobuf(args) => generate_protobuf(&args.repository),
    }
}

fn check(repository: &Path) -> Result {
    let repository = repository.canonicalize()?;
    let header = fs::read_to_string(repository.join("rust/revault_bindings/revault_api.h"))?;
    let declarations = declarations(&header);
    let operations: BTreeSet<_> = declarations
        .iter()
        .filter(|name| name.as_str() != "api_abi_version")
        .cloned()
        .collect();
    if declarations.len() != 211 || operations.len() != 210 {
        return Err(format!(
            "expected 211 ABI declarations and 210 domain operations, found {} and {}",
            declarations.len(),
            operations.len()
        )
        .into());
    }
    let complete_abi = [
        "bindings/csharp/RevaultNative.cs",
        "bindings/dart/lib/revault_native.dart",
        "bindings/go/revault_native.go",
        "bindings/java/src/com/onepub/revault/RevaultAbiSymbols.java",
        "bindings/php/src/BindingOperations.php",
        "bindings/python/revault_api/revault_native.py",
    ];
    for relative in complete_abi {
        require_names(&repository.join(relative), &declarations, relative)?;
    }
    let complete_routes = [
        "bindings/cpp/revault_api.hpp",
        "bindings/csharp/BindingOperations.cs",
        "bindings/dart/lib/src/binding_operations.dart",
        "bindings/go/revault.go",
        "bindings/java/src/com/onepub/revault/BindingOperations.java",
        "bindings/javascript/native.js",
        "bindings/lua/revault_api.lua",
        "bindings/php/src/BindingOperations.php",
        "bindings/python/revault_api/facade.py",
        "bindings/ruby/lib/revault/binding_operations.rb",
        "bindings/swift/Sources/RevaultAPI/RevaultAPI.swift",
    ];
    for relative in complete_routes {
        require_names(&repository.join(relative), &operations, relative)?;
    }
    let messages = schema_messages(&repository)?;
    for relative in [
        "bindings/cpp/generated/revault_bindings.pb.h",
        "bindings/csharp/Generated/RevaultBindings.cs",
        "bindings/dart/lib/src/generated/revault_bindings.pb.dart",
        "bindings/go/messages/revault_bindings.pb.go",
        "bindings/java/generated/revault/bindings/RevaultBindings.java",
        "bindings/javascript/generated/messages.js",
        "bindings/lua/revault_api.lua",
        "bindings/ruby/generated/revault_bindings_pb.rb",
        "bindings/swift/Sources/RevaultAPI/revault_bindings.pb.swift",
    ] {
        require_names(&repository.join(relative), &messages, relative)?;
    }
    let php_models: BTreeSet<_> =
        fs::read_dir(repository.join("bindings/php/generated/Revault/Bindings"))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .collect();
    if !messages.is_subset(&php_models) {
        let missing: Vec<_> = messages.difference(&php_models).cloned().collect();
        return Err(format!("PHP generated models are missing: {}", missing.join(", ")).into());
    }
    require_features(
        &repository.join("bindings/wasm/index.js"),
        &[
            "class Vault",
            "new Runtime()",
            "runtime.before_call('buffer_free')",
            "return wrap(result)",
        ],
        "hosted WebAssembly facade",
    )?;
    require_features(
        &repository.join("rust/revault_wasm_bindings/src/lib.rs"),
        &["pub struct Runtime", "before_call", "operations.tsv"],
        "WebAssembly dispatcher",
    )?;
    check_results(&repository, &operations)?;
    println!(
        "verified 211 ABI declarations and 210 operations across all generated binding surfaces"
    );
    Ok(())
}

fn require_features(path: &Path, features: &[&str], label: &str) -> Result {
    let source = fs::read_to_string(path)?;
    let missing: Vec<_> = features
        .iter()
        .filter(|feature| !source.contains(**feature))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(format!("{label} is missing: {}", missing.join(", ")).into());
    }
    Ok(())
}

fn declarations(header: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in header
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(");"))
    {
        let Some(open) = line.find('(') else {
            continue;
        };
        let prefix = line[..open].trim();
        let name = prefix
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_start_matches('*');
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        {
            names.insert(name.to_owned());
        }
    }
    names
}

fn require_names(path: &Path, names: &BTreeSet<String>, label: &str) -> Result {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read {label} as UTF-8: {error}"))?;
    let missing: Vec<_> = names
        .iter()
        .filter(|name| !source.contains(name.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{label} is missing {} operations: {}",
            missing.len(),
            missing.join(", ")
        )
        .into());
    }
    Ok(())
}

fn check_results(repository: &Path, _operations: &BTreeSet<String>) -> Result {
    let source = fs::read_to_string(repository.join("bindings/proto/results.tsv"))?;
    let mut rows = BTreeMap::new();
    for line in source
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err(format!("invalid result manifest row: {line}").into());
        }
        if rows.insert(fields[0], (fields[1], fields[2])).is_some() {
            return Err(format!("duplicate result manifest operation: {}", fields[0]).into());
        }
    }
    let inventory = fs::read_to_string(repository.join("bindings/e2e/operations.tsv"))?;
    let buffer_operations: BTreeSet<_> = inventory
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            (fields.len() == 4 && fields[2] == "RevaultBuffer").then(|| fields[0].to_string())
        })
        .collect();
    let classified: BTreeSet<_> = rows.keys().map(|name| name.to_string()).collect();
    if classified != buffer_operations {
        let missing: Vec<_> = buffer_operations.difference(&classified).cloned().collect();
        let extra: Vec<_> = classified.difference(&buffer_operations).cloned().collect();
        return Err(format!(
            "result manifest mismatch; missing [{}], extra [{}]",
            missing.join(", "),
            extra.join(", ")
        )
        .into());
    }
    let messages = schema_messages(repository)?;
    for (symbol, (encoding, message)) in &rows {
        match *encoding {
            "lbwf" if !messages.contains(*message) => {
                return Err(format!("{symbol}: unknown protobuf message {message}").into())
            }
            "raw" if !matches!(*message, "bytes" | "utf8") => {
                return Err(format!("{symbol}: invalid raw result type {message}").into())
            }
            "lbwf" | "raw" => {}
            _ => return Err(format!("{symbol}: invalid encoding {encoding}").into()),
        }
    }
    println!("verified {} native buffer result encodings", rows.len());
    Ok(())
}

fn schema_messages(repository: &Path) -> Result<BTreeSet<String>> {
    let schema = fs::read_to_string(repository.join("bindings/proto/revault_bindings.proto"))?;
    Ok(schema
        .lines()
        .filter_map(|line| line.trim().strip_prefix("message "))
        .filter_map(|tail| tail.split_whitespace().next())
        .map(str::to_string)
        .collect())
}

fn generate_protobuf(repository: &Path) -> Result {
    let repository = repository.canonicalize()?;
    let proto = repository.join("bindings/proto/revault_bindings.proto");
    let include = repository.join("bindings/proto");
    let commands: Vec<(&str, Vec<String>)> = vec![
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--python_out={}",
                    repository.join("bindings/python/revault_api").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--cpp_out={}",
                    repository.join("bindings/cpp/generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--csharp_out={}",
                    repository.join("bindings/csharp/Generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--ruby_out={}",
                    repository.join("bindings/ruby/generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--dart_out={}",
                    repository.join("bindings/dart/lib/src/generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--go_out={}",
                    repository.join("bindings/go/messages").display()
                ),
                "--go_opt=paths=source_relative".into(),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                "--swift_opt=Visibility=Public".into(),
                format!(
                    "--swift_out={}",
                    repository
                        .join("bindings/swift/Sources/RevaultAPI")
                        .display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--descriptor_set_out={}",
                    repository
                        .join("bindings/lua/revault_bindings.pb")
                        .display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--java_out={}",
                    repository.join("bindings/java/generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
        (
            "protoc",
            vec![
                format!("-I{}", include.display()),
                format!(
                    "--php_out={}",
                    repository.join("bindings/php/generated").display()
                ),
                proto.display().to_string(),
            ],
        ),
    ];
    for (program, arguments) in commands {
        let status = Command::new(program).args(arguments).status()?;
        if !status.success() {
            return Err(format!("{program} failed with {status}").into());
        }
    }
    println!("regenerated canonical Protobuf models; ecosystem-specific generated facades are validated by bindings check");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_pointer_and_value_declarations() {
        let names = declarations("uint32_t api_abi_version(void);\nvoid * lockbox_create(void);\nRevaultBuffer buffer_last_error(void);");
        assert_eq!(names.len(), 3);
        assert!(names.contains("lockbox_create"));
    }
}
