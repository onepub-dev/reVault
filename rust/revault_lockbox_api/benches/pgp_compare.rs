use revault_lockbox_api::{
    ExtractPolicy, Lockbox, LockboxOpen, LockboxPath, LockboxProtection, OwnerSigningKeyPair,
    SecretString, WorkloadProfile,
};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const DEFAULT_ITERATIONS: usize = 3;
const PASSPHRASE: &[u8] = b"revault pgp comparison pass phrase";
const LARGE_BYTES: usize = 16 * 1024 * 1024;
const SMALL_FILE_COUNT: usize = 2_000;
const SMALL_FILE_BYTES: usize = 1024;

fn main() -> revault_lockbox_api::Result<()> {
    let config = Config::parse()?;
    let root = config.root;
    reset_dir(&root)?;

    let corpus_root = root.join("corpus");
    fs::create_dir_all(&corpus_root).map_err(io_err)?;
    let scenarios = build_corpora(&corpus_root)?;

    let gpg_home = root.join("gnupg");
    fs::create_dir_all(&gpg_home).map_err(io_err)?;
    make_private_dir(&gpg_home)?;

    let mut results = Vec::new();
    for scenario in &scenarios {
        if config
            .scenario
            .as_deref()
            .is_some_and(|wanted| wanted != scenario.name)
        {
            continue;
        }
        results.push(run_scenario(
            &root,
            &gpg_home,
            scenario,
            config.iterations,
            config.lockbox_only,
        )?);
    }

    if results.is_empty() {
        return Err(revault_lockbox_api::Error::InvalidInput(
            "no benchmark scenarios matched".to_string(),
        ));
    }

    let report = render_report(&results, config.iterations, config.lockbox_only);
    if let Some(output) = config.output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        fs::write(&output, report).map_err(io_err)?;
        println!("wrote {}", output.display());
    } else {
        print!("{report}");
    }
    Ok(())
}

#[derive(Debug)]
struct Config {
    root: PathBuf,
    output: Option<PathBuf>,
    iterations: usize,
    scenario: Option<String>,
    lockbox_only: bool,
}

impl Config {
    fn parse() -> revault_lockbox_api::Result<Self> {
        let mut root = PathBuf::from("target/lockbox-gpg-bench");
        let mut output = Some(root.join("results.md"));
        let mut output_set = false;
        let mut iterations = DEFAULT_ITERATIONS;
        let mut scenario = None;
        let mut lockbox_only = false;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => {
                    root = PathBuf::from(required_value(&mut args, "--root")?);
                    if !output_set {
                        output = Some(root.join("results.md"));
                    }
                }
                "--output" => {
                    let value = required_value(&mut args, "--output")?;
                    output_set = true;
                    output = if value == "-" {
                        None
                    } else {
                        Some(PathBuf::from(value))
                    };
                }
                "--iterations" => {
                    let value = required_value(&mut args, "--iterations")?;
                    iterations = value.parse::<usize>().map_err(|err| {
                        revault_lockbox_api::Error::InvalidInput(format!(
                            "invalid --iterations value {value}: {err}"
                        ))
                    })?;
                    if iterations == 0 {
                        return Err(revault_lockbox_api::Error::InvalidInput(
                            "--iterations must be greater than zero".to_string(),
                        ));
                    }
                }
                "--scenario" => {
                    scenario = Some(required_value(&mut args, "--scenario")?);
                }
                "--lockbox-only" => {
                    lockbox_only = true;
                }
                "--bench" => {}
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    return Err(revault_lockbox_api::Error::InvalidInput(format!(
                        "unknown argument: {other}"
                    )));
                }
            }
        }

        Ok(Self {
            root,
            output,
            iterations,
            scenario,
            lockbox_only,
        })
    }
}

