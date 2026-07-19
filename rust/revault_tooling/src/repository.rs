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
        "bindings/dart/lib/src/revault_native.dart",
        "bindings/go/revault_native.go",
        "bindings/java/src/com/onepub/revault/RevaultAbiSymbols.java",
        "bindings/php/src/BindingOperations.php",
        "bindings/python/revault_api/_revault_native.py",
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
    check_registry_quality(&repository)?;
    check_schema_documentation(&repository)?;
    check_public_api_documentation(&repository)?;
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

fn check_registry_quality(repository: &Path) -> Result {
    let checks: &[(&str, &[&str], &str)] = &[
        (
            "bindings/dart/pubspec.yaml",
            &["dev_dependencies:", "topics:", "platforms:"],
            "pub.dev metadata",
        ),
        (
            "bindings/csharp/RevaultBindings.csproj",
            &[
                "<PackageTags>",
                "<PackageReadmeFile>",
                "<RepositoryType>git",
            ],
            "NuGet metadata",
        ),
        (
            "bindings/javascript/package.json",
            &["\"homepage\"", "\"bugs\"", "\"keywords\"", "\"engines\""],
            "npm metadata",
        ),
        (
            "bindings/wasm/package.json",
            &["\"homepage\"", "\"bugs\"", "\"keywords\"", "\"engines\""],
            "WASM npm metadata",
        ),
        (
            "bindings/python/pyproject.toml",
            &[
                "keywords =",
                "classifiers =",
                "Documentation =",
                "Security =",
            ],
            "PyPI metadata",
        ),
        (
            "bindings/java/build.gradle",
            &["connection =", "developerConnection =", "organizationUrl ="],
            "Java Maven Central metadata",
        ),
        (
            "bindings/kotlin/build.gradle.kts",
            &[
                "connection.set",
                "developerConnection.set",
                "organizationUrl.set",
            ],
            "Kotlin Maven Central metadata",
        ),
        (
            "bindings/php/composer.json",
            &["\"keywords\"", "\"support\"", "\"security\""],
            "Packagist metadata",
        ),
        (
            "bindings/ruby/revault_api.gemspec",
            &["spec.email", "spec.metadata", "source_code_uri"],
            "RubyGems metadata",
        ),
        (
            "bindings/lua/revault_api-0.2.0-1.rockspec",
            &["detailed =", "homepage =", "license ="],
            "LuaRocks metadata",
        ),
        (
            "bindings/go/revault.go",
            &["// Package revault"],
            "pkg.go.dev package overview",
        ),
        (
            "bindings/swift/Package.swift",
            &["platforms:", ".testTarget("],
            "Swift Package Index metadata",
        ),
        (
            "bindings/swift/CModule/revault_api.h",
            &["../../../rust/revault_bindings/revault_api.h"],
            "Swift source-tree C ABI bridge",
        ),
        (
            "bindings/rust/Cargo.toml",
            &["keywords =", "categories =", "readme ="],
            "crates.io metadata",
        ),
        (
            "bindings/release/package-managers/conan/conanfile.py",
            &["description =", "topics =", "def package_info"],
            "Conan metadata",
        ),
        (
            "bindings/release/package-managers/vcpkg/vcpkg.json",
            &["\"description\"", "\"homepage\"", "\"supports\""],
            "vcpkg metadata",
        ),
    ];
    for (relative, features, label) in checks {
        require_features(&repository.join(relative), features, label)?;
    }
    for relative in [
        "bindings/dart/example/revault_api_example.dart",
        "bindings/swift/Tests/RevaultAPITests/RevaultAPITests.swift",
        "SECURITY.md",
    ] {
        if !repository.join(relative).is_file() {
            return Err(format!("package quality file is missing: {relative}").into());
        }
    }
    println!("verified registry quality metadata for 15 package surfaces");
    Ok(())
}

#[derive(Clone, Copy)]
enum DocumentationSurface {
    C,
    CSharp,
    Go,
    Java,
    Kotlin,
    Lua,
    Php,
    Python,
    Ruby,
    Swift,
    TypeScript,
}

