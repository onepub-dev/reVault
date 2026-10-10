use clap::ArgMatches;
use revault_lockbox_api::{
    lock_path_for, Error, ExtractPolicy, ListOptions, Lockbox, LockboxEntry, LockboxEntryKind,
    LockboxPath, MirrorMissingFilePolicy, MirrorProject, WorkloadProfile,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::context::{cli_error, open_existing, Access, CliResult};
use super::default_lockbox_for_add;
use super::filters::{excluded, included, normalize as normalize_rules};
use super::mirror_index::BinaryTable;
use super::output::{human_size, output_format_from_matches, print_records};

const LARGE_DELETE_PERCENT: usize = 50;

/// A complete, inspectable one-way mirror update plan.
#[derive(Debug)]
pub(crate) struct MirrorPlan {
    pub additions: usize,
    pub replacements: usize,
    pub removals: usize,
    pub unchanged: usize,
    directories: usize,
    removed_files: usize,
}

/// Safety controls that affect mirror application.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MirrorUpdateOptions {
    pub allow_empty: bool,
    pub allow_large_delete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileFingerprint {
    len: u64,
    modified_nanos: Option<u128>,
    digest: String,
}

#[derive(Debug)]
enum SourceEntryValue {
    Directory,
    File {
        host_path: PathBuf,
        fingerprint: FileFingerprint,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum PlanOperation {
    Add = 1,
    Replace = 2,
    Remove = 3,
    MakeDirectory = 4,
}

#[derive(Debug)]
struct MirrorRequest {
    lockbox: String,
    project: MirrorProject,
    destination: LockboxPath,
    includes: Vec<String>,
    excludes: Vec<String>,
    options: MirrorUpdateOptions,
    force: bool,
    format: String,
    quiet: bool,
}

struct MirrorProgress {
    enabled: bool,
    redraw: bool,
    line_width: usize,
    line_active: bool,
    next_percent: usize,
    scanned_files: usize,
    scanned_bytes: u64,
    last_scan_write: Instant,
}

impl MirrorProgress {
    fn new(enabled: bool) -> Self {
        let redraw = enabled
            && io::stderr().is_terminal()
            && std::env::var("TERM").map_or(true, |term| term != "dumb");
        Self {
            enabled,
            redraw,
            line_width: 0,
            line_active: false,
            next_percent: 10,
            scanned_files: 0,
            scanned_bytes: 0,
            last_scan_write: Instant::now(),
        }
    }

    fn scanned(&mut self, bytes: usize) {
        self.scanned_bytes = self.scanned_bytes.saturating_add(bytes as u64);
        if self.enabled && self.last_scan_write.elapsed() >= Duration::from_secs(1) {
            self.write(
                format!(
                    "scanning ({} files complete, {} read).",
                    self.scanned_files,
                    human_size(self.scanned_bytes)
                ),
                false,
            );
            self.last_scan_write = Instant::now();
        }
    }

    fn scanned_file(&mut self) {
        self.scanned_files += 1;
    }

    fn stage(&mut self, message: impl AsRef<str>) {
        self.write(message, false);
    }

    fn finish(&mut self, message: impl AsRef<str>) {
        self.write(message, true);
    }

    fn begin_counted(&mut self, message: impl AsRef<str>) {
        self.next_percent = 10;
        self.stage(message);
    }

    fn counted(&mut self, label: &str, completed: usize, total: usize) {
        if !self.enabled || total == 0 {
            return;
        }
        let percent = completed.saturating_mul(100) / total;
        if percent >= self.next_percent || completed == total {
            self.write(format!("{label} {completed}/{total} ({percent}%)"), false);
            while self.next_percent <= percent {
                self.next_percent += 10;
            }
        }
    }

    fn write(&mut self, message: impl AsRef<str>, finish: bool) {
        if !self.enabled {
            return;
        }
        let rendered = format!("Mirror: {}", message.as_ref());
        if !self.redraw {
            eprintln!("{rendered}");
            return;
        }
        let padding = self.line_width.saturating_sub(rendered.len());
        let mut stderr = io::stderr().lock();
        let _ = write!(stderr, "\r{rendered}{:padding$}", "", padding = padding);
        if finish {
            let _ = writeln!(stderr);
            self.line_width = 0;
            self.line_active = false;
        } else {
            let _ = stderr.flush();
            self.line_width = rendered.len().max(self.line_width);
            self.line_active = true;
        }
    }
}

impl Drop for MirrorProgress {
    fn drop(&mut self) {
        if self.redraw && self.line_active {
            eprintln!();
        }
    }
}

struct SourceSnapshot {
    _temporary_directory: tempfile::TempDir,
    source: BinaryTable,
    plan: BinaryTable,
    verification: BinaryTable,
    file_count: usize,
    directory_count: usize,
}

impl SourceSnapshot {
    fn files_empty(&self) -> bool {
        self.file_count == 0
    }
}

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let lockbox = default_lockbox_for_add()?;
    let explicit = matches.get_one::<String>("project").map(String::as_str);
    let (action, action_matches) = matches
        .subcommand()
        .ok_or_else(|| cli_error("mirror requires an action"))?;
    match action {
        "create" => create_project(&lockbox, explicit, action_matches, access),
        "projects" if explicit.is_none() => list_projects(&lockbox, action_matches, access),
        "projects" => Err(cli_error(
            "mirror projects lists every project and does not accept a project name",
        )),
        _ => {
            let project = select_project(&lockbox, explicit, access)?;
            match action {
                "info" => show_project(&project, action_matches),
                "status" => status_project(&lockbox, project, action_matches, access),
                "update" => update_project(&lockbox, project, action_matches, access),
                "configure" => configure_project(&lockbox, project, action_matches, access),
                "rebind" => rebind_project(&lockbox, project, action_matches, access),
                "forget" => forget_project(&lockbox, project, action_matches, access),
                "destroy" | "delete-project" => {
                    delete_project(&lockbox, project, action_matches, access)
                }
                "rule" => rules_project(&lockbox, project, action_matches, access),
                "add" | "extract" | "cat" | "list" | "remove" | "move" => {
                    project_file_command(&lockbox, project, action, action_matches, access)
                }
                other => Err(cli_error(format!("unknown mirror action: {other}"))),
            }
        }
    }
}

fn configure_project(
    lockbox_path: &str,
    mut project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    if let Some(policy) = matches.get_one::<String>("missing-files") {
        project.missing_file_policy = match policy.as_str() {
            "remove" => MirrorMissingFilePolicy::Remove,
            "retain" => MirrorMissingFilePolicy::Retain,
            _ => return Err(cli_error("missing-files must be remove or retain")),
        };
    }
    if matches.get_flag("strict") {
        project.strict = true;
    } else if matches.get_flag("no-strict") {
        project.strict = false;
    }
    let mut lb = open_existing(lockbox_path, access)?;
    lb.update_mirror_project(&project)?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!("Updated mirror '{}'.", project.name);
    println!(
        "  missing files: {}",
        policy_name(project.missing_file_policy)
    );
    println!("  strict:        {}", yes_no(project.strict));
    Ok(())
}

fn rebind_project(
    lockbox_path: &str,
    mut project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let source = canonical_source(Path::new(
        matches
            .get_one::<String>("from")
            .ok_or_else(|| cli_error("mirror rebind requires --from"))?,
    ))?;
    let identity = platform_identity(&source)?;
    println!("Rebind mirror '{}':", project.name);
    println!("  old: {}", project.source);
    println!("  new: {}", source.display());
    if !matches.get_flag("force")
        && !confirm("Store this host binding? Type y or yes to confirm: ")?
    {
        println!("Mirror rebind cancelled.");
        return Ok(());
    }
    project.source = source
        .to_str()
        .ok_or_else(|| cli_error("mirror source path is not valid UTF-8"))?
        .to_string();
    project.host_identity = identity;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.update_mirror_project(&project)?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!("Rebound mirror '{}'.", project.name);
    Ok(())
}

fn forget_project(
    lockbox_path: &str,
    project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    if !matches.get_flag("force")
        && !confirm("Forget this mirror but retain its files? Type y or yes to confirm: ")?
    {
        println!("Mirror forget cancelled.");
        return Ok(());
    }
    let mut lb = open_existing(lockbox_path, access)?;
    lb.forget_mirror_project(&project.name)?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!(
        "Forgot mirror '{}'; files under {} are now unmanaged.",
        project.name, project.destination
    );
    Ok(())
}

fn delete_project(
    lockbox_path: &str,
    project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    if !matches.get_flag("force")
        && !confirm("Delete this mirror and all managed files? Type y or yes to confirm: ")?
    {
        println!("Mirror deletion cancelled.");
        return Ok(());
    }
    let mut lb = open_existing(lockbox_path, access)?;
    let destination = project.destination.clone();
    let root_entries = if destination.as_str() == "/" {
        let mut options = ListOptions::new(&destination);
        options.recursive = true;
        lb.list(options)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry| entry.path)
            .collect()
    } else {
        vec![destination.clone()]
    };
    lb.with_mirror_project_mutation(&project.name, |lb, _| {
        for path in removal_roots(&root_entries, lb) {
            if lb.is_dir(&path) {
                lb.remove_dir_recursive(&path)?;
            } else if lb.stat(&path).is_some() {
                lb.delete(&path)?;
            }
        }
        Ok(())
    })?;
    lb.forget_mirror_project(&project.name)?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!("Deleted mirror '{}' and its managed files.", project.name);
    Ok(())
}

fn rules_project(
    lockbox_path: &str,
    mut project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let (action, action_matches) = matches
        .subcommand()
        .ok_or_else(|| cli_error("mirror rule requires an action"))?;
    if action == "list" {
        let kind = action_matches.get_one::<String>("kind").map(String::as_str);
        let format = selected_format(action_matches);
        if format == "json" {
            println!(
                "{}",
                json!({
                    "include": if kind != Some("exclude") { Some(&project.includes) } else { None },
                    "exclude": if kind != Some("include") { Some(&project.excludes) } else { None },
                })
            );
        } else if format == "tsv" {
            if kind != Some("exclude") {
                for rule in &project.includes {
                    println!("include\t{rule}");
                }
            }
            if kind != Some("include") {
                for rule in &project.excludes {
                    println!("exclude\t{rule}");
                }
            }
        } else {
            if kind != Some("exclude") {
                println!("include:");
                print_rule_values(&project.includes, "(all source paths)");
            }
            if kind != Some("include") {
                println!("exclude:");
                print_rule_values(&project.excludes, "(none)");
            }
        }
        return Ok(());
    }
    let kind = action_matches
        .get_one::<String>("kind")
        .map(String::as_str)
        .ok_or_else(|| cli_error("rule kind is required"))?;
    let rules = if kind == "include" {
        &mut project.includes
    } else {
        &mut project.excludes
    };
    match action {
        "add" => {
            let values = action_matches
                .get_many::<String>("patterns")
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>();
            rules.extend(values);
            normalize_rules(rules);
        }
        "remove" => {
            let values = action_matches
                .get_many::<String>("patterns")
                .into_iter()
                .flatten()
                .collect::<BTreeSet<_>>();
            let old_len = rules.len();
            rules.retain(|rule| !values.contains(rule));
            if rules.len() == old_len {
                return Err(cli_error("none of the supplied rules were stored"));
            }
        }
        "clear" => match kind {
            "include" => project.includes.clear(),
            "exclude" => project.excludes.clear(),
            "all" => {
                project.includes.clear();
                project.excludes.clear();
            }
            _ => unreachable!(),
        },
        _ => return Err(cli_error(format!("unknown rule action: {action}"))),
    }
    let mut lb = open_existing(lockbox_path, access)?;
    lb.update_mirror_project(&project)?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!(
        "Updated {} rules for mirror '{}'. Run mirror status before updating.",
        kind, project.name
    );
    Ok(())
}

fn print_rules(project: &MirrorProject) {
    println!("  include:");
    print_rule_values(&project.includes, "(all source paths)");
    println!("  exclude:");
    print_rule_values(&project.excludes, "(none)");
}

fn print_rule_values(values: &[String], empty: &str) {
    if values.is_empty() {
        println!("    {empty}");
    } else {
        for value in values {
            println!("    {value}");
        }
    }
}

fn project_json(project: &MirrorProject) -> serde_json::Value {
    json!({
        "name": project.name,
        "source": project.source,
        "destination": project.destination.to_string(),
        "include": project.includes,
        "exclude": project.excludes,
        "missing_files": policy_name(project.missing_file_policy),
        "strict": project.strict,
        "host_identity": project.host_identity,
    })
}

fn policy_name(policy: MirrorMissingFilePolicy) -> &'static str {
    match policy {
        MirrorMissingFilePolicy::Remove => "remove",
        MirrorMissingFilePolicy::Retain => "retain",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn selected_format(matches: &ArgMatches) -> &str {
    matches
        .try_get_one::<String>("format")
        .ok()
        .flatten()
        .map(String::as_str)
        .unwrap_or("table")
}

fn validate_project_action_name(name: &str) -> CliResult<()> {
    const RESERVED: &[&str] = &[
        "create",
        "projects",
        "info",
        "status",
        "update",
        "configure",
        "rebind",
        "forget",
        "delete",
        "destroy",
        "delete-project",
        "rule",
        "rules",
        "add",
        "extract",
        "cat",
        "list",
        "ls",
        "remove",
        "rm",
        "move",
        "mv",
        "rename",
        "help",
    ];
    if RESERVED.contains(&name) {
        return Err(cli_error(format!(
            "mirror project name '{name}' conflicts with a command"
        )));
    }
    Ok(())
}

fn print_status_warnings(
    plan: &MirrorPlan,
    source: &SourceSnapshot,
    request: &MirrorRequest,
    lb: &Lockbox,
) -> CliResult<()> {
    let warning = |message: String| {
        if request.format == "table" {
            println!("\nBlocked update: {message}");
        } else {
            eprintln!("Blocked update: {message}");
        }
    };
    if source.files_empty() && plan.removals > 0 {
        warning("no source files match; update requires --allow-empty.".to_string());
    }
    let count = destination_file_count(lb, &request.destination)?;
    if count > 0 && plan.removed_files * 100 > count * LARGE_DELETE_PERCENT {
        warning(format!(
            "removing {} of {} managed files requires --allow-large-delete.",
            plan.removed_files, count
        ));
    }
    Ok(())
}

fn project_file_command(
    lockbox_path: &str,
    project: MirrorProject,
    action: &str,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    match action {
        "add" => project_add(lockbox_path, &project, matches, access),
        "extract" => project_extract(lockbox_path, &project, matches, access),
        "cat" => project_cat(lockbox_path, &project, matches, access),
        "list" => project_list(lockbox_path, &project, matches, access),
        "remove" | "delete" => project_remove(lockbox_path, &project, matches, access),
        "move" => project_move(lockbox_path, &project, matches, access),
        _ => Err(cli_error(format!("unknown mirror file action: {action}"))),
    }
}

fn project_add(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let sources = matches
        .get_many::<String>("sources")
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let recursive = matches.get_flag("recursive");
    let overwrite = matches.get_flag("overwrite");
    let includes = filter_values(matches, "include");
    let excludes = filter_values(matches, "exclude");
    let destination_value = matches.get_one::<String>("to").map(String::as_str);
    if sources.len() > 1 && destination_value.is_some_and(|value| !value.ends_with('/')) {
        return Err(cli_error(
            "adding multiple sources requires --to to end with '/'",
        ));
    }
    let mut additions = Vec::new();
    let mut directories = BTreeSet::new();
    for source in &sources {
        let metadata = fs::metadata(source).map_err(|err| {
            cli_error(format!("cannot access source {}: {err}", source.display()))
        })?;
        if metadata.is_dir() {
            if !recursive {
                return Err(cli_error(format!(
                    "{} is a directory; pass --recursive",
                    source.display()
                )));
            }
            if sources.len() != 1 {
                return Err(cli_error("recursive add accepts one directory source"));
            }
            if let Some(destination) = destination_value {
                directories.insert(project_join(project, destination)?);
            }
            let canonical = canonical_source(source)?;
            let ignored_paths = mirror_ignored_host_paths(Path::new(lockbox_path))?;
            let mut progress = MirrorProgress::new(false);
            let snapshot = walk_source(
                &canonical,
                &includes,
                &excludes,
                &ignored_paths,
                &mut progress,
            )?;
            for record in snapshot.source.iter()? {
                let (relative, value) = decode_source_record(record?)?;
                match value {
                    SourceEntryValue::Directory => {
                        directories.insert(project_join(
                            project,
                            &join_relative_destination(destination_value, &relative),
                        )?);
                    }
                    SourceEntryValue::File { host_path, .. } => {
                        additions.push((
                            host_path,
                            project_join(
                                project,
                                &join_relative_destination(destination_value, &relative),
                            )?,
                        ));
                    }
                }
            }
        } else if metadata.is_file() {
            let name = source
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| cli_error("source filename is not valid UTF-8"))?;
            if !included(name, &includes) || excluded(name, &excludes) {
                continue;
            }
            let relative = match destination_value {
                Some(destination) if sources.len() > 1 || destination.ends_with('/') => {
                    format!("{destination}{name}")
                }
                Some(destination) => destination.to_string(),
                None => name.to_string(),
            };
            additions.push((source.clone(), project_join(project, &relative)?));
        } else {
            return Err(cli_error(format!(
                "unsupported source entry: {}",
                source.display()
            )));
        }
    }
    let worker_policy = super::read_worker_policy(matches)?;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.set_worker_policy(worker_policy);
    lb.set_workload_profile(WorkloadProfile::BulkImport);
    let mut added = 0usize;
    let mut replaced = 0usize;
    lb.with_mirror_project_mutation(&project.name, |lb, _| {
        for directory in &directories {
            if !lb.is_dir(directory) {
                lb.create_dir(directory, true)?;
            }
        }
        for (source, destination) in &additions {
            let exists = lb.stat(destination).is_some();
            lb.create_parent_dirs_for(destination)?;
            lb.add_file_from_path(source, destination, overwrite && exists)?;
            if exists {
                replaced += 1;
            } else {
                added += 1;
            }
        }
        Ok(())
    })?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!(
        "Added {added} files and replaced {replaced} files in mirror '{}'.",
        project.name
    );
    if !additions.is_empty() || !directories.is_empty() {
        print_direct_change_warning(project);
    }
    Ok(())
}

fn project_extract(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let lb = open_existing(lockbox_path, access)?;
    let args = matches
        .get_many::<String>("args")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let policy = ExtractPolicy {
        overwrite: matches.get_flag("overwrite"),
        restore_permissions: matches.get_flag("restore-permissions"),
        restore_symlinks: matches.get_flag("restore-symlinks"),
        ..ExtractPolicy::default()
    };
    if let Some(destination) = matches.get_one::<String>("to") {
        if args.len() > 1 {
            return Err(cli_error(
                "mirror extract --to accepts at most one project path",
            ));
        }
        if let Some(source) = args.first() {
            extract_project_entry(&lb, project, source, Path::new(destination), &policy)?;
            println!("Extracted {source} to {destination}.");
            return Ok(());
        }
        extract_project_tree(&lb, project, Path::new(destination), &policy)?;
        println!("Extracted mirror '{}' to {}.", project.name, destination);
        return Ok(());
    }
    if args.len() != 2 {
        return Err(cli_error(
            "mirror extract requires PATH DESTINATION or --to DESTINATION",
        ));
    }
    extract_project_entry(&lb, project, args[0], Path::new(args[1]), &policy)?;
    println!("Extracted {} to {}.", args[0], args[1]);
    Ok(())
}

fn extract_project_entry(
    lb: &Lockbox,
    project: &MirrorProject,
    relative: &str,
    destination: &Path,
    policy: &ExtractPolicy,
) -> CliResult<()> {
    let source = project_join(project, relative)?;
    let entry = lb
        .stat(&source)
        .ok_or_else(|| Error::NotFound(source.to_string()))?;
    match entry.kind {
        LockboxEntryKind::Directory => {
            lb.extract_directory_to(&source, destination, policy)?;
        }
        LockboxEntryKind::File => {
            if entry.len > policy.max_file_bytes {
                return Err(cli_error(format!(
                    "{} is {}, exceeding the per-file extraction limit of {}",
                    entry.path,
                    human_size(entry.len),
                    human_size(policy.max_file_bytes)
                )));
            }
            let replace = policy.overwrite && destination.exists();
            lb.extract_file_to(&source, destination, replace)?;
            restore_host_permissions(destination, entry.permissions, policy.restore_permissions)?;
        }
        LockboxEntryKind::Symlink => {
            return Err(cli_error(format!(
                "{source} is a symlink; extract its containing directory with --restore-symlinks"
            )));
        }
    }
    Ok(())
}

fn extract_project_tree(
    lb: &Lockbox,
    project: &MirrorProject,
    destination: &Path,
    policy: &ExtractPolicy,
) -> CliResult<()> {
    fs::create_dir_all(destination)?;
    let mut options = ListOptions::new(&project.destination);
    options.recursive = true;
    let entries = lb.list(options)?.collect::<Result<Vec<_>, _>>()?;
    if entries.len() > policy.max_files {
        return Err(cli_error(format!(
            "project contains {} entries, exceeding the extraction limit of {}",
            entries.len(),
            policy.max_files
        )));
    }
    let mut total_bytes = 0_u64;
    let mut directories = Vec::new();
    for entry in entries {
        let relative = relative_lockbox_path(&project.destination, &entry.path);
        if relative.is_empty() {
            continue;
        }
        let host = destination.join(&relative);
        match entry.kind {
            LockboxEntryKind::Directory => {
                prepare_extract_destination(&host, policy.overwrite, true)?;
                fs::create_dir_all(&host)?;
                directories.push((host, entry.permissions));
            }
            LockboxEntryKind::File => {
                if entry.len > policy.max_file_bytes {
                    return Err(cli_error(format!(
                        "{} is {}, exceeding the per-file extraction limit of {}",
                        entry.path,
                        human_size(entry.len),
                        human_size(policy.max_file_bytes)
                    )));
                }
                total_bytes = total_bytes
                    .checked_add(entry.len)
                    .ok_or_else(|| cli_error("total extracted size overflow"))?;
                if total_bytes > policy.max_total_bytes {
                    return Err(cli_error(format!(
                        "project extraction exceeds the total size limit of {}",
                        human_size(policy.max_total_bytes)
                    )));
                }
                if let Some(parent) = host.parent() {
                    fs::create_dir_all(parent)?;
                }
                let replace = policy.overwrite && fs::symlink_metadata(&host).is_ok();
                lb.extract_file_to(&entry.path, &host, replace)?;
                restore_host_permissions(&host, entry.permissions, policy.restore_permissions)?;
            }
            LockboxEntryKind::Symlink if policy.restore_symlinks => {
                if let Some(parent) = host.parent() {
                    fs::create_dir_all(parent)?;
                }
                prepare_extract_destination(&host, policy.overwrite, false)?;
                let target = lb.get_symlink_target(&entry.path)?;
                create_host_symlink(target.as_str(), &host)?;
            }
            LockboxEntryKind::Symlink => {}
        }
    }
    for (directory, permissions) in directories.into_iter().rev() {
        restore_host_permissions(&directory, permissions, policy.restore_permissions)?;
    }
    Ok(())
}

fn prepare_extract_destination(path: &Path, overwrite: bool, directory: bool) -> CliResult<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if directory && metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Ok(());
    }
    if !overwrite {
        return Err(cli_error(format!("destination exists: {}", path.display())));
    }
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_host_symlink(target: &str, path: &Path) -> CliResult<()> {
    std::os::unix::fs::symlink(target, path)?;
    Ok(())
}

