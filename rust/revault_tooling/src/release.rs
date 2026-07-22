use crate::Result;
use clap::{Args, Subcommand};
use flate2::{Compression, GzBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tar::{Builder as TarBuilder, Header};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

const NATIVE_ABI_VERSION: u32 = 3;

#[derive(Subcommand)]
pub enum ReleaseCommand {
    /// Create a deterministic native SDK archive and SHA-256 sidecar.
    PackageNative(PackageNative),
    /// Verify and expand all six native archives into ecosystem layouts.
    StageEcosystems(StageEcosystems),
    /// Assemble publishable language package trees from a staged layout.
    AssemblePackages(AssemblePackages),
    /// Build, validate, or publish one registry-native package set.
    Publish(crate::publication::PublishPackages),
    /// Promote a package tree to a Git-native distribution repository.
    PromoteGit(crate::publication::PromoteGitPackage),
    /// Verify one canonical native archive without installing it.
    VerifyArchive(VerifyArchive),
    /// Verify and install one canonical native SDK under a clean prefix.
    InstallNative(InstallNative),
    /// Assemble the accepted macOS archives into a Swift XCFramework zip.
    PackageXcframework(PackageXcframework),
}

#[derive(Args)]
pub struct PackageNative {
    #[arg(long)]
    version: String,
    #[arg(long)]
    target: String,
    #[arg(long)]
    library: PathBuf,
    #[arg(long)]
    static_library: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
}

#[derive(Args)]
pub struct StageEcosystems {
    #[arg(long)]
    version: String,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
    #[arg(required = true)]
    archives: Vec<PathBuf>,
    #[arg(long)]
    allow_partial: bool,
}

#[derive(Args)]
pub struct AssemblePackages {
    #[arg(long)]
    version: String,
    #[arg(long)]
    layout: PathBuf,
    #[arg(long, default_value = ".")]
    source: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    allow_partial: bool,
}

#[derive(Args)]
pub struct VerifyArchive {
    #[arg(long)]
    archive: PathBuf,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    target: Option<String>,
}

#[derive(Args)]
pub struct InstallNative {
    #[arg(long)]
    archive: PathBuf,
    #[arg(long)]
    prefix: PathBuf,
}

#[derive(Args)]
pub struct PackageXcframework {
    #[arg(long)]
    x86_64_archive: PathBuf,
    #[arg(long)]
    aarch64_archive: PathBuf,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Clone, Debug)]