fn check_public_api_documentation(repository: &Path) -> Result {
    let surfaces = [
        (
            "rust/revault_bindings/revault_api.h",
            DocumentationSurface::C,
        ),
        ("bindings/csharp/Vault.cs", DocumentationSurface::CSharp),
        (
            "bindings/java/src/com/onepub/revault/Revault.java",
            DocumentationSurface::Java,
        ),
        ("bindings/go/revault.go", DocumentationSurface::Go),
        (
            "bindings/javascript/index.d.ts",
            DocumentationSurface::TypeScript,
        ),
        (
            "bindings/javascript/index.js",
            DocumentationSurface::TypeScript,
        ),
        (
            "bindings/kotlin/src/main/kotlin/Vault.kt",
            DocumentationSurface::Kotlin,
        ),
        ("bindings/php/src/Vault.php", DocumentationSurface::Php),
        (
            "bindings/python/revault_api/facade.py",
            DocumentationSurface::Python,
        ),
        (
            "bindings/ruby/lib/revault/vault.rb",
            DocumentationSurface::Ruby,
        ),
        ("bindings/lua/revault_api.lua", DocumentationSurface::Lua),
        (
            "bindings/swift/Sources/RevaultAPI/RevaultAPI.swift",
            DocumentationSurface::Swift,
        ),
        (
            "bindings/typescript/index.ts",
            DocumentationSurface::TypeScript,
        ),
        ("bindings/wasm/index.d.ts", DocumentationSurface::TypeScript),
        ("bindings/wasm/index.js", DocumentationSurface::TypeScript),
    ];
    let mut declarations = 0usize;
    for (relative, style) in surfaces {
        declarations += check_documented_surface(repository, relative, style)?;
    }
    declarations += check_cpp_documentation(repository, "bindings/cpp/revault_api.hpp")?;
    println!("verified documentation for {declarations} public binding declarations");
    Ok(())
}

fn check_cpp_documentation(repository: &Path, relative: &str) -> Result<usize> {
    let source = fs::read_to_string(repository.join(relative))?;
    let lines: Vec<_> = source.lines().collect();
    let detail_start = lines
        .iter()
        .position(|line| line.trim() == "namespace detail {");
    let detail_end = detail_start.and_then(|start| {
        lines[start + 1..]
            .iter()
            .position(|line| line.contains("namespace detail"))
            .map(|offset| start + 1 + offset)
    });
    let mut depth = 0isize;
    let mut classes: Vec<(isize, bool)> = Vec::new();
    let mut pending_signature = false;
    let mut count = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        classes.retain(|(class_depth, _)| depth >= *class_depth);
        let internal_detail = detail_start
            .zip(detail_end)
            .is_some_and(|(start, end)| index >= start && index <= end);
        let class_declaration = !internal_detail
            && (trimmed.starts_with("class ") || trimmed.starts_with("struct "))
            && trimmed.contains('{');
        if class_declaration {
            count += 1;
            if !previous_significant_line(&lines, index)
                .is_some_and(|previous| previous.starts_with("/**") || previous.starts_with('*'))
            {
                return Err(format!(
                    "{relative}:{} public type lacks documentation: {trimmed}",
                    index + 1
                )
                .into());
            }
            let public = trimmed.starts_with("struct ");
            classes.push((depth + brace_delta(trimmed), public));
        }
        if let Some((class_depth, public)) = classes.last_mut() {
            if depth == *class_depth {
                if trimmed == "public:" {
                    *public = true;
                } else if trimmed == "private:" || trimmed == "protected:" {
                    *public = false;
                }
            }
        }
        let continuation = pending_signature;
        if continuation && (trimmed.contains('{') || trimmed.ends_with(';')) {
            pending_signature = false;
        }
        let public_member = classes
            .last()
            .is_some_and(|(class_depth, public)| depth == *class_depth && *public);
        let declaration = !internal_detail
            && public_member
            && !continuation
            && !trimmed.starts_with(':')
            && !trimmed.starts_with('}')
            && !trimmed.starts_with("friend ")
            && !trimmed.starts_with("using ")
            && (trimmed.contains('(') || trimmed.ends_with(';'));
        if declaration {
            count += 1;
            if !previous_significant_line(&lines, index)
                .is_some_and(|previous| previous.starts_with("/**") || previous.starts_with('*'))
            {
                return Err(format!(
                    "{relative}:{} public declaration lacks documentation: {trimmed}",
                    index + 1
                )
                .into());
            }
            pending_signature =
                trimmed.contains('(') && !trimmed.contains('{') && !trimmed.ends_with(';');
        }
        depth += brace_delta(trimmed);
    }
    Ok(count)
}

fn brace_delta(line: &str) -> isize {
    line.bytes().filter(|byte| *byte == b'{').count() as isize
        - line.bytes().filter(|byte| *byte == b'}').count() as isize
}