#[cfg(windows)]
fn create_host_symlink(target: &str, path: &Path) -> CliResult<()> {
    std::os::windows::fs::symlink_file(target, path)?;
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn create_host_symlink(_target: &str, _path: &Path) -> CliResult<()> {
    Err(cli_error(
        "restoring symbolic links is unsupported on this platform",
    ))
}

#[cfg(unix)]
fn restore_host_permissions(path: &Path, permissions: u32, restore: bool) -> CliResult<()> {
    if restore {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(permissions))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn restore_host_permissions(_path: &Path, _permissions: u32, _restore: bool) -> CliResult<()> {
    Ok(())
}

fn project_cat(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let path = project_join(
        project,
        matches
            .get_one::<String>("path")
            .ok_or_else(|| cli_error("mirror cat requires a path"))?,
    )?;
    let lb = open_existing(lockbox_path, access)?;
    let stdout = io::stdout();
    lb.extract_file_to_writer(&path, &mut stdout.lock())?;
    Ok(())
}

fn project_list(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let target = matches
        .get_one::<String>("path")
        .map(String::as_str)
        .unwrap_or("/");
    let glob = target.contains('*') || target.contains('?');
    let base = if glob {
        project.destination.clone()
    } else {
        project_join(project, target)?
    };
    let mut options = ListOptions::new(&base);
    options.recursive = matches.get_flag("recursive") || glob;
    if glob {
        options.set_glob(target.trim_start_matches('/'));
    }
    let lb = open_existing(lockbox_path, access)?;
    let format = output_format_from_matches(matches)?;
    if !options.recursive {
        let mut rows = super::files::direct_listing_rows(&lb, &base)?;
        super::files::humanize_listing_rows(&mut rows, format);
        return print_records(&["kind", "len", "name"], rows, format);
    }
    let mut rows = Vec::new();
    for entry in lb.list(options)? {
        let entry = entry?;
        rows.push(vec![
            entry_kind_name(&entry.kind).to_string(),
            if format == super::output::OutputFormat::Table {
                human_size(entry.len)
            } else {
                entry.len.to_string()
            },
            relative_lockbox_path(&project.destination, &entry.path),
        ]);
    }
    print_records(&["kind", "len", "path"], rows, format)?;
    Ok(())
}

fn project_remove(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let patterns = matches
        .get_many::<String>("paths")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let lb = open_existing(lockbox_path, access)?;
    let mut entries = BTreeMap::<String, LockboxEntry>::new();
    for pattern in patterns {
        if pattern.contains('*') || pattern.contains('?') {
            let mut options = ListOptions::new(&project.destination);
            options.recursive = pattern.contains('/') || pattern.contains("**");
            options.set_glob(pattern.trim_start_matches('/'));
            for entry in lb.list(options)? {
                let entry = entry?;
                entries.insert(entry.path.to_string(), entry);
            }
        } else {
            let path = project_join(project, pattern)?;
            let entry = lb
                .stat(&path)
                .ok_or_else(|| cli_error(format!("project entry not found: {pattern}")))?;
            entries.insert(path.to_string(), entry);
        }
    }
    drop(lb);
    if entries.is_empty() {
        return Err(cli_error("no project entries matched"));
    }
    if entries
        .values()
        .any(|entry| entry.kind == LockboxEntryKind::Directory)
        && !matches.get_flag("recursive")
    {
        return Err(cli_error(
            "removing a directory requires --recursive (-r or -R)",
        ));
    }
    if !matches.get_flag("force")
        && !confirm(&format!(
            "Remove {} project entries? Type y or yes to confirm: ",
            entries.len()
        ))?
    {
        println!("No entries removed.");
        return Ok(());
    }
    let mut lb = open_existing(lockbox_path, access)?;
    let paths = entries
        .values()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    lb.with_mirror_project_mutation(&project.name, |lb, _| {
        for path in removal_roots(&paths, lb) {
            if lb.is_dir(&path) {
                lb.remove_dir_recursive(&path)?;
            } else {
                lb.delete(&path)?;
            }
        }
        Ok(())
    })?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!("Removed {} project entries.", entries.len());
    print_direct_change_warning(project);
    Ok(())
}

fn project_move(
    lockbox_path: &str,
    project: &MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let from_value = matches
        .get_one::<String>("from")
        .ok_or_else(|| cli_error("mirror move requires FROM"))?;
    let to_value = matches
        .get_one::<String>("to")
        .ok_or_else(|| cli_error("mirror move requires TO"))?;
    let from = project_join(project, from_value)?;
    let to = project_join(project, to_value)?;
    let mut lb = open_existing(lockbox_path, access)?;
    lb.with_mirror_project_mutation(&project.name, |lb, _| lb.rename(&from, &to))?;
    commit_mirror_change(lb, lockbox_path, access)?;
    println!("Moved {from_value} to {to_value}.");
    print_direct_change_warning(project);
    Ok(())
}

fn print_direct_change_warning(project: &MirrorProject) {
    println!(
        "Direct changes may be reversed by the next update; run `mirror {} status`.",
        project.name
    );
}

fn filter_values(matches: &ArgMatches, name: &str) -> Vec<String> {
    let mut values = matches
        .get_many::<String>(name)
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    normalize_rules(&mut values);
    values
}

fn join_relative_destination(destination: Option<&str>, relative: &str) -> String {
    match destination {
        Some(destination) => format!(
            "{}/{}",
            destination.trim_end_matches('/'),
            relative.trim_start_matches('/')
        ),
        None => relative.to_string(),
    }
}

fn project_join(project: &MirrorProject, relative: &str) -> CliResult<LockboxPath> {
    if relative.split('/').any(|component| component == "..") {
        return Err(cli_error("project paths cannot contain '..'"));
    }
    let relative = relative.trim_start_matches('/');
    if relative.is_empty() {
        return Ok(project.destination.clone());
    }
    join_destination(&project.destination, relative)
}

fn entry_kind_name(kind: &LockboxEntryKind) -> &'static str {
    match kind {
        LockboxEntryKind::File => "file",
        LockboxEntryKind::Directory => "directory",
        LockboxEntryKind::Symlink => "symlink",
    }
}

fn create_project(
    archive_path: &str,
    name: Option<&str>,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    if let Some(misplaced) = matches.get_one::<String>("misplaced-project") {
        let intended = name.unwrap_or(misplaced);
        return Err(cli_error(format!(
            "the mirror project name must appear before 'create'; use:\n  lbx mirror {intended} create --from <HOST_DIRECTORY> --to <LOCKBOX_DIRECTORY>"
        )));
    }
    let name = name.ok_or_else(|| {
        cli_error(
            "mirror create requires a project name before 'create'; use:\n  lbx mirror <NAME> create --from <HOST_DIRECTORY> --to <LOCKBOX_DIRECTORY>",
        )
    })?;
    validate_project_action_name(name)?;
    let source = PathBuf::from(
        matches
            .get_one::<String>("from")
            .ok_or_else(|| cli_error("mirror create requires --from"))?,
    );
    let canonical = canonical_source(&source)?;
    let identity = platform_identity(&canonical)?;
    let destination = lockbox_path(
        matches
            .get_one::<String>("to")
            .ok_or_else(|| cli_error("mirror create requires --to"))?,
    )?;
    let project = MirrorProject {
        name: name.to_string(),
        source: canonical
            .to_str()
            .ok_or_else(|| cli_error("mirror source path is not valid UTF-8"))?
            .to_string(),
        destination,
        includes: Vec::new(),
        excludes: Vec::new(),
        missing_file_policy: MirrorMissingFilePolicy::Remove,
        strict: matches.get_flag("strict"),
        host_identity: identity,
    };
    let mut lb = open_existing(archive_path, access)?;
    lb.create_mirror_project(project.clone(), matches.get_flag("adopt"))?;
    commit_mirror_change(lb, archive_path, access)?;
    println!(
        "Created mirror '{}': {} -> {}.\nNo files were copied. Add them with:\n  lbx {} mirror {} update",
        project.name, project.source, project.destination, archive_path, project.name
    );
    Ok(())
}

fn select_project(
    lockbox_path: &str,
    explicit: Option<&str>,
    access: &Access,
) -> CliResult<MirrorProject> {
    let lb = open_existing(lockbox_path, access)?;
    let projects = lb.list_mirror_projects()?;
    if let Some(name) = explicit {
        return projects
            .into_iter()
            .find(|project| project.name == name)
            .ok_or_else(|| cli_error(format!("mirror project not found: {name}")));
    }
    match projects.as_slice() {
        [] => Err(cli_error("no mirror projects are configured")),
        [project] => Ok(project.clone()),
        _ => Err(cli_error(format!(
            "more than one mirror project is configured; specify one of: {}",
            projects
                .iter()
                .map(|project| project.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn list_projects(lockbox_path: &str, matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let lb = open_existing(lockbox_path, access)?;
    let projects = lb.list_mirror_projects()?;
    let format = selected_format(matches);
    if format == "json" {
        let values = projects.iter().map(project_json).collect::<Vec<_>>();
        println!("{}", serde_json::to_string(&values)?);
    } else {
        let rows = projects
            .into_iter()
            .map(|project| {
                vec![
                    project.name,
                    project.source,
                    project.destination.to_string(),
                    policy_name(project.missing_file_policy).to_string(),
                    yes_no(project.strict).to_string(),
                ]
            })
            .collect();
        print_records(
            &["name", "source", "destination", "missing files", "strict"],
            rows,
            output_format_from_matches(matches)?,
        )?;
    }
    Ok(())
}

fn show_project(project: &MirrorProject, matches: &ArgMatches) -> CliResult<()> {
    let format = selected_format(matches);
    if format == "json" {
        println!("{}", project_json(project));
    } else if format == "tsv" {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            project.name,
            project.source,
            project.destination,
            policy_name(project.missing_file_policy),
            project.strict,
            project.host_identity.as_deref().unwrap_or("")
        );
        for rule in &project.includes {
            println!("include\t{rule}");
        }
        for rule in &project.excludes {
            println!("exclude\t{rule}");
        }
    } else {
        println!("Mirror: {}", project.name);
        println!("  source:        {}", project.source);
        println!("  destination:   {}", project.destination);
        println!(
            "  missing files: {}",
            policy_name(project.missing_file_policy)
        );
        println!("  strict:        {}", yes_no(project.strict));
        println!(
            "  host identity: {}",
            project.host_identity.as_deref().unwrap_or("unavailable")
        );
        print_rules(project);
    }
    Ok(())
}

fn status_project(
    lockbox_path: &str,
    project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let request = mirror_request(lockbox_path, project, matches, false);
    run_mirror(request, access, false)
}

fn update_project(
    lockbox_path: &str,
    project: MirrorProject,
    matches: &ArgMatches,
    access: &Access,
) -> CliResult<()> {
    let request = mirror_request(lockbox_path, project, matches, true);
    run_mirror(request, access, true)
}

fn mirror_request(
    lockbox: &str,
    project: MirrorProject,
    matches: &ArgMatches,
    update: bool,
) -> MirrorRequest {
    MirrorRequest {
        lockbox: lockbox.to_string(),
        destination: project.destination.clone(),
        includes: project.includes.clone(),
        excludes: project.excludes.clone(),
        project,
        options: MirrorUpdateOptions {
            allow_empty: update && matches.get_flag("allow-empty"),
            allow_large_delete: update && matches.get_flag("allow-large-delete"),
        },
        force: update && matches.get_flag("force"),
        format: selected_format(matches).to_string(),
        quiet: matches.get_flag("quiet"),
    }
}

fn run_mirror(request: MirrorRequest, access: &Access, apply: bool) -> CliResult<()> {
    let mut progress = MirrorProgress::new(!request.quiet);
    progress.stage(format!("scanning source '{}'.", request.project.source));
    let canonical = canonical_source(Path::new(&request.project.source))?;
    let identity = platform_identity(&canonical)?;
    if request.project.host_identity.as_deref() != identity.as_deref() {
        return Err(cli_error(format!(
            "host directory identity changed for mirror '{}'; use mirror {} rebind --from <HOST_DIRECTORY>",
            request.project.name, request.project.name
        )));
    }
    let mut lb = open_existing(&request.lockbox, access)?;
    lb.set_workload_profile(WorkloadProfile::BulkImport);
    let ignored_paths = mirror_ignored_host_paths(Path::new(&request.lockbox))?;
    let source_entries = walk_source(
        &canonical,
        &request.includes,
        &request.excludes,
        &ignored_paths,
        &mut progress,
    )?;
    progress.stage(format!(
        "source scan found {} files and {} directories.",
        source_entries.file_count, source_entries.directory_count
    ));
    progress.begin_counted(format!(
        "comparing {} source files with the lockbox.",
        source_entries.file_count
    ));
    let plan = build_plan(&lb, &request, &source_entries, &mut progress, true)?;
    progress.finish("comparison complete.");
    print_plan(&plan, &source_entries, &canonical, &request)?;
    if !apply {
        print_status_warnings(&plan, &source_entries, &request, &lb)?;
        return Ok(());
    }
    if source_entries.files_empty() && plan.removals > 0 && !request.options.allow_empty {
        return Err(cli_error(
            "no source files match the project rules; pass --allow-empty after inspecting mirror status",
        ));
    }
    validate_large_delete(
        &plan,
        request.options,
        destination_file_count(&lb, &request.destination)?,
    )?;
    if plan.additions == 0 && plan.replacements == 0 && plan.removals == 0 && plan.directories == 0
    {
        println!("Mirror '{}' is up to date.", request.project.name);
        return Ok(());
    }
    if !request.force && !confirm("Apply this mirror update? Type y or yes to confirm: ")? {
        println!("Mirror update cancelled.");
        return Ok(());
    }
    let project_name = request.project.name.clone();
    let change_count = plan.removals + plan.directories + plan.additions + plan.replacements;
    progress.begin_counted(format!("applying {change_count} lockbox changes."));
    let mut completed = 0;
    lb.with_mirror_project_mutation(&project_name, |lb, _| {
        for record in source_entries.plan.iter().map_err(mirror_inventory_error)? {
            let (operation, destination, _) =
                decode_plan_record(record.map_err(mirror_inventory_error)?)
                    .map_err(|error| Error::InvalidOperation(error.to_string()))?;
            if operation != PlanOperation::Remove {
                continue;
            }
            if lb.is_dir(&destination) {
                lb.remove_dir_recursive(&destination)?;
            } else {
                lb.delete(&destination)?;
            }
            completed += 1;
            progress.counted("applied", completed, change_count);
        }
        for record in source_entries.plan.iter().map_err(mirror_inventory_error)? {
            let (operation, destination, _) =
                decode_plan_record(record.map_err(mirror_inventory_error)?)
                    .map_err(|error| Error::InvalidOperation(error.to_string()))?;
            if operation != PlanOperation::MakeDirectory {
                continue;
            }
            if !lb.is_dir(&destination) {
                lb.create_dir(&destination, true)?;
            }
            completed += 1;
            progress.counted("applied", completed, change_count);
        }
        for (operation, replace) in [(PlanOperation::Add, false), (PlanOperation::Replace, true)] {
            for record in source_entries.plan.iter().map_err(mirror_inventory_error)? {
                let (record_operation, destination, value) =
                    decode_plan_record(record.map_err(mirror_inventory_error)?)
                        .map_err(|error| Error::InvalidOperation(error.to_string()))?;
                if record_operation != operation {
                    continue;
                }
                let SourceEntryValue::File {
                    host_path,
                    fingerprint,
                } = decode_source_value(&value)
                    .map_err(|error| Error::InvalidOperation(error.to_string()))?
                else {
                    return Err(Error::InvalidOperation(
                        "mirror file plan has no source file".to_string(),
                    ));
                };
                lb.create_parent_dirs_for(&destination)?;
                lb.add_file_from_path_verified(
                    &host_path,
                    &destination,
                    replace,
                    &parse_sha256(&fingerprint.digest)?,
                )?;
                completed += 1;
                progress.counted("applied", completed, change_count);
            }
        }
        Ok(())
    })?;
    let mut applied_progress = MirrorProgress::new(false);
    let applied_plan = build_plan(&lb, &request, &source_entries, &mut applied_progress, false)?;
    if applied_plan.additions > 0
        || applied_plan.replacements > 0
        || applied_plan.removals > 0
        || applied_plan.directories > 0
    {
        return Err(cli_error(format!(
            "mirror '{}' could not apply the planned contents before commit",
            request.project.name
        )));
    }
    progress.stage("checking the source for changes before commit.");
    verify_source_tree(
        &canonical,
        &request.includes,
        &request.excludes,
        &ignored_paths,
        &source_entries,
        request.project.strict,
    )?;
    progress.stage("committing the encrypted update.");
    commit_mirror_change(lb, &request.lockbox, access)?;
    progress.begin_counted("verifying the committed mirror contents.");
    let persisted = open_existing(&request.lockbox, access)?;
    let persisted_plan = build_plan(&persisted, &request, &source_entries, &mut progress, false)?;
    if persisted_plan.additions > 0
        || persisted_plan.replacements > 0
        || persisted_plan.removals > 0
        || persisted_plan.directories > 0
    {
        return Err(cli_error(format!(
            "mirror '{}' update verification failed: the committed lockbox contents do not match the source",
            request.project.name
        )));
    }
    progress.finish("update complete.");
    if request.format != "json" {
        println!(
            "Updated mirror '{}': {} added, {} replaced, {} directories created, {} removed, {} unchanged.",
            request.project.name,
            plan.additions,
            plan.replacements,
            plan.directories,
            plan.removed_files,
            plan.unchanged
        );
    }
    Ok(())
}

fn commit_mirror_change(
    mut lockbox: Lockbox,
    lockbox_path: &str,
    access: &Access,
) -> CliResult<()> {
    match lockbox.commit() {
        Ok(()) => Ok(()),
        Err(Error::RecoveryRequired { .. }) => {
            drop(lockbox);
            // The logical transaction is already published. A fresh writable
            // open authenticates it and completes its secure cleanup before
            // the mirror command reports success.
            drop(open_existing(lockbox_path, access)?);
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn canonical_source(source: &Path) -> CliResult<PathBuf> {
    let canonical = fs::canonicalize(source).map_err(|err| {
        cli_error(format!(
            "cannot access source directory {}: {err}",
            source.display()
        ))
    })?;
    if !canonical.is_dir() {
        return Err(cli_error(format!(
            "mirror source is not a directory: {}",
            source.display()
        )));
    }
    if canonical.parent().is_none() {
        return Err(cli_error("refusing to mirror a filesystem root"));
    }
    Ok(canonical)
}

fn mirror_ignored_host_paths(lockbox: &Path) -> CliResult<BTreeSet<PathBuf>> {
    let absolute = if lockbox.is_absolute() {
        lockbox.to_path_buf()
    } else {
        std::env::current_dir()?.join(lockbox)
    };
    let canonical = fs::canonicalize(lockbox).map_err(|err| {
        cli_error(format!(
            "cannot resolve lockbox path {}: {err}",
            lockbox.display()
        ))
    })?;
    Ok([
        absolute.clone(),
        lock_path_for(&absolute),
        canonical.clone(),
        lock_path_for(&canonical),
    ]
    .into_iter()
    .collect())
}

fn walk_source(
    root: &Path,
    includes: &[String],
    excludes: &[String],
    ignored_paths: &BTreeSet<PathBuf>,
    progress: &mut MirrorProgress,
) -> CliResult<SourceSnapshot> {
    struct WalkOptions<'a> {
        root: &'a Path,
        includes: &'a [String],
        excludes: &'a [String],
        ignored_paths: &'a BTreeSet<PathBuf>,
    }

    fn visit(
        current: &Path,
        options: &WalkOptions<'_>,
        progress: &mut MirrorProgress,
        database: &BinaryTable,
        file_count: &mut usize,
        directory_count: &mut usize,
    ) -> CliResult<()> {
        let entries = fs::read_dir(current).map_err(|err| {
            cli_error(format!(
                "cannot completely read {}: {err}",
                current.display()
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|err| {
                cli_error(format!(
                    "cannot completely read {}: {err}",
                    current.display()
                ))
            })?;
            let path = entry.path();
            if options.ignored_paths.contains(&path) {
                continue;
            }
            let relative = slash_path(
                path.strip_prefix(options.root)
                    .map_err(|err| cli_error(err.to_string()))?,
            )?;
            if excluded(&relative, options.excludes) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(cli_error(format!(
                    "symbolic links are not supported by mirror projects: {}",
                    path.display()
                )));
            }
            if kind.is_dir() {
                if options.includes.is_empty() {
                    insert_source_directory(database, &relative, directory_count)?;
                }
                visit(
                    &path,
                    options,
                    progress,
                    database,
                    file_count,
                    directory_count,
                )?;
            } else if kind.is_file() {
                if included(&relative, options.includes) {
                    for directory in Path::new(&relative).ancestors().skip(1) {
                        let directory = slash_path(directory)?;
                        if directory.is_empty() {
                            break;
                        }
                        insert_source_directory(database, &directory, directory_count)?;
                    }
                    let fingerprint = fingerprint_with_progress(&path, progress)?;
                    progress.scanned_file();
                    path.to_str().ok_or_else(|| {
                        cli_error(format!(
                            "source path is not valid UTF-8: {}",
                            path.display()
                        ))
                    })?;
                    let encoded = encode_source_file(&path, &fingerprint)?;
                    if !database.insert_if_absent(relative.as_bytes(), &encoded)? {
                        return Err(cli_error(format!(
                            "duplicate source path while scanning: {relative}"
                        )));
                    }
                    *file_count += 1;
                }
            } else {
                return Err(cli_error(format!(
                    "unsupported source entry type: {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }
    let temporary_directory = tempfile::Builder::new()
        .prefix("revault-mirror-")
        .tempdir()?;
    let database = BinaryTable::create(temporary_directory.path(), "source")?;
    let options = WalkOptions {
        root,
        includes,
        excludes,
        ignored_paths,
    };
    let mut file_count = 0;
    let mut directory_count = 0;
    visit(
        root,
        &options,
        progress,
        &database,
        &mut file_count,
        &mut directory_count,
    )?;
    let plan = BinaryTable::create(temporary_directory.path(), "plan")?;
    let verification = BinaryTable::create(temporary_directory.path(), "verification")?;
    Ok(SourceSnapshot {
        _temporary_directory: temporary_directory,
        source: database,
        plan,
        verification,
        file_count,
        directory_count,
    })
}

fn insert_source_directory(
    database: &BinaryTable,
    relative: &str,
    directory_count: &mut usize,
) -> CliResult<()> {
    if database.insert_if_absent(relative.as_bytes(), &[0])? {
        *directory_count += 1;
    }
    Ok(())
}

fn build_plan(
    lb: &Lockbox,
    request: &MirrorRequest,
    source: &SourceSnapshot,
    progress: &mut MirrorProgress,
    store: bool,
) -> CliResult<MirrorPlan> {
    if store {
        source.plan.clear()?;
    }
    let mut plan = MirrorPlan {
        additions: 0,
        replacements: 0,
        removals: 0,
        unchanged: 0,
        directories: 0,
        removed_files: 0,
    };
    if request.destination.as_str() != "/" && lb.stat(&request.destination).is_none() {
        plan.directories += 1;
        store_plan_path(
            source,
            store,
            PlanOperation::MakeDirectory,
            &request.destination,
        )?;
    }
    let source_file_count = source.file_count;
    let mut compared = 0;
    for record in source.source.iter()? {
        let (relative, value) = decode_source_record(record?)?;
        let destination = join_destination(&request.destination, &relative)?;
        match value {
            SourceEntryValue::Directory => match lb.stat(&destination) {
                Some(entry) if entry.kind != LockboxEntryKind::Directory => {
                    plan.removed_files += 1;
                    plan.removals += 1;
                    store_plan_path(source, store, PlanOperation::Remove, &entry.path)?;
                    plan.directories += 1;
                    store_plan_path(source, store, PlanOperation::MakeDirectory, &destination)?;
                }
                Some(_) => {}
                None => {
                    plan.directories += 1;
                    store_plan_path(source, store, PlanOperation::MakeDirectory, &destination)?;
                }
            },
            SourceEntryValue::File {
                host_path,
                fingerprint: host_fingerprint,
            } => {
                match lb.stat(&destination) {
                    None => {
                        plan.additions += 1;
                        store_plan_file(
                            source,
                            store,
                            PlanOperation::Add,
                            &host_path,
                            &destination,
                            &host_fingerprint,
                        )?;
                    }
                    Some(entry) if entry.kind == LockboxEntryKind::Directory => {
                        let directory = entry.path.clone();
                        plan.removed_files += destination_file_count(lb, &directory)?;
                        plan.removals += 1;
                        store_plan_path(source, store, PlanOperation::Remove, &directory)?;
                        plan.additions += 1;
                        store_plan_file(
                            source,
                            store,
                            PlanOperation::Add,
                            &host_path,
                            &destination,
                            &host_fingerprint,
                        )?;
                    }
                    Some(entry) if entry.kind != LockboxEntryKind::File => {
                        plan.removed_files += 1;
                        plan.removals += 1;
                        store_plan_path(source, store, PlanOperation::Remove, &entry.path)?;
                        plan.additions += 1;
                        store_plan_file(
                            source,
                            store,
                            PlanOperation::Add,
                            &host_path,
                            &destination,
                            &host_fingerprint,
                        )?;
                    }
                    Some(entry) => {
                        let archive_digest = archive_digest(lb, &entry.path)?;
                        if entry.len == host_fingerprint.len
                            && archive_digest == host_fingerprint.digest
                        {
                            plan.unchanged += 1;
                        } else {
                            plan.replacements += 1;
                            store_plan_file(
                                source,
                                store,
                                PlanOperation::Replace,
                                &host_path,
                                &destination,
                                &host_fingerprint,
                            )?;
                        }
                    }
                }
                compared += 1;
                progress.counted("compared", compared, source_file_count);
            }
        }
    }
    if request.project.missing_file_policy == MirrorMissingFilePolicy::Remove {
        let mut options = ListOptions::new(&request.destination);
        options.recursive = true;
        for entry in lb.list(options)? {
            let entry = entry?;
            let relative = relative_lockbox_path(&request.destination, &entry.path);
            if relative.is_empty() || source_entry_exists(source, &relative)? {
                continue;
            }
            if plan_removal_contains(source, store, &entry.path)? {
                continue;
            }
            plan.removals += 1;
            plan.removed_files += if entry.kind == LockboxEntryKind::Directory {
                destination_file_count(lb, &entry.path)?
            } else {
                1
            };
            store_plan_path(source, store, PlanOperation::Remove, &entry.path)?;
        }
    }
    Ok(plan)
}

fn source_entry_exists(source: &SourceSnapshot, relative: &str) -> CliResult<bool> {
    Ok(source.source.contains(relative.as_bytes())?)
}

fn plan_removal_contains(
    source: &SourceSnapshot,
    store: bool,
    path: &LockboxPath,
) -> CliResult<bool> {
    if !store {
        // Check-only plans do not need exact operation roots; any difference
        // is sufficient for verification.
        return Ok(false);
    }
    let mut candidate = path.as_str();
    loop {
        if source
            .plan
            .contains(&plan_key(PlanOperation::Remove, candidate))?
        {
            return Ok(true);
        }
        let Some(index) = candidate.rfind('/') else {
            return Ok(false);
        };
        if index == 0 {
            return Ok(false);
        }
        candidate = &candidate[..index];
    }
}

fn store_plan_path(
    source: &SourceSnapshot,
    store: bool,
    operation: PlanOperation,
    destination: &LockboxPath,
) -> CliResult<()> {
    if store {
        source
            .plan
            .insert_if_absent(&plan_key(operation, destination.as_str()), &[])?;
    }
    Ok(())
}

fn store_plan_file(
    source: &SourceSnapshot,
    store: bool,
    operation: PlanOperation,
    host_path: &Path,
    destination: &LockboxPath,
    fingerprint: &FileFingerprint,
) -> CliResult<()> {
    if store {
        let value = encode_source_file(host_path, fingerprint)?;
        if !source
            .plan
            .insert_if_absent(&plan_key(operation, destination.as_str()), &value)?
        {
            return Err(cli_error(format!(
                "duplicate mirror plan entry: {destination}"
            )));
        }
    }
    Ok(())
}

fn encode_source_file(path: &Path, fingerprint: &FileFingerprint) -> CliResult<Vec<u8>> {
    let path = path
        .to_str()
        .ok_or_else(|| cli_error("source path is not valid UTF-8"))?;
    let path_bytes = path.as_bytes();
    let path_len = u32::try_from(path_bytes.len())
        .map_err(|_| cli_error("source path is too long for the mirror index"))?;
    let mut value = Vec::with_capacity(1 + 8 + 1 + 16 + 32 + 4 + path_bytes.len());
    value.push(1);
    value.extend_from_slice(&fingerprint.len.to_le_bytes());
    match fingerprint.modified_nanos {
        Some(modified) => {
            value.push(1);
            value.extend_from_slice(&modified.to_le_bytes());
        }
        None => {
            value.push(0);
            value.extend_from_slice(&0_u128.to_le_bytes());
        }
    }
    value.extend_from_slice(&parse_sha256(&fingerprint.digest)?);
    value.extend_from_slice(&path_len.to_le_bytes());
    value.extend_from_slice(path_bytes);
    Ok(value)
}

fn decode_source_record(record: (Vec<u8>, Vec<u8>)) -> CliResult<(String, SourceEntryValue)> {
    Ok((
        utf8_string(record.0, "source path")?,
        decode_source_value(&record.1)?,
    ))
}

fn decode_source_value(value: &[u8]) -> CliResult<SourceEntryValue> {
    let Some(kind) = value.first().copied() else {
        return Err(cli_error("temporary mirror index contains an empty record"));
    };
    if kind == 0 {
        if value.len() != 1 {
            return Err(cli_error(
                "temporary mirror index contains an invalid directory record",
            ));
        }
        return Ok(SourceEntryValue::Directory);
    }
    if kind != 1 {
        return Err(cli_error(
            "temporary mirror index contains an unknown record type",
        ));
    }
    let mut cursor = io::Cursor::new(&value[1..]);
    let len = read_index_u64(&mut cursor)?;
    let mut present = [0_u8; 1];
    cursor.read_exact(&mut present)?;
    let mut modified_bytes = [0_u8; 16];
    cursor.read_exact(&mut modified_bytes)?;
    let modified_nanos = match present[0] {
        0 => None,
        1 => Some(u128::from_le_bytes(modified_bytes)),
        _ => {
            return Err(cli_error(
                "temporary mirror index has invalid file metadata",
            ))
        }
    };
    let mut digest = [0_u8; 32];
    cursor.read_exact(&mut digest)?;
    let path_len = read_index_u32(&mut cursor)? as usize;
    let mut path = vec![0_u8; path_len];
    cursor.read_exact(&mut path)?;
    if cursor.position() != (value.len() - 1) as u64 {
        return Err(cli_error(
            "temporary mirror index file record has trailing data",
        ));
    }
    Ok(SourceEntryValue::File {
        host_path: PathBuf::from(utf8_string(path, "source path")?),
        fingerprint: FileFingerprint {
            len,
            modified_nanos,
            digest: hex_digest(&digest),
        },
    })
}

fn plan_key(operation: PlanOperation, destination: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + destination.len());
    key.push(operation as u8);
    key.extend_from_slice(destination.as_bytes());
    key
}

fn decode_plan_record(
    record: (Vec<u8>, Vec<u8>),
) -> CliResult<(PlanOperation, LockboxPath, Vec<u8>)> {
    let (key, value) = record;
    let Some((&operation, destination)) = key.split_first() else {
        return Err(cli_error("temporary mirror plan contains an empty key"));
    };
    let operation = match operation {
        1 => PlanOperation::Add,
        2 => PlanOperation::Replace,
        3 => PlanOperation::Remove,
        4 => PlanOperation::MakeDirectory,
        _ => return Err(cli_error("temporary mirror plan has an unknown operation")),
    };
    let destination = LockboxPath::new(utf8_string(destination.to_vec(), "lockbox path")?)?;
    Ok((operation, destination, value))
}

fn utf8_string(value: Vec<u8>, label: &str) -> CliResult<String> {
    String::from_utf8(value).map_err(|_| {
        cli_error(format!(
            "temporary mirror index has invalid UTF-8 in {label}"
        ))
    })
}

fn read_index_u32(reader: &mut impl Read) -> CliResult<u32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_index_u64(reader: &mut impl Read) -> CliResult<u64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn fingerprint(path: &Path) -> CliResult<FileFingerprint> {
    fingerprint_inner(path, |_| {})
}

fn fingerprint_with_progress(
    path: &Path,
    progress: &mut MirrorProgress,
) -> CliResult<FileFingerprint> {
    fingerprint_inner(path, |bytes| progress.scanned(bytes))
}

fn fingerprint_inner(path: &Path, mut on_read: impl FnMut(usize)) -> CliResult<FileFingerprint> {
    let before = fs::metadata(path)?;
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 128 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        on_read(count);
    }
    let after = fs::metadata(path)?;
    let result = FileFingerprint {
        len: after.len(),
        modified_nanos: modified_nanos(&after),
        digest: hex_digest(hasher.finalize().as_slice()),
    };
    if before.len() != after.len() || modified_nanos(&before) != result.modified_nanos {
        return Err(cli_error(format!(
            "source file changed while being read: {}",
            path.display()
        )));
    }
    Ok(result)
}

fn archive_digest(lb: &Lockbox, path: &LockboxPath) -> CliResult<String> {
    struct HashWriter(Sha256);
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(Sha256::new());
    lb.extract_file_to_writer(path, &mut writer)?;
    Ok(hex_digest(writer.0.finalize().as_slice()))
}

fn verify_source_tree(
    root: &Path,
    includes: &[String],
    excludes: &[String],
    ignored_paths: &BTreeSet<PathBuf>,
    source: &SourceSnapshot,
    strict: bool,
) -> CliResult<()> {
    fn mark_seen(source: &SourceSnapshot, relative: &str) -> CliResult<()> {
        if !source.source.contains(relative.as_bytes())? {
            return Err(cli_error(format!(
                "source changed while updating the mirror; new entry found: {relative}"
            )));
        }
        source
            .verification
            .insert_if_absent(relative.as_bytes(), &[])?;
        Ok(())
    }

    fn visit(
        current: &Path,
        root: &Path,
        includes: &[String],
        excludes: &[String],
        ignored_paths: &BTreeSet<PathBuf>,
        source: &SourceSnapshot,
        strict: bool,
    ) -> CliResult<()> {
        let entries = fs::read_dir(current).map_err(|err| {
            cli_error(format!(
                "source changed while updating the mirror; cannot read {}: {err}",
                current.display()
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|err| {
                cli_error(format!(
                    "source changed while updating the mirror; cannot read {}: {err}",
                    current.display()
                ))
            })?;
            let path = entry.path();
            if ignored_paths.contains(&path) {
                continue;
            }
            let relative = slash_path(
                path.strip_prefix(root)
                    .map_err(|err| cli_error(err.to_string()))?,
            )?;
            if excluded(&relative, excludes) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err(cli_error(format!(
                    "source changed while updating the mirror; symbolic link found: {}",
                    path.display()
                )));
            }
            if kind.is_dir() {
                if includes.is_empty() {
                    mark_seen(source, &relative)?;
                }
                visit(
                    &path,
                    root,
                    includes,
                    excludes,
                    ignored_paths,
                    source,
                    strict,
                )?;
            } else if kind.is_file() && included(&relative, includes) {
                for directory in Path::new(&relative).ancestors().skip(1) {
                    let directory = slash_path(directory)?;
                    if directory.is_empty() {
                        break;
                    }
                    mark_seen(source, &directory)?;
                }
                let Some(expected) = source.source.get(relative.as_bytes())? else {
                    return Err(cli_error(format!(
                        "source changed while updating the mirror; new file found: {}",
                        path.display()
                    )));
                };
                let SourceEntryValue::File {
                    fingerprint: expected,
                    ..
                } = decode_source_value(&expected)?
                else {
                    return Err(cli_error(format!(
                        "source changed while updating the mirror; file replaced another entry: {}",
                        path.display()
                    )));
                };
                mark_seen(source, &relative)?;
                let metadata = entry.metadata()?;
                let metadata_matches = metadata.len() == expected.len
                    && modified_nanos(&metadata) == expected.modified_nanos;
                if (strict || !metadata_matches) && fingerprint(&path)?.digest != expected.digest {
                    return Err(cli_error(format!(
                        "source file changed while updating the mirror: {}",
                        path.display()
                    )));
                }
            } else if !kind.is_file() {
                return Err(cli_error(format!(
                    "source changed while updating the mirror; unsupported entry found: {}",
                    path.display()
                )));
            }
        }
        Ok(())
    }

    source.verification.clear()?;
    visit(
        root,
        root,
        includes,
        excludes,
        ignored_paths,
        source,
        strict,
    )?;
    for record in source.source.iter()? {
        let (key, _) = record?;
        if !source.verification.contains(&key)? {
            return Err(cli_error(format!(
                "source changed while updating the mirror; entry was removed: {}",
                utf8_string(key, "source path")?
            )));
        }
    }
    Ok(())
}

fn validate_large_delete(
    plan: &MirrorPlan,
    options: MirrorUpdateOptions,
    destination_files: usize,
) -> CliResult<()> {
    if !options.allow_large_delete
        && destination_files > 0
        && plan.removed_files * 100 > destination_files * LARGE_DELETE_PERCENT
    {
        return Err(cli_error(format!(
            "mirror update would delete {} of {} managed files; pass --allow-large-delete",
            plan.removed_files, destination_files
        )));
    }
    Ok(())
}

fn destination_file_count(lb: &Lockbox, destination: &LockboxPath) -> CliResult<usize> {
    let mut options = ListOptions::new(destination);
    options.recursive = true;
    let mut count = 0;
    for entry in lb.list(options)? {
        if entry?.kind != LockboxEntryKind::Directory {
            count += 1;
        }
    }
    Ok(count)
}

fn print_plan(
    plan: &MirrorPlan,
    inventory: &SourceSnapshot,
    source: &Path,
    request: &MirrorRequest,
) -> CliResult<()> {
    if request.format == "json" {
        let mut stdout = io::stdout().lock();
        write!(
            stdout,
            "{{\"source\":{},\"destination\":{}",
            serde_json::to_string(&source.display().to_string())?,
            serde_json::to_string(&request.destination.to_string())?,
        )?;
        for (field, operation) in [
            ("add", PlanOperation::Add),
            ("replace", PlanOperation::Replace),
            ("create_directory", PlanOperation::MakeDirectory),
            ("remove", PlanOperation::Remove),
        ] {
            write!(stdout, ",\"{field}\":[")?;
            let mut first = true;
            for_each_plan_path(inventory, operation, |path| {
                if !first {
                    write!(stdout, ",")?;
                }
                first = false;
                write!(stdout, "{}", serde_json::to_string(&path)?)?;
                Ok(())
            })?;
            write!(stdout, "]")?;
        }
        writeln!(
            stdout,
            ",\"remove_file_count\":{},\"unchanged\":{}}}",
            plan.removed_files, plan.unchanged
        )?;
        return Ok(());
    }
    if request.format == "tsv" {
        for (label, operation) in [
            ("add", PlanOperation::Add),
            ("replace", PlanOperation::Replace),
            ("create-directory", PlanOperation::MakeDirectory),
            ("remove", PlanOperation::Remove),
        ] {
            for_each_plan_path(inventory, operation, |path| {
                println!("{label}\t{path}");
                Ok(())
            })?;
        }
        println!("unchanged\t{}", plan.unchanged);
        return Ok(());
    }
    println!("Mirror status for '{}'", request.project.name);
    println!();
    println!("  source:    {}", source.display());
    println!("  add:       {} files", plan.additions);
    println!("  replace:   {} files", plan.replacements);
    println!("  mkdir:     {} directories", plan.directories);
    println!("  remove:    {} files", plan.removed_files);
    println!("  unchanged: {} files", plan.unchanged);
    if plan.removals > 0 {
        println!("\nEntries to remove:");
        for_each_plan_path(inventory, PlanOperation::Remove, |path| {
            println!("  {path}");
            Ok(())
        })?;
    }
    Ok(())
}

fn for_each_plan_path(
    inventory: &SourceSnapshot,
    operation: PlanOperation,
    mut action: impl FnMut(String) -> CliResult<()>,
) -> CliResult<()> {
    for record in inventory.plan.iter()? {
        let (record_operation, destination, _) = decode_plan_record(record?)?;
        if record_operation == operation {
            action(destination.to_string())?;
        }
    }
    Ok(())
}

fn confirm(prompt: &str) -> CliResult<bool> {
    if !io::stdin().is_terminal() {
        return Err(cli_error(
            "confirmation requires a terminal; inspect with `mirror status`, then pass --force",
        ));
    }
    print!("{prompt}");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn lockbox_path(value: &str) -> CliResult<LockboxPath> {
    LockboxPath::new(if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    })
    .map_err(Into::into)
}

fn join_destination(root: &LockboxPath, relative: &str) -> CliResult<LockboxPath> {
    lockbox_path(&format!(
        "{}/{}",
        root.as_str().trim_end_matches('/'),
        relative
    ))
}

fn relative_lockbox_path(root: &LockboxPath, path: &LockboxPath) -> String {
    if root.as_str() == "/" {
        path.as_str().trim_start_matches('/').to_string()
    } else {
        path.as_str()
            .strip_prefix(root.as_str())
            .unwrap_or(path.as_str())
            .trim_start_matches('/')
            .to_string()
    }
}

fn slash_path(path: &Path) -> CliResult<String> {
    let components = path
        .components()
        .map(|part| {
            part.as_os_str().to_str().ok_or_else(|| {
                cli_error(format!(
                    "source path is not valid UTF-8: {}",
                    path.display()
                ))
            })
        })
        .collect::<CliResult<Vec<_>>>()?;
    Ok(components.join("/"))
}

fn removal_roots(paths: &[LockboxPath], lb: &Lockbox) -> Vec<LockboxPath> {
    let mut paths = paths.to_vec();
    paths.sort_by_key(|path| path.as_str().len());
    let mut roots: Vec<LockboxPath> = Vec::new();
    for path in paths {
        if roots
            .iter()
            .any(|root| lb.is_dir(root) && path.is_descendant_of(root))
        {
            continue;
        }
        roots.push(path);
    }
    roots
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn parse_sha256(value: &str) -> Result<[u8; 32], Error> {
    if value.len() != 64 {
        return Err(Error::InvalidOperation(
            "invalid SHA-256 checksum in mirror inventory".to_string(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let encoded = std::str::from_utf8(pair)
            .map_err(|_| Error::InvalidOperation("invalid mirror checksum".to_string()))?;
        digest[index] = u8::from_str_radix(encoded, 16)
            .map_err(|_| Error::InvalidOperation("invalid mirror checksum".to_string()))?;
    }
    Ok(digest)
}

fn mirror_inventory_error(error: io::Error) -> Error {
    Error::InvalidOperation(format!("cannot read temporary mirror inventory: {error}"))
}

#[cfg(unix)]
fn platform_identity(path: &Path) -> CliResult<Option<String>> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(path)?;
    Ok(Some(format!("unix:{}:{}", metadata.dev(), metadata.ino())))
}

#[cfg(not(unix))]
fn platform_identity(_path: &Path) -> CliResult<Option<String>> {
    Ok(None)
}

fn modified_nanos(metadata: &fs::Metadata) -> Option<u128> {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_inventory_is_disk_backed_and_tree_changes_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("nested")).unwrap();
        fs::write(root.path().join("nested/file.txt"), b"planned bytes").unwrap();
        let mut progress = MirrorProgress::new(false);
        let ignored = BTreeSet::new();
        let inventory = walk_source(root.path(), &[], &[], &ignored, &mut progress).unwrap();

        assert!(inventory
            ._temporary_directory
            .path()
            .join("source.records")
            .is_file());
        let SourceEntryValue::File { fingerprint, .. } =
            decode_source_value(&inventory.source.get(b"nested/file.txt").unwrap().unwrap())
                .unwrap()
        else {
            panic!("file record was stored as a directory")
        };
        assert_eq!(
            fingerprint.digest,
            hex_digest(Sha256::digest(b"planned bytes").as_slice())
        );

        fs::write(root.path().join("new.txt"), b"new").unwrap();
        let error = verify_source_tree(root.path(), &[], &[], &ignored, &inventory, false)
            .unwrap_err()
            .to_string();
        assert!(error.contains("new file found"), "{error}");
    }
}
