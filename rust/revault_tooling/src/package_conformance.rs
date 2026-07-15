use crate::e2e::{self, Container, RustSourceConformance};
use crate::Result;
use clap::Args;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Args)]
pub struct PackageConformance {
    #[arg(long)]
    language: String,
    #[arg(long)]
    target: String,
    #[arg(long)]
    archive: Option<PathBuf>,
    #[arg(long)]
    packages: PathBuf,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
    #[arg(long)]
    work: PathBuf,
}

struct Prepared {
    program: PathBuf,
    args: Vec<String>,
    native_root: PathBuf,
    native_file: String,
    native_kind: &'static str,
    environment: Vec<(String, String)>,
}

pub fn run(args: PackageConformance) -> Result {
    let repository = args.repository.canonicalize()?;
    let packages = args.packages.canonicalize()?;
    fs::create_dir_all(&args.work)?;
    let work = args.work.canonicalize()?;
    configure_isolated_caches(&work)?;
    if args.language == "rust" {
        let source_archive = prepare_rust_package(&repository, &packages, &work)?;
        return e2e::run(e2e::E2eCommand::RustSourceConformance(
            RustSourceConformance {
                repository,
                target: args.target,
                source_archive: Some(source_archive),
            },
        ));
    }
    let archive_input = args
        .archive
        .ok_or("--archive is required for foreign-language package conformance")?;
    let archive = resolve_archive(&archive_input, &args.target)?.canonicalize()?;
    let inspected = crate::release::install_archive(&archive, &work.join("archive-inspection"))?;
    if inspected.target != args.target {
        return Err(format!(
            "native archive target is {}, expected {}",
            inspected.target, args.target
        )
        .into());
    }
    fs::remove_dir_all(&inspected.prefix)?;
    let prepared = prepare(
        &args.language,
        &args.target,
        &repository,
        &packages,
        &work,
        &archive,
    )?;
    std::env::set_var("REVAULT_E2E_PROGRAM", &prepared.program);
    std::env::set_var(
        "REVAULT_E2E_ARGS_JSON",
        serde_json::to_string(&prepared.args)?,
    );
    std::env::set_var("REVAULT_E2E_NATIVE_ROOT", &prepared.native_root);
    std::env::set_var("REVAULT_E2E_NATIVE_FILE", &prepared.native_file);
    std::env::set_var("REVAULT_E2E_NATIVE_KIND", prepared.native_kind);
    std::env::set_var("REVAULT_E2E_TARGET", &args.target);
    std::env::set_var("REVAULT_E2E_ARTIFACT_DIR", work.join("artifacts"));
    for (name, value) in prepared.environment {
        std::env::set_var(name, value);
    }
    std::env::set_current_dir(&repository)?;
    e2e::container(Container {
        language: args.language,
    })
}

fn prepare_rust_package(repository: &Path, packages: &Path, work: &Path) -> Result<PathBuf> {
    let publication = work.join("rust-publication");
    let package = publication.join("cargo/revault-api");
    copy_tree(&packages.join("cargo/revault-api"), &package)?;
    for dependency in [
        "revault_lockbox_api",
        "revault_page_api",
        "revault_vault_api",
    ] {
        copy_tree(
            &repository.join("rust").join(dependency),
            &publication.join("rust").join(dependency),
        )?;
    }
    let target = work.join("rust-package-target");
    run_status(
        Command::new("cargo")
            .args([
                "package",
                "--locked",
                "--allow-dirty",
                "--no-verify",
                "--target-dir",
            ])
            .arg(&target)
            .current_dir(&package),
    )?;
    let archive = target.join("package/revault-api-0.1.0.crate");
    if !archive.is_file() {
        return Err("cargo package did not produce revault-api-0.1.0.crate".into());
    }
    let unpacked = work.join("rust-unpacked");
    fs::create_dir_all(&unpacked)?;
    let decoder = flate2::read::GzDecoder::new(fs::File::open(&archive)?);
    let mut tar = tar::Archive::new(decoder);
    for item in tar.entries()? {
        let mut entry = item?;
        let path = entry.path()?.into_owned();
        if path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err("unsafe path in Rust source package".into());
        }
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_dir() {
            return Err("links and special files are forbidden in Rust source packages".into());
        }
        if !entry.unpack_in(&unpacked)? {
            return Err("Rust source package entry escaped its install root".into());
        }
    }
    let consumer = work.join("rust-consumer");
    fs::create_dir_all(consumer.join("src"))?;
    fs::write(
        consumer.join("Cargo.toml"),
        format!(
            "[package]\nname = \"revault-api-consumer\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\nrevault-api = {{ path = {:?} }}\n\n[patch.crates-io]\nrevault_lockbox_api = {{ path = {:?} }}\nrevault_vault_api = {{ path = {:?} }}\n",
            unpacked.join("revault-api-0.1.0"),
            repository.join("rust/revault_lockbox_api"),
            repository.join("rust/revault_vault_api"),
        ),
    )?;
    fs::write(
        consumer.join("src/main.rs"),
        "use revault_api::{ContactKeyPair, VaultDirectory};\nfn main() { let _ = std::mem::size_of::<Option<ContactKeyPair>>(); let _ = std::mem::size_of::<Option<VaultDirectory>>(); }\n",
    )?;
    run_status(
        Command::new("cargo")
            .args(["check", "--manifest-path"])
            .arg(consumer.join("Cargo.toml")),
    )?;
    Ok(archive)
}

