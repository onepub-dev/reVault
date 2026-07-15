use crate::command::{self, TaskResult};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const PASSPHRASE: &str = "lockbox-bench";

pub fn run(args: &[String]) -> TaskResult {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("Usage: cargo xtask compare-archive-compression [OUTPUT_DIRECTORY]");
        return Ok(());
    }
    if args.len() > 1 {
        return Err("compare-archive-compression accepts at most one output directory".to_owned());
    }
    command::require_commands(&["cargo", "du", "gpg", "tar", "zstd", "openssl"])?;
    command::require_file(Path::new("/usr/bin/time"))?;
    let rust_dir = command::workspace_root()?;
    let repo = command::repo_root()?;
    let out = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| rust_dir.join("target/archive-comparison"));
    let fixtures = out.join("fixtures");
    let results = out.join("results");
    let gnupg = out.join("gnupg");
    fs::create_dir_all(&fixtures).map_err(to_string)?;
    fs::create_dir_all(&results).map_err(to_string)?;
    fs::create_dir_all(&gnupg).map_err(to_string)?;
    set_owner_only(&gnupg)?;

    command::run(command::command("cargo").args([
        "build",
        "--release",
        "--manifest-path",
        "revault_cli/Cargo.toml",
    ]))?;
    let lockbox = rust_dir.join("target/release/lockbox");
    generate_fixtures(&fixtures, &repo)?;

    let jobs = env::var("LOCKBOX_JOBS").unwrap_or_else(|_| "auto".to_owned());
    let summary_path = results.join("summary.tsv");
    let mut summary = File::create(&summary_path).map_err(to_string)?;
    writeln!(
        summary,
        "fixture\ttool\tlogical_bytes\toutput_bytes\tseconds\tmax_rss_kib"
    )
    .map_err(to_string)?;
    let fixture_names = [
        "repeated-small",
        "text-tree",
        "mixed-tree",
        "high-entropy",
        "revault-source",
    ];
    let tools = [
        "lockbox",
        "gpg-default",
        "gpg-zlib9",
        "zstd1-gpg-none",
        "zstd19-gpg-none",
    ];
    for fixture in fixture_names {
        for tool in tools {
            let row = run_tool(&out, &lockbox, &gnupg, fixture, tool, &jobs)?;
            println!("{row}");
            writeln!(summary, "{row}").map_err(to_string)?;
            summary.flush().map_err(to_string)?;
        }
    }
    println!("summary: {}", summary_path.display());
    Ok(())
}

fn generate_fixtures(fixtures: &Path, repo: &Path) -> TaskResult {
    repeated_small(&fixtures.join("repeated-small"))?;
    text_tree(&fixtures.join("text-tree"))?;
    mixed_tree(&fixtures.join("mixed-tree"))?;
    high_entropy(&fixtures.join("high-entropy"))?;
    source_tree(&fixtures.join("revault-source"), repo)
}

fn repeated_small(dir: &Path) -> TaskResult {
    if dir.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dir).map_err(to_string)?;
    let payload = vec![b'x'; 25_600];
    for index in 0..4096 {
        fs::write(dir.join(format!("file-{index:04}.bin")), &payload).map_err(to_string)?;
    }
    Ok(())
}

fn text_tree(dir: &Path) -> TaskResult {
    if dir.is_dir() {
        return Ok(());
    }
    for index in 0..1024 {
        let sub = dir.join(format!("service-{:02}", index % 16));
        fs::create_dir_all(&sub).map_err(to_string)?;
        let mut file =
            File::create(sub.join(format!("event-{index:04}.jsonl"))).map_err(to_string)?;
        for line in 0..160 {
            let level = if line % 17 == 0 { "WARN" } else { "INFO" };
            let ok = if line % 23 == 0 { "false" } else { "true" };
            writeln!(file, "{{\"ts\":\"2026-05-{:02}T{:02}:{:02}:{:02}Z\",\"level\":\"{level}\",\"service\":\"svc-{:02}\",\"request_id\":\"req-{:06}\",\"message\":\"{}\",\"value\":{},\"ok\":{ok}}}",
                1 + ((index + line) % 28), line % 24, (index + line) % 60, (index * 7 + line) % 60,
                index % 16, index * 1000 + line, "cache lookup completed ".repeat(1 + line % 4),
                (index * 31 + line * 17) % 100_000).map_err(to_string)?;
        }
    }
    Ok(())
}

