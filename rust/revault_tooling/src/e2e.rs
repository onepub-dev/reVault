use crate::Result;
use clap::{Args, Subcommand};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const LANGUAGES: [&str; 16] = [
    "c",
    "cpp",
    "csharp",
    "dart",
    "go",
    "java",
    "javascript",
    "kotlin",
    "lua",
    "php",
    "python",
    "ruby",
    "rust",
    "swift",
    "typescript",
    "wasm",
];

#[derive(Subcommand)]
pub enum E2eCommand {
    /// Verify complete per-language operation and artifact records.
    VerifyResults(VerifyResults),
    /// Verify every directed non-self archive and vault path.
    VerifyInterop(VerifyInterop),
    /// Regenerate the operation inventory from the C header.
    GenerateInventory(GenerateInventory),
    /// Run every Linux package suite followed by canonical interoperability.
    Matrix(Matrix),
    /// Emit the claimed language/target GitHub Actions matrix.
    MatrixJson,
    /// Run all foreign-producer checks for one installed consumer.
    InteropConsumer(InteropConsumer),
    /// Emit machine-verifiable evidence for an installed native artifact.
    Evidence(Evidence),
    /// Install a canonical archive and run the complete C ABI suite on its host target.
    NativeConformance(NativeConformance),
    /// Run the source-native Rust lockbox and vault public API suites.
    RustSourceConformance(RustSourceConformance),
    /// Install one assembled ecosystem package and run its complete host suite.
    PackageConformance(crate::package_conformance::PackageConformance),
    /// Execute one installed language suite inside its service-enabled image.
    Container(Container),
}

#[derive(Args)]
pub struct VerifyResults {
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "c,cpp,csharp,dart,go,java,javascript,kotlin,lua,php,python,ruby,rust,swift,typescript,wasm"
    )]
    languages: Vec<String>,
    #[arg(long, default_value = "bindings/e2e/operations.tsv")]
    operations: PathBuf,
    #[arg(required = true)]
    results: Vec<PathBuf>,
}

#[derive(Args)]
pub struct VerifyInterop {
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "c,cpp,csharp,dart,go,java,javascript,kotlin,lua,php,python,ruby,rust,swift,typescript,wasm"
    )]
    languages: Vec<String>,
    #[arg()]
    results: Vec<PathBuf>,
    #[arg(long)]
    results_dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct GenerateInventory {
    #[arg(long, default_value = "rust/revault_bindings/revault_api.h")]
    header: PathBuf,
    #[arg(long, default_value = "bindings/e2e/operations.tsv")]
    output: PathBuf,
    #[arg(long)]
    check: bool,
}

#[derive(Args)]
pub struct Matrix {
    #[arg(long, default_value = "bindings/e2e/compose.yaml")]
    compose: PathBuf,
    #[arg(long)]
    skip_interop: bool,
}

#[derive(Args)]
pub struct InteropConsumer {
    #[arg(long)]
    consumer: String,
    #[arg(long, default_value = "/artifacts")]
    artifacts: PathBuf,
}

#[derive(Args)]
pub struct Evidence {
    #[arg(long)]
    language: String,
    #[arg(long)]
    target: String,
    #[arg(long)]
    kind: String,
    #[arg(long)]
    root: PathBuf,
    #[arg(long)]
    file: String,
}

#[derive(Args)]
pub struct NativeConformance {
    #[arg(long)]
    archive: PathBuf,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
    #[arg(long)]
    work: PathBuf,
}

#[derive(Args)]
pub struct RustSourceConformance {
    #[arg(long, default_value = ".")]
    pub(crate) repository: PathBuf,
    #[arg(long)]
    pub(crate) target: String,
    #[arg(long)]
    pub(crate) source_archive: Option<PathBuf>,
}

#[derive(Args)]
pub struct Container {
    #[arg(long)]
    pub(crate) language: String,
}

pub fn run(command: E2eCommand) -> Result {
    match command {
        E2eCommand::VerifyResults(args) => verify_results(args),
        E2eCommand::VerifyInterop(args) => verify_interop(args),
        E2eCommand::GenerateInventory(args) => generate_inventory(args),
        E2eCommand::Matrix(args) => matrix(args),
        E2eCommand::MatrixJson => matrix_json(),
        E2eCommand::InteropConsumer(args) => interop_consumer(args),
        E2eCommand::Evidence(args) => evidence(args),
        E2eCommand::NativeConformance(args) => native_conformance(args),
        E2eCommand::RustSourceConformance(args) => rust_source_conformance(args),
        E2eCommand::PackageConformance(args) => crate::package_conformance::run(args),
        E2eCommand::Container(args) => container(args),
    }
}

