use clap::ArgMatches;
use revault_lockbox_api::{
    ExtractPolicy, ListOptions, Lockbox, LockboxEntry, LockboxEntryKind, LockboxPath,
    MirrorMissingFilePolicy, MirrorProject, WorkloadProfile,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use super::context::{cli_error, open_existing, Access, CliResult};
use super::default_lockbox_for_add;
use super::filters::{excluded, included, normalize as normalize_rules};
use super::output::{output_format_from_matches, print_records};

const LARGE_DELETE_PERCENT: usize = 50;

/// A complete, inspectable one-way mirror update plan.
#[derive(Debug)]
pub(crate) struct MirrorPlan {
    pub additions: Vec<MirrorAddition>,
    pub replacements: Vec<MirrorReplacement>,
    pub removals: Vec<LockboxPath>,
    pub unchanged: usize,
    directories: Vec<LockboxPath>,
    removed_files: usize,
}

#[derive(Debug)]
pub(crate) struct MirrorAddition {
    source: PathBuf,
    destination: LockboxPath,
    fingerprint: FileFingerprint,
}

#[derive(Debug)]
pub(crate) struct MirrorReplacement {
    source: PathBuf,
    destination: LockboxPath,
    fingerprint: FileFingerprint,
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
struct MirrorRequest {
    lockbox: String,
    project: MirrorProject,
    destination: LockboxPath,
    includes: Vec<String>,
    excludes: Vec<String>,
    options: MirrorUpdateOptions,
    force: bool,
    format: String,
}

#[derive(Debug)]
struct SourceSnapshot {
    files: BTreeMap<String, (PathBuf, FileFingerprint)>,
    directories: BTreeSet<String>,
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
    project.missing_file_policy = match matches
        .get_one::<String>("missing-files")
        .map(String::as_str)
    {
        Some("remove") => MirrorMissingFilePolicy::Remove,
        Some("retain") => MirrorMissingFilePolicy::Retain,
        _ => return Err(cli_error("missing-files must be remove or retain")),
    };
    let mut lb = open_existing(lockbox_path, access)?;
    lb.update_mirror_project(&project)?;
    lb.commit()?;
    println!(
        "Mirror '{}' now {} archive-only files.",
        project.name,
        if project.missing_file_policy == MirrorMissingFilePolicy::Remove {
            "removes"
        } else {
            "retains"
        }
    );
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
    lb.commit()?;
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
    lb.commit()?;
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
    lb.commit()?;
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
    lb.commit()?;
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
        "host_identity": project.host_identity,
    })
}

fn policy_name(policy: MirrorMissingFilePolicy) -> &'static str {
    match policy {
        MirrorMissingFilePolicy::Remove => "remove",
        MirrorMissingFilePolicy::Retain => "retain",
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
    if source.files.is_empty() && !plan.removals.is_empty() {
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
            let snapshot = walk_source(source, &includes, &excludes)?;
            for directory in snapshot.directories {
                directories.insert(project_join(
                    project,
                    &join_relative_destination(destination_value, &directory),
                )?);
            }
            for (relative, (path, _)) in snapshot.files {
                additions.push((
                    path,
                    project_join(
                        project,
                        &join_relative_destination(destination_value, &relative),
                    )?,
                ));
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
    lb.commit()?;
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
    if let Some(destination) = matches.get_one::<String>("to") {
        let policy = ExtractPolicy {
            overwrite: matches.get_flag("overwrite"),
            restore_permissions: matches.get_flag("restore-permissions"),
            restore_symlinks: matches.get_flag("restore-symlinks"),
            ..ExtractPolicy::default()
        };
        extract_project_tree(&lb, project, Path::new(destination), &policy)?;
        println!("Extracted mirror '{}' to {}.", project.name, destination);
        return Ok(());
    }
    let args = matches
        .get_many::<String>("args")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(cli_error(
            "mirror extract requires PATH DESTINATION or --to DESTINATION",
        ));
    }
    let source = project_join(project, args[0])?;
    lb.extract_file_to(&source, Path::new(args[1]), matches.get_flag("overwrite"))?;
    println!("Extracted {} to {}.", args[0], args[1]);
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
                        "{} is {} bytes, exceeding the per-file extraction limit of {}",
                        entry.path, entry.len, policy.max_file_bytes
                    )));
                }
                total_bytes = total_bytes
                    .checked_add(entry.len)
                    .ok_or_else(|| cli_error("total extracted size overflow"))?;
                if total_bytes > policy.max_total_bytes {
                    return Err(cli_error(format!(
                        "project extraction exceeds the total byte limit of {}",
                        policy.max_total_bytes
                    )));
                }
                if let Some(parent) = host.parent() {
                    fs::create_dir_all(parent)?;
                }
                lb.extract_file_to(&entry.path, &host, policy.overwrite)?;
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
    if !options.recursive {
        return print_records(
            &["kind", "len", "name"],
            super::files::direct_listing_rows(&lb, &base)?,
            output_format_from_matches(matches)?,
        );
    }
    let mut rows = Vec::new();
    for entry in lb.list(options)? {
        let entry = entry?;
        rows.push(vec![
            entry_kind_name(&entry.kind).to_string(),
            entry.len.to_string(),
            relative_lockbox_path(&project.destination, &entry.path),
        ]);
    }
    print_records(
        &["kind", "len", "path"],
        rows,
        output_format_from_matches(matches)?,
    )?;
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
    lb.commit()?;
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
    lb.commit()?;
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
    let name = name.ok_or_else(|| cli_error("mirror create requires an explicit project name"))?;
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
        host_identity: identity,
    };
    let mut lb = open_existing(archive_path, access)?;
    lb.create_mirror_project(project.clone(), matches.get_flag("adopt"))?;
    lb.commit()?;
    println!(
        "Created mirror '{}': {} -> {}.\nNo files were copied. Run `lbx {} mirror {} status` to inspect the first update.",
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
                ]
            })
            .collect();
        print_records(
            &["name", "source", "destination", "missing files"],
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
            "{}\t{}\t{}\t{}\t{}",
            project.name,
            project.source,
            project.destination,
            policy_name(project.missing_file_policy),
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
    }
}