fn mixed_tree(dir: &Path) -> TaskResult {
    if marker_matches(dir, "mixed-v3")? {
        return Ok(());
    }
    reset_dir(dir)?;
    for name in ["text", "bin", "tiny", "media"] {
        fs::create_dir_all(dir.join(name)).map_err(to_string)?;
    }
    for index in 0..512 {
        let mut file =
            File::create(dir.join(format!("text/doc-{index:04}.md"))).map_err(to_string)?;
        writeln!(file, "# Document {index}\n").map_err(to_string)?;
        for line in 0..81 {
            writeln!(file, "This paragraph has repeated project vocabulary, endpoint names, and status fields. item={index} line={line} path=/api/v1/resource/{}", index % 37).map_err(to_string)?;
        }
    }
    for index in 0..400 {
        fs::write(
            dir.join(format!("tiny/key-{index:04}.txt")),
            format!("flag-{}\n", index % 13),
        )
        .map_err(to_string)?;
    }
    for index in 0..128 {
        encrypt_zeros(
            65_536,
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            &format!("{:032x}", (index + 1) * 1_048_576),
            &dir.join(format!("bin/blob-{index:04}.dat")),
            &[],
        )?;
    }
    for index in 0..32 {
        encrypt_zeros(
            262_144,
            "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100",
            &format!("{:032x}", (index + 1000) * 1_048_576),
            &dir.join(format!("media/image-like-{index:04}.bin")),
            format!("PSEUDOIMAGE{index:08}").as_bytes(),
        )?;
    }
    fs::write(dir.join(".fixture-version"), "mixed-v3\n").map_err(to_string)
}

fn high_entropy(dir: &Path) -> TaskResult {
    if marker_matches(dir, "high-entropy-v3")? {
        return Ok(());
    }
    reset_dir(dir)?;
    for index in 0..64 {
        encrypt_zeros(
            1_048_576,
            "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567",
            &format!("{:032x}", (index + 1) * 1_048_576),
            &dir.join(format!("random-{index:04}.bin")),
            &[],
        )?;
    }
    fs::write(dir.join(".fixture-version"), "high-entropy-v3\n").map_err(to_string)
}