pub(crate) fn container(args: Container) -> Result {
    selected_languages(std::slice::from_ref(&args.language))?;
    if std::env::var_os("REVAULT_LIBRARY").is_some() {
        return Err("REVAULT_LIBRARY is forbidden in installed-package conformance".into());
    }
    if args.language == "rust" {
        rust_source_conformance(RustSourceConformance {
            repository: PathBuf::from("."),
            target: "linux-x86_64-gnu".into(),
            source_archive: None,
        })?;
        let output = Command::new("/opt/revault-rust-conformance").output()?;
        if !output.status.success() {
            return Err(format!(
                "Rust artifact conformance failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        verify_rust_artifacts(&output.stdout)?;
        return Ok(());
    }
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/revault-runtime"));
    fs::create_dir_all(&runtime)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700))?;
    }
    if cfg!(target_os = "linux") && std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        let status = Command::new("dbus-run-session")
            .arg("--")
            .arg(std::env::current_exe()?)
            .args(["e2e", "container", "--language", &args.language])
            .status()?;
        if !status.success() {
            return Err(format!("service session failed with {status}").into());
        }
        return Ok(());
    }
    let service_env = linux_secret_service_env()?;
    let results = std::env::temp_dir().join(format!("{}-results.tsv", args.language));
    let native = std::env::temp_dir().join(format!("{}-native.tsv", args.language));
    let root = PathBuf::from(std::env::var("REVAULT_E2E_NATIVE_ROOT")?);
    let file = std::env::var("REVAULT_E2E_NATIVE_FILE")?;
    let kind = std::env::var("REVAULT_E2E_NATIVE_KIND")?;
    fs::create_dir_all(&root)?;
    let mut combined = Vec::new();
    for invocation in invocations(&args.language) {
        prepare_invocation_directories(&invocation.env)?;
        let mut server = if matches!(args.language.as_str(), "lua" | "swift")
            && invocation.args.last().is_some_and(|arg| arg == "--agent")
        {
            let mut server_args = invocation.args.clone();
            *server_args.last_mut().unwrap() = "--serve-agent".into();
            let mut child = Command::new(&invocation.program);
            child
                .args(server_args)
                .envs(&service_env)
                .envs(&invocation.env)
                .env_remove("REVAULT_LIBRARY")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            Some(child.spawn()?)
        } else {
            None
        };
        let mut command = Command::new(&invocation.program);
        command
            .args(&invocation.args)
            .envs(&service_env)
            .env_remove("REVAULT_LIBRARY");
        for (key, value) in invocation.env {
            command.env(key, value);
        }
        let output = command.output()?;
        if !output.status.success() {
            return Err(format!(
                "{} conformance failed: {}",
                args.language,
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        if let Some(mut child) = server.take() {
            let status = child.wait()?;
            if !status.success() {
                return Err(format!("{} agent server failed with {status}", args.language).into());
            }
        }
        combined.extend_from_slice(&output.stdout);
    }
    fs::write(&results, combined)?;
    let evidence_output = Command::new(std::env::current_exe()?)
        .args([
            "e2e",
            "evidence",
            "--language",
            &args.language,
            "--target",
            &std::env::var("REVAULT_E2E_TARGET").unwrap_or_else(|_| "linux-x86_64-gnu".into()),
            "--kind",
            &kind,
            "--root",
        ])
        .arg(root)
        .args(["--file", &file])
        .output()?;
    if !evidence_output.status.success() {
        return Err(String::from_utf8_lossy(&evidence_output.stderr)
            .into_owned()
            .into());
    }
    fs::write(&native, evidence_output.stdout)?;
    std::env::set_var("REVAULT_REQUIRE_INSTALLED_NATIVE", "1");
    verify_results(VerifyResults {
        languages: vec![args.language],
        operations: PathBuf::from("bindings/e2e/operations.tsv"),
        results: vec![results, native],
    })
}

fn prepare_invocation_directories(environment: &BTreeMap<String, String>) -> Result {
    for name in ["LOCKBOX_VAULT_DIR", "LOCKBOX_SESSION_AGENT_DIR"] {
        let Some(path) = environment.get(name) else {
            continue;
        };
        fs::create_dir_all(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

struct Invocation {
    program: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
}

fn invocations(language: &str) -> Vec<Invocation> {
    let (program, base): (&str, &[&str]) = match language {
        "c" => ("/tmp/revault-c-conformance/revault_c_conformance", &[]),
        "cpp" => ("/tmp/revault-cpp-conformance/revault_cpp_conformance", &[]),
        "csharp" => ("dotnet", &["/opt/revault-csharp/Conformance.dll"]),
        "dart" => ("/opt/revault-dart/conformance", &[]),
        "go" => ("/tmp/revault-go-conformance", &[]),
        "java" => ("java", &["--enable-native-access=ALL-UNNAMED", "-Djava.io.tmpdir=/tmp/revault-java-extract", "-Drevault.keepExtracted=true", "-cp", "/opt/revault-java:/root/.m2/repository/dev/onepub/revault-api/0.2.0/revault-api-0.2.0.jar:/root/.m2/repository/com/google/flatbuffers/flatbuffers-java/25.2.10/flatbuffers-java-25.2.10.jar", "com.onepub.revault.e2e.Conformance"]),
        "javascript" | "wasm" => ("node", &["bindings/e2e/javascript/conformance.js"]),
        "kotlin" => ("bindings/e2e/kotlin/build/install/revault-api-kotlin-conformance/bin/revault-api-kotlin-conformance", &[]),
        "lua" => ("luajit", &["bindings/e2e/lua/conformance.lua"]),
        "php" => ("php", &["-d", "ffi.enable=true", "bindings/e2e/php/conformance.php"]),
        "python" => ("python3", &["bindings/e2e/python/conformance.py"]),
        "ruby" => ("ruby", &["bindings/e2e/ruby/conformance.rb"]),
        "rust" => ("/opt/revault-rust-conformance", &[]),
        "swift" => ("/tmp/packages/swift/.build/release/revault-swift-conformance", &[]),
        "typescript" => ("/opt/revault-ts-consumer/node_modules/.bin/tsx", &["bindings/e2e/typescript/conformance.ts"]),
        _ => unreachable!(),
    };
    let program = std::env::var("REVAULT_E2E_PROGRAM").unwrap_or_else(|_| program.into());
    let base: Vec<String> = std::env::var("REVAULT_E2E_ARGS_JSON")
        .ok()
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .expect("REVAULT_E2E_ARGS_JSON must be a JSON string array")
        .unwrap_or_else(|| base.iter().map(|value| value.to_string()).collect());
    let language_env = reported_language_env(language);
    let make = |mode: Option<&str>, root: &str| {
        let mut args = base.clone();
        if let Some(mode) = mode {
            args.push(mode.to_string());
        }
        let temporary = std::env::temp_dir();
        let mut env = BTreeMap::from([(
            "LOCKBOX_VAULT_DIR".into(),
            temporary
                .join(format!("revault-{language}-{root}"))
                .display()
                .to_string(),
        )]);
        if mode == Some("--agent") {
            env.insert(
                "LOCKBOX_SESSION_AGENT_DIR".into(),
                temporary
                    .join(format!("revault-{language}-agent"))
                    .display()
                    .to_string(),
            );
            env.insert(
                "LOCKBOX_VAULT_DIR".into(),
                temporary
                    .join(format!("revault-{language}-agent-vault"))
                    .display()
                    .to_string(),
            );
            env.insert(
                "LOCKBOX_VAULT_PASSWORD".into(),
                "agent vault password".into(),
            );
        }
        if let Some((key, value)) = language_env.clone() {
            env.insert(key, value);
        }
        if language == "kotlin" && std::env::var_os("JAVA_TOOL_OPTIONS").is_none() {
            env.insert(
                "JAVA_TOOL_OPTIONS".into(),
                "-Djava.io.tmpdir=/tmp/revault-kotlin-extract -Drevault.keepExtracted=true".into(),
            );
        }
        Invocation {
            program: program.clone(),
            args,
            env,
        }
    };
    if matches!(language, "c" | "cpp" | "go") {
        return vec![make(None, "core")];
    }
    if language == "python" {
        return vec![make(None, "core"), make(Some("--platform"), "platform")];
    }
    vec![
        make(None, "core"),
        make(Some("--default"), "default"),
        make(Some("--agent"), "agent"),
        make(Some("--platform"), "platform"),
    ]
}

fn reported_language_env(language: &str) -> Option<(String, String)> {
    matches!(language, "kotlin" | "wasm")
        .then(|| ("REVAULT_E2E_LANGUAGE".to_string(), language.to_string()))
}

fn verify_rust_artifacts(output: &[u8]) -> Result {
    let required: BTreeSet<_> = [
        "archive-created",
        "archive-opened",
        "vault-created",
        "vault-opened",
    ]
    .into_iter()
    .collect();
    let mut seen = BTreeSet::new();
    let output = String::from_utf8_lossy(output);
    for line in output.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() == 4 && fields[0] == "ARTIFACT" && fields[1] == "rust" {
            if !Path::new(fields[3]).exists() {
                return Err(format!("Rust artifact does not exist: {}", fields[3]).into());
            }
            seen.insert(fields[2]);
        }
    }
    let missing: Vec<_> = required.difference(&seen).copied().collect();
    if !missing.is_empty() {
        return Err(format!(
            "Rust source conformance missing artifacts: {}",
            missing.join(", ")
        )
        .into());
    }
    Ok(())
}

fn rust_source_conformance(args: RustSourceConformance) -> Result {
    if std::env::var_os("REVAULT_LIBRARY").is_some() {
        return Err("REVAULT_LIBRARY is forbidden in Rust source conformance".into());
    }
    let repository = args.repository.canonicalize()?;
    let workspace = repository.join("rust");
    let suites = [
        ("revault_lockbox_api", "public_api_suite"),
        ("revault_vault_api", "vault_api"),
    ];
    for (package, suite) in suites {
        run_status(
            Command::new("cargo")
                .current_dir(&workspace)
                .args(["test", "--locked", "-p", package, "--test", suite]),
        )?;
        println!("SUITE\trust\t{suite}\tpassed");
    }
    run_status(
        Command::new("cargo")
            .current_dir(&workspace)
            .args(["check", "--locked", "--manifest-path"])
            .arg(repository.join("bindings/rust/Cargo.toml")),
    )?;
    let source = args
        .source_archive
        .unwrap_or_else(|| repository.join("bindings/rust/Cargo.lock"))
        .canonicalize()?;
    println!(
        "SOURCE\trust\t{}\tsource-native\t{}\t{}\tinstalled",
        args.target,
        source.display(),
        sha256_file(&source)?
    );
    println!("verified Rust source-native lockbox and vault API suites");
    Ok(())
}

fn native_conformance(args: NativeConformance) -> Result {
    let runtime = args.work.join("runtime");
    fs::create_dir_all(&runtime)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700))?;
    }
    if cfg!(target_os = "linux") && std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        let status = Command::new("dbus-run-session")
            .env("XDG_RUNTIME_DIR", &runtime)
            .arg("--")
            .arg(std::env::current_exe()?)
            .args(["e2e", "native-conformance", "--archive"])
            .arg(&args.archive)
            .args(["--repository"])
            .arg(&args.repository)
            .args(["--work"])
            .arg(&args.work)
            .status()?;
        if !status.success() {
            return Err(format!("native conformance service session failed with {status}").into());
        }
        return Ok(());
    }
    let service_env = linux_secret_service_env()?;
    fs::create_dir_all(&args.work)?;
    let work = PathBuf::from(crate::release::msvc_path(&args.work.canonicalize()?));
    let repository = PathBuf::from(crate::release::msvc_path(&args.repository.canonicalize()?));
    let archive = PathBuf::from(crate::release::msvc_path(&args.archive.canonicalize()?));
    let install = crate::release::install_archive(&archive, &work.join("installed"))?;
    let build = work.join("build");
    run_status(
        Command::new("cmake")
            .arg("-S")
            .arg(crate::release::msvc_path(
                &repository.join("bindings/e2e/c"),
            ))
            .arg("-B")
            .arg(crate::release::msvc_path(&build))
            .arg("-DCMAKE_BUILD_TYPE=Release")
            .arg(format!(
                "-DCMAKE_PREFIX_PATH={}",
                crate::release::msvc_path(&install.prefix)
            )),
    )?;
    run_status(
        Command::new("cmake")
            .arg("--build")
            .arg(crate::release::msvc_path(&build))
            .args(["--config", "Release"]),
    )?;
    let executable = if cfg!(windows) {
        build.join("Release/revault_c_conformance.exe")
    } else {
        build.join("revault_c_conformance")
    };
    let mut command = Command::new(&executable);
    let library_dir = install.prefix.join("lib");
    if cfg!(target_os = "linux") {
        command.env("LD_LIBRARY_PATH", &library_dir);
    }
    if cfg!(target_os = "macos") {
        command.env("DYLD_LIBRARY_PATH", &library_dir);
    }
    if cfg!(windows) {
        let mut paths = vec![library_dir.clone()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        command.env("PATH", std::env::join_paths(paths)?);
    }
    command.env_remove("REVAULT_LIBRARY");
    command.envs(service_env);
    command.env("REVAULT_E2E_LANGUAGE", "c");
    command.env("REVAULT_E2E_ARTIFACT_DIR", work.join("artifacts"));
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "native conformance failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let results = work.join("results.tsv");
    fs::write(&results, output.stdout)?;
    let evidence_path = library_dir.join(&install.library);
    let native = work.join("native.tsv");
    let evidence_output = Command::new(std::env::current_exe()?)
        .args([
            "e2e",
            "evidence",
            "--language",
            "c",
            "--target",
            &install.target,
            "--kind",
            "dynamic",
            "--root",
        ])
        .arg(&library_dir)
        .args(["--file", &install.library])
        .output()?;
    if !evidence_output.status.success() {
        return Err(String::from_utf8_lossy(&evidence_output.stderr)
            .into_owned()
            .into());
    }
    fs::write(&native, evidence_output.stdout)?;
    if !evidence_path.is_file() {
        return Err("installed native library disappeared before verification".into());
    }
    std::env::set_var("REVAULT_REQUIRE_INSTALLED_NATIVE", "1");
    verify_results(VerifyResults {
        languages: vec!["c".into()],
        operations: repository.join("bindings/e2e/operations.tsv"),
        results: vec![results, native],
    })
}

fn linux_secret_service_env() -> Result<BTreeMap<String, String>> {
    let mut service_env = BTreeMap::new();
    if !cfg!(target_os = "linux") {
        return Ok(service_env);
    }
    let mut daemon = Command::new("gnome-keyring-daemon")
        .args(["--daemonize", "--login", "--components=secrets"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    daemon
        .stdin
        .take()
        .ok_or("gnome-keyring-daemon did not expose stdin")?
        .write_all(b"\n")?;
    let output = daemon.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("gnome-keyring-daemon failed with {}", output.status).into());
    }
    for raw in String::from_utf8_lossy(&output.stdout).lines() {
        let line = raw
            .trim()
            .trim_start_matches("export ")
            .trim_end_matches(';');
        if let Some((key, value)) = line.split_once('=') {
            service_env.insert(key.to_string(), value.trim_matches(['\'', '"']).to_string());
        }
    }
    Ok(service_env)
}

fn evidence(args: Evidence) -> Result {
    selected_languages(std::slice::from_ref(&args.language))?;
    let matches: Vec<_> = WalkDir::new(&args.root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file() && entry.file_name() == std::ffi::OsStr::new(&args.file)
        })
        .map(|entry| entry.into_path())
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "expected one installed {}, found {} under {}",
            args.file,
            matches.len(),
            args.root.display()
        )
        .into());
    }
    let path = matches[0].canonicalize()?;
    println!(
        "NATIVE\t{}\t{}\t{}\t{}\t{}\tinstalled",
        args.language,
        args.target,
        args.kind,
        path.display(),
        sha256_file(&path)?
    );
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut source = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn matrix_json() -> Result {
    let targets = [
        ("linux-x86_64-gnu", "ubuntu-24.04"),
        ("linux-aarch64-gnu", "ubuntu-24.04-arm"),
        ("macos-x86_64", "macos-15-intel"),
        ("macos-aarch64", "macos-15"),
        ("windows-x86_64-msvc", "windows-2025"),
        ("windows-aarch64-msvc", "windows-11-arm"),
    ];
    let mut include = Vec::new();
    for (target, runner) in targets {
        for language in LANGUAGES {
            if language == "swift" && target.starts_with("windows-") {
                continue;
            }
            include.push(
                serde_json::json!({"target": target, "runner": runner, "language": language}),
            );
        }
    }
    assert_eq!(include.len(), 94);
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({"include": include}))?
    );
    Ok(())
}