struct TargetRow {
    target: String,
    os: String,
    arch: String,
    rust_target: String,
    library: String,
    static_library: String,
    import_library: Option<String>,
    archive: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NativeMetadata {
    name: String,
    version: String,
    target: String,
    os: String,
    arch: String,
    rust_target: String,
    library: String,
    abi: u32,
    wire: String,
    library_sha256: String,
    static_library: String,
    static_library_sha256: String,
    ruby_shim: String,
    ruby_shim_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    import_library: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    import_library_sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NativeManifest {
    version: String,
    abi: u32,
    targets: Vec<ManifestTarget>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ManifestTarget {
    target: String,
    library: String,
    sha256: String,
    static_library: String,
    static_sha256: String,
}

pub fn run(command: ReleaseCommand) -> Result {
    match command {
        ReleaseCommand::PackageNative(args) => package_native(args),
        ReleaseCommand::StageEcosystems(args) => stage_ecosystems(args),
        ReleaseCommand::AssemblePackages(args) => assemble_packages(args),
        ReleaseCommand::Publish(args) => crate::publication::publish(args),
        ReleaseCommand::PromoteGit(args) => crate::publication::promote_git(args),
        ReleaseCommand::VerifyArchive(args) => {
            let temporary = TempDir::new()?;
            let (_, metadata) = extract_verified(&args.archive, temporary.path())?;
            if args
                .version
                .as_ref()
                .is_some_and(|value| value != &metadata.version)
            {
                return Err(format!(
                    "archive version is {}, expected {}",
                    metadata.version,
                    args.version.unwrap()
                )
                .into());
            }
            if args
                .target
                .as_ref()
                .is_some_and(|value| value != &metadata.target)
            {
                return Err(format!(
                    "archive target is {}, expected {}",
                    metadata.target,
                    args.target.unwrap()
                )
                .into());
            }
            println!("verified {} {}", metadata.target, metadata.version);
            Ok(())
        }
        ReleaseCommand::InstallNative(args) => install_native(args),
        ReleaseCommand::PackageXcframework(args) => package_xcframework(args),
    }
}

fn package_xcframework(args: PackageXcframework) -> Result {
    let temporary = TempDir::new()?;
    let x86 = install_archive(&args.x86_64_archive, &temporary.path().join("x86_64"))?;
    let arm = install_archive(&args.aarch64_archive, &temporary.path().join("aarch64"))?;
    if x86.target != "macos-x86_64" || arm.target != "macos-aarch64" {
        return Err("XCFramework inputs must be macos-x86_64 and macos-aarch64 archives".into());
    }
    fs::create_dir_all(&args.output)?;
    let universal_library = temporary.path().join("librevault_api.a");
    run_status(
        Command::new("lipo")
            .arg("-create")
            .arg(x86.prefix.join("lib/librevault_api.a"))
            .arg(arm.prefix.join("lib/librevault_api.a"))
            .arg("-output")
            .arg(&universal_library),
    )?;
    let framework = args.output.join("RevaultC.xcframework");
    run_status(
        Command::new("xcodebuild")
            .args(["-create-xcframework", "-library"])
            .arg(&universal_library)
            .arg("-headers")
            .arg(x86.prefix.join("include"))
            .arg("-output")
            .arg(&framework),
    )?;
    let archive = args.output.join("RevaultC.xcframework.zip");
    write_zip_tree(&archive, &framework, "RevaultC.xcframework")?;
    fs::write(
        args.output.join("RevaultC.xcframework.zip.sha256"),
        format!("{}\n", sha256(&archive)?),
    )?;
    println!("{}", archive.display());
    Ok(())
}

fn install_native(args: InstallNative) -> Result {
    install_archive(&args.archive, &args.prefix).map(|_| ())
}

fn msvc_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    if let Some(tail) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{tail}")
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(&value).to_owned()
    }
}

fn build_ruby_shim(
    repository: &Path,
    row: &TargetRow,
    library: &Path,
    import_library: Option<&Path>,
    output: &Path,
) -> Result<PathBuf> {
    let source = repository.join("bindings/ruby/native/revault_ruby_shim.c");
    let include = repository.join("rust/revault_bindings");
    let library_dir = library.parent().ok_or("native library has no parent")?;
    let name = match row.os.as_str() {
        "linux" => "librevault_ruby_shim.so",
        "macos" => "librevault_ruby_shim.dylib",
        "windows" => "revault_ruby_shim.dll",
        value => return Err(format!("unsupported Ruby shim operating system: {value}").into()),
    };
    let destination = output.join(name);
    let mut command = if row.os == "windows" {
        let import_library =
            import_library.ok_or("Windows Ruby shim requires an import library")?;
        let mut command = Command::new("cl.exe");
        command
            .arg("/nologo")
            .arg("/LD")
            .arg(format!("/I{}", msvc_path(&include)))
            .arg(msvc_path(&source))
            .arg(msvc_path(import_library))
            .arg("/link")
            .arg(format!("/OUT:{}", msvc_path(&destination)));
        for symbol in ruby_shim_symbols(&source)? {
            command.arg(format!("/EXPORT:{symbol}"));
        }
        command
    } else {
        let mut command = Command::new("cc");
        if row.os == "macos" {
            command.args(["-dynamiclib", "-Wl,-rpath,@loader_path"]);
        } else {
            command.args(["-shared", "-fPIC", "-Wl,-z,defs", "-Wl,-rpath,$ORIGIN"]);
        }
        command
            .arg(format!("-I{}", include.display()))
            .arg(&source)
            .arg(format!("-L{}", library_dir.display()))
            .arg("-lrevault_api")
            .arg("-o")
            .arg(&destination);
        command
    };
    run_status(&mut command)?;
    if !destination.is_file() {
        return Err(format!(
            "Ruby shim compiler did not create {}",
            destination.display()
        )
        .into());
    }
    Ok(destination)
}

fn ruby_shim_symbols(source: &Path) -> Result<Vec<String>> {
    let source = fs::read_to_string(source)?;
    let symbols: Vec<_> = source
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("void ruby_")
                .and_then(|tail| tail.split_once('('))
                .map(|(name, _)| format!("ruby_{name}"))
        })
        .collect();
    if symbols.is_empty() {
        return Err("Ruby shim has no exported functions".into());
    }
    Ok(symbols)
}

pub(crate) fn install_archive(archive: &Path, prefix: &Path) -> Result<NativeInstall> {
    let temporary = TempDir::new()?;
    let (root, metadata) = extract_verified(archive, temporary.path())?;
    copy_tree(&root, prefix)?;
    println!(
        "installed {} {} under {}",
        metadata.target,
        metadata.version,
        prefix.display()
    );
    Ok(NativeInstall {
        target: metadata.target,
        library: metadata.library,
        prefix: prefix.to_path_buf(),
    })
}

pub(crate) struct NativeInstall {
    pub target: String,
    pub library: String,
    pub prefix: PathBuf,
}

