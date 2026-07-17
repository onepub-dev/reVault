use crate::Result;
use clap::{Args, ValueEnum};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use ureq::unversioned::multipart::Form;
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Ecosystem {
    NpmNative,
    Npm,
    Wasm,
    Python,
    MavenJava,
    MavenKotlin,
    Nuget,
    Dart,
    Ruby,
    Lua,
    RustFoundation,
    Rust,
}

#[derive(Args, Debug)]
pub struct PublishPackages {
    /// Release version without a leading `v`.
    #[arg(long)]
    version: String,
    /// Root produced by `release assemble-packages`.
    #[arg(long)]
    packages: PathBuf,
    /// Registry-native package set to process.
    #[arg(long, value_enum)]
    ecosystem: Ecosystem,
    /// Directory for wheels, gems, NuGet packages, and other upload payloads.
    #[arg(long, default_value = "dist/publication")]
    output: PathBuf,
    /// Perform the irreversible registry upload after local package validation.
    #[arg(long)]
    publish: bool,
    /// Restrict a platform package set to one canonical native target.
    #[arg(long)]
    target: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum GitPackage {
    Api,
    Homebrew,
}

#[derive(Args, Debug)]
pub struct PromoteGitPackage {
    #[arg(long)]
    version: String,
    #[arg(long)]
    packages: PathBuf,
    #[arg(long)]
    destination: PathBuf,
    /// Downloaded GitHub release assets, required by Swift and Homebrew.
    #[arg(long)]
    release_assets: Option<PathBuf>,
    #[arg(long, value_enum)]
    package: GitPackage,
    /// Commit, tag, and push the prepared tree. The default validates only.
    #[arg(long)]
    publish: bool,
}

#[derive(Deserialize)]
struct NpmPackage {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct LuaRocksVersion {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct LuaRocksVersionResponse {
    version: Option<LuaRocksVersion>,
}

#[derive(Debug, Deserialize)]
struct LuaRocksErrorResponse {
    #[serde(default)]
    errors: Vec<String>,
}

pub fn publish(args: PublishPackages) -> Result {
    validate_version(&args.version)?;
    let packages = args.packages.canonicalize()?;
    fs::create_dir_all(&args.output)?;
    let output = args.output.canonicalize()?;
    match args.ecosystem {
        Ecosystem::NpmNative => npm_native(&packages, &args.version, args.publish, args.target),
        Ecosystem::Npm => npm_package(
            &packages.join("npm/revault-api"),
            &args.version,
            args.publish,
        ),
        Ecosystem::Wasm => npm_package(
            &packages.join("npm/revault-api-wasm"),
            &args.version,
            args.publish,
        ),
        Ecosystem::Python => python(&packages, &args.version, &output, args.publish, args.target),
        Ecosystem::MavenJava => maven(
            &packages.join("maven/java"),
            &args.version,
            "mavenJava",
            args.publish,
        ),
        Ecosystem::MavenKotlin => maven(
            &packages.join("maven/kotlin"),
            &args.version,
            "mavenKotlin",
            args.publish,
        ),
        Ecosystem::Nuget => nuget(
            &packages.join("nuget"),
            &args.version,
            &output,
            args.publish,
        ),
        Ecosystem::Dart => dart(&packages.join("dart"), &args.version, args.publish),
        Ecosystem::Ruby => ruby(&packages, &args.version, &output, args.publish, args.target),
        Ecosystem::Lua => lua(&packages, &args.version, &output, args.publish, args.target),
        Ecosystem::RustFoundation => rust_foundation(&packages, args.publish),
        Ecosystem::Rust => rust_package(
            &packages.join("cargo/revault-api"),
            "revault-api",
            &args.version,
            args.publish,
            true,
        ),
    }
}

fn npm_native(packages: &Path, version: &str, publish: bool, target: Option<String>) -> Result {
    let root = packages.join("npm");
    let mut package_dirs = child_directories(&root)?;
    package_dirs.retain(|path| {
        path.file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("revault-api-native-"))
    });
    if let Some(target) = target {
        package_dirs.retain(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name == format!("revault-api-native-{target}"))
        });
    }
    if package_dirs.is_empty() {
        return Err("no matching npm native carrier packages".into());
    }
    for package in package_dirs {
        npm_package(&package, version, publish)?;
    }
    Ok(())
}