fn selected_languages(values: &[String]) -> Result<BTreeSet<String>> {
    let all: BTreeSet<_> = LANGUAGES.iter().map(|value| value.to_string()).collect();
    let selected: BTreeSet<_> = values.iter().cloned().collect();
    let unknown: Vec<_> = selected.difference(&all).cloned().collect();
    if !unknown.is_empty() {
        return Err(format!("unknown languages: {}", unknown.join(", ")).into());
    }
    Ok(selected)
}

fn verify_results(args: VerifyResults) -> Result {
    let languages = selected_languages(&args.languages)?;
    let operations = read_operations(&args.operations)?;
    let mut seen: BTreeMap<String, BTreeMap<String, u64>> = languages
        .iter()
        .map(|language| (language.clone(), BTreeMap::new()))
        .collect();
    let mut artifacts: BTreeMap<String, BTreeSet<String>> = languages
        .iter()
        .map(|language| (language.clone(), BTreeSet::new()))
        .collect();
    let mut installs: BTreeSet<String> = BTreeSet::new();
    let mut errors = Vec::new();
    for file in &args.results {
        for (number, raw) in fs::read_to_string(file)?.lines().enumerate() {
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let fields: Vec<_> = raw.split('\t').collect();
            if fields.len() < 3 {
                errors.push(format!(
                    "{}:{}: malformed result",
                    file.display(),
                    number + 1
                ));
                continue;
            }
            let (kind, language, item) = (fields[0], fields[1], fields[2]);
            if !languages.contains(language) {
                errors.push(format!(
                    "{}:{}: unknown language {language}",
                    file.display(),
                    number + 1
                ));
                continue;
            }
            match kind {
                "PASS" => {
                    if !operations.contains(item) {
                        errors.push(format!(
                            "{}:{}: unknown operation {item}",
                            file.display(),
                            number + 1
                        ));
                        continue;
                    }
                    let assertions = fields
                        .get(3)
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    if assertions == 0 {
                        errors.push(format!("{language}:{item}: no observable assertion"));
                    }
                    *seen
                        .get_mut(language)
                        .unwrap()
                        .entry(item.to_string())
                        .or_default() += assertions;
                }
                "ARTIFACT" => {
                    if fields.get(3).is_none_or(|path| !Path::new(path).exists()) {
                        errors.push(format!(
                            "{}:{}: artifact path does not exist",
                            file.display(),
                            number + 1
                        ));
                    } else {
                        artifacts
                            .get_mut(language)
                            .unwrap()
                            .insert(item.to_string());
                    }
                }
                "NATIVE" => {
                    if fields.len() != 7 {
                        errors.push(format!(
                            "{}:{}: malformed NATIVE evidence",
                            file.display(),
                            number + 1
                        ));
                        continue;
                    }
                    let path = Path::new(fields[4]);
                    let display = path.to_string_lossy().replace('\\', "/");
                    let staged = display.contains("/tmp/packages/")
                        || display.contains("/tmp/native/")
                        || display.contains("/rust/target/");
                    let valid_hash = sha256_file(path).is_ok_and(|digest| digest == fields[5]);
                    if !path.is_file()
                        || fields[5].len() != 64
                        || fields[6] != "installed"
                        || staged
                        || !valid_hash
                    {
                        errors.push(format!(
                            "{}:{}: invalid installed native evidence",
                            file.display(),
                            number + 1
                        ));
                    } else {
                        installs.insert(language.to_string());
                    }
                }
                "SKIP" | "XFAIL" => errors.push(format!(
                    "{}:{}: {kind} is not conformance",
                    file.display(),
                    number + 1
                )),
                _ => errors.push(format!(
                    "{}:{}: unknown result kind {kind}",
                    file.display(),
                    number + 1
                )),
            }
        }
    }
    let required_artifacts: BTreeSet<_> = [
        "archive-created",
        "archive-opened",
        "vault-created",
        "vault-opened",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    for language in &languages {
        let actual: BTreeSet<_> = seen[language].keys().cloned().collect();
        let missing: Vec<_> = operations.difference(&actual).cloned().collect();
        if !missing.is_empty() {
            errors.push(format!(
                "{language}: missing {} operations: {}",
                missing.len(),
                missing.join(", ")
            ));
        }
        let missing_artifacts: Vec<_> = required_artifacts
            .difference(&artifacts[language])
            .cloned()
            .collect();
        if !missing_artifacts.is_empty() {
            errors.push(format!(
                "{language}: missing artifact checks: {}",
                missing_artifacts.join(", ")
            ));
        }
        if std::env::var_os("REVAULT_REQUIRE_INSTALLED_NATIVE").is_some()
            && !installs.contains(language)
        {
            errors.push(format!(
                "{language}: missing installed native artifact evidence"
            ));
        }
    }
    finish(
        errors,
        format!(
            "verified {} languages x {} operations",
            languages.len(),
            operations.len()
        ),
    )
}

fn verify_interop(args: VerifyInterop) -> Result {
    let languages = selected_languages(&args.languages)?;
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    let mut result_files = args.results;
    if let Some(directory) = args.results_dir {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("interop-") && name.ends_with(".tsv"))
            {
                result_files.push(path);
            }
        }
    }
    if result_files.is_empty() {
        return Err("no interoperability result files supplied".into());
    }
    for file in &result_files {
        for (number, raw) in fs::read_to_string(file)?.lines().enumerate() {
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let fields: Vec<_> = raw.split('\t').collect();
            if fields.first() != Some(&"INTEROP") {
                continue;
            }
            if fields.len() != 5 {
                errors.push(format!(
                    "{}:{}: malformed INTEROP record",
                    file.display(),
                    number + 1
                ));
                continue;
            }
            let (consumer, producer, artifact) = (fields[1], fields[2], fields[3]);
            if !languages.contains(consumer) || !languages.contains(producer) {
                errors.push(format!(
                    "{}:{}: language outside selected matrix",
                    file.display(),
                    number + 1
                ));
            } else if consumer == producer {
                errors.push(format!(
                    "{}:{}: self-interop is not cross-language",
                    file.display(),
                    number + 1
                ));
            } else if !matches!(artifact, "archive" | "vault") {
                errors.push(format!(
                    "{}:{}: unknown artifact {artifact}",
                    file.display(),
                    number + 1
                ));
            } else if fields[4].parse::<u64>().ok().is_none_or(|value| value < 1) {
                errors.push(format!(
                    "{}:{}: no observable assertion",
                    file.display(),
                    number + 1
                ));
            } else {
                seen.insert((
                    consumer.to_string(),
                    producer.to_string(),
                    artifact.to_string(),
                ));
            }
        }
    }
    for consumer in &languages {
        for producer in &languages {
            if consumer != producer {
                for artifact in ["archive", "vault"] {
                    if !seen.contains(&(consumer.clone(), producer.clone(), artifact.to_string())) {
                        errors.push(format!(
                            "missing {artifact} interop: {consumer} opening {producer}"
                        ));
                    }
                }
            }
        }
    }
    let count = languages.len() * (languages.len() - 1) * 2;
    finish(
        errors,
        format!("verified {count} cross-language artifact paths"),
    )
}