fn required_value(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> revault_lockbox_api::Result<String> {
    args.next()
        .ok_or_else(|| revault_lockbox_api::Error::InvalidInput(format!("{name} requires a value")))
}

fn print_help() {
    println!(
        "Usage: cargo bench -p revault_lockbox_api --bench pgp_compare -- [options]\n\
\n\
Options:\n\
  --root <path>        Working directory (default: target/lockbox-gpg-bench)\n\
  --output <path|->    Markdown report path, or '-' for stdout\n\
  --iterations <n>     Repetitions per operation (default: {DEFAULT_ITERATIONS})\n\
  --scenario <name>    large-text, large-randomish, or small-tree\n\
  --lockbox-only       Skip GPG/tar runs; useful under profilers\n"
    );
}

#[derive(Debug)]
struct Scenario {
    name: &'static str,
    input: PathBuf,
    kind: ScenarioKind,
    logical_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
enum ScenarioKind {
    SingleFile,
    Directory,
}

#[derive(Debug)]
struct ScenarioResult {
    name: &'static str,
    logical_bytes: u64,
    lockbox_size: u64,
    pgp_size: Option<u64>,
    lockbox_create: Duration,
    pgp_create: Option<Duration>,
    lockbox_extract: Duration,
    pgp_extract: Option<Duration>,
}

fn build_corpora(root: &Path) -> revault_lockbox_api::Result<Vec<Scenario>> {
    let large_text = root.join("large-text.txt");
    write_repeated_text(&large_text, LARGE_BYTES)?;

    let large_randomish = root.join("large-randomish.bin");
    write_randomish(&large_randomish, LARGE_BYTES, 0xC0DE_1234_5566_7788)?;

    let small_tree = root.join("small-tree");
    fs::create_dir_all(&small_tree).map_err(io_err)?;
    for index in 0..SMALL_FILE_COUNT {
        let dir = small_tree.join(format!("group-{:02}", index % 32));
        fs::create_dir_all(&dir).map_err(io_err)?;
        let path = dir.join(format!("file-{index:06}.txt"));
        write_small_file(&path, index, SMALL_FILE_BYTES)?;
    }

    Ok(vec![
        Scenario {
            name: "large-text",
            input: large_text,
            kind: ScenarioKind::SingleFile,
            logical_bytes: LARGE_BYTES as u64,
        },
        Scenario {
            name: "large-randomish",
            input: large_randomish,
            kind: ScenarioKind::SingleFile,
            logical_bytes: LARGE_BYTES as u64,
        },
        Scenario {
            name: "small-tree",
            input: small_tree,
            kind: ScenarioKind::Directory,
            logical_bytes: (SMALL_FILE_COUNT * SMALL_FILE_BYTES) as u64,
        },
    ])
}

fn run_scenario(
    root: &Path,
    gpg_home: &Path,
    scenario: &Scenario,
    iterations: usize,
    lockbox_only: bool,
) -> revault_lockbox_api::Result<ScenarioResult> {
    let scenario_root = root.join("runs").join(scenario.name);
    reset_dir(&scenario_root)?;

    let mut lockbox_create = Vec::with_capacity(iterations);
    let mut lockbox_extract = Vec::with_capacity(iterations);
    let mut lockbox_size = 0;
    let mut lockbox_archive = PathBuf::new();

    for iteration in 0..iterations {
        let iter_root = scenario_root.join(format!("lockbox-{iteration}"));
        reset_dir(&iter_root)?;
        let archive = iter_root.join("archive.lbox");
        let extract_dir = iter_root.join("extract");

        let started = Instant::now();
        create_lockbox_archive(scenario, &archive)?;
        lockbox_create.push(started.elapsed());
        lockbox_size = fs::metadata(&archive).map_err(io_err)?.len();
        lockbox_archive = archive.clone();

        let pass_phrase = pass_phrase()?;
        let started = Instant::now();
        let mut lockbox = Lockbox::open(&archive, LockboxOpen::Password(&pass_phrase))?;
        lockbox.set_workload_profile(WorkloadProfile::ExtractMany);
        lockbox.extract_to_directory(&extract_dir, &bench_extract_policy())?;
        lockbox_extract.push(started.elapsed());
    }

    let (pgp_size, pgp_create, pgp_extract) = if lockbox_only {
        (None, None, None)
    } else {
        let mut pgp_create = Vec::with_capacity(iterations);
        let mut pgp_extract = Vec::with_capacity(iterations);
        let mut pgp_size = 0;
        for iteration in 0..iterations {
            let iter_root = scenario_root.join(format!("pgp-{iteration}"));
            reset_dir(&iter_root)?;
            let encrypted = iter_root.join("archive.gpg");
            let started = Instant::now();
            create_pgp_archive(gpg_home, scenario, &iter_root, &encrypted)?;
            pgp_create.push(started.elapsed());
            pgp_size = fs::metadata(&encrypted).map_err(io_err)?.len();

            let extract_dir = iter_root.join("extract");
            fs::create_dir_all(&extract_dir).map_err(io_err)?;
            let started = Instant::now();
            extract_pgp_archive(gpg_home, scenario, &iter_root, &encrypted, &extract_dir)?;
            pgp_extract.push(started.elapsed());
        }
        (
            Some(pgp_size),
            Some(median_duration(&mut pgp_create)),
            Some(median_duration(&mut pgp_extract)),
        )
    };

    println!(
        "{}: lockbox {} in {}, out {}, extract {}",
        scenario.name,
        format_bytes(lockbox_size),
        format_duration(median_duration(&mut lockbox_create.clone())),
        lockbox_archive.display(),
        format_duration(median_duration(&mut lockbox_extract.clone()))
    );

    Ok(ScenarioResult {
        name: scenario.name,
        logical_bytes: scenario.logical_bytes,
        lockbox_size,
        pgp_size,
        lockbox_create: median_duration(&mut lockbox_create),
        pgp_create,
        lockbox_extract: median_duration(&mut lockbox_extract),
        pgp_extract,
    })
}

fn create_lockbox_archive(scenario: &Scenario, archive: &Path) -> revault_lockbox_api::Result<()> {
    let pass_phrase = pass_phrase()?;
    let signing_key = OwnerSigningKeyPair::generate()?;
    let mut lockbox = Lockbox::create_file(
        archive,
        LockboxProtection::Password(&pass_phrase),
        &signing_key,
    )?;
    lockbox.set_workload_profile(WorkloadProfile::BulkImport);
    match scenario.kind {
        ScenarioKind::SingleFile => {
            lockbox.add_file_from_path(&scenario.input, &LockboxPath::new("/payload")?, false)?;
        }
        ScenarioKind::Directory => {
            for file in list_regular_files(&scenario.input)? {
                let relative = file.strip_prefix(&scenario.input).map_err(|err| {
                    revault_lockbox_api::Error::InvalidPath(format!(
                        "cannot relativize {}: {err}",
                        file.display()
                    ))
                })?;
                let lockbox_path = LockboxPath::new(format!("/{}", slash_path(relative)?))?;
                lockbox.add_file_from_path(&file, &lockbox_path, false)?;
            }
        }
    }
    lockbox.commit()
}

fn create_pgp_archive(
    gpg_home: &Path,
    scenario: &Scenario,
    iter_root: &Path,
    encrypted: &Path,
) -> revault_lockbox_api::Result<()> {
    match scenario.kind {
        ScenarioKind::SingleFile => gpg_symmetric(gpg_home, &scenario.input, encrypted),
        ScenarioKind::Directory => {
            let tar_path = iter_root.join("payload.tar");
            run_command(
                "tar",
                [
                    OsString::from("-cf"),
                    tar_path.as_os_str().to_os_string(),
                    OsString::from("-C"),
                    scenario.input.as_os_str().to_os_string(),
                    OsString::from("."),
                ],
            )?;
            gpg_symmetric(gpg_home, &tar_path, encrypted)
        }
    }
}

fn extract_pgp_archive(
    gpg_home: &Path,
    scenario: &Scenario,
    iter_root: &Path,
    encrypted: &Path,
    extract_dir: &Path,
) -> revault_lockbox_api::Result<()> {
    match scenario.kind {
        ScenarioKind::SingleFile => {
            let out = extract_dir.join("payload");
            gpg_decrypt(gpg_home, encrypted, &out)
        }
        ScenarioKind::Directory => {
            let tar_path = iter_root.join("decrypted.tar");
            gpg_decrypt(gpg_home, encrypted, &tar_path)?;
            run_command(
                "tar",
                [
                    OsString::from("-xf"),
                    tar_path.as_os_str().to_os_string(),
                    OsString::from("-C"),
                    extract_dir.as_os_str().to_os_string(),
                ],
            )
        }
    }
}

fn gpg_symmetric(gpg_home: &Path, input: &Path, output: &Path) -> revault_lockbox_api::Result<()> {
    let mut args = gpg_args(gpg_home);
    args.extend([
        OsString::from("--batch"),
        OsString::from("--yes"),
        OsString::from("--pinentry-mode"),
        OsString::from("loopback"),
        OsString::from("--no-symkey-cache"),
        OsString::from("--passphrase"),
        OsString::from("revault pgp comparison pass phrase"),
        OsString::from("--cipher-algo"),
        OsString::from("AES256"),
        OsString::from("--compress-algo"),
        OsString::from("ZLIB"),
        OsString::from("--compress-level"),
        OsString::from("6"),
        OsString::from("--symmetric"),
        OsString::from("--output"),
        output.as_os_str().to_os_string(),
        input.as_os_str().to_os_string(),
    ]);
    run_command("gpg", args)
}

fn gpg_decrypt(gpg_home: &Path, input: &Path, output: &Path) -> revault_lockbox_api::Result<()> {
    let mut args = gpg_args(gpg_home);
    args.extend([
        OsString::from("--batch"),
        OsString::from("--yes"),
        OsString::from("--pinentry-mode"),
        OsString::from("loopback"),
        OsString::from("--no-symkey-cache"),
        OsString::from("--passphrase"),
        OsString::from("revault pgp comparison pass phrase"),
        OsString::from("--decrypt"),
        OsString::from("--output"),
        output.as_os_str().to_os_string(),
        input.as_os_str().to_os_string(),
    ]);
    run_command("gpg", args)
}

fn gpg_args(gpg_home: &Path) -> Vec<OsString> {
    vec![
        OsString::from("--homedir"),
        gpg_home.as_os_str().to_os_string(),
    ]
}

fn run_command(
    program: &str,
    args: impl IntoIterator<Item = OsString>,
) -> revault_lockbox_api::Result<()> {
    let args = args.into_iter().collect::<Vec<_>>();
    let output = Command::new(program)
        .args(&args)
        .output()
        .map_err(|err| revault_lockbox_api::Error::Io(format!("run {program}: {err}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(revault_lockbox_api::Error::Io(format!(
        "{program} failed with status {}: {}",
        output.status,
        stderr.trim()
    )))
}

fn list_regular_files(root: &Path) -> revault_lockbox_api::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_regular_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_regular_files(root: &Path, files: &mut Vec<PathBuf>) -> revault_lockbox_api::Result<()> {
    let mut entries = fs::read_dir(root)
        .map_err(|err| revault_lockbox_api::Error::Io(format!("read {}: {err}", root.display())))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(io_err)?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let metadata = entry.metadata().map_err(io_err)?;
        if metadata.is_dir() {
            collect_regular_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn slash_path(path: &Path) -> revault_lockbox_api::Result<String> {
    let mut out = String::new();
    for component in path.components() {
        if !out.is_empty() {
            out.push('/');
        }
        let value = component.as_os_str().to_str().ok_or_else(|| {
            revault_lockbox_api::Error::InvalidPath(format!(
                "non UTF-8 corpus path: {}",
                path.display()
            ))
        })?;
        out.push_str(value);
    }
    Ok(out)
}

fn write_repeated_text(path: &Path, bytes: usize) -> revault_lockbox_api::Result<()> {
    let mut file = fs::File::create(path).map_err(io_err)?;
    let line = b"The quick brown fox stores a deterministic reVault benchmark line.\n";
    let mut written = 0usize;
    while written < bytes {
        let count = line.len().min(bytes - written);
        file.write_all(&line[..count]).map_err(io_err)?;
        written += count;
    }
    Ok(())
}

fn write_randomish(path: &Path, bytes: usize, seed: u64) -> revault_lockbox_api::Result<()> {
    let mut file = fs::File::create(path).map_err(io_err)?;
    let mut state = seed;
    let mut buffer = [0u8; 8192];
    let mut written = 0usize;
    while written < bytes {
        for chunk in buffer.chunks_mut(8) {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let bytes = state.to_le_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&bytes[..len]);
        }
        let count = buffer.len().min(bytes - written);
        file.write_all(&buffer[..count]).map_err(io_err)?;
        written += count;
    }
    Ok(())
}

fn write_small_file(path: &Path, index: usize, bytes: usize) -> revault_lockbox_api::Result<()> {
    let mut file = fs::File::create(path).map_err(io_err)?;
    let header = format!("file={index:06}\n");
    file.write_all(header.as_bytes()).map_err(io_err)?;
    let remaining = bytes.saturating_sub(header.len());
    let mut state = index as u64 ^ 0xA55A_5AA5_C3C3_3C3C;
    let mut buffer = vec![0u8; remaining];
    for byte in &mut buffer {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *byte = (state >> 32) as u8;
    }
    file.write_all(&buffer).map_err(io_err)?;
    Ok(())
}

fn pass_phrase() -> revault_lockbox_api::Result<SecretString> {
    Ok(SecretString::try_from_slice(PASSPHRASE)?)
}

fn bench_extract_policy() -> ExtractPolicy {
    ExtractPolicy {
        max_file_bytes: u64::MAX,
        max_total_bytes: u64::MAX,
        max_files: usize::MAX,
        restore_symlinks: false,
        restore_permissions: false,
        overwrite: false,
    }
}

fn reset_dir(path: &Path) -> revault_lockbox_api::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(io_err(err)),
    }
    fs::create_dir_all(path).map_err(io_err)
}

#[cfg(unix)]
fn make_private_dir(path: &Path) -> revault_lockbox_api::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(io_err)
}