fn configure_isolated_caches(work: &Path) -> Result {
    for (name, directory) in [
        ("NPM_CONFIG_CACHE", "cache/npm"),
        ("PIP_CACHE_DIR", "cache/pip"),
        ("GRADLE_USER_HOME", "cache/gradle"),
        ("PUB_CACHE", "cache/dart"),
        ("COMPOSER_HOME", "cache/composer"),
        ("GOPATH", "cache/go"),
        ("GOMODCACHE", "cache/go/pkg/mod"),
        ("GEM_SPEC_CACHE", "cache/gem-specs"),
    ] {
        let path = work.join(directory);
        fs::create_dir_all(&path)?;
        std::env::set_var(name, path);
    }
    Ok(())
}

fn resolve_archive(input: &Path, target: &str) -> Result<PathBuf> {
    if input.is_file() {
        return Ok(input.to_path_buf());
    }
    let suffix = if target.starts_with("windows-") {
        format!("-{target}.zip")
    } else {
        format!("-{target}.tar.gz")
    };
    let matches: Vec<_> = WalkDir::new(input)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.ends_with(&suffix))
        })
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "expected one native archive ending {suffix} under {}, found {}",
            input.display(),
            matches.len()
        )
        .into());
    }
    Ok(matches[0].clone())
}

fn prepare(
    language: &str,
    target: &str,
    repository: &Path,
    packages: &Path,
    work: &Path,
    archive: &Path,
) -> Result<Prepared> {
    match language {
        "c" => prepare_c(false, repository, work, archive),
        "cpp" => prepare_c(true, repository, work, archive),
        "csharp" => prepare_csharp(target, repository, packages, work),
        "dart" => prepare_dart(target, repository, packages, work),
        "go" => prepare_go(target, repository, packages, work),
        "java" => prepare_gradle(false, target, repository, packages, work),
        "kotlin" => prepare_gradle(true, target, repository, packages, work),
        "javascript" | "typescript" | "wasm" => {
            prepare_npm(language, target, repository, packages, work)
        }
        "lua" => prepare_lua(target, repository, packages, work),
        "php" => prepare_php(target, repository, packages, work),
        "python" => prepare_python(target, repository, packages, work),
        "ruby" => prepare_ruby(target, repository, packages, work),
        "swift" => prepare_swift(target, packages, work, archive),
        _ => Err(format!("unsupported package conformance language {language}").into()),
    }
}