fn generate_inventory(args: GenerateInventory) -> Result {
    let source = fs::read_to_string(&args.header)?;
    let mut rows = Vec::new();
    for raw in source.lines() {
        let line = normalize_pointer_name(raw);
        let Some(open) = line.find('(') else {
            continue;
        };
        if !line.ends_with(");") {
            continue;
        }
        let prefix = line[..open].trim();
        let Some(split) = prefix.rfind(char::is_whitespace) else {
            continue;
        };
        let result = prefix[..split].trim();
        let name = prefix[split..].trim();
        if name.is_empty()
            || name == "api_abi_version"
            || !name
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        {
            continue;
        }
        let arguments = line[open + 1..line.len() - 2].trim();
        rows.push(format!("{name}\t{}\t{result}\t{arguments}", domain(name)));
    }
    if rows.is_empty() {
        return Err("no ABI functions found".into());
    }
    let generated = format!(
        "symbol\tdomain\treturn_type\targuments\n{}\n",
        rows.join("\n")
    );
    if args.check {
        if fs::read_to_string(&args.output)? != generated {
            return Err(format!(
                "{} is stale; run revault-tool e2e generate-inventory",
                args.output.display()
            )
            .into());
        }
    } else {
        fs::write(&args.output, generated)?;
        println!(
            "generated {} operations in {}",
            rows.len(),
            args.output.display()
        );
    }
    Ok(())
}