#[cfg(not(unix))]
fn make_private_dir(_path: &Path) -> revault_lockbox_api::Result<()> {
    Ok(())
}

fn median_duration(values: &mut [Duration]) -> Duration {
    values.sort_by(|left, right| {
        left.partial_cmp(right)
            .unwrap_or_else(|| left.as_nanos().cmp(&right.as_nanos()))
    });
    values[values.len() / 2]
}

fn render_report(results: &[ScenarioResult], iterations: usize, lockbox_only: bool) -> String {
    let mut out = String::new();
    out.push_str("# Lockbox vs PGP Benchmark\n\n");
    out.push_str(&format!("Iterations per operation: `{iterations}`\n\n"));
    out.push_str("Lockbox uses the core archive API directly with `BulkImport` for create ");
    out.push_str("and `ExtractMany` for extract. PGP uses `gpg --symmetric --no-symkey-cache ");
    out.push_str("--cipher-algo AES256 --compress-algo ZLIB --compress-level 6`; directory ");
    out.push_str("scenarios include the required `tar` archive and extract steps.\n\n");
    if lockbox_only {
        out.push_str("This run used `--lockbox-only`, so PGP columns are omitted.\n\n");
    }
    out.push_str("| Scenario | Logical bytes | Lockbox size | PGP size | Lockbox create | PGP create | Lockbox extract | PGP extract | Size ratio L/PGP | Create ratio L/PGP | Extract ratio L/PGP |\n");
    out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for result in results {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            result.name,
            format_bytes(result.logical_bytes),
            format_bytes(result.lockbox_size),
            result
                .pgp_size
                .map(format_bytes)
                .unwrap_or_else(|| "-".to_string()),
            format_duration(result.lockbox_create),
            result
                .pgp_create
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string()),
            format_duration(result.lockbox_extract),
            result
                .pgp_extract
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string()),
            result
                .pgp_size
                .map(|size| format_ratio(result.lockbox_size as f64, size as f64))
                .unwrap_or_else(|| "-".to_string()),
            result
                .pgp_create
                .map(|duration| {
                    format_ratio(result.lockbox_create.as_secs_f64(), duration.as_secs_f64())
                })
                .unwrap_or_else(|| "-".to_string()),
            result
                .pgp_extract
                .map(|duration| {
                    format_ratio(result.lockbox_extract.as_secs_f64(), duration.as_secs_f64())
                })
                .unwrap_or_else(|| "-".to_string()),
        ));
    }
    out
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.3}s", duration.as_secs_f64())
    } else {
        format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
    }
}

fn format_bytes(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    const KIB: f64 = 1024.0;
    match bytes.cmp(&(1024 * 1024)) {
        Ordering::Greater | Ordering::Equal => format!("{:.2} MiB", bytes as f64 / MIB),
        Ordering::Less if bytes >= 1024 => format!("{:.2} KiB", bytes as f64 / KIB),
        Ordering::Less => format!("{bytes} B"),
    }
}

fn format_ratio(left: f64, right: f64) -> String {
    if right == 0.0 {
        "-".to_string()
    } else {
        format!("{:.2}x", left / right)
    }
}

fn io_err(err: std::io::Error) -> revault_lockbox_api::Error {
    revault_lockbox_api::Error::Io(err.to_string())
}