fn npm_package(package: &Path, version: &str, publish: bool) -> Result {
    let manifest: NpmPackage = serde_json::from_slice(&fs::read(package.join("package.json"))?)?;
    require_version(&manifest.version, version, package)?;
    run(Command::new("npm")
        .args(["pack", "--dry-run"])
        .current_dir(package))?;
    if publish && !npm_version_exists(&manifest.name, version) {
        run(Command::new("npm")
            .args(["publish", "--access", "public", "--provenance"])
            .current_dir(package))?;
    } else if publish {
        println!("already published: {}@{version}", manifest.name);
    }
    Ok(())
}

fn npm_version_exists(name: &str, version: &str) -> bool {
    Command::new("npm")
        .args(["view", &format!("{name}@{version}"), "version"])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn python(
    packages: &Path,
    version: &str,
    output: &Path,
    publish: bool,
    target: Option<String>,
) -> Result {
    let linux_target = target
        .as_deref()
        .is_some_and(|value| value.starts_with("linux-"));
    let roots = platform_roots(&packages.join("python"), target)?;
    let wheel_dir = output.join("python");
    fs::create_dir_all(&wheel_dir)?;
    for root in roots {
        require_text_version(&root.join("pyproject.toml"), version)?;
        run(Command::new("python")
            .args(["-m", "build", "--wheel", "--outdir"])
            .arg(&wheel_dir)
            .arg(&root))?;
    }
    if linux_target {
        repair_linux_wheels(&wheel_dir)?;
    }
    let wheels = files_with_extension(&wheel_dir, "whl")?;
    if wheels.is_empty() {
        return Err("Python build produced no wheels".into());
    }
    let mut check = Command::new("python");
    check.args(["-m", "twine", "check"]);
    check.args(&wheels);
    run(&mut check)?;
    if publish {
        let mut upload = Command::new("python");
        upload.args(["-m", "twine", "upload", "--skip-existing"]);
        upload.args(&wheels);
        run(&mut upload)?;
    }
    Ok(())
}

fn repair_linux_wheels(wheel_dir: &Path) -> Result {
    let inputs = files_with_extension(wheel_dir, "whl")?;
    let repaired = wheel_dir.join("manylinux");
    fs::create_dir_all(&repaired)?;
    for wheel in &inputs {
        run(Command::new("python")
            .args(["-m", "auditwheel", "repair", "--wheel-dir"])
            .arg(&repaired)
            .arg(wheel))?;
    }
    for wheel in inputs {
        fs::remove_file(wheel)?;
    }
    for wheel in files_with_extension(&repaired, "whl")? {
        fs::rename(
            &wheel,
            wheel_dir.join(wheel.file_name().ok_or("wheel has no filename")?),
        )?;
    }
    fs::remove_dir(repaired)?;
    Ok(())
}

fn maven(package: &Path, version: &str, publication: &str, publish: bool) -> Result {
    require_text_version(&gradle_manifest(package)?, version)?;
    let task = if publish {
        "publishAndReleaseToMavenCentral"
    } else {
        "publishToMavenLocal"
    };
    println!("validating Maven publication {publication}");
    run(Command::new(gradle())
        .arg("-p")
        .arg(package)
        .args(["--no-daemon", task]))
}

fn nuget(package: &Path, version: &str, output: &Path, publish: bool) -> Result {
    require_text_version(&package.join("RevaultBindings.csproj"), version)?;
    let destination = output.join("nuget");
    fs::create_dir_all(&destination)?;
    run(Command::new("dotnet")
        .args(["pack", "--configuration", "Release", "--output"])
        .arg(&destination)
        .arg(package.join("RevaultBindings.csproj")))?;
    let packages = files_with_extension(&destination, "nupkg")?;
    if packages.is_empty() {
        return Err("dotnet pack produced no NuGet package".into());
    }
    if publish {
        if nuget_version_is_public(version)? {
            println!("Revault.Api {version} is already publicly available on NuGet");
            return Ok(());
        }
        let api_key = std::env::var("NUGET_API_KEY")
            .map_err(|_| "NUGET_API_KEY is required for NuGet publication")?;
        for file in packages {
            run_redacted(
                Command::new("dotnet")
                    .args(["nuget", "push"])
                    .arg(file)
                    .args([
                        "--api-key",
                        &api_key,
                        "--source",
                        "https://api.nuget.org/v3/index.json",
                    ]),
                "dotnet nuget push <package> --api-key <redacted> --source nuget.org",
            )?;
        }
    }
    Ok(())
}

fn nuget_version_is_public(version: &str) -> Result<bool> {
    let url = nuget_package_url(version);
    let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let output = Command::new("curl")
        .args([
            "--silent",
            "--output",
            null_device,
            "--write-out",
            "%{http_code}",
            "--head",
            &url,
        ])
        .output()
        .map_err(|error| format!("failed to check NuGet package availability: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "NuGet package availability check failed with {}",
            output.status
        )
        .into());
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "200" => Ok(true),
        "404" => Ok(false),
        status => Err(format!("NuGet package availability check returned HTTP {status}").into()),
    }
}