fn matrix(args: Matrix) -> Result {
    for language in LANGUAGES {
        println!("running installed {language} conformance");
        run_status(
            Command::new("docker")
                .args(["compose", "-f"])
                .arg(&args.compose)
                .args(["run", "--rm", language]),
        )?;
    }
    if !args.skip_interop {
        for language in LANGUAGES {
            run_status(
                Command::new("docker")
                    .args(["compose", "-f"])
                    .arg(&args.compose)
                    .args([
                        "run",
                        "--rm",
                        language,
                        "revault-tool",
                        "e2e",
                        "interop-consumer",
                        "--consumer",
                        language,
                    ]),
            )?;
        }
        run_status(
            Command::new("docker")
                .args(["compose", "-f"])
                .arg(&args.compose)
                .args([
                    "run",
                    "--rm",
                    "kotlin",
                    "revault-tool",
                    "e2e",
                    "verify-interop",
                    "--results-dir",
                    "/artifacts",
                ]),
        )?;
    }
    Ok(())
}

fn interop_consumer(args: InteropConsumer) -> Result {
    if !LANGUAGES.contains(&args.consumer.as_str()) {
        return Err(format!("unknown consumer {}", args.consumer).into());
    }
    fs::create_dir_all(&args.artifacts)?;
    for producer in LANGUAGES
        .into_iter()
        .filter(|producer| *producer != args.consumer)
    {
        let invocation = invocations(&args.consumer)
            .into_iter()
            .next()
            .ok_or("missing language invocation")?;
        let mut command = Command::new(invocation.program);
        command
            .args(invocation.args)
            .args(["--interop", producer])
            .envs(invocation.env)
            .env_remove("REVAULT_LIBRARY");
        let output = command.output()?;
        if !output.status.success() {
            return Err(format!("{} failed opening {producer} artifacts", args.consumer).into());
        }
        fs::write(
            args.artifacts
                .join(format!("interop-{}-{producer}.tsv", args.consumer)),
            output.stdout,
        )?;
    }
    Ok(())
}

