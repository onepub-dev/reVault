use super::context::{cli_error, open_existing, open_or_create, require_arg, Access, CliResult};
use super::output::{output_format_from_matches, print_records, OutputFormat};
use super::{
    default_lockbox_for_add, default_lockbox_for_command, looks_like_lockbox_path,
    optional_lockbox_positionals, positional_values,
};
use clap::ArgMatches;
use lockbox_core::{
    Error, ExtractPolicy, ListOptions, Lockbox, LockboxPath, WorkerPolicy, WorkloadProfile,
};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant};

const ADD_PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) fn add_matches(
    matches: &ArgMatches,
    access: &Access,
    worker_policy: WorkerPolicy,
) -> CliResult<()> {
    add(&add_args_from_matches(matches)?, access, worker_policy)
}

pub(crate) fn extract_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    extract(&extract_args_from_matches(matches)?, access)
}

pub(crate) fn cat_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    cat(
        &optional_lockbox_positionals(positional_values(matches, "args"), 1)?,
        access,
    )
}

pub(crate) fn list_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let mut args = optional_lockbox_positionals(positional_values(matches, "args"), 0)?;
    if matches.get_flag("recursive") {
        args.push("--recursive".to_string());
    }
    list_with_format(&args, access, output_format_from_matches(matches)?)
}

pub(crate) fn remove_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let mut args = optional_lockbox_positionals(positional_values(matches, "args"), 1)?;
    if matches.get_flag("force") {
        args.push("--force".to_string());
    }
    remove(&args, access)
}

pub(crate) fn rename_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    rename(
        &optional_lockbox_positionals(positional_values(matches, "args"), 2)?,
        access,
    )
}

fn add_args_from_matches(matches: &ArgMatches) -> CliResult<Vec<String>> {
    let first = matches
        .get_one::<String>("lockbox-or-source")
        .ok_or_else(|| Error::InvalidInput("missing source".to_string()))?
        .clone();
    let second = matches.get_one::<String>("source-or-lockbox-path").cloned();
    let third = matches.get_one::<String>("lockbox-path").cloned();
    let mut args = match (second, third) {
        (None, None) => vec![default_lockbox_for_add()?, first],
        (Some(second), None) => {
            if looks_like_lockbox_path(&first) {
                vec![first, second]
            } else {
                vec![default_lockbox_for_add()?, first, second]
            }
        }
        (Some(second), Some(third)) => vec![first, second, third],
        (None, Some(_)) => unreachable!("clap does not provide third positional without second"),
    };
    if matches.get_flag("recursive") {
        args.push("--recursive".to_string());
    }
    Ok(args)
}

fn extract_args_from_matches(matches: &ArgMatches) -> CliResult<Vec<String>> {
    if let Some(destination) = matches.get_one::<String>("to") {
        let values = positional_values(matches, "args");
        if values.len() > 1 {
            return Err(cli_error("extract --to accepts at most one lockbox path"));
        }
        let mut args = if let Some(lockbox) = values.first() {
            vec![lockbox.clone()]
        } else {
            vec![default_lockbox_for_command()?]
        };
        args.push("--to".to_string());
        args.push(destination.clone());
        for (name, flag) in [
            ("overwrite", "--overwrite"),
            ("restore-symlinks", "--restore-symlinks"),
            ("restore-permissions", "--restore-permissions"),
        ] {
            if matches.get_flag(name) {
                args.push(flag.to_string());
            }
        }
        return Ok(args);
    }
    let mut args = optional_lockbox_positionals(positional_values(matches, "args"), 2)?;
    if matches.get_flag("overwrite") {
        args.push("--overwrite".to_string());
    }
    Ok(args)
}

