use super::context::{cli_error, open_existing, open_or_create, require_arg, Access, CliResult};
use super::filters::{excluded, included, normalize as normalize_rules};
use super::output::{output_format_from_matches, print_records, OutputFormat};
use super::{
    default_lockbox_for_add, default_lockbox_for_command, optional_lockbox_positionals,
    positional_values,
};
use clap::ArgMatches;
use revault_lockbox_api::{
    Error, ExtractPolicy, ListOptions, Lockbox, LockboxEntry, LockboxEntryKind, LockboxPath,
    WorkerPolicy, WorkloadProfile,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const ADD_PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) fn add_matches(
    matches: &ArgMatches,
    access: &Access,
    worker_policy: WorkerPolicy,
) -> CliResult<()> {
    add(add_request_from_matches(matches)?, access, worker_policy)
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
    if matches.get_flag("recursive") {
        args.push("--recursive".to_string());
    }
    remove(&args, access)
}

pub(crate) fn rename_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    rename(
        &optional_lockbox_positionals(positional_values(matches, "args"), 2)?,
        access,
    )
}

struct AddRequest {
    lockbox_path: String,
    sources: Vec<AddSource>,
    overwrite: bool,
    includes: Vec<String>,
    excludes: Vec<String>,
}

struct AddSource {
    path: PathBuf,
    destination: LockboxPath,
    is_directory: bool,
}

#[derive(Default)]
struct AddOutcome {
    added: usize,
    replaced: usize,
}

impl AddOutcome {
    fn record_file(&mut self, replaced: bool) {
        if replaced {
            self.replaced += 1;
        } else {
            self.added += 1;
        }
    }

    fn merge(&mut self, other: Self) {
        self.added += other.added;
        self.replaced += other.replaced;
    }
}

fn add_request_from_matches(matches: &ArgMatches) -> CliResult<AddRequest> {
    let source_values = matches
        .get_many::<String>("sources")
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let lockbox_path = default_lockbox_for_add()?;
    if source_values.is_empty() {
        return Err(cli_error("add requires at least one source"));
    }
    let recursive = matches.get_flag("recursive");
    let destination = matches.get_one::<String>("to").map(String::as_str);
    let sources = prepare_add_sources(&source_values, destination, recursive)?;
    let mut includes = matches
        .get_many::<String>("include")
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let mut excludes = matches
        .get_many::<String>("exclude")
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    normalize_rules(&mut includes);
    normalize_rules(&mut excludes);
    Ok(AddRequest {
        lockbox_path,
        sources,
        overwrite: matches.get_flag("overwrite"),
        includes,
        excludes,
    })
}

fn prepare_add_sources(
    values: &[String],
    destination: Option<&str>,
    recursive: bool,
) -> CliResult<Vec<AddSource>> {
    if values.len() > 1 && destination.is_some_and(|path| !path.ends_with('/')) {
        return Err(cli_error(
            "--to must end with / when adding multiple sources",
        ));
    }
    let mut sources = Vec::with_capacity(values.len());
    for value in values {
        let path = PathBuf::from(value);
        let metadata = source_metadata(&path)?;
        let is_directory = metadata.is_dir();
        if is_directory && !recursive {
            return Err(cli_error(format!(
                "source is a directory: {}; pass --recursive to import its files",
                path.display()
            )));
        }
        if is_directory && values.len() > 1 {
            return Err(cli_error(
                "add directory sources separately so their stored roots are unambiguous",
            ));
        }
        let destination = add_destination(&path, is_directory, values.len(), destination)?;
        sources.push(AddSource {
            path,
            destination,
            is_directory,
        });
    }
    let mut destinations = BTreeSet::new();
    for source in &sources {
        if !destinations.insert(source.destination.to_string()) {
            return Err(cli_error(format!(
                "multiple sources map to the same lockbox path: {}",
                source.destination
            )));
        }
    }
    Ok(sources)
}

fn add_destination(
    source: &Path,
    is_directory: bool,
    source_count: usize,
    destination: Option<&str>,
) -> CliResult<LockboxPath> {
    let Some(destination) = destination else {
        return if is_directory {
            Ok(LockboxPath::new("/")?)
        } else {
            cli_lockbox_path(source_file_name(source)?)
        };
    };
    let destination_path = cli_lockbox_path(destination)?;
    if is_directory {
        return Ok(destination_path);
    }
    if source_count > 1 || destination.ends_with('/') {
        return join_lockbox_leaf(&destination_path, source_file_name(source)?);
    }
    Ok(destination_path)
}

fn cli_lockbox_path(value: &str) -> CliResult<LockboxPath> {
    let rooted = if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    };
    Ok(LockboxPath::new(rooted)?)
}