fn package_native(args: PackageNative) -> Result {
    let repository = args.repository.canonicalize()?;
    let rows = target_rows(&repository)?;
    let row = rows
        .get(&args.target)
        .ok_or_else(|| format!("unsupported native target: {}", args.target))?;
    let library = repository_relative(&repository, &args.library);
    let static_library = repository_relative(&repository, &args.static_library);
    require_file_name(&library, &row.library, &args.target)?;
    require_file_name(&static_library, &row.static_library, &args.target)?;
    let library = library.canonicalize()?;
    let static_library = static_library.canonicalize()?;
    let import_library = row
        .import_library
        .as_ref()
        .map(|name| library.with_file_name(name))
        .map(|path| path.canonicalize())
        .transpose()?;
    let epoch = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(315_532_800);
    let prefix = format!("revault-api-native-{}-{}", args.version, args.target);
    let temporary = TempDir::new()?;
    let ruby_shim = build_ruby_shim(
        &repository,
        row,
        &library,
        import_library.as_deref(),
        temporary.path(),
    )?;
    let ruby_shim_name = ruby_shim
        .file_name()
        .ok_or("Ruby shim has no file name")?
        .to_string_lossy()
        .into_owned();
    let metadata = NativeMetadata {
        name: "revault-api-native".into(),
        version: args.version.clone(),
        target: row.target.clone(),
        os: row.os.clone(),
        arch: row.arch.clone(),
        rust_target: row.rust_target.clone(),
        library: row.library.clone(),
        abi: NATIVE_ABI_VERSION,
        wire: "FlatBuffers/25.2.10".into(),
        library_sha256: sha256(&library)?,
        static_library: row.static_library.clone(),
        static_library_sha256: sha256(&static_library)?,
        ruby_shim: ruby_shim_name.clone(),
        ruby_shim_sha256: sha256(&ruby_shim)?,
        import_library: row.import_library.clone(),
        import_library_sha256: import_library
            .as_ref()
            .map(|path| sha256(path))
            .transpose()?,
    };
    let metadata_path = temporary.path().join("metadata.json");
    write_json(&metadata_path, &metadata)?;
    let sbom_path = temporary.path().join("sbom.spdx.json");
    write_json(
        &sbom_path,
        &serde_json::json!({
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": prefix,
            "documentNamespace": format!("https://github.com/onepub-dev/reVault/releases/{prefix}"),
            "creationInfo": {"created": "1980-01-01T00:00:00Z", "creators": ["Tool: revault-tool"]},
            "packages": [{
                "name": "revault-api-native", "SPDXID": "SPDXRef-Package-revault-api-native",
                "versionInfo": args.version, "downloadLocation": "NOASSERTION", "filesAnalyzed": false,
                "licenseConcluded": "LicenseRef-reVault-Source-Available-1.0",
                "checksums": [{"algorithm": "SHA256", "checksumValue": metadata.library_sha256}]
            }]
        }),
    )?;
    let mut files = vec![
        (library, format!("{prefix}/lib/{}", row.library)),
        (
            static_library,
            format!("{prefix}/lib/{}", row.static_library),
        ),
        (ruby_shim, format!("{prefix}/lib/{ruby_shim_name}")),
        (
            repository.join("rust/revault_bindings/revault_api.h"),
            format!("{prefix}/include/revault_api.h"),
        ),
        (
            repository.join("bindings/flatbuffers/revault_bindings.fbs"),
            format!("{prefix}/flatbuffers/revault_bindings.fbs"),
        ),
        (
            repository.join("rust/revault_lockbox_api/LICENSE"),
            format!("{prefix}/LICENSE"),
        ),
        (metadata_path, format!("{prefix}/metadata.json")),
        (sbom_path, format!("{prefix}/sbom.spdx.json")),
    ];
    if let (Some(source), Some(name)) = (import_library, &row.import_library) {
        files.push((source, format!("{prefix}/lib/{name}")));
    }
    fs::create_dir_all(&args.output)?;
    let suffix = if row.archive == "zip" {
        ".zip"
    } else {
        ".tar.gz"
    };
    let destination = args.output.join(format!("{prefix}{suffix}"));
    if row.archive == "zip" {
        write_zip(&destination, &files)?;
    } else {
        write_tar_gz(&destination, &files, epoch)?;
    }
    let checksum = sha256(&destination)?;
    fs::write(
        destination.with_file_name(format!(
            "{}.sha256",
            destination.file_name().unwrap().to_string_lossy()
        )),
        format!(
            "{checksum}  {}\n",
            destination.file_name().unwrap().to_string_lossy()
        ),
    )?;
    println!("{}", destination.display());
    Ok(())
}