pub(crate) fn add(args: &[String], access: &Access, worker_policy: WorkerPolicy) -> CliResult<()> {
    let recursive = args.iter().any(|arg| arg == "--recursive" || arg == "-r");
    let args = args
        .iter()
        .filter(|arg| !matches!(arg.as_str(), "--recursive" | "-r"))
        .cloned()
        .collect::<Vec<_>>();
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let source = require_arg(&args, 1, "source")?;
    let source_path = Path::new(source);
    let source_metadata = source_metadata(source_path)?;
    if source_metadata.is_dir() && !recursive {
        return Err(cli_error(
            "source is a directory; pass --recursive to import its files",
        ));
    }
    let path = match args.get(2) {
        Some(path) => destination_lockbox_path(source_path, &source_metadata, path)?,
        None => default_lockbox_path_for_source(source_path)?,
    };
    let creates_lockbox = !Path::new(lockbox_path).exists();
    let mut lb = open_or_create(lockbox_path, access)?;
    lb.set_worker_policy(worker_policy);
    if creates_lockbox || source_path.is_dir() {
        lb.set_workload_profile(WorkloadProfile::BulkImport);
    }
    lb.reset_import_stats();
    let add_start = Instant::now();
    let mut progress = AddProgress::for_source(source_path);
    let add_result = add_source_path(&mut lb, source_path, &path, &mut progress);
    let progress_result = progress.finish();
    add_result?;
    progress_result?;
    let add_wall = add_start.elapsed();
    let commit_start = Instant::now();
    lb.commit()?;
    let commit_wall = commit_start.elapsed();
    if std::env::var_os("LOCKBOX_IMPORT_TIMINGS").is_some() {
        let stats = lb.import_stats();
        eprintln!(
            "lockbox_import_timings\tadd_wall_s={:.6}\tcommit_wall_s={:.6}\thost_stat_s={:.6}\thost_read_s={:.6}\tframe_prepare_s={:.6}\tpage_write_s={:.6}",
            add_wall.as_secs_f64(),
            commit_wall.as_secs_f64(),
            nanos_to_secs(stats.host_stat_nanos),
            nanos_to_secs(stats.host_read_nanos),
            nanos_to_secs(stats.frame_prepare_nanos),
            nanos_to_secs(stats.page_write_nanos),
        );
    }
    Ok(())
}

fn source_metadata(source: &Path) -> CliResult<fs::Metadata> {
    match fs::metadata(source) {
        Ok(metadata) if metadata.is_file() || metadata.is_dir() => Ok(metadata),
        Ok(_) => Err(Error::UnsupportedHostPath(source.display().to_string()).into()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Err(cli_error(format!("file not found: {}", source.display())))
        }
        Err(err) => Err(cli_error(format!(
            "cannot access source {}: {err}",
            source.display()
        ))),
    }
}

pub(crate) fn extract(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let mut lb = open_existing(lockbox_path, access)?;
    if args.get(1).map(String::as_str) == Some("--to") {
        let dest = require_arg(args, 2, "destination")?;
        let policy = extract_policy_from_args(&args[3..]);
        lb.set_workload_profile(WorkloadProfile::ExtractMany);
        lb.extract_to_directory(Path::new(dest), &policy)?;
    } else {
        let path = LockboxPath::new(require_arg(args, 1, "lockbox path")?)?;
        let dest = require_arg(args, 2, "destination")?;
        let replace = args.iter().skip(3).any(|arg| arg == "--overwrite");
        lb.set_workload_profile(WorkloadProfile::ReadMostly);
        lb.extract_file_to(&path, Path::new(dest), replace)?;
    }
    Ok(())
}

pub(crate) fn cat(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let path = LockboxPath::new(require_arg(args, 1, "lockbox path")?)?;
    let lb = open_existing(lockbox_path, access)?;
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    lb.extract_file_to_writer(&path, &mut lock)?;
    Ok(())
}

fn list_with_format(args: &[String], access: &Access, format: OutputFormat) -> CliResult<()> {
    let recursive = args.iter().any(|arg| arg == "--recursive" || arg == "-R");
    let args = args
        .iter()
        .filter(|arg| !matches!(arg.as_str(), "--recursive" | "-R"))
        .cloned()
        .collect::<Vec<_>>();
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let target = args.get(1).map(String::as_str).unwrap_or("/");
    let glob = contains_glob(target);
    let path = if glob {
        LockboxPath::new("/")?
    } else {
        LockboxPath::new(target)?
    };
    let lb = open_existing(lockbox_path, access)?;
    if recursive || glob {
        let mut options = ListOptions::new(&path);
        options.recursive = true;
        if glob {
            options.set_glob(target.trim_start_matches('/'));
        }
        let mut rows = Vec::new();
        for entry in lb.list(options)? {
            let entry = entry?;
            rows.push(vec![
                kind_name(&entry.kind).to_string(),
                entry.len.to_string(),
                entry.path.to_string(),
            ]);
        }
        print_records(&["kind", "len", "path"], rows, format)?;
    } else {
        let rows = direct_listing_rows(&lb, &path)?;
        print_records(&["kind", "len", "name"], rows, format)?;
    }
    Ok(())
}

fn contains_glob(value: &str) -> bool {
    value.contains('*') || value.contains('?')
}