fn join_lockbox_leaf(directory: &LockboxPath, leaf: &str) -> CliResult<LockboxPath> {
    let path = if directory.as_str() == "/" {
        format!("/{leaf}")
    } else {
        format!("{}/{leaf}", directory.as_str().trim_end_matches('/'))
    };
    Ok(LockboxPath::new(path)?)
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

fn add(request: AddRequest, access: &Access, worker_policy: WorkerPolicy) -> CliResult<()> {
    let creates_lockbox = !Path::new(&request.lockbox_path).exists();
    let mut lb = open_or_create(&request.lockbox_path, access)?;
    lb.set_worker_policy(worker_policy);
    if creates_lockbox
        || request.sources.len() > 1
        || request.sources.iter().any(|source| source.is_directory)
    {
        lb.set_workload_profile(WorkloadProfile::BulkImport);
    }
    for source in &request.sources {
        if !request.overwrite && !source.is_directory && lb.stat(&source.destination).is_some() {
            return Err(Error::AlreadyExists(source.destination.to_string()).into());
        }
    }
    lb.reset_import_stats();
    let add_start = Instant::now();
    let mut progress = AddProgress::for_source(&request.sources[0].path);
    let add_result: CliResult<AddOutcome> = (|| {
        let mut outcome = AddOutcome::default();
        for source in &request.sources {
            let source_name = source_file_name(&source.path)?;
            if source.path.is_file()
                && (!included(source_name, &request.includes)
                    || excluded(source_name, &request.excludes))
            {
                continue;
            }
            outcome.merge(add_source_path(
                &mut lb,
                &source.path,
                source.destination.as_str(),
                request.overwrite,
                &request.includes,
                &request.excludes,
                &mut progress,
            )?);
        }
        Ok(outcome)
    })();
    let progress_result = progress.finish();
    let outcome = add_result?;
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
    print_add_outcome(&outcome, &request.lockbox_path);
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
        println!("Extracted lockbox to {dest}.");
    } else {
        let path = cli_lockbox_path(require_arg(args, 1, "lockbox path")?)?;
        let dest = require_arg(args, 2, "destination")?;
        let replace = args.iter().skip(3).any(|arg| arg == "--overwrite");
        lb.set_workload_profile(WorkloadProfile::ReadMostly);
        lb.extract_file_to(&path, Path::new(dest), replace)?;
        println!("Extracted {path} to {dest}.");
    }
    Ok(())
}

