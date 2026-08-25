use crate::Result;
use clap::{Args, Subcommand};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
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

const CONSUMER_PHASE_TIMEOUT: Duration = Duration::from_secs(300);
static COMMAND_OUTPUT_ID: AtomicU64 = AtomicU64::new(0);

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
    /// Maximum number of language containers to execute concurrently.
    #[arg(long, default_value_t = 1)]
    jobs: usize,
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
        let output = command_output_with_timeout(
            &mut Command::new("/opt/revault-rust-conformance"),
            CONSUMER_PHASE_TIMEOUT,
            "Rust artifact conformance",
        )?;
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
        let mut command = Command::new("dbus-run-session");
        command.arg("--").arg(std::env::current_exe()?).args([
            "e2e",
            "container",
            "--language",
            &args.language,
        ]);
        let mut child = command.spawn()?;
        let status = wait_child_with_timeout(
            &mut child,
            Duration::from_secs(1200),
            &format!("{} service session", args.language),
        )?;
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
        let output = command_output_with_timeout(
            &mut command,
            CONSUMER_PHASE_TIMEOUT,
            &format!("{} {:?} phase", args.language, invocation.args.last()),
        )?;
        if !output.status.success() {
            return Err(format!(
                "{} conformance failed: {}",
                args.language,
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        if let Some(mut child) = server.take() {
            let status = wait_child_with_timeout(
                &mut child,
                CONSUMER_PHASE_TIMEOUT,
                &format!("{} agent server", args.language),
            )?;
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
        .arg(&root)
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
        languages: vec![args.language.clone()],
        operations: PathBuf::from("bindings/e2e/operations.tsv"),
        results: vec![results, native],
    })?;
    verify_loader_resolution(&args.language, &root, &file, &service_env)
}

fn verify_loader_resolution(
    language: &str,
    native_root: &Path,
    native_file: &str,
    service_env: &BTreeMap<String, String>,
) -> Result {
    if !matches!(
        language,
        "csharp"
            | "dart"
            | "java"
            | "javascript"
            | "kotlin"
            | "lua"
            | "php"
            | "python"
            | "ruby"
            | "typescript"
    ) {
        return Ok(());
    }
    let invocation = invocations(language)
        .into_iter()
        .next()
        .ok_or("missing loader conformance invocation")?;
    let full_path = native_root.join(native_file).canonicalize()?;
    let invalid_environment_path = native_root.join(format!(
        ".revault-e2e-missing-library-{}",
        std::process::id()
    ));
    if invalid_environment_path.exists() {
        return Err(format!(
            "loader negative-control path unexpectedly exists: {}",
            invalid_environment_path.display()
        )
        .into());
    }

    // This negative control closes an important false-positive path. Without it,
    // a binding could ignore both REVAULT_LIBRARY and the explicit argument,
    // discover its packaged carrier, and still make the explicit-precedence
    // check appear to pass.
    let mut invalid_environment_control = Command::new(&invocation.program);
    invalid_environment_control
        .args(&invocation.args)
        .envs(service_env)
        .envs(&invocation.env)
        .env_remove("REVAULT_E2E_LOAD_PATH")
        .env("REVAULT_LIBRARY", &invalid_environment_path)
        .env("REVAULT_E2E_LOADER_SMOKE", "invalid-environment-control");
    let invalid_output = command_output_with_timeout(
        &mut invalid_environment_control,
        CONSUMER_PHASE_TIMEOUT,
        &format!("{language} invalid-environment loader negative control"),
    )?;
    if invalid_output.status.success() {
        return Err(format!(
            "{language} ignored invalid REVAULT_LIBRARY and fell back to another carrier: {}",
            String::from_utf8_lossy(&invalid_output.stdout).trim()
        )
        .into());
    }

    for mode in [
        "packaged",
        "empty-environment",
        "explicit-overrides-invalid-environment",
        "environment",
        "search",
    ] {
        let mut command = Command::new(&invocation.program);
        command
            .args(&invocation.args)
            .envs(service_env)
            .envs(&invocation.env)
            .env_remove("REVAULT_LIBRARY")
            .env_remove("REVAULT_E2E_LOAD_PATH")
            .env("REVAULT_E2E_LOADER_SMOKE", mode);
        match mode {
            "packaged" => {}
            "empty-environment" => {
                command.env("REVAULT_LIBRARY", "");
            }
            "explicit-overrides-invalid-environment" => {
                command
                    .env("REVAULT_E2E_LOAD_PATH", &full_path)
                    .env("REVAULT_LIBRARY", &invalid_environment_path);
            }
            "environment" => {
                command.env("REVAULT_LIBRARY", &full_path);
            }
            "search" => {
                command.env("REVAULT_E2E_LOAD_PATH", native_file);
                let search_variable = if cfg!(windows) {
                    "PATH"
                } else if cfg!(target_os = "macos") {
                    "DYLD_LIBRARY_PATH"
                } else {
                    "LD_LIBRARY_PATH"
                };
                let mut paths = vec![native_root.to_path_buf()];
                paths.extend(std::env::split_paths(
                    &std::env::var_os(search_variable).unwrap_or_default(),
                ));
                command.env(search_variable, std::env::join_paths(paths)?);
            }
            _ => unreachable!(),
        }
        let output = command_output_with_timeout(
            &mut command,
            CONSUMER_PHASE_TIMEOUT,
            &format!("{language} {mode} loader conformance"),
        )?;
        if !output.status.success() {
            return Err(format!(
                "{language} {mode} loader conformance failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        let line = String::from_utf8_lossy(&output.stdout);
        let fields: Vec<_> = line.trim().split('\t').collect();
        let version = fields
            .get(3)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or_default();
        if fields.len() != 4
            || fields[0] != "LOADER"
            || fields[1] != language
            || fields[2] != mode
            || version == 0
        {
            return Err(format!(
                "{language} {mode} loader conformance returned unexpected output: {}",
                line.trim()
            )
            .into());
        }
    }
    println!(
        "verified {language} invalid-environment rejection plus packaged, empty-environment, explicit-overrides-invalid-environment, environment, and search-path loading"
    );
    Ok(())
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
        "dart" => ("/opt/revault-dart/bundle/bin/conformance", &[]),
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
            .args(["check", "--manifest-path"])
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
    let library_dir = install.prefix.join("lib");
    let mut combined = Vec::new();
    for phase in ["--core", "--agent", "--platform", "--last-error"] {
        println!("running native conformance phase {phase}");
        let phase_output = work.join(format!("{}.tsv", phase.trim_start_matches('-')));
        let mut command = Command::new(&executable);
        command
            .arg(phase)
            .stdout(fs::File::create(&phase_output)?)
            .stderr(Stdio::inherit());
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
        command.envs(&service_env);
        command.env("REVAULT_E2E_LANGUAGE", "c");
        command.env("REVAULT_E2E_ARTIFACT_DIR", work.join("artifacts"));
        let mut child = command.spawn()?;
        let deadline = Instant::now() + Duration::from_secs(120);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Err(format!("native conformance phase {phase} timed out").into());
            }
            thread::sleep(Duration::from_millis(50));
        };
        if !status.success() {
            return Err(format!("native conformance phase {phase} failed with {status}").into());
        }
        combined.extend_from_slice(&fs::read(&phase_output)?);
    }
    let results = work.join("results.tsv");
    fs::write(&results, combined)?;
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
    // Windows executables default to a 1 MiB main-thread stack. Keep the
    // evidence buffer on the heap so hashing an installed artifact cannot
    // exhaust the stack before the first read.
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
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
    let groups = [
        (
            "native-jvm",
            ["c", "cpp", "go", "java", "kotlin", "rust", "swift"].as_slice(),
        ),
        (
            "managed-script",
            [
                "csharp",
                "dart",
                "javascript",
                "lua",
                "php",
                "python",
                "ruby",
                "typescript",
                "wasm",
            ]
            .as_slice(),
        ),
    ];
    let mut include = Vec::new();
    let mut combinations = 0;
    for (target, runner) in targets {
        for (group, languages) in groups {
            let languages: Vec<_> = languages
                .iter()
                .copied()
                .filter(|language| *language != "swift" || !target.starts_with("windows-"))
                .collect();
            combinations += languages.len();
            include.push(serde_json::json!({
                "target": target,
                "runner": runner,
                "group": group,
                "languages": languages.join(","),
            }));
        }
    }
    assert_eq!(include.len(), 12);
    assert_eq!(combinations, 94);
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
    if args.jobs == 0 {
        return Err("matrix jobs must be greater than zero".into());
    }

    let compose = args.compose.canonicalize()?;
    let e2e_directory = compose.parent().ok_or("compose path has no parent")?;
    let repository = e2e_directory
        .parent()
        .and_then(Path::parent)
        .ok_or("compose path is not below bindings/e2e")?;
    println!(
        "building the shared native image, then language images with at most {} concurrent builds",
        args.jobs
    );
    let mut native_build = Command::new("docker");
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        native_build.args([
            "buildx",
            "build",
            "--cache-from",
            "type=gha,scope=bindings-e2e-native",
            "--cache-to",
            "type=gha,mode=max,scope=bindings-e2e-native",
            "--load",
        ]);
    } else {
        native_build.arg("build");
    }
    native_build
        .arg("--file")
        .arg(e2e_directory.join("containers/native/Dockerfile"))
        .args(["--tag", "revault-e2e-native:local"])
        .arg(repository);
    run_status(&mut native_build)?;
    run_status(
        Command::new("docker")
            .env("COMPOSE_PARALLEL_LIMIT", args.jobs.to_string())
            .args(["compose", "-f"])
            .arg(&compose)
            .arg("build"),
    )?;

    let conformance_errors = run_language_batches(args.jobs, |language| {
        println!("running installed {language} conformance");
        run_status(
            Command::new("docker")
                .args(["compose", "-f"])
                .arg(&compose)
                .args(["run", "--rm", language]),
        )
    });
    if !conformance_errors.is_empty() {
        return Err(conformance_errors.join("\n").into());
    }

    if !args.skip_interop {
        let interop_errors = run_language_batches(args.jobs, |language| {
            run_status(
                Command::new("docker")
                    .args(["compose", "-f"])
                    .arg(&compose)
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
            )
        });
        if !interop_errors.is_empty() {
            return Err(interop_errors.join("\n").into());
        }
        run_status(
            Command::new("docker")
                .args(["compose", "-f"])
                .arg(&compose)
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

fn run_language_batches<F>(jobs: usize, run: F) -> Vec<String>
where
    F: Fn(&str) -> Result + Sync,
{
    let mut errors = Vec::new();
    for batch in LANGUAGES.chunks(jobs) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|language| {
                    let language = *language;
                    let run = &run;
                    (language, scope.spawn(move || run(language)))
                })
                .collect();
            for (language, handle) in handles {
                match handle.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => errors.push(format!("{language}: {error}")),
                    Err(_) => errors.push(format!("{language}: conformance worker panicked")),
                }
            }
        });
    }
    errors
}