fn direct_listing_rows(lb: &Lockbox, path: &LockboxPath) -> CliResult<Vec<Vec<String>>> {
    if let Some(entry) = lb.stat(path) {
        if entry.kind != lockbox_core::LockboxEntryKind::Directory {
            return Ok(vec![vec![
                kind_name(&entry.kind).to_string(),
                entry.len.to_string(),
                leaf_name(entry.path.as_str()).to_string(),
            ]]);
        }
    }

    let mut options = ListOptions::new(path);
    options.recursive = true;
    let mut rows = BTreeMap::new();
    let prefix = listing_prefix(path.as_str());
    for entry in lb.list(options)? {
        let entry = entry?;
        let rest = entry
            .path
            .as_str()
            .strip_prefix(&prefix)
            .unwrap_or(entry.path.as_str());
        let Some((name, is_directory)) = direct_child(rest) else {
            continue;
        };
        let row = if is_directory || entry.kind == lockbox_core::LockboxEntryKind::Directory {
            vec!["directory".to_string(), "-".to_string(), format!("{name}/")]
        } else {
            vec![
                kind_name(&entry.kind).to_string(),
                entry.len.to_string(),
                name.to_string(),
            ]
        };
        rows.entry(name.to_string()).or_insert(row);
    }
    Ok(rows.into_values().collect())
}

fn listing_prefix(path: &str) -> String {
    if path == "/" {
        "/".to_string()
    } else {
        format!("{}/", path.trim_end_matches('/'))
    }
}

fn direct_child(rest: &str) -> Option<(&str, bool)> {
    let rest = rest.trim_start_matches('/');
    if rest.is_empty() {
        return None;
    }
    match rest.split_once('/') {
        Some((name, _)) if !name.is_empty() => Some((name, true)),
        None => Some((rest, false)),
        _ => None,
    }
}

fn leaf_name(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
}

fn default_lockbox_path_for_source(source: &Path) -> CliResult<String> {
    if source.is_dir() {
        return Ok("/".to_string());
    }
    Ok(format!("/{}", source_file_name(source)?))
}

fn destination_lockbox_path(
    source: &Path,
    source_metadata: &fs::Metadata,
    destination: &str,
) -> CliResult<String> {
    if source_metadata.is_file() && destination_looks_like_directory(destination) {
        let path = format!(
            "{}/{}",
            destination.trim_end_matches('/'),
            source_file_name(source)?
        );
        LockboxPath::new(&path)?;
        return Ok(path);
    }
    LockboxPath::new(destination)?;
    Ok(destination.to_string())
}

fn destination_looks_like_directory(destination: &str) -> bool {
    destination == "/" || destination.ends_with('/') || !leaf_name(destination).contains('.')
}

fn source_file_name(source: &Path) -> CliResult<&str> {
    let Some(name) = source.file_name().and_then(|name| name.to_str()) else {
        return Err(Error::UnsupportedHostPath(format!(
            "source path is not valid UTF-8: {}",
            source.display()
        ))
        .into());
    };
    Ok(name)
}

pub(crate) fn remove(args: &[String], access: &Access) -> CliResult<()> {
    let force = args.iter().any(|arg| arg == "--force");
    let args = args
        .iter()
        .filter(|arg| !matches!(arg.as_str(), "--force"))
        .cloned()
        .collect::<Vec<_>>();
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let path = LockboxPath::new(root_relative_lockbox_path(require_arg(
        &args,
        1,
        "lockbox path",
    )?))?;
    let mut lb = open_existing(lockbox_path, access)?;
    let Some(entry) = lb.stat(&path) else {
        return Err(Error::NotFound(path.to_string()).into());
    };
    if !force && !confirm_remove(path.as_str())? {
        println!("No entries removed.");
        return Ok(());
    }
    lb.delete(&path)?;
    lb.commit()?;
    println!("Removed 1 {}: {}", kind_name(&entry.kind), entry.path);
    Ok(())
}