fn read_operations(path: &Path) -> Result<BTreeSet<String>> {
    let source = fs::read_to_string(path)?;
    Ok(source
        .lines()
        .skip(1)
        .filter_map(|line| line.split('\t').next())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect())
}

fn normalize_pointer_name(raw: &str) -> String {
    let mut output = raw.to_string();
    if let Some(open) = output.find('(') {
        if let Some(star) = output[..open].rfind('*') {
            let after = output[star + 1..open].trim();
            if !after.is_empty() {
                output.replace_range(star + 1..open, &format!(" {after}"));
            }
        }
    }
    output
}

fn domain(name: &str) -> &'static str {
    if name.starts_with("lockbox_") {
        if name.contains("form") {
            "archive.forms"
        } else if name.contains("recovery") {
            "archive.recovery"
        } else if ["password", "contact", "key_slot", "owner"]
            .iter()
            .any(|part| name.contains(part))
        {
            "archive.keys"
        } else if ["cache", "import_stats", "inspection", "runtime", "stream"]
            .iter()
            .any(|part| name.contains(part))
        {
            "archive.diagnostics"
        } else {
            "archive.lifecycle"
        }
    } else if name.starts_with("key_contact_") {
        "keys.contact"
    } else if name.starts_with("key_signing_") || name.starts_with("vault_key_") {
        "keys.signing"
    } else if name.starts_with("vault_directory_") {
        "vault.directory"
    } else if name.starts_with("vault_agent_")
        || matches!(name, "vault_is_running" | "vault_forget_all")
    {
        "vault.agent"
    } else if name.starts_with("vault_platform_") {
        "vault.platform"
    } else if name.starts_with("vault_") {
        "vault.local"
    } else {
        "support.memory"
    }
}

fn finish(errors: Vec<String>, success: String) -> Result {
    if errors.is_empty() {
        println!("{success}");
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}

fn run_status(command: &mut Command) -> Result {
    let display = format!("{command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed ({status}): {display}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_languages_are_unique() {
        assert_eq!(
            selected_languages(&LANGUAGES.map(str::to_string))
                .unwrap()
                .len(),
            16
        );
    }
    #[test]
    fn domains_remain_descriptive() {
        assert_eq!(domain("lockbox_form_list"), "archive.forms");
        assert_eq!(domain("vault_agent_start"), "vault.agent");
    }
    #[test]
    fn shared_conformance_runners_report_the_selected_language() {
        assert_eq!(
            reported_language_env("kotlin"),
            Some(("REVAULT_E2E_LANGUAGE".into(), "kotlin".into()))
        );
        assert_eq!(
            reported_language_env("wasm"),
            Some(("REVAULT_E2E_LANGUAGE".into(), "wasm".into()))
        );
        assert_eq!(reported_language_env("java"), None);
        assert_eq!(reported_language_env("javascript"), None);
    }
}