fn interop_consumer(args: InteropConsumer) -> Result {
    if !LANGUAGES.contains(&args.consumer.as_str()) {
        return Err(format!("unknown consumer {}", args.consumer).into());
    }
    fs::create_dir_all(&args.artifacts)?;
    let producers: Vec<_> = LANGUAGES
        .into_iter()
        .filter(|producer| *producer != args.consumer)
        .collect();
    let invocation = invocations(&args.consumer)
        .into_iter()
        .next()
        .ok_or("missing language invocation")?;
    let mut command = Command::new(invocation.program);
    command
        .args(invocation.args)
        .arg("--interop")
        .args(&producers)
        .envs(invocation.env)
        .env_remove("REVAULT_LIBRARY");
    let output = command_output_with_timeout(
        &mut command,
        CONSUMER_PHASE_TIMEOUT,
        &format!("{} batched interoperability", args.consumer),
    )?;
    if !output.status.success() {
        return Err(format!(
            "{} failed opening foreign artifacts: {}",
            args.consumer,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    fs::write(
        args.artifacts
            .join(format!("interop-{}-batch.tsv", args.consumer)),
        output.stdout,
    )?;
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

fn wait_child_with_timeout(
    child: &mut Child,
    timeout: Duration,
    description: &str,
) -> Result<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            return Err(format!(
                "{description} timed out after {} seconds",
                timeout.as_secs()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn command_output_with_timeout(
    command: &mut Command,
    timeout: Duration,
    description: &str,
) -> Result<Output> {
    let output_id = COMMAND_OUTPUT_ID.fetch_add(1, Ordering::Relaxed);
    let output_prefix = std::env::temp_dir().join(format!(
        "revault-command-output-{}-{output_id}",
        std::process::id()
    ));
    let stdout_path = output_prefix.with_extension("stdout");
    let stderr_path = output_prefix.with_extension("stderr");
    command
        .stdout(fs::File::create(&stdout_path)?)
        .stderr(fs::File::create(&stderr_path)?);
    let display = format!("{command:?}");
    let mut child = command.spawn()?;
    let status = wait_child_with_timeout(&mut child, timeout, description);
    let stdout = fs::read(&stdout_path)?;
    let stderr = fs::read(&stderr_path)?;
    let _ = fs::remove_file(stdout_path);
    let _ = fs::remove_file(stderr_path);
    let status = status.map_err(|error| {
        format!(
            "{error}; command: {display}; stderr: {}",
            String::from_utf8_lossy(&stderr).trim()
        )
    })?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn command_output_timeout_is_bounded() {
        let started = Instant::now();
        let result = command_output_with_timeout(
            Command::new("sh").args(["-c", "while :; do :; done"]),
            Duration::from_millis(50),
            "test child",
        );
        assert!(result.unwrap_err().to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn command_output_capture_preserves_both_streams() {
        let output = command_output_with_timeout(
            Command::new("sh").args(["-c", "printf stdout; printf stderr >&2"]),
            Duration::from_secs(1),
            "test child",
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stdout");
        assert_eq!(output.stderr, b"stderr");
    }

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