fn nuget_package_url(version: &str) -> String {
    format!(
        "https://api.nuget.org/v3-flatcontainer/revault.api/{version}/revault.api.{version}.nupkg"
    )
}

fn dart(package: &Path, version: &str, publish: bool) -> Result {
    require_text_version(&package.join("pubspec.yaml"), version)?;
    let mut command = Command::new("dart");
    command.args(["pub", "publish"]);
    if publish {
        command.arg("--force");
    } else {
        command.arg("--dry-run");
    }
    run(command.current_dir(package))
}

fn ruby(
    packages: &Path,
    version: &str,
    output: &Path,
    publish: bool,
    target: Option<String>,
) -> Result {
    let roots = platform_roots(&packages.join("ruby"), target)?;
    let destination = output.join("ruby");
    fs::create_dir_all(&destination)?;
    for root in roots {
        require_text_version(&root.join("revault_api.gemspec"), version)?;
        run(Command::new("gem")
            .args(["build", "revault_api.gemspec", "--output"])
            .arg(destination.join(format!(
                "revault_api-{version}-{}.gem",
                root.file_name().unwrap().to_string_lossy()
            )))
            .current_dir(&root))?;
    }
    let gems = files_with_extension(&destination, "gem")?;
    if publish {
        for gem in gems {
            push_gem(&gem)?;
        }
    }
    Ok(())
}

fn push_gem(gem: &Path) -> Result {
    let output = Command::new("gem").arg("push").arg(gem).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{stdout}");
    eprint!("{stderr}");
    if output.status.success() {
        return Ok(());
    }
    let message = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    if message.contains("repushing of gem versions is not allowed")
        || message.contains("already been pushed")
    {
        println!("already published: {}", gem.display());
        return Ok(());
    }
    Err(format!(
        "command failed ({}): gem push {}",
        output.status,
        gem.display()
    )
    .into())
}

fn lua(
    packages: &Path,
    version: &str,
    output: &Path,
    publish: bool,
    target: Option<String>,
) -> Result {
    let roots = platform_roots(&packages.join("lua"), target)?;
    let destination = output.join("lua");
    fs::create_dir_all(&destination)?;
    let api_key = if publish {
        let key = std::env::var("LUAROCKS_API_KEY")
            .map_err(|_| "LUAROCKS_API_KEY is required for LuaRocks publication")?;
        let key = key.trim().to_owned();
        if key.is_empty() {
            return Err("LUAROCKS_API_KEY must not be empty".into());
        }
        Some(key)
    } else {
        None
    };
    for root in roots {
        let rockspec = root.join(format!("revault_api-{version}-1.rockspec"));
        let rockspec_name = rockspec
            .file_name()
            .ok_or("LuaRocks rockspec path has no filename")?;
        require_text_version(&rockspec, version)?;
        run(Command::new("luarocks")
            .args(["--lua-version=5.1", "lint"])
            .arg(rockspec_name)
            .current_dir(&root))?;
        run(Command::new("luarocks")
            .args([
                "--lua-version=5.1",
                "make",
                "--pack-binary-rock",
                "--deps-mode=none",
            ])
            .arg(rockspec_name)
            .current_dir(&root))?;
        let rocks = files_with_extension(&root, "rock")?;
        if rocks.is_empty() {
            return Err(
                format!("LuaRocks produced no binary rock under {}", root.display()).into(),
            );
        }
        for rock in rocks {
            let staged = destination.join(rock.file_name().ok_or("binary rock has no filename")?);
            fs::copy(&rock, &staged)?;
            if let Some(api_key) = api_key.as_deref() {
                let version_id = ensure_luarocks_version(api_key, &rockspec, version)?;
                upload_binary_rock(api_key, version_id, &staged)?;
            }
        }
    }
    Ok(())
}