fn stage_ecosystems(args: StageEcosystems) -> Result {
    let rows = target_rows(&args.repository.canonicalize()?)?;
    fs::create_dir_all(&args.output)?;
    let temporary = TempDir::new()?;
    let mut seen = BTreeSet::new();
    let mut manifest = Vec::new();
    for (index, archive) in args.archives.iter().enumerate() {
        let destination = temporary.path().join(index.to_string());
        let (root, metadata) = extract_verified(archive, &destination)?;
        let row = rows
            .get(&metadata.target)
            .ok_or_else(|| format!("unknown target {}", metadata.target))?;
        if metadata.version != args.version || !seen.insert(metadata.target.clone()) {
            return Err(format!(
                "invalid, duplicate, or wrong-version target: {}",
                metadata.target
            )
            .into());
        }
        let library = root.join("lib").join(&row.library);
        let static_library = root.join("lib").join(&row.static_library);
        let target = &metadata.target;
        copy_file(
            &root.join("include/revault_api.h"),
            &args.output.join("swift/include/revault_api.h"),
        )?;
        let (npm_os, npm_cpu) = npm_platform(target)?;
        let npm_root = args
            .output
            .join("npm")
            .join(format!("revault-api-native-{target}"));
        copy_file(&library, &npm_root.join("lib").join(&row.library))?;
        copy_file(&root.join("LICENSE"), &npm_root.join("LICENSE"))?;
        write_json(
            &npm_root.join("package.json"),
            &serde_json::json!({
                "name": format!("@onepub-dev/revault-api-native-{target}"), "version": args.version,
                "description": format!("reVault native runtime for {target}"),
                "license": "SEE LICENSE IN LICENSE", "repository": "github:onepub-dev/reVault",
                "os": [npm_os], "cpu": [npm_cpu], "files": ["lib", "LICENSE"]
            }),
        )?;
        copy_file(
            &library,
            &args
                .output
                .join("nuget/runtimes")
                .join(rid(target)?)
                .join("native")
                .join(&row.library),
        )?;
        copy_file(
            &library,
            &args
                .output
                .join("maven/META-INF/native")
                .join(target)
                .join(&row.library),
        )?;
        for (ecosystem, tail) in [("python", "_native"), ("ruby", "native"), ("lua", "native")] {
            copy_file(
                &library,
                &args
                    .output
                    .join(ecosystem)
                    .join(target)
                    .join(tail)
                    .join(target)
                    .join(&row.library),
            )?;
        }
        copy_file(
            &root.join("lib").join(&metadata.ruby_shim),
            &args
                .output
                .join("ruby")
                .join(target)
                .join("native")
                .join(target)
                .join(&metadata.ruby_shim),
        )?;
        copy_file(
            &library,
            &args
                .output
                .join("dart/lib/src/native")
                .join(target)
                .join(&row.library),
        )?;
        copy_file(
            &library,
            &args
                .output
                .join("php/native")
                .join(target)
                .join(&row.library),
        )?;
        copy_file(
            &static_library,
            &args
                .output
                .join("go/native")
                .join(target)
                .join(&row.static_library),
        )?;
        manifest.push(ManifestTarget {
            target: target.clone(),
            library: row.library.clone(),
            sha256: metadata.library_sha256,
            static_library: row.static_library.clone(),
            static_sha256: metadata.static_library_sha256,
        });
    }
    let missing: Vec<_> = rows
        .keys()
        .filter(|target| !seen.contains(*target))
        .cloned()
        .collect();
    if !args.allow_partial && !missing.is_empty() {
        return Err(format!("missing native targets: {}", missing.join(", ")).into());
    }
    manifest.sort_by(|a, b| a.target.cmp(&b.target));
    let count = manifest.len();
    write_json(
        &args.output.join("native-manifest.json"),
        &NativeManifest {
            version: args.version,
            abi: NATIVE_ABI_VERSION,
            targets: manifest,
        },
    )?;
    println!("staged {count} native targets for ecosystem packages");
    Ok(())
}

fn assemble_packages(args: AssemblePackages) -> Result {
    let source = args.source.canonicalize()?;
    let layout = args.layout.canonicalize()?;
    let bindings = source.join("bindings");
    let manifest: NativeManifest =
        serde_json::from_slice(&fs::read(layout.join("native-manifest.json"))?)?;
    if manifest.version != args.version
        || manifest.abi != NATIVE_ABI_VERSION
        || (!args.allow_partial && manifest.targets.len() != 6)
        || manifest.targets.is_empty()
    {
        return Err(
            "layout must contain the requested version, ABI 2, and required native targets".into(),
        );
    }
    let output = &args.output;
    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output)?;
    copy_file(&source.join("LICENSE"), &output.join("LICENSE"))?;
    copy_tree(
        &bindings.join("javascript"),
        &output.join("npm/revault-api"),
    )?;
    copy_tree(&bindings.join("wasm"), &output.join("npm/revault-api-wasm"))?;
    copy_tree(&layout.join("npm"), &output.join("npm"))?;
    replace_release_version(&output.join("npm/revault-api/package.json"), &args.version)?;
    replace_release_version(
        &output.join("npm/revault-api-wasm/package.json"),
        &args.version,
    )?;
    require_version(&output.join("npm/revault-api/package.json"), &args.version)?;
    require_version(
        &output.join("npm/revault-api-wasm/package.json"),
        &args.version,
    )?;
    for entry in &manifest.targets {
        let target = &entry.target;
        let python = output.join("python").join(target);
        copy_tree(&bindings.join("python"), &python)?;
        copy_tree(
            &layout.join("python").join(target).join("_native"),
            &python.join("revault_api/_native"),
        )?;
        replace_release_version(&python.join("pyproject.toml"), &args.version)?;
        require_version(&python.join("pyproject.toml"), &args.version)?;
        for ecosystem in ["ruby", "lua"] {
            let destination = output.join(ecosystem).join(target);
            copy_tree(&bindings.join(ecosystem), &destination)?;
            copy_tree(
                &layout.join(ecosystem).join(target).join("native"),
                &destination.join("native"),
            )?;
            if ecosystem == "lua" {
                let template = find_single_file(&destination, "revault_api-", "-1.rockspec")?;
                let rockspec = destination.join(format!("revault_api-{}-1.rockspec", args.version));
                let source = replace_version_text(
                    &fs::read_to_string(&template)?,
                    &format!("{}-1", args.version),
                )?
                .replace("@NATIVE_TARGET@", target)
                .replace("@NATIVE_LIBRARY@", &entry.library);
                fs::write(&rockspec, source)?;
                if template != rockspec {
                    fs::remove_file(template)?;
                }
            } else {
                let gemspec = destination.join("revault_api.gemspec");
                let source = replace_version_text(&fs::read_to_string(&gemspec)?, &args.version)?
                    .replace("@GEM_PLATFORM@", gem_platform(target)?);
                fs::write(gemspec, source)?;
            }
        }
    }
    let java = output.join("maven/java");
    copy_tree(&bindings.join("java"), &java)?;
    copy_tree(&layout.join("maven"), &java.join("resources"))?;
    replace_release_version(&java.join("build.gradle"), &args.version)?;
    require_version(&java.join("build.gradle"), &args.version)?;
    copy_tree(&bindings.join("kotlin"), &output.join("maven/kotlin"))?;
    replace_release_version(&output.join("maven/kotlin/build.gradle.kts"), &args.version)?;
    copy_tree(&bindings.join("csharp"), &output.join("nuget"))?;
    copy_tree(
        &layout.join("nuget/runtimes"),
        &output.join("nuget/runtimes"),
    )?;
    replace_release_version(&output.join("nuget/RevaultBindings.csproj"), &args.version)?;
    require_version(&output.join("nuget/RevaultBindings.csproj"), &args.version)?;
    copy_tree(&bindings.join("dart"), &output.join("dart"))?;
    copy_tree(&layout.join("dart/lib"), &output.join("dart/lib"))?;
    replace_release_version(&output.join("dart/pubspec.yaml"), &args.version)?;
    require_version(&output.join("dart/pubspec.yaml"), &args.version)?;
    copy_tree(&bindings.join("php"), &output.join("composer"))?;
    copy_tree(&layout.join("php/native"), &output.join("composer/native"))?;
    assemble_go(&source, &layout, output, &manifest)?;
    for (name, destination) in [
        ("rust", "cargo/revault-api"),
        ("swift", "swift"),
        ("c", "native-sdk/c"),
        ("cpp", "native-sdk/cpp"),
    ] {
        copy_tree(&bindings.join(name), &output.join(destination))?;
    }
    let cargo_manifest = output.join("cargo/revault-api/Cargo.toml");
    replace_release_version(&cargo_manifest, &args.version)?;
    replace_cargo_lock_package_version(
        &output.join("cargo/revault-api/Cargo.lock"),
        "revault-api",
        &args.version,
    )?;
    for dependency in [
        "revault_lockbox_api",
        "revault_page_api",
        "revault_vault_api",
    ] {
        copy_tree(
            &source.join("rust").join(dependency),
            &output.join("rust").join(dependency),
        )?;
    }
    copy_file(
        &layout.join("swift/include/revault_api.h"),
        &output.join("swift/CModule/revault_api.h"),
    )?;
    copy_tree(
        &bindings.join("release/package-managers"),
        &output.join("native-sdk/package-managers"),
    )?;
    copy_file(
        &source.join("rust/revault_bindings/revault_api.h"),
        &output.join("native-sdk/include/revault_api.h"),
    )?;
    copy_file(
        &layout.join("native-manifest.json"),
        &output.join("native-manifest.json"),
    )?;
    println!("assembled complete revault-api publication trees for all sixteen language APIs");
    Ok(())
}