fn source_tree(dir: &Path, repo: &Path) -> TaskResult {
    if dir.is_dir() {
        return Ok(());
    }
    let paths = [
        "docs",
        "rust/Cargo.toml",
        "rust/Cargo.lock",
        "rust/revault_cli/src",
        "rust/revault_cli/tests",
        "rust/revault_lockbox_api/src",
        "rust/revault_lockbox_api/tests",
        "rust/revault_lockbox_api/examples",
        "rust/revault_page_api/src",
        "rust/revault_vault_api/src",
        "rust/revault_vault_api/tests",
        "rust/tools",
        "rust/xtask",
    ];
    for relative in paths {
        let source = repo.join(relative);
        let destination = dir.join(relative);
        copy_tree(&source, &destination)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> TaskResult {
    if source.is_dir() {
        fs::create_dir_all(destination).map_err(to_string)?;
        for entry in fs::read_dir(source).map_err(to_string)? {
            let entry = entry.map_err(to_string)?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(to_string)?;
        }
        fs::copy(source, destination).map_err(to_string)?;
    }
    Ok(())
}

fn encrypt_zeros(size: usize, key: &str, iv: &str, path: &Path, prefix: &[u8]) -> TaskResult {
    let mut output = File::create(path).map_err(to_string)?;
    output.write_all(prefix).map_err(to_string)?;
    let command_output = output.try_clone().map_err(to_string)?;
    let mut child = Command::new("openssl")
        .args(["enc", "-aes-256-ctr", "-nosalt", "-K", key, "-iv", iv])
        .stdin(Stdio::piped())
        .stdout(Stdio::from(command_output))
        .spawn()
        .map_err(to_string)?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| "openssl stdin was not piped".to_owned())?;
    input.write_all(&vec![0_u8; size]).map_err(to_string)?;
    drop(input);
    let status = child.wait().map_err(to_string)?;
    if !status.success() {
        return Err(format!("openssl failed with {status}"));
    }
    Ok(())
}

fn run_tool(
    out: &Path,
    lockbox: &Path,
    gnupg: &Path,
    fixture: &str,
    tool: &str,
    jobs: &str,
) -> TaskResult<String> {
    let source = out.join("fixtures").join(fixture);
    let result_dir = out.join("results").join(fixture);
    fs::create_dir_all(&result_dir).map_err(to_string)?;
    let metric = result_dir.join(format!("{tool}.time"));
    let mut artifact = result_dir.join(format!("{tool}.out"));
    remove_if_exists(&artifact)?;
    remove_if_exists(&metric)?;

    match tool {
        "lockbox" => {
            artifact = result_dir.join("lockbox.lbx");
            remove_if_exists(&artifact)?;
            timed(&metric, lockbox, &["--key", KEY, "create", &artifact.to_string_lossy()], &[])?;
            let add_metric = result_dir.join("lockbox.add.time");
            timed(&add_metric, lockbox, &["--key", KEY, "--jobs", jobs, "add", &artifact.to_string_lossy(), &source.to_string_lossy(), "/"], &[])?;
            fs::copy(add_metric, &metric).map_err(to_string)?;
        }
        "gpg-default" => pipeline(&metric, gnupg, &source, &artifact, "gpg --batch --yes --pinentry-mode loopback --passphrase \"$PASSPHRASE\" --symmetric --cipher-algo AES256 -o \"$ARTIFACT\"")?,
        "gpg-zlib9" => pipeline(&metric, gnupg, &source, &artifact, "gpg --batch --yes --pinentry-mode loopback --passphrase \"$PASSPHRASE\" --symmetric --cipher-algo AES256 --compress-algo zlib --compress-level 9 -o \"$ARTIFACT\"")?,
        "zstd1-gpg-none" => pipeline(&metric, gnupg, &source, &artifact, "zstd -q -1 | gpg --batch --yes --pinentry-mode loopback --passphrase \"$PASSPHRASE\" --symmetric --cipher-algo AES256 --compress-algo none -o \"$ARTIFACT\"")?,
        "zstd19-gpg-none" => pipeline(&metric, gnupg, &source, &artifact, "zstd -q -19 | gpg --batch --yes --pinentry-mode loopback --passphrase \"$PASSPHRASE\" --symmetric --cipher-algo AES256 --compress-algo none -o \"$ARTIFACT\"")?,
        _ => return Err(format!("unknown tool: {tool}")),
    }
    let (seconds, rss) = read_metric(&metric)?;
    Ok(format!(
        "{fixture}\t{tool}\t{}\t{}\t{seconds}\t{rss}",
        directory_bytes(&source)?,
        fs::metadata(&artifact).map_err(to_string)?.len()
    ))
}

fn timed(metric: &Path, program: &Path, args: &[&str], environment: &[(&str, &str)]) -> TaskResult {
    let mut timer = Command::new("/usr/bin/time");
    timer
        .args(["-f", "%e\t%M", "-o", &metric.to_string_lossy()])
        .arg(program)
        .args(args);
    for (key, value) in environment {
        timer.env(key, value);
    }
    command::run(&mut timer)
}

fn pipeline(metric: &Path, gnupg: &Path, source: &Path, artifact: &Path, tail: &str) -> TaskResult {
    let script = format!("tar -C \"$SOURCE\" -cf - . | {tail}");
    let mut timer = Command::new("/usr/bin/time");
    timer
        .args([
            "-f",
            "%e\t%M",
            "-o",
            &metric.to_string_lossy(),
            "bash",
            "-c",
            &script,
        ])
        .env("SOURCE", source)
        .env("ARTIFACT", artifact)
        .env("PASSPHRASE", PASSPHRASE)
        .env("GNUPGHOME", gnupg);
    command::run(&mut timer)
}

fn read_metric(path: &Path) -> TaskResult<(String, String)> {
    let text = fs::read_to_string(path).map_err(to_string)?;
    let mut fields = text.split_whitespace();
    Ok((
        fields.next().unwrap_or("-").to_owned(),
        fields.next().unwrap_or("-").to_owned(),
    ))
}

fn directory_bytes(path: &Path) -> TaskResult<u64> {
    let output =
        command::output_lossy(command::command("du").args(["-sb", &path.to_string_lossy()]))?;
    output
        .split_whitespace()
        .next()
        .ok_or_else(|| "du returned no byte count".to_owned())?
        .parse::<u64>()
        .map_err(|error| format!("invalid du byte count: {error}"))
}

fn marker_matches(dir: &Path, expected: &str) -> TaskResult<bool> {
    match fs::read_to_string(dir.join(".fixture-version")) {
        Ok(value) => Ok(value.trim() == expected),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

fn reset_dir(path: &Path) -> TaskResult {
    if path.exists() {
        fs::remove_dir_all(path).map_err(to_string)?;
    }
    fs::create_dir_all(path).map_err(to_string)
}

fn remove_if_exists(path: &Path) -> TaskResult {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> TaskResult {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(to_string)
}

#[cfg(not(unix))]
fn set_owner_only(_path: &Path) -> TaskResult {
    Ok(())
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}