fn ensure_luarocks_version(api_key: &str, rockspec: &Path, version: &str) -> Result<u64> {
    if let Some(id) = check_luarocks_version(api_key, version)? {
        return Ok(id);
    }

    let form = Form::new().file("rockspec_file", rockspec)?;
    let upload = (|| -> Result<LuaRocksVersionResponse> {
        let response = ureq::post("https://luarocks.org/api/1/bearer/upload")
            .config()
            .http_status_as_error(false)
            .build()
            .header("Authorization", &format!("Bearer {api_key}"))
            .send(form)?;
        decode_luarocks_response(response, "rockspec upload")
    })();
    match upload {
        Ok(uploaded) => {
            let uploaded: LuaRocksVersionResponse = uploaded;
            uploaded
                .version
                .map(|version| version.id)
                .ok_or_else(|| "LuaRocks rockspec upload returned no version".into())
        }
        Err(upload_error) => {
            // Parallel platform jobs can all observe the version as absent. One
            // creates it and the others resolve the resulting version below.
            for _ in 0..10 {
                if let Some(id) = check_luarocks_version(api_key, version)? {
                    return Ok(id);
                }
                thread::sleep(Duration::from_millis(500));
            }
            Err(format!("LuaRocks rockspec upload failed: {upload_error}").into())
        }
    }
}

fn check_luarocks_version(api_key: &str, version: &str) -> Result<Option<u64>> {
    let response = ureq::get("https://luarocks.org/api/1/bearer/check_rockspec")
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", &format!("Bearer {api_key}"))
        .query("package", "revault_api")
        .query("version", luarocks_version(version))
        .call()?;
    let checked: LuaRocksVersionResponse = decode_luarocks_response(response, "version check")?;
    Ok(checked.version.map(|version| version.id))
}