fn replace_release_version(path: &Path, version: &str) -> Result {
    let source = fs::read_to_string(path)?;
    fs::write(path, replace_version_text(&source, version)?)?;
    Ok(())
}

fn replace_version_text(source: &str, version: &str) -> Result<String> {
    const MARKERS: &[(&str, &str)] = &[
        ("\"version\": \"", "\""),
        ("version = \"", "\""),
        ("version = '", "'"),
        ("spec.version = \"", "\""),
        ("<Version>", "</Version>"),
        ("version: ", "\n"),
    ];
    let current = MARKERS.iter().find_map(|(prefix, suffix)| {
        let start = source.find(prefix)? + prefix.len();
        let end = source[start..].find(suffix)? + start;
        Some(&source[start..end])
    });
    let Some(current) = current else {
        return Err("release manifest does not contain a recognizable version".into());
    };
    Ok(source.replace(current, version))
}

fn replace_cargo_lock_package_version(path: &Path, package: &str, version: &str) -> Result {
    let source = fs::read_to_string(path)?;
    let name = format!("name = \"{package}\"");
    let start = source.find(&name).ok_or_else(|| {
        format!(
            "{} does not contain lock entry for {package}",
            path.display()
        )
    })?;
    let block_end = source[start..]
        .find("\n\n")
        .map_or(source.len(), |end| start + end);
    let block = &source[start..block_end];
    let prefix = "version = \"";
    let version_start = block
        .find(prefix)
        .ok_or_else(|| format!("lock entry for {package} has no version"))?
        + prefix.len();
    let version_end = block[version_start..]
        .find('"')
        .ok_or_else(|| format!("lock entry for {package} has an invalid version"))?
        + version_start;
    let marker = format!(
        "name = \"{package}\"\nversion = \"{}\"",
        &block[version_start..version_end]
    );
    let replacement = format!("name = \"{package}\"\nversion = \"{version}\"");
    if !source.contains(&marker) {
        return Err(format!(
            "{} does not contain lock entry for {package}",
            path.display()
        )
        .into());
    }
    fs::write(path, source.replacen(&marker, &replacement, 1))?;
    Ok(())
}

fn find_single_file(directory: &Path, prefix: &str, suffix: &str) -> Result<PathBuf> {
    let mut matches = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        });
    let file = matches
        .next()
        .ok_or_else(|| format!("{} has no {prefix}*{suffix} file", directory.display()))?;
    if matches.next().is_some() {
        return Err(format!(
            "{} has multiple {prefix}*{suffix} files",
            directory.display()
        )
        .into());
    }
    Ok(file)
}