fn prepare_c(cpp: bool, repository: &Path, work: &Path, archive: &Path) -> Result<Prepared> {
    let install = crate::release::install_archive(archive, &work.join("installed-sdk"))?;
    let prefix = install.prefix;
    if cpp {
        run_status(
            Command::new("cmake")
                .args(["-S"])
                .arg(repository.join("bindings/cpp"))
                .args(["-B"])
                .arg(work.join("cpp-api"))
                .arg(format!("-DCMAKE_PREFIX_PATH={}", prefix.display()))
                .arg("-DCMAKE_BUILD_TYPE=Release"),
        )?;
        run_status(
            Command::new("cmake")
                .arg("--build")
                .arg(work.join("cpp-api"))
                .args(["--config", "Release"]),
        )?;
        run_status(
            Command::new("cmake")
                .arg("--install")
                .arg(work.join("cpp-api"))
                .args(["--config", "Release", "--prefix"])
                .arg(&prefix),
        )?;
    }
    let source = if cpp {
        "bindings/e2e/cpp"
    } else {
        "bindings/e2e/c"
    };
    let build = work.join(if cpp {
        "cpp-conformance"
    } else {
        "c-conformance"
    });
    run_status(
        Command::new("cmake")
            .args(["-S"])
            .arg(repository.join(source))
            .args(["-B"])
            .arg(&build)
            .arg(format!("-DCMAKE_PREFIX_PATH={}", prefix.display()))
            .arg("-DCMAKE_BUILD_TYPE=Release"),
    )?;
    run_status(
        Command::new("cmake")
            .arg("--build")
            .arg(&build)
            .args(["--config", "Release"]),
    )?;
    let executable = executable(
        &build,
        if cpp {
            "revault_cpp_conformance"
        } else {
            "revault_c_conformance"
        },
    );
    Ok(dynamic_prepared(
        executable,
        vec![],
        prefix.join("lib"),
        &install.library,
    ))
}

fn prepare_npm(
    language: &str,
    target: &str,
    repository: &Path,
    packages: &Path,
    work: &Path,
) -> Result<Prepared> {
    let tarballs = work.join("npm-tarballs");
    let consumer = work.join("npm-consumer");
    fs::create_dir_all(&tarballs)?;
    fs::create_dir_all(&consumer)?;
    let mut package_dirs = vec![
        packages.join("npm/revault-api"),
        packages.join(format!("npm/revault-api-native-{target}")),
    ];
    if language == "wasm" {
        package_dirs.push(packages.join("npm/revault-api-wasm"));
    }
    for package in &package_dirs {
        run_status(
            Command::new("npm")
                .arg("pack")
                .arg(package)
                .arg("--pack-destination")
                .arg(&tarballs),
        )?;
    }
    run_status(
        Command::new("npm")
            .arg("init")
            .arg("-y")
            .current_dir(&consumer),
    )?;
    let mut install = Command::new("npm");
    install
        .args(["install", "--omit=optional"])
        .current_dir(&consumer);
    for tarball in files_with_extension(&tarballs, "tgz")? {
        install.arg(tarball);
    }
    if language == "typescript" {
        install.args(["typescript@5.8.3", "tsx@4.20.0", "@types/node@22"]);
    }
    run_status(&mut install)?;
    let module = if language == "wasm" {
        "@onepub/revault-api-wasm"
    } else {
        "@onepub/revault-api"
    };
    let mut environment = vec![(
        "REVAULT_E2E_MODULE".into(),
        consumer
            .join("node_modules")
            .join(module)
            .join("index.js")
            .display()
            .to_string(),
    )];
    let (program, program_args) = if language == "typescript" {
        environment.push((
            "NODE_PATH".into(),
            consumer.join("node_modules").display().to_string(),
        ));
        (
            consumer.join(if cfg!(windows) {
                "node_modules/.bin/tsx.cmd"
            } else {
                "node_modules/.bin/tsx"
            }),
            vec![repository
                .join("bindings/e2e/typescript/conformance.ts")
                .display()
                .to_string()],
        )
    } else {
        (
            PathBuf::from("node"),
            vec![repository
                .join("bindings/e2e/javascript/conformance.js")
                .display()
                .to_string()],
        )
    };
    let (native_root, native_file, kind) = if language == "wasm" {
        (
            consumer.join("node_modules/@onepub/revault-api-wasm/generated"),
            "revault_wasm_bindings_bg.wasm".into(),
            "wasm",
        )
    } else {
        (
            consumer
                .join("node_modules")
                .join(format!("@onepub/revault-api-native-{target}/lib")),
            dynamic_library(target),
            "dynamic",
        )
    };
    Ok(Prepared {
        program,
        args: program_args,
        native_root,
        native_file,
        native_kind: kind,
        environment,
    })
}