fn root_relative_lockbox_path(path: &str) -> String {
    if path.starts_with('/') || path.contains('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

pub(crate) fn rename(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let from = LockboxPath::new(require_arg(args, 1, "from")?)?;
    let to = LockboxPath::new(require_arg(args, 2, "to")?)?;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.create_parent_dirs_for(&to)?;
    lb.rename(&from, &to)?;
    lb.commit()?;
    Ok(())
}

fn kind_name(kind: &lockbox_core::LockboxEntryKind) -> &'static str {
    match kind {
        lockbox_core::LockboxEntryKind::File => "file",
        lockbox_core::LockboxEntryKind::Symlink => "symlink",
        lockbox_core::LockboxEntryKind::Directory => "directory",
    }
}

fn nanos_to_secs(nanos: u128) -> f64 {
    nanos as f64 / 1_000_000_000.0
}

fn extract_policy_from_args(args: &[String]) -> ExtractPolicy {
    let mut policy = ExtractPolicy::default();
    for arg in args {
        match arg.as_str() {
            "--overwrite" => policy.overwrite = true,
            "--restore-symlinks" => policy.restore_symlinks = true,
            "--restore-permissions" => policy.restore_permissions = true,
            _ => {}
        }
    }
    policy
}

fn add_source_path(
    lockbox: &mut Lockbox,
    source: &Path,
    lockbox_root: &str,
    progress: &mut AddProgress,
) -> CliResult<()> {
    let lockbox_root = LockboxPath::new(lockbox_root)?;
    if source.is_file() {
        progress.record(source)?;
        lockbox.create_parent_dirs_for(&lockbox_root)?;
        lockbox.add_file_from_path(source, &lockbox_root, false)?;
        return Ok(());
    }
    if source.is_dir() {
        if lockbox_root.as_str() != "/" {
            create_lockbox_dir_if_missing(lockbox, &lockbox_root, true)?;
        }
        add_directory(lockbox, source, source, &lockbox_root, progress)?;
        return Ok(());
    }
    Err(Error::UnsupportedHostPath(source.display().to_string()).into())
}

fn add_directory(
    lockbox: &mut Lockbox,
    root: &Path,
    current: &Path,
    lockbox_root: &LockboxPath,
    progress: &mut AddProgress,
) -> CliResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            progress.record(&path)?;
            let relative = path.strip_prefix(root)?;
            let lockbox_path = join_lockbox_path(lockbox_root, relative)?;
            create_lockbox_dir_if_missing(lockbox, &lockbox_path, true)?;
            add_directory(lockbox, root, &path, lockbox_root, progress)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root)?;
            let lockbox_path = join_lockbox_path(lockbox_root, relative)?;
            progress.record(&path)?;
            lockbox.create_parent_dirs_for(&lockbox_path)?;
            lockbox.add_file_from_path(&path, &lockbox_path, false)?;
        }
    }
    Ok(())
}

fn create_lockbox_dir_if_missing(
    lockbox: &mut Lockbox,
    path: &LockboxPath,
    create_parents: bool,
) -> CliResult<()> {
    if lockbox.is_dir(path) {
        return Ok(());
    }
    match lockbox.create_dir(path, create_parents) {
        Ok(()) | Err(Error::AlreadyExists(_)) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

struct AddProgress {
    enabled: bool,
    terminal: bool,
    last_write: Option<Instant>,
    pending: Option<String>,
    wrote: bool,
}

impl AddProgress {
    fn for_source(source: &Path) -> Self {
        let mode = std::env::var("LOCKBOX_ADD_PROGRESS").ok();
        let terminal = io::stderr().is_terminal();
        let enabled = match mode.as_deref() {
            Some("0" | "off" | "false" | "never") => false,
            Some("1" | "on" | "true" | "always") => true,
            _ => source.is_dir() && terminal,
        };
        Self {
            enabled,
            terminal,
            last_write: None,
            pending: None,
            wrote: false,
        }
    }

    fn record(&mut self, path: &Path) -> CliResult<()> {
        if !self.enabled {
            return Ok(());
        }
        self.pending = Some(path.display().to_string());
        if self
            .last_write
            .is_none_or(|last_write| last_write.elapsed() >= ADD_PROGRESS_INTERVAL)
        {
            self.write_pending()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> CliResult<()> {
        if !self.enabled {
            return Ok(());
        }
        self.write_pending()?;
        if self.wrote {
            if self.terminal {
                eprint!("\r{}\r", " ".repeat(terminal_width_fallback()));
            } else {
                eprintln!();
            }
            io::stderr().flush()?;
        }
        Ok(())
    }

    fn write_pending(&mut self) -> CliResult<()> {
        let Some(path) = self.pending.take() else {
            return Ok(());
        };
        eprint!("\rAdding: {path}");
        io::stderr().flush()?;
        self.last_write = Some(Instant::now());
        self.wrote = true;
        Ok(())
    }
}

fn terminal_width_fallback() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(120)
}

fn join_lockbox_path(lockbox_root: &LockboxPath, relative: &Path) -> CliResult<LockboxPath> {
    let mut out = lockbox_root.as_str().trim_end_matches('/').to_string();
    if out.is_empty() {
        out.push('/');
    }
    for component in relative.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(Error::UnsupportedHostPath(format!(
                "unsupported source path component in {}",
                relative.display()
            ))
            .into());
        };
        let Some(part) = part.to_str() else {
            return Err(Error::UnsupportedHostPath(format!(
                "source path is not valid UTF-8: {}",
                relative.display()
            ))
            .into());
        };
        if !out.ends_with('/') {
            out.push('/');
        }
        out.push_str(part);
    }
    Ok(LockboxPath::new(out)?)
}

fn confirm_remove(path: &str) -> CliResult<bool> {
    eprint!("Remove lockbox entry '{path}'? Type y or yes to confirm: ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