fn assemble_go(source: &Path, layout: &Path, output: &Path, manifest: &NativeManifest) -> Result {
    let go = output.join("go");
    copy_tree(&source.join("bindings/go"), &go)?;
    copy_tree(&layout.join("go/native"), &go.join("native"))?;
    for name in ["revault.go", "revault_native.go"] {
        let file = go.join(name);
        let text = fs::read_to_string(&file)?
            .replace("#cgo LDFLAGS: -lrevault_api\n", "")
            .replace(
                "#include \"../../rust/revault_bindings/revault_api.h\"",
                "#include \"revault_api.h\"",
            );
        fs::write(file, text)?;
    }
    for entry in &manifest.targets {
        let (os, arch, system) = go_platform(&entry.target)?;
        fs::write(
            go.join(format!("link_{os}_{arch}.go")),
            format!("//go:build {os} && {arch}\n\npackage revault\n\n/*\n#cgo LDFLAGS: ${{SRCDIR}}/native/{}/{} {system}\n*/\nimport \"C\"\n", entry.target, entry.static_library),
        )?;
    }
    copy_file(
        &source.join("rust/revault_bindings/revault_api.h"),
        &go.join("revault_api.h"),
    )
}

fn extract_verified(archive: &Path, destination: &Path) -> Result<(PathBuf, NativeMetadata)> {
    verify_sidecar_if_present(archive)?;
    fs::create_dir_all(destination)?;
    if archive.extension() == Some(OsStr::new("zip")) {
        let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index)?;
            let enclosed = entry
                .enclosed_name()
                .ok_or("unsafe path in zip archive")?
                .to_path_buf();
            let output = destination.join(enclosed);
            if entry.is_dir() {
                fs::create_dir_all(&output)?;
            } else {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                std::io::copy(&mut entry, &mut File::create(output)?)?;
            }
        }
    } else {
        let decoder = flate2::read::GzDecoder::new(File::open(archive)?);
        let mut tar = tar::Archive::new(decoder);
        for item in tar.entries()? {
            let mut entry = item?;
            let path = entry.path()?.into_owned();
            require_safe_relative(&path)?;
            entry.unpack_in(destination)?;
        }
    }
    let roots: Vec<_> = fs::read_dir(destination)?
        .filter_map(|entry| entry.ok())
        .collect();
    if roots.len() != 1 || !roots[0].file_type()?.is_dir() {
        return Err(format!(
            "archive must contain exactly one root directory: {}",
            archive.display()
        )
        .into());
    }
    let root = roots[0].path();
    let metadata: NativeMetadata = serde_json::from_slice(&fs::read(root.join("metadata.json"))?)?;
    if metadata.abi != NATIVE_ABI_VERSION || metadata.wire != "FlatBuffers/25.2.10" {
        return Err("unsupported native archive ABI or wire protocol".into());
    }
    let library = root.join("lib").join(&metadata.library);
    let static_library = root.join("lib").join(&metadata.static_library);
    if sha256(&library)? != metadata.library_sha256
        || sha256(&static_library)? != metadata.static_library_sha256
        || sha256(&root.join("lib").join(&metadata.ruby_shim))? != metadata.ruby_shim_sha256
    {
        return Err(format!("native library checksum mismatch: {}", metadata.target).into());
    }
    if let (Some(name), Some(expected)) =
        (&metadata.import_library, &metadata.import_library_sha256)
    {
        if sha256(&root.join("lib").join(name))? != *expected {
            return Err(format!(
                "native import library checksum mismatch: {}",
                metadata.target
            )
            .into());
        }
    }
    Ok((root, metadata))
}

fn target_rows(repository: &Path) -> Result<BTreeMap<String, TargetRow>> {
    let source = fs::read_to_string(repository.join("bindings/release/native-targets.tsv"))?;
    let mut lines = source.lines();
    let header: Vec<_> = lines
        .next()
        .ok_or("native target table is empty")?
        .split('\t')
        .collect();
    let required = [
        "target",
        "os",
        "arch",
        "rust_target",
        "runner",
        "library",
        "static_library",
        "import_library",
        "archive",
    ];
    if header != required {
        return Err("unexpected native-targets.tsv header".into());
    }
    let mut rows = BTreeMap::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let values: Vec<_> = line.split('\t').collect();
        if values.len() != required.len() {
            return Err(format!("invalid native target row: {line}").into());
        }
        let row = TargetRow {
            target: values[0].into(),
            os: values[1].into(),
            arch: values[2].into(),
            rust_target: values[3].into(),
            library: values[5].into(),
            static_library: values[6].into(),
            import_library: (!values[7].is_empty()).then(|| values[7].into()),
            archive: values[8].into(),
        };
        if rows.insert(row.target.clone(), row).is_some() {
            return Err("duplicate native target".into());
        }
    }
    Ok(rows)
}

fn write_tar_gz(destination: &Path, files: &[(PathBuf, String)], epoch: u64) -> Result {
    let gzip = GzBuilder::new()
        .mtime(epoch as u32)
        .write(File::create(destination)?, Compression::default());
    let mut tar = TarBuilder::new(gzip);
    for (source, name) in files {
        let mut header = Header::new_gnu();
        header.set_size(fs::metadata(source)?.len());
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(epoch);
        header.set_cksum();
        tar.append_data(&mut header, name, File::open(source)?)?;
    }
    tar.into_inner()?.finish()?;
    Ok(())
}