fn prepare_python(
    target: &str,
    repository: &Path,
    packages: &Path,
    work: &Path,
) -> Result<Prepared> {
    let site = work.join("python-site");
    run_status(
        Command::new(python())
            .args(["-m", "pip", "install", "--target"])
            .arg(&site)
            .arg(packages.join("python").join(target)),
    )?;
    let native_root = site.join("revault_api/_native").join(target);
    Ok(Prepared {
        program: PathBuf::from(python()),
        args: vec![repository
            .join("bindings/e2e/python/conformance.py")
            .display()
            .to_string()],
        native_root,
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![
            ("PYTHONPATH".into(), site.display().to_string()),
            ("REVAULT_E2E_INSTALLED".into(), "1".into()),
        ],
    })
}

fn prepare_go(target: &str, repository: &Path, packages: &Path, work: &Path) -> Result<Prepared> {
    let package = work.join("go-package");
    copy_tree(&packages.join("go"), &package)?;
    let output = work.join(if cfg!(windows) {
        "revault-go-conformance.exe"
    } else {
        "revault-go-conformance"
    });
    run_status(
        Command::new("go")
            .arg("build")
            .arg("-o")
            .arg(&output)
            .arg(repository.join("bindings/e2e/go/conformance.go"))
            .current_dir(&package),
    )?;
    let native_root = package.join("native").join(target);
    Ok(Prepared {
        program: output,
        args: vec![],
        native_root,
        native_file: static_library(target),
        native_kind: "static",
        environment: vec![],
    })
}

fn prepare_csharp(
    target: &str,
    repository: &Path,
    packages: &Path,
    work: &Path,
) -> Result<Prepared> {
    let feed = work.join("nuget-feed");
    let output = work.join("csharp-conformance");
    fs::create_dir_all(&feed)?;
    run_status(
        Command::new("dotnet")
            .arg("pack")
            .arg(packages.join("nuget/RevaultBindings.csproj"))
            .args(["-c", "Release", "-o"])
            .arg(&feed)
            .arg("--nologo"),
    )?;
    run_status(
        Command::new("dotnet")
            .arg("publish")
            .arg(repository.join("bindings/e2e/csharp/Conformance.csproj"))
            .args(["-c", "Release", "-o"])
            .arg(&output)
            .arg("--source")
            .arg(&feed)
            .args([
                "--source",
                "https://api.nuget.org/v3/index.json",
                "--nologo",
            ]),
    )?;
    Ok(Prepared {
        program: PathBuf::from("dotnet"),
        args: vec![output.join("Conformance.dll").display().to_string()],
        native_root: output
            .join("runtimes")
            .join(dotnet_rid(target))
            .join("native"),
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![],
    })
}

fn prepare_dart(target: &str, repository: &Path, packages: &Path, work: &Path) -> Result<Prepared> {
    let root = work.join("dart-tree");
    copy_tree(&packages.join("dart"), &root.join("bindings/dart"))?;
    copy_tree(
        &repository.join("bindings/e2e/dart"),
        &root.join("bindings/e2e/dart"),
    )?;
    let project = root.join("bindings/e2e/dart");
    run_status(
        Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project),
    )?;
    let install = work.join("dart-install");
    let program = install.join(if cfg!(windows) {
        "conformance.exe"
    } else {
        "conformance"
    });
    fs::create_dir_all(&install)?;
    run_status(
        Command::new("dart")
            .args(["compile", "exe", "conformance.dart", "-o"])
            .arg(&program)
            .current_dir(&project),
    )?;
    let native_root = install.join("native").join(target);
    fs::create_dir_all(&native_root)?;
    fs::copy(
        root.join("bindings/dart/lib/src/native")
            .join(target)
            .join(dynamic_library(target)),
        native_root.join(dynamic_library(target)),
    )?;
    Ok(Prepared {
        program,
        args: vec![],
        native_root,
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![],
    })
}