fn upload_binary_rock(api_key: &str, version_id: u64, rock: &Path) -> Result {
    let form = Form::new().file("rock_file", rock)?;
    let url = format!("https://luarocks.org/api/1/bearer/upload_rock/{version_id}");
    let response = ureq::post(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", &format!("Bearer {api_key}"))
        .send(form)?;
    let _: serde_json::Value = decode_luarocks_response(response, "binary rock upload")?;
    println!("published binary rock: {}", rock.display());
    Ok(())
}

fn decode_luarocks_response<T: DeserializeOwned>(
    mut response: ureq::http::Response<ureq::Body>,
    operation: &str,
) -> Result<T> {
    let status = response.status();
    let body = response.body_mut().read_to_string()?;
    if !status.is_success() {
        let details = serde_json::from_str::<LuaRocksErrorResponse>(&body)
            .ok()
            .filter(|error| !error.errors.is_empty())
            .map(|error| error.errors.join("; "))
            .unwrap_or_else(|| body.trim().to_owned());
        return Err(format!("LuaRocks {operation} failed with HTTP {status}: {details}").into());
    }
    serde_json::from_str(&body)
        .map_err(|error| format!("invalid LuaRocks {operation} response: {error}").into())
}

fn luarocks_version(version: &str) -> String {
    format!("{version}-1")
}

fn rust_foundation(packages: &Path, publish: bool) -> Result {
    for (name, version) in [
        ("revault_page_api", "0.0.3"),
        ("revault_lockbox_api", "0.0.4"),
        ("revault_vault_api", "0.0.4"),
    ] {
        rust_package(
            &packages.join("rust").join(name),
            name,
            version,
            publish,
            false,
        )?;
    }
    Ok(())
}

fn rust_package(package: &Path, name: &str, version: &str, publish: bool, locked: bool) -> Result {
    require_text_version(&package.join("Cargo.toml"), version)?;
    if publish && crate_version_exists(name, version) {
        println!("already published: {name}@{version}");
        return Ok(());
    }
    let mut command = Command::new("cargo");
    command.arg(if publish { "publish" } else { "package" });
    command.arg("--allow-dirty");
    if locked {
        command.arg("--locked");
    }
    if !publish {
        // `--list` validates the final package file set without requiring a
        // just-versioned dependency to have reached the crates.io index yet.
        command.arg("--list");
    }
    run(command.current_dir(package))
}

fn crate_version_exists(name: &str, version: &str) -> bool {
    Command::new("cargo")
        .args([
            "info",
            "--registry",
            "crates-io",
            &format!("{name}@{version}"),
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn promote_git(args: PromoteGitPackage) -> Result {
    validate_version(&args.version)?;
    let packages = args.packages.canonicalize()?;
    let destination = args.destination.canonicalize()?;
    if !destination.join(".git").is_dir() {
        return Err(format!(
            "destination is not a Git checkout: {}",
            destination.display()
        )
        .into());
    }
    clear_checkout_tree(&destination)?;
    match args.package {
        GitPackage::Api => {
            for source in [
                packages.join("go"),
                packages.join("swift"),
                packages.join("composer"),
            ] {
                if !source.is_dir() {
                    return Err(
                        format!("publication source is missing: {}", source.display()).into(),
                    );
                }
                copy_tree(&source, &destination)?;
            }
            prepare_swift(&destination, &args.version, args.release_assets.as_deref())?
        }
        GitPackage::Homebrew => {
            let source = packages.join("native-sdk/package-managers/homebrew");
            if !source.is_dir() {
                return Err(format!("publication source is missing: {}", source.display()).into());
            }
            copy_tree(&source, &destination)?;
            prepare_homebrew(&destination, &args.version, args.release_assets.as_deref())?
        }
    }
    run(Command::new("git")
        .args(["diff", "--check"])
        .current_dir(&destination))?;
    if !args.publish {
        println!(
            "validated {:?} promotion for v{} in {}",
            args.package,
            args.version,
            destination.display()
        );
        return Ok(());
    }
    run(Command::new("git")
        .args(["add", "-A"])
        .current_dir(&destination))?;
    let changed = !Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&destination)
        .status()?
        .success();
    if changed {
        run(Command::new("git")
            .args([
                "-c",
                "user.name=Brett Sutton",
                "-c",
                "user.email=bsutton@onepub.dev",
                "commit",
                "-m",
                &format!("Release {}", args.version),
            ])
            .current_dir(&destination))?;
    }
    let tag = format!("v{}", args.version);
    let tag_exists = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/tags/{tag}")])
        .current_dir(&destination)
        .output()
        .is_ok_and(|output| output.status.success());
    if !tag_exists {
        run(Command::new("git")
            .args([
                "-c",
                "user.name=Brett Sutton",
                "-c",
                "user.email=bsutton@onepub.dev",
                "tag",
                "-a",
                &tag,
                "-m",
                &format!("Release {tag}"),
            ])
            .current_dir(&destination))?;
    }
    run(Command::new("git")
        .args(["push", "origin", "HEAD"])
        .current_dir(&destination))?;
    run(Command::new("git")
        .args(["push", "origin", &tag])
        .current_dir(&destination))
}

fn prepare_swift(destination: &Path, version: &str, assets: Option<&Path>) -> Result {
    let assets = assets.ok_or("--release-assets is required for Swift promotion")?;
    let archive = assets.join("RevaultC.xcframework.zip");
    let checksum = sha256(&archive)?;
    let url = release_url(version, "RevaultC.xcframework.zip");
    let manifest = format!(
        r#"// swift-tools-version: 5.9
import PackageDescription

#if os(Linux)
let revaultC: Target = .systemLibrary(name: "RevaultC", path: "CModule")
#else
let revaultC: Target = .binaryTarget(name: "RevaultC", url: "{url}", checksum: "{checksum}")
#endif

let package = Package(
    name: "RevaultAPI",
    products: [.library(name: "RevaultAPI", targets: ["RevaultAPI"])],
    dependencies: [
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.29.0"),
    ],
    targets: [
        revaultC,
        .target(
            name: "RevaultAPI",
            dependencies: ["RevaultC", .product(name: "SwiftProtobuf", package: "swift-protobuf")],
            path: "Sources/RevaultAPI"
        ),
    ]
)
"#
    );
    fs::write(destination.join("Package.swift"), manifest)?;
    Ok(())
}

fn prepare_homebrew(destination: &Path, version: &str, assets: Option<&Path>) -> Result {
    let assets = assets.ok_or("--release-assets is required for Homebrew promotion")?;
    let template = destination.join("revault-api.rb.in");
    let mut formula = fs::read_to_string(&template)?.replace("@VERSION@", version);
    for (placeholder, target) in [
        ("MACOS_ARM64", "macos-aarch64"),
        ("MACOS_X86_64", "macos-x86_64"),
        ("LINUX_ARM64", "linux-aarch64-gnu"),
        ("LINUX_X86_64", "linux-x86_64-gnu"),
    ] {
        let name = format!("revault-api-native-{version}-{target}.tar.gz");
        formula = formula
            .replace(
                &format!("@{placeholder}_URL@"),
                &release_url(version, &name),
            )
            .replace(
                &format!("@{placeholder}_SHA256@"),
                &sha256(&assets.join(name))?,
            );
    }
    let formula_dir = destination.join("Formula");
    fs::create_dir_all(&formula_dir)?;
    fs::write(formula_dir.join("revault-api.rb"), formula)?;
    fs::remove_file(template)?;
    Ok(())
}

fn release_url(version: &str, asset: &str) -> String {
    format!(
        "https://github.com/onepub-dev/reVault/releases/download/revault-api-v{version}/{asset}"
    )
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn clear_checkout_tree(destination: &Path) -> Result {
    for entry in fs::read_dir(destination)? {
        let entry = entry?;
        if entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result {
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let output = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(output)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), output)?;
        }
    }
    Ok(())
}

fn platform_roots(root: &Path, target: Option<String>) -> Result<Vec<PathBuf>> {
    let mut roots = child_directories(root)?;
    if let Some(target) = target {
        roots.retain(|path| path.file_name() == Some(OsStr::new(&target)));
    }
    if roots.is_empty() {
        return Err(format!("no platform packages under {}", root.display()).into());
    }
    Ok(roots)
}

fn child_directories(root: &Path) -> Result<Vec<PathBuf>> {
    let mut values = fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_type().ok()?.is_dir().then_some(entry.path()))
        .collect::<Vec<_>>();
    values.sort();
    Ok(values)
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut values = fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            (entry.path().extension() == Some(OsStr::new(extension))).then_some(entry.path())
        })
        .collect::<Vec<_>>();
    values.sort();
    Ok(values)
}