fn write_zip(destination: &Path, files: &[(PathBuf, String)]) -> Result {
    let mut zip = zip::ZipWriter::new(File::create(destination)?);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (source, name) in files {
        zip.start_file(name, options)?;
        std::io::copy(&mut File::open(source)?, &mut zip)?;
    }
    zip.finish()?;
    Ok(())
}

fn write_zip_tree(destination: &Path, source: &Path, prefix: &str) -> Result {
    let mut zip = zip::ZipWriter::new(File::create(destination)?);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for entry in WalkDir::new(source).sort_by_file_name() {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(source)?
            .to_string_lossy()
            .replace('\\', "/");
        zip.start_file(format!("{prefix}/{relative}"), options)?;
        std::io::copy(&mut File::open(entry.path())?, &mut zip)?;
    }
    zip.finish()?;
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result {
    if !source.is_dir() {
        return Err(format!("source directory does not exist: {}", source.display()).into());
    }
    for entry in WalkDir::new(source).into_iter().filter_entry(|entry| {
        entry.path() == source
            || entry
                .path()
                .strip_prefix(source)
                .is_ok_and(|relative| !ignored(relative))
    }) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let output = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(output)?;
        } else if entry.file_type().is_file() {
            copy_file(entry.path(), &output)?;
        }
    }
    Ok(())
}

fn ignored(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str().to_str().is_some_and(|value| {
            matches!(
                value,
                "node_modules"
                    | "build"
                    | "dist"
                    | "target"
                    | ".dart_tool"
                    | ".gradle"
                    | ".build"
                    | "vendor"
                    | "__pycache__"
            ) || value.ends_with(".egg-info")
        })
    }) || path.extension().is_some_and(|extension| extension == "pyc")
}

fn copy_file(source: &Path, destination: &Path) -> Result {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn repository_relative(repository: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository.join(path)
    }
}