fn prepare_php(target: &str, repository: &Path, packages: &Path, work: &Path) -> Result<Prepared> {
    let root = work.join("php-tree");
    copy_tree(&packages.join("composer"), &root.join("bindings/php"))?;
    copy_tree(
        &repository.join("bindings/e2e/php"),
        &root.join("bindings/e2e/php"),
    )?;
    let project = root.join("bindings/e2e/php");
    run_status(
        Command::new("composer")
            .args([
                "install",
                "--no-interaction",
                "--no-progress",
                "--prefer-dist",
                "--optimize-autoloader",
            ])
            .current_dir(&project),
    )?;
    Ok(Prepared {
        program: PathBuf::from("php"),
        args: vec![
            "-d".into(),
            "ffi.enable=true".into(),
            project.join("conformance.php").display().to_string(),
        ],
        native_root: project
            .join("vendor/onepub/revault-api/native")
            .join(target),
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![],
    })
}

fn prepare_ruby(target: &str, repository: &Path, packages: &Path, work: &Path) -> Result<Prepared> {
    let package = packages.join("ruby").join(target);
    let gem_home = work.join("ruby-gems");
    run_status(
        Command::new("gem")
            .args(["build", "revault_api.gemspec"])
            .current_dir(&package),
    )?;
    let gem = one_file(&package, "gem")?;
    run_status(
        Command::new("gem")
            .arg("install")
            .arg(gem)
            .args(["--no-document", "--install-dir"])
            .arg(&gem_home),
    )?;
    let native_root = find_parent(&gem_home, &dynamic_library(target))?;
    Ok(Prepared {
        program: PathBuf::from("ruby"),
        args: vec![repository
            .join("bindings/e2e/ruby/conformance.rb")
            .display()
            .to_string()],
        native_root,
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![
            ("GEM_HOME".into(), gem_home.display().to_string()),
            ("GEM_PATH".into(), gem_home.display().to_string()),
        ],
    })
}

fn prepare_lua(target: &str, repository: &Path, packages: &Path, work: &Path) -> Result<Prepared> {
    let package = packages.join("lua").join(target);
    let tree = work.join("lua-tree");
    run_status(
        Command::new("luarocks")
            .arg("make")
            .arg("--tree")
            .arg(&tree)
            .arg("revault_api-0.1.0-1.rockspec")
            .current_dir(&package),
    )?;
    let native_root = find_parent(&tree, &dynamic_library(target))?;
    let lua_path = format!(
        "{}{}?.lua;{}{}?{}init.lua;;",
        tree.join("share/lua/5.1").display(),
        std::path::MAIN_SEPARATOR,
        tree.join("share/lua/5.1").display(),
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );
    let lua_cpath = format!(
        "{}{}?.{};;",
        native_root.display(),
        std::path::MAIN_SEPARATOR,
        if cfg!(windows) { "dll" } else { "so" }
    );
    Ok(Prepared {
        program: PathBuf::from(if cfg!(windows) {
            "luajit.exe"
        } else {
            "luajit"
        }),
        args: vec![repository
            .join("bindings/e2e/lua/conformance.lua")
            .display()
            .to_string()],
        native_root,
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![
            ("LUA_PATH".into(), lua_path),
            ("LUA_CPATH".into(), lua_cpath),
        ],
    })
}

fn prepare_gradle(
    kotlin: bool,
    target: &str,
    repository: &Path,
    packages: &Path,
    work: &Path,
) -> Result<Prepared> {
    run_status(
        Command::new(gradle())
            .arg("-p")
            .arg(packages.join("maven/java"))
            .args(["--no-daemon", "publishToMavenLocal"]),
    )?;
    if kotlin {
        run_status(
            Command::new(gradle())
                .arg("-p")
                .arg(packages.join("maven/kotlin"))
                .args(["--no-daemon", "publishToMavenLocal"]),
        )?;
    }
    let project = repository.join(if kotlin {
        "bindings/e2e/kotlin"
    } else {
        "bindings/e2e/java"
    });
    run_status(
        Command::new(gradle())
            .arg("-p")
            .arg(&project)
            .args(["--no-daemon", "installDist"]),
    )?;
    let name = if kotlin {
        "revault-api-kotlin-conformance"
    } else {
        "revault-api-java-conformance"
    };
    let program = project
        .join("build/install")
        .join(name)
        .join("bin")
        .join(if cfg!(windows) {
            format!("{name}.bat")
        } else {
            name.into()
        });
    let native_root = work.join(if kotlin {
        "kotlin-native"
    } else {
        "java-native"
    });
    let java_options = format!(
        "-Djava.io.tmpdir={} -Drevault.keepExtracted=true",
        native_root.display()
    );
    Ok(Prepared {
        program,
        args: vec![],
        native_root,
        native_file: dynamic_library(target),
        native_kind: "dynamic",
        environment: vec![("JAVA_TOOL_OPTIONS".into(), java_options)],
    })
}