fn check_documented_surface(
    repository: &Path,
    relative: &str,
    style: DocumentationSurface,
) -> Result<usize> {
    let source = fs::read_to_string(repository.join(relative))?;
    let lines: Vec<_> = source.lines().collect();
    let lua_facade_start = lines
        .iter()
        .position(|line| line.contains("local Vault = owned(\"Vault\")"));
    let mut count = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let declaration = match style {
            DocumentationSurface::C => trimmed.ends_with(");") && trimmed.contains('('),
            DocumentationSurface::CSharp => trimmed.starts_with("public "),
            DocumentationSurface::Java => {
                trimmed.starts_with("public ") || trimmed.starts_with("@Override public ")
            }
            DocumentationSurface::Kotlin => trimmed.starts_with("typealias "),
            DocumentationSurface::Go => exported_go_declaration(trimmed),
            DocumentationSurface::Lua => {
                lua_facade_start.is_some_and(|start| index >= start)
                    && trimmed.starts_with("function ")
                    && !trimmed.starts_with("function Operations:")
                    && !trimmed.starts_with("function Operations.")
            }
            DocumentationSurface::Php => trimmed.starts_with("public function "),
            DocumentationSurface::Python => trimmed
                .strip_prefix("def _")
                .and_then(|rest| rest.chars().next())
                .is_some_and(|character| character.is_ascii_uppercase()),
            DocumentationSurface::Ruby => {
                trimmed.starts_with("class ") || trimmed.starts_with("def ")
            }
            DocumentationSurface::Swift => trimmed.starts_with("public "),
            DocumentationSurface::TypeScript => {
                trimmed.starts_with("export ")
                    || (line.starts_with("  ")
                        && !line.starts_with("    ")
                        && (trimmed.contains('(') || trimmed.starts_with("readonly ")))
                        && ![
                            "if ", "for ", "while ", "return ", "throw ", "const ", "let ",
                        ]
                        .iter()
                        .any(|prefix| trimmed.starts_with(prefix))
            }
        };
        if !declaration {
            continue;
        }
        count += 1;
        let documented = match style {
            DocumentationSurface::Python => lines
                .get(index + 1)
                .is_some_and(|next| next.trim_start().starts_with("\"\"\"")),
            _ => previous_significant_line(&lines, index).is_some_and(|previous| match style {
                DocumentationSurface::C
                | DocumentationSurface::Java
                | DocumentationSurface::Kotlin
                | DocumentationSurface::Php
                | DocumentationSurface::TypeScript => {
                    previous.starts_with("/**") || previous.starts_with('*')
                }
                DocumentationSurface::CSharp | DocumentationSurface::Swift => {
                    previous.starts_with("///")
                }
                DocumentationSurface::Go => previous.starts_with("//"),
                DocumentationSurface::Lua => previous.starts_with("---"),
                DocumentationSurface::Ruby => previous.starts_with('#'),
                DocumentationSurface::Python => unreachable!(),
            }),
        };
        if !documented {
            return Err(format!(
                "{relative}:{} public declaration lacks documentation: {trimmed}",
                index + 1
            )
            .into());
        }
    }
    Ok(count)
}

fn previous_significant_line<'a>(lines: &'a [&str], index: usize) -> Option<&'a str> {
    lines[..index]
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| {
            !line.is_empty()
                && !line.starts_with("#[")
                && !line.starts_with('[')
                && !line.starts_with('@')
        })
}

fn exported_go_declaration(line: &str) -> bool {
    let name = if let Some(rest) = line.strip_prefix("type ") {
        rest.split_whitespace().next()
    } else if let Some(mut rest) = line.strip_prefix("func ") {
        if rest.starts_with('(') {
            let Some(end) = rest.find(')') else {
                return false;
            };
            rest = rest[end + 1..].trim_start();
        }
        rest.split(|character: char| character == '(' || character.is_whitespace())
            .next()
    } else {
        None
    };
    name.and_then(|value| value.chars().next())
        .is_some_and(|character| character.is_ascii_uppercase())
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
    for generated in [
        "revault_bindings.pb.dart",
        "revault_bindings.pbenum.dart",
        "revault_bindings.pbjson.dart",
    ] {
        let path = repository
            .join("bindings/dart/lib/src/generated")
            .join(generated);
        let source = fs::read_to_string(&path)?;
        if !source.contains("// ignore_for_file: public_member_api_docs") {
            let marker = "// Generated from revault_bindings.proto.\n";
            let source = source.replacen(
                marker,
                &format!("{marker}\n// ignore_for_file: public_member_api_docs\n"),
                1,
            );
            fs::write(path, source)?;
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