fn require_version(path: &Path, version: &str) -> Result {
    if !fs::read_to_string(path)?.contains(version) {
        return Err(format!(
            "{} does not declare release version {version}",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn require_file_name(path: &Path, expected: &str, target: &str) -> Result {
    if path.file_name() != Some(OsStr::new(expected)) {
        return Err(format!("{target} requires {expected}, got {}", path.display()).into());
    }
    if !path.is_file() {
        return Err(format!("native library does not exist: {}", path.display()).into());
    }
    Ok(())
}

fn require_safe_relative(path: &Path) -> Result {
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe archive path: {}", path.display()).into());
    }
    Ok(())
}

fn verify_sidecar_if_present(archive: &Path) -> Result {
    let sidecar = archive.with_file_name(format!(
        "{}.sha256",
        archive.file_name().unwrap().to_string_lossy()
    ));
    if sidecar.is_file() {
        let expected = fs::read_to_string(sidecar)?
            .split_whitespace()
            .next()
            .ok_or("empty checksum sidecar")?
            .to_owned();
        if sha256(archive)? != expected {
            return Err(format!("archive checksum mismatch: {}", archive.display()).into());
        }
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String> {
    let mut source = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn run_status(command: &mut Command) -> Result {
    let display = format!("{command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed ({status}): {display}").into());
    }
    Ok(())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn npm_platform(target: &str) -> Result<(&'static str, &'static str)> {
    Ok(match target {
        "linux-x86_64-gnu" => ("linux", "x64"),
        "linux-aarch64-gnu" => ("linux", "arm64"),
        "macos-x86_64" => ("darwin", "x64"),
        "macos-aarch64" => ("darwin", "arm64"),
        "windows-x86_64-msvc" => ("win32", "x64"),
        "windows-aarch64-msvc" => ("win32", "arm64"),
        _ => return Err(format!("unsupported npm target {target}").into()),
    })
}

fn rid(target: &str) -> Result<&'static str> {
    Ok(match target {
        "linux-x86_64-gnu" => "linux-x64",
        "linux-aarch64-gnu" => "linux-arm64",
        "macos-x86_64" => "osx-x64",
        "macos-aarch64" => "osx-arm64",
        "windows-x86_64-msvc" => "win-x64",
        "windows-aarch64-msvc" => "win-arm64",
        _ => return Err(format!("unsupported NuGet target {target}").into()),
    })
}

fn gem_platform(target: &str) -> Result<&'static str> {
    match target {
        "linux-x86_64-gnu" => Ok("x86_64-linux"),
        "linux-aarch64-gnu" => Ok("aarch64-linux"),
        "macos-x86_64" => Ok("x86_64-darwin"),
        "macos-aarch64" => Ok("arm64-darwin"),
        "windows-x86_64-msvc" => Ok("x64-mingw-ucrt"),
        "windows-aarch64-msvc" => Ok("arm64-mingw-ucrt"),
        _ => Err(format!("unsupported Ruby target {target}").into()),
    }
}

fn go_platform(target: &str) -> Result<(&'static str, &'static str, &'static str)> {
    Ok(match target {
        "linux-x86_64-gnu" => ("linux", "amd64", "-ldbus-1 -lpthread -ldl -lm"),
        "linux-aarch64-gnu" => ("linux", "arm64", "-ldbus-1 -lpthread -ldl -lm"),
        "macos-x86_64" => (
            "darwin",
            "amd64",
            "-framework Security -framework CoreFoundation -framework Foundation",
        ),
        "macos-aarch64" => (
            "darwin",
            "arm64",
            "-framework Security -framework CoreFoundation -framework Foundation",
        ),
        "windows-x86_64-msvc" => (
            "windows",
            "amd64",
            "-lbcrypt -lcrypt32 -lncrypt -luserenv -lws2_32 -ladvapi32",
        ),
        "windows-aarch64-msvc" => (
            "windows",
            "arm64",
            "-lbcrypt -lcrypt32 -lncrypt -luserenv -lws2_32 -ladvapi32",
        ),
        _ => return Err(format!("unsupported Go target {target}").into()),
    })
}

pub fn publish_cli(repository: &Path, publish: bool) -> Result {
    let rust = repository.canonicalize()?.join("rust");
    let packages = [
        "revault_lockbox_api",
        "revault_migration_format",
        "revault_migrate_vault_v1",
        "revault_vault_api",
        "revault_migrate_archive_v1",
        "revault_migration",
        "revault_publish_protocol",
        "revault_cli",
    ];
    for package in packages {
        let manifest = find_package_manifest(&rust, package)?;
        let version = manifest_value(&manifest, "version")?;
        let release = format!("{package}@{version}");
        if crate_is_published(&rust, &release)? {
            println!("skipping {release}: already published on crates.io");
            continue;
        }
        let mut command = Command::new("cargo");
        command
            .current_dir(&rust)
            .args(["publish", "-p", package, "--allow-dirty"]);
        if !publish {
            command.arg("--dry-run");
        }
        run_process(&mut command)?;
        if publish {
            let mut visible = false;
            for _ in 0..30 {
                if crate_is_published(&rust, &release)? {
                    visible = true;
                    break;
                }
                thread::sleep(Duration::from_secs(2));
            }
            if !visible {
                return Err(format!(
                    "{release} was uploaded but did not become visible on crates.io"
                )
                .into());
            }
        }
    }
    Ok(())
}

fn crate_is_published(rust: &Path, release: &str) -> Result<bool> {
    Ok(Command::new("cargo")
        .current_dir(rust)
        .args(["info", release, "--registry", "crates-io"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success())
}

fn find_package_manifest(rust: &Path, package: &str) -> Result<PathBuf> {
    let direct = rust.join(package).join("Cargo.toml");
    if direct.is_file() {
        Ok(direct)
    } else {
        Err(format!("package manifest not found for {package}").into())
    }
}

fn manifest_value(path: &Path, key: &str) -> Result<String> {
    let source = fs::read_to_string(path)?;
    let mut in_package = false;
    for line in source.lines() {
        if line.trim() == "[package]" {
            in_package = true;
            continue;
        }
        if in_package && line.starts_with('[') {
            break;
        }
        if in_package {
            if let Some(value) = line.trim().strip_prefix(&format!("{key} = \"")) {
                return Ok(value.trim_end_matches('"').to_owned());
            }
        }
    }
    Err(format!("{} has no package {key}", path.display()).into())
}

fn run_process(command: &mut Command) -> Result {
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
    fn rejects_parent_archive_paths() {
        assert!(require_safe_relative(Path::new("../escape")).is_err());
        assert!(require_safe_relative(Path::new("safe/file")).is_ok());
    }

    #[test]
    fn maps_every_native_target() {
        for target in [
            "linux-x86_64-gnu",
            "linux-aarch64-gnu",
            "macos-x86_64",
            "macos-aarch64",
            "windows-x86_64-msvc",
            "windows-aarch64-msvc",
        ] {
            assert!(npm_platform(target).is_ok());
            assert!(rid(target).is_ok());
            assert!(go_platform(target).is_ok());
        }
    }

    #[test]
    fn converts_verbatim_paths_for_msvc() {
        assert_eq!(
            msvc_path(Path::new(r"\\?\C:\work\revault_api.dll")),
            r"C:\work\revault_api.dll"
        );
        assert_eq!(
            msvc_path(Path::new(r"\\?\UNC\server\share\revault_api.dll")),
            r"\\server\share\revault_api.dll"
        );
    }

    #[test]
    fn copies_sources_below_a_target_directory() {
        let temporary = TempDir::new().unwrap();
        let source = temporary.path().join("target/source");
        let destination = temporary.path().join("installed");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("artifact"), b"accepted").unwrap();
        copy_tree(&source, &destination).unwrap();
        assert_eq!(fs::read(destination.join("artifact")).unwrap(), b"accepted");
    }

    #[test]
    fn replaces_detected_manifest_versions_without_a_baseline_constant() {
        for source in [
            "{\"version\": \"1.2.3\", \"dependency\": \"1.2.3\"}",
            "version = \"1.2.3\"\n",
            "version = '1.2.3'\n",
            "spec.version = \"1.2.3\"\n",
            "<Version>1.2.3</Version>",
            "version: 1.2.3\n",
        ] {
            let replaced = replace_version_text(source, "9.8.7").unwrap();
            assert!(replaced.contains("9.8.7"));
            assert!(!replaced.contains("1.2.3"));
        }
    }
}