fn prepare_swift(target: &str, packages: &Path, work: &Path, archive: &Path) -> Result<Prepared> {
    let install = crate::release::install_archive(archive, &work.join("swift-native"))?;
    let package = work.join("swift-package");
    copy_tree(&packages.join("swift"), &package)?;
    run_status(
        Command::new("swift")
            .args(["build", "-c", "release", "--package-path"])
            .arg(&package)
            .env("LIBRARY_PATH", install.prefix.join("lib")),
    )?;
    let program = package.join(".build/release/revault-swift-conformance");
    Ok(dynamic_prepared(
        program,
        vec![],
        install.prefix.join("lib"),
        &dynamic_library(target),
    ))
}

fn dynamic_prepared(
    program: PathBuf,
    args: Vec<String>,
    native_root: PathBuf,
    native_file: &str,
) -> Prepared {
    let variable = if cfg!(windows) {
        "PATH"
    } else if cfg!(target_os = "macos") {
        "DYLD_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    };
    let separator = if cfg!(windows) { ";" } else { ":" };
    let existing = std::env::var(variable).unwrap_or_default();
    Prepared {
        program,
        args,
        native_root: native_root.clone(),
        native_file: native_file.into(),
        native_kind: "dynamic",
        environment: vec![(
            variable.into(),
            format!("{}{separator}{existing}", native_root.display()),
        )],
    }
}

fn dynamic_library(target: &str) -> String {
    if target.starts_with("windows-") {
        "revault_api.dll"
    } else if target.starts_with("macos-") {
        "librevault_api.dylib"
    } else {
        "librevault_api.so"
    }
    .into()
}

fn static_library(target: &str) -> String {
    if target.starts_with("windows-") {
        "revault_api.lib"
    } else {
        "librevault_api.a"
    }
    .into()
}

fn dotnet_rid(target: &str) -> &'static str {
    match target {
        "linux-x86_64-gnu" => "linux-x64",
        "linux-aarch64-gnu" => "linux-arm64",
        "macos-x86_64" => "osx-x64",
        "macos-aarch64" => "osx-arm64",
        "windows-x86_64-msvc" => "win-x64",
        "windows-aarch64-msvc" => "win-arm64",
        _ => unreachable!(),
    }
}

fn executable(build: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        build.join("Release").join(format!("{name}.exe"))
    } else {
        build.join(name)
    }
}

fn python() -> &'static str {
    if cfg!(windows) {
        "python"
    } else {
        "python3"
    }
}
fn gradle() -> &'static str {
    if cfg!(windows) {
        "gradle.bat"
    } else {
        "gradle"
    }
}

fn run_status(command: &mut Command) -> Result {
    let display = format!("{command:?}");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed ({status}): {display}").into())
    }
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<_> = fs::read_dir(root)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension() == Some(OsStr::new(extension)))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("no .{extension} files under {}", root.display()).into());
    }
    Ok(files)
}

fn one_file(root: &Path, extension: &str) -> Result<PathBuf> {
    let files = files_with_extension(root, extension)?;
    if files.len() != 1 {
        return Err(format!("expected one .{extension} file under {}", root.display()).into());
    }
    Ok(files[0].clone())
}

fn find_parent(root: &Path, name: &str) -> Result<PathBuf> {
    let matches: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == OsStr::new(name))
        .map(|entry| entry.path().parent().unwrap().to_path_buf())
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "expected one {name} under {}, found {}",
            root.display(),
            matches.len()
        )
        .into());
    }
    Ok(matches[0].clone())
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