pub(crate) fn cat(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let lb = open_existing(lockbox_path, access)?;
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    for path in args.get(1..).unwrap_or_default() {
        let path = cli_lockbox_path(path)?;
        lb.extract_file_to_writer(&path, &mut lock)?;
    }
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
        cli_lockbox_path(target)?
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

pub(crate) fn direct_listing_rows(lb: &Lockbox, path: &LockboxPath) -> CliResult<Vec<Vec<String>>> {
    if let Some(entry) = lb.stat(path) {
        if entry.kind != revault_lockbox_api::LockboxEntryKind::Directory {
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
        let row = if is_directory || entry.kind == revault_lockbox_api::LockboxEntryKind::Directory
        {
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
    let recursive = args.iter().any(|arg| arg == "--recursive");
    let args = args
        .iter()
        .filter(|arg| !matches!(arg.as_str(), "--force" | "--recursive"))
        .cloned()
        .collect::<Vec<_>>();
    let lockbox_path = require_arg(&args, 0, "lockbox")?;
    let mut lb = open_existing(lockbox_path, access)?;
    let patterns = args.get(1..).unwrap_or_default();
    let entries = resolve_remove_entries(&lb, patterns)?;
    if entries
        .iter()
        .any(|entry| entry.kind == LockboxEntryKind::Directory)
        && !recursive
    {
        return Err(cli_error(
            "removing a directory requires --recursive (-r or -R)",
        ));
    }
    let entries = collapse_descendants_of_selected_directories(entries);
    if !force && !confirm_remove(&entries)? {
        println!("No entries removed.");
        return Ok(());
    }
    for entry in &entries {
        if entry.kind == LockboxEntryKind::Directory {
            lb.remove_dir_recursive(&entry.path)?;
        } else {
            lb.delete(&entry.path)?;
        }
    }
    lb.commit()?;
    if entries.len() == 1 {
        println!(
            "Removed 1 {}: {}",
            kind_name(&entries[0].kind),
            entries[0].path
        );
    } else {
        println!("Removed {} entries.", entries.len());
    }
    Ok(())
}

fn resolve_remove_entries(lb: &Lockbox, patterns: &[String]) -> CliResult<Vec<LockboxEntry>> {
    if patterns.is_empty() {
        return Err(cli_error(
            "remove requires at least one stored path or glob",
        ));
    }
    let mut entries = BTreeMap::new();
    for pattern in patterns {
        if contains_glob(pattern) {
            let mut options = ListOptions::new(&LockboxPath::new("/")?);
            let archive_pattern = pattern.trim_start_matches('/');
            options.recursive = archive_pattern.contains('/') || archive_pattern.contains("**");
            options.set_glob(archive_pattern);
            let matches = lb.list(options)?.collect::<Result<Vec<_>, _>>()?;
            if matches.is_empty() {
                return Err(Error::NotFound(format!("no lockbox entries match {pattern}")).into());
            }
            for entry in matches {
                entries.insert(entry.path.to_string(), entry);
            }
        } else {
            let path = cli_lockbox_path(pattern)?;
            let Some(entry) = lb.stat(&path) else {
                return Err(Error::NotFound(path.to_string()).into());
            };
            entries.insert(entry.path.to_string(), entry);
        }
    }
    Ok(entries.into_values().collect())
}

fn collapse_descendants_of_selected_directories(
    mut entries: Vec<LockboxEntry>,
) -> Vec<LockboxEntry> {
    entries.sort_by_key(|entry| entry.path.as_str().len());
    let mut targets: Vec<LockboxEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        if targets.iter().any(|target| {
            target.kind == LockboxEntryKind::Directory && entry.path.is_descendant_of(&target.path)
        }) {
            continue;
        }
        targets.push(entry);
    }
    targets
}

pub(crate) fn rename(args: &[String], access: &Access) -> CliResult<()> {
    let lockbox_path = require_arg(args, 0, "lockbox")?;
    let from = cli_lockbox_path(require_arg(args, 1, "from")?)?;
    let to = cli_lockbox_path(require_arg(args, 2, "to")?)?;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.create_parent_dirs_for(&to)?;
    lb.rename(&from, &to)?;
    lb.commit()?;
    println!("Moved {from} to {to}.");
    Ok(())
}

fn kind_name(kind: &revault_lockbox_api::LockboxEntryKind) -> &'static str {
    match kind {
        revault_lockbox_api::LockboxEntryKind::File => "file",
        revault_lockbox_api::LockboxEntryKind::Symlink => "symlink",
        revault_lockbox_api::LockboxEntryKind::Directory => "directory",
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
    overwrite: bool,
    includes: &[String],
    excludes: &[String],
    progress: &mut AddProgress,
) -> CliResult<AddOutcome> {
    let lockbox_root = LockboxPath::new(lockbox_root)?;
    if source.is_file() {
        let replaced = lockbox.stat(&lockbox_root).is_some();
        progress.record(source)?;
        lockbox.create_parent_dirs_for(&lockbox_root)?;
        lockbox.add_file_from_path(source, &lockbox_root, overwrite)?;
        let mut outcome = AddOutcome::default();
        outcome.record_file(replaced);
        return Ok(outcome);
    }
    if source.is_dir() {
        if lockbox_root.as_str() != "/" {
            create_lockbox_dir_if_missing(lockbox, &lockbox_root, true)?;
        }
        let options = AddDirectoryOptions {
            root: source,
            lockbox_root: &lockbox_root,
            overwrite,
            includes,
            excludes,
        };
        return add_directory(lockbox, source, &options, progress);
    }
    Err(Error::UnsupportedHostPath(source.display().to_string()).into())
}

struct AddDirectoryOptions<'a> {
    root: &'a Path,
    lockbox_root: &'a LockboxPath,
    overwrite: bool,
    includes: &'a [String],
    excludes: &'a [String],
}

fn add_directory(
    lockbox: &mut Lockbox,
    current: &Path,
    options: &AddDirectoryOptions<'_>,
    progress: &mut AddProgress,
) -> CliResult<AddOutcome> {
    let mut outcome = AddOutcome::default();
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(options.root)?;
        let relative_rule = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if excluded(&relative_rule, options.excludes) {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            progress.record(&path)?;
            if options.includes.is_empty() {
                let lockbox_path = join_lockbox_path(options.lockbox_root, relative)?;
                create_lockbox_dir_if_missing(lockbox, &lockbox_path, true)?;
            }
            outcome.merge(add_directory(lockbox, &path, options, progress)?);
        } else if file_type.is_file() {
            if !included(&relative_rule, options.includes) {
                continue;
            }
            let lockbox_path = join_lockbox_path(options.lockbox_root, relative)?;
            let replaced = lockbox.stat(&lockbox_path).is_some();
            progress.record(&path)?;
            lockbox.create_parent_dirs_for(&lockbox_path)?;
            lockbox.add_file_from_path(&path, &lockbox_path, options.overwrite)?;
            outcome.record_file(replaced);
        }
    }
    Ok(outcome)
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

fn print_add_outcome(outcome: &AddOutcome, lockbox_path: &str) {
    match (outcome.added, outcome.replaced) {
        (added, 0) => println!(
            "Added {added} {} to {lockbox_path}.",
            plural(added, "file", "files")
        ),
        (0, replaced) => println!(
            "Replaced {replaced} {} in {lockbox_path}.",
            plural(replaced, "file", "files")
        ),
        (added, replaced) => println!(
            "Added {added} {} and replaced {replaced} {} in {lockbox_path}.",
            plural(added, "file", "files"),
            plural(replaced, "file", "files")
        ),
    }
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

fn confirm_remove(entries: &[LockboxEntry]) -> CliResult<bool> {
    if entries.len() == 1 {
        eprint!(
            "Remove lockbox entry '{}'? Type y or yes to confirm: ",
            entries[0].path
        );
    } else {
        eprintln!("Remove {} lockbox entries?", entries.len());
        for entry in entries {
            eprintln!("  {}", entry.path);
        }
        eprint!("Type y or yes to confirm: ");
    }
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
