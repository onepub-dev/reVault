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
    let inventory = operation_inventory(&repository)?;
    if operations != inventory {
        let missing: Vec<_> = operations.difference(&inventory).cloned().collect();
        let extra: Vec<_> = inventory.difference(&operations).cloned().collect();
        return Err(format!(
            "ABI inventory mismatch; missing [{}], extra [{}]",
            missing.join(", "),
            extra.join(", ")
        )
        .into());
    }
    check_rust_api_coverage(&repository, &operations)?;
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
    check_package_documentation(&repository)?;
    check_schema_documentation(&repository)?;
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
        "verified {} ABI declarations and {} operations across all generated binding surfaces",
        declarations.len(),
        operations.len()
    );
    Ok(())
}

fn check_schema_documentation(repository: &Path) -> Result {
    let relative = "bindings/proto/revault_bindings.proto";
    let schema = fs::read_to_string(repository.join(relative))?;
    let mut previous = "";
    let mut declaration_depth = 0usize;
    for (index, line) in schema.lines().enumerate() {
        let trimmed = line.trim();
        let opens_declaration = trimmed.starts_with("message ") || trimmed.starts_with("enum ");
        let is_member = declaration_depth > 0 && trimmed.contains(" = ") && trimmed.ends_with(';');
        if (opens_declaration || is_member) && !previous.starts_with("//") {
            return Err(format!(
                "{relative}:{} public schema declaration lacks documentation: {trimmed}",
                index + 1
            )
            .into());
        }
        if opens_declaration || declaration_depth > 0 {
            declaration_depth += trimmed.matches('{').count();
        }
        declaration_depth = declaration_depth.saturating_sub(trimmed.matches('}').count());
        if !trimmed.is_empty() {
            previous = trimmed;
        }
    }
    println!("verified every generated Protobuf model and field is documented");
    Ok(())
}

fn check_package_documentation(repository: &Path) -> Result {
    const README_URL: &str = "https://github.com/onepub-dev/reVault#readme";
    let package_surfaces = [
        "rust/revault_bindings/revault_api.h",
        "bindings/cpp/revault_api.hpp",
        "bindings/csharp/Vault.cs",
        "bindings/dart/lib/revault_api.dart",
        "bindings/go/doc.go",
        "bindings/java/src/com/onepub/revault/package-info.java",
        "bindings/javascript/index.d.ts",
        "bindings/kotlin/src/main/kotlin/Vault.kt",
        "bindings/lua/revault_api.lua",
        "bindings/php/src/Vault.php",
        "bindings/python/revault_api/__init__.py",
        "bindings/ruby/revault_api.rb",
        "bindings/rust/src/lib.rs",
        "bindings/swift/Sources/RevaultAPI/RevaultAPI.docc/RevaultAPI.md",
        "bindings/typescript/index.ts",
        "bindings/wasm/index.d.ts",
    ];
    for relative in package_surfaces {
        require_features(
            &repository.join(relative),
            &[README_URL],
            &format!("{relative} package documentation"),
        )?;
    }
    println!(
        "verified package overviews and repository README links for {} language bindings",
        package_surfaces.len()
    );
    Ok(())
}

fn check_rust_api_coverage(repository: &Path, operations: &BTreeSet<String>) -> Result {
    let mappings = read_tsv(
        &repository.join("bindings/api/rust-to-abi.tsv"),
        "Rust-to-ABI mapping",
    )?;
    let exclusions = read_tsv(
        &repository.join("bindings/api/rust-only-exclusions.tsv"),
        "Rust-only exclusion",
    )?;
    let mut reviewed = BTreeSet::new();
    for fields in &mappings {
        if fields.len() != 3 || fields[2].trim().is_empty() {
            return Err(format!("invalid Rust-to-ABI mapping row: {}", fields.join("\t")).into());
        }
        if !reviewed.insert(rust_item_name(&fields[0]).to_string()) {
            return Err(format!("duplicate reviewed Rust API name: {}", fields[0]).into());
        }
        for symbol in fields[1].split(',').map(str::trim) {
            if !operations.contains(symbol) {
                return Err(format!("{} maps to unknown ABI operation {symbol}", fields[0]).into());
            }
        }
    }
    for fields in &exclusions {
        if fields.len() != 2 || fields[1].trim().is_empty() {
            return Err(format!("invalid Rust-only exclusion row: {}", fields.join("\t")).into());
        }
        if !reviewed.insert(rust_item_name(&fields[0]).to_string()) {
            return Err(format!("duplicate reviewed Rust API name: {}", fields[0]).into());
        }
    }

    let public_functions = public_rust_function_names(repository)?;
    let missing: Vec<_> = public_functions.difference(&reviewed).cloned().collect();
    let stale: Vec<_> = reviewed.difference(&public_functions).cloned().collect();
    if !missing.is_empty() || !stale.is_empty() {
        return Err(format!(
            "Rust public API review mismatch; unreviewed [{}], stale [{}]",
            missing.join(", "),
            stale.join(", ")
        )
        .into());
    }
    println!(
        "verified {} unique public Rust function names have ABI mappings or reviewed Rust-only exclusions",
        public_functions.len()
    );
    Ok(())
}

fn read_tsv(path: &Path, label: &str) -> Result<Vec<Vec<String>>> {
    let source = fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for line in source
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
    {
        let fields: Vec<_> = line.split('\t').map(str::to_string).collect();
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err(format!("{label} contains an empty field: {line}").into());
        }
        rows.push(fields);
    }
    Ok(rows)
}

fn rust_item_name(item: &str) -> &str {
    item.rsplit("::").next().unwrap_or(item)
}

fn public_rust_function_names(repository: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for crate_path in ["rust/revault_lockbox_api/src", "rust/revault_vault_api/src"] {
        for entry in walkdir::WalkDir::new(repository.join(crate_path)) {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("rs")
                || path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.contains("test"))
            {
                continue;
            }
            for line in fs::read_to_string(path)?.lines() {
                let Some(rest) = line.trim_start().strip_prefix("pub fn ") else {
                    continue;
                };
                let name: String = rest
                    .chars()
                    .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                    .collect();
                if !name.is_empty() {
                    names.insert(name);
                }
            }
        }
    }
    Ok(names)
}

fn operation_inventory(repository: &Path) -> Result<BTreeSet<String>> {
    let source = fs::read_to_string(repository.join("bindings/e2e/operations.tsv"))?;
    let mut operations = BTreeSet::new();
    for line in source
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err(format!("invalid operation manifest row: {line}").into());
        }
        if !operations.insert(fields[0].to_string()) {
            return Err(format!("duplicate operation manifest symbol: {}", fields[0]).into());
        }
    }
    Ok(operations)
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