fn run_mirror(request: MirrorRequest, access: &Access, apply: bool) -> CliResult<()> {
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
    let source_entries = walk_source(&canonical, &request.includes, &request.excludes)?;
    let plan = build_plan(&lb, &request, &source_entries)?;
    print_plan(&plan, &canonical, &request)?;
    if !apply {
        print_status_warnings(&plan, &source_entries, &request, &lb)?;
        return Ok(());
    }
    if source_entries.files.is_empty() && !plan.removals.is_empty() && !request.options.allow_empty
    {
        return Err(cli_error(
            "no source files match the project rules; pass --allow-empty after inspecting mirror status",
        ));
    }
    validate_large_delete(
        &plan,
        request.options,
        destination_file_count(&lb, &request.destination)?,
    )?;
    if plan.additions.is_empty()
        && plan.replacements.is_empty()
        && plan.removals.is_empty()
        && plan.directories.is_empty()
    {
        println!("Mirror '{}' is up to date.", request.project.name);
        return Ok(());
    }
    if !request.force && !confirm("Apply this mirror update? Type y or yes to confirm: ")? {
        println!("Mirror update cancelled.");
        return Ok(());
    }
    verify_source_snapshot(&plan)?;
    let project_name = request.project.name.clone();
    lb.with_mirror_project_mutation(&project_name, |lb, _| {
        for path in removal_roots(&plan.removals, lb) {
            if lb.is_dir(&path) {
                lb.remove_dir_recursive(&path)?;
            } else {
                lb.delete(&path)?;
            }
        }
        for directory in &plan.directories {
            if !lb.is_dir(directory) {
                lb.create_dir(directory, true)?;
            }
        }
        for addition in &plan.additions {
            lb.create_parent_dirs_for(&addition.destination)?;
            lb.add_file_from_path(&addition.source, &addition.destination, false)?;
        }
        for replacement in &plan.replacements {
            lb.create_parent_dirs_for(&replacement.destination)?;
            lb.add_file_from_path(&replacement.source, &replacement.destination, true)?;
        }
        Ok(())
    })?;
    verify_source_snapshot(&plan)?;
    lb.commit()?;
    if request.format != "json" {
        println!(
            "Updated mirror '{}': {} added, {} replaced, {} directories created, {} removed, {} unchanged.",
            request.project.name,
            plan.additions.len(),
            plan.replacements.len(),
            plan.directories.len(),
            plan.removed_files,
            plan.unchanged
        );
    }
    Ok(())
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

fn walk_source(root: &Path, includes: &[String], excludes: &[String]) -> CliResult<SourceSnapshot> {
    fn visit(
        root: &Path,
        current: &Path,
        includes: &[String],
        excludes: &[String],
        files: &mut BTreeMap<String, (PathBuf, FileFingerprint)>,
        directories: &mut BTreeSet<String>,
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
                    "symbolic links are not supported by mirror projects: {}",
                    path.display()
                )));
            }
            if kind.is_dir() {
                if includes.is_empty() {
                    directories.insert(relative);
                }
                visit(root, &path, includes, excludes, files, directories)?;
            } else if kind.is_file() {
                if included(&relative, includes) {
                    for directory in Path::new(&relative).ancestors().skip(1) {
                        let directory = slash_path(directory)?;
                        if directory.is_empty() {
                            break;
                        }
                        directories.insert(directory);
                    }
                    files.insert(relative, (path.clone(), fingerprint(&path)?));
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
    let mut files = BTreeMap::new();
    let mut directories = BTreeSet::new();
    visit(root, root, includes, excludes, &mut files, &mut directories)?;
    Ok(SourceSnapshot { files, directories })
}

fn build_plan(
    lb: &Lockbox,
    request: &MirrorRequest,
    source: &SourceSnapshot,
) -> CliResult<MirrorPlan> {
    let mut archive = BTreeMap::new();
    let mut options = ListOptions::new(&request.destination);
    options.recursive = true;
    for entry in lb.list(options)? {
        let entry = entry?;
        let relative = relative_lockbox_path(&request.destination, &entry.path);
        if !relative.is_empty() {
            archive.insert(relative, entry);
        }
    }
    let mut plan = MirrorPlan {
        additions: Vec::new(),
        replacements: Vec::new(),
        removals: Vec::new(),
        unchanged: 0,
        directories: Vec::new(),
        removed_files: 0,
    };
    if request.destination.as_str() != "/" && lb.stat(&request.destination).is_none() {
        plan.directories.push(request.destination.clone());
    }
    for relative in &source.directories {
        let destination = join_destination(&request.destination, relative)?;
        match archive.remove(relative) {
            Some(entry) if entry.kind != LockboxEntryKind::Directory => {
                plan.removed_files += 1;
                plan.removals.push(entry.path);
                plan.directories.push(destination);
            }
            Some(_) => {}
            None => plan.directories.push(destination),
        }
    }
    for (relative, (host_path, host_fingerprint)) in &source.files {
        let destination = join_destination(&request.destination, relative)?;
        match archive.remove(relative) {
            None => plan.additions.push(MirrorAddition {
                source: host_path.clone(),
                destination,
                fingerprint: host_fingerprint.clone(),
            }),
            Some(entry) if entry.kind == LockboxEntryKind::Directory => {
                let directory = entry.path.clone();
                plan.removed_files += archive
                    .values()
                    .filter(|candidate| {
                        candidate.path.is_descendant_of(&directory)
                            && candidate.kind != LockboxEntryKind::Directory
                    })
                    .count();
                plan.removals.push(directory.clone());
                archive.retain(|_, candidate| !candidate.path.is_descendant_of(&directory));
                plan.additions.push(MirrorAddition {
                    source: host_path.clone(),
                    destination,
                    fingerprint: host_fingerprint.clone(),
                });
            }
            Some(entry) if entry.kind != LockboxEntryKind::File => {
                plan.removed_files += 1;
                plan.removals.push(entry.path);
                plan.additions.push(MirrorAddition {
                    source: host_path.clone(),
                    destination,
                    fingerprint: host_fingerprint.clone(),
                });
            }
            Some(entry) => {
                let archive_digest = archive_digest(lb, &entry.path)?;
                if entry.len == host_fingerprint.len && archive_digest == host_fingerprint.digest {
                    plan.unchanged += 1;
                } else {
                    plan.replacements.push(MirrorReplacement {
                        source: host_path.clone(),
                        destination,
                        fingerprint: host_fingerprint.clone(),
                    });
                }
            }
        }
    }
    if request.project.missing_file_policy == MirrorMissingFilePolicy::Remove {
        for entry in archive.into_values() {
            if entry.kind != LockboxEntryKind::Directory {
                plan.removed_files += 1;
            }
            plan.removals.push(entry.path);
        }
    }
    Ok(plan)
}

fn fingerprint(path: &Path) -> CliResult<FileFingerprint> {
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

fn verify_source_snapshot(plan: &MirrorPlan) -> CliResult<()> {
    for (path, expected) in plan
        .additions
        .iter()
        .map(|item| (&item.source, &item.fingerprint))
        .chain(
            plan.replacements
                .iter()
                .map(|item| (&item.source, &item.fingerprint)),
        )
    {
        if fingerprint(path)? != *expected {
            return Err(cli_error(format!(
                "source file changed while updating the mirror: {}",
                path.display()
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

fn print_plan(plan: &MirrorPlan, source: &Path, request: &MirrorRequest) -> CliResult<()> {
    if request.format == "json" {
        println!(
            "{}",
            json!({
                "source": source.display().to_string(),
                "destination": request.destination.to_string(),
                "add": plan.additions.iter().map(|v| v.destination.to_string()).collect::<Vec<_>>(),
                "replace": plan.replacements.iter().map(|v| v.destination.to_string()).collect::<Vec<_>>(),
                "create_directory": plan.directories.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "remove": plan.removals.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "remove_file_count": plan.removed_files,
                "unchanged": plan.unchanged,
            })
        );
        return Ok(());
    }
    if request.format == "tsv" {
        for addition in &plan.additions {
            println!("add\t{}", addition.destination);
        }
        for replacement in &plan.replacements {
            println!("replace\t{}", replacement.destination);
        }
        for directory in &plan.directories {
            println!("create-directory\t{directory}");
        }
        for removal in &plan.removals {
            println!("remove\t{removal}");
        }
        println!("unchanged\t{}", plan.unchanged);
        return Ok(());
    }
    println!("Mirror status for '{}'", request.project.name);
    println!();
    println!("  source:    {}", source.display());
    println!("  add:       {} files", plan.additions.len());
    println!("  replace:   {} files", plan.replacements.len());
    println!("  mkdir:     {} directories", plan.directories.len());
    println!("  remove:    {} files", plan.removed_files);
    println!("  unchanged: {} files", plan.unchanged);
    if !plan.removals.is_empty() {
        println!("\nEntries to remove:");
        for path in &plan.removals {
            println!("  {path}");
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