fn require_text_version(path: &Path, version: &str) -> Result {
    if !fs::read_to_string(path)?.contains(version) {
        return Err(format!("{} does not declare version {version}", path.display()).into());
    }
    Ok(())
}

fn require_version(actual: &str, expected: &str, path: &Path) -> Result {
    if actual != expected {
        return Err(format!("{} declares {actual}, expected {expected}", path.display()).into());
    }
    Ok(())
}

fn validate_version(version: &str) -> Result {
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|value| value.is_ascii_digit()))
    {
        return Err(format!("release version must be numeric MAJOR.MINOR.PATCH: {version}").into());
    }
    Ok(())
}

fn gradle_manifest(package: &Path) -> Result<PathBuf> {
    for name in ["build.gradle", "build.gradle.kts"] {
        let path = package.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!("Gradle manifest is missing under {}", package.display()).into())
}

fn gradle() -> &'static str {
    if cfg!(windows) {
        "gradle.bat"
    } else {
        "gradle"
    }
}

fn run(command: &mut Command) -> Result {
    let display = format!("{command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed ({status}): {display}").into());
    }
    Ok(())
}

fn run_redacted(command: &mut Command, display: &str) -> Result {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed ({status}): {display}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn release_versions_are_strict_numeric_triples() {
        for value in ["0.1.0", "12.34.56"] {
            validate_version(value).unwrap();
        }
        for value in ["v0.1.0", "0.1", "0.1.0-alpha", "0..1"] {
            assert!(validate_version(value).is_err(), "{value}");
        }
    }

    #[test]
    fn nuget_publication_url_uses_the_flat_container_identity() {
        assert_eq!(
            nuget_package_url("0.1.0"),
            "https://api.nuget.org/v3-flatcontainer/revault.api/0.1.0/revault.api.0.1.0.nupkg"
        );
    }

    #[test]
    fn luarocks_version_includes_the_rockspec_revision() {
        assert_eq!(luarocks_version("0.1.0"), "0.1.0-1");
    }

    #[test]
    fn luarocks_error_response_decodes_error_messages() {
        let response: LuaRocksErrorResponse =
            serde_json::from_str(r#"{"errors":["first", "second"]}"#).unwrap();
        assert_eq!(response.errors, ["first", "second"]);
    }

    #[test]
    fn checkout_replacement_preserves_git_metadata() {
        let source = TempDir::new().unwrap();
        let destination = TempDir::new().unwrap();
        fs::create_dir(destination.path().join(".git")).unwrap();
        fs::write(destination.path().join("old"), b"old").unwrap();
        fs::write(source.path().join("new"), b"new").unwrap();
        clear_checkout_tree(destination.path()).unwrap();
        copy_tree(source.path(), destination.path()).unwrap();
        assert!(destination.path().join(".git").is_dir());
        assert!(!destination.path().join("old").exists());
        assert_eq!(fs::read(destination.path().join("new")).unwrap(), b"new");
    }
}
