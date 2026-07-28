use clap::ArgMatches;
use revault_lockbox_api::{
    ListOptions, Lockbox, LockboxEntryKind, LockboxPath, VariableName, WorkloadProfile,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use super::context::{cli_error, open_existing, Access, CliResult};
use super::default_lockbox_for_add;

const PROFILE_PREFIX: &str = "/.revault/sync/";
const LARGE_DELETE_PERCENT: usize = 50;

/// A complete, inspectable one-way synchronization plan.
#[derive(Debug)]
pub(crate) struct SyncPlan {
    pub additions: Vec<SyncAddition>,
    pub replacements: Vec<SyncReplacement>,
    pub removals: Vec<LockboxPath>,
    pub unchanged: usize,
    directories: Vec<LockboxPath>,
}

#[derive(Debug)]
pub(crate) struct SyncAddition {
    source: PathBuf,
    destination: LockboxPath,
    fingerprint: FileFingerprint,
}

#[derive(Debug)]
pub(crate) struct SyncReplacement {
    source: PathBuf,
    destination: LockboxPath,
    fingerprint: FileFingerprint,
}

/// Safety controls that affect synchronization planning.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SyncOptions {
    pub delete: bool,
    pub delete_excluded: bool,
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
struct SyncRequest {
    lockbox: String,
    source: PathBuf,
    destination: LockboxPath,
    includes: Vec<String>,
    excludes: Vec<String>,
    options: SyncOptions,
    dry_run: bool,
    force: bool,
    rebind_host_path: bool,
    update_rules: bool,
    json: bool,
}

#[derive(Debug)]
struct SyncProfile {
    id: String,
    source: String,
    destination: String,
    includes: Vec<String>,
    excludes: Vec<String>,
    identity: Option<String>,
}

#[derive(Debug)]
struct SourceSnapshot {
    files: BTreeMap<String, (PathBuf, FileFingerprint)>,
    directories: BTreeSet<String>,
}

pub(crate) fn run_matches(matches: &ArgMatches, access: &Access) -> CliResult<()> {
    let source = matches
        .get_one::<String>("source")
        .ok_or_else(|| cli_error("sync requires a source directory"))?;
    let destination = matches
        .get_one::<String>("to")
        .ok_or_else(|| cli_error("sync requires --to <LOCKBOX_PATH>"))?;
    let delete_excluded = matches.get_flag("delete-excluded");
    let request = SyncRequest {
        lockbox: default_lockbox_for_add()?,
        source: PathBuf::from(source),
        destination: lockbox_path(destination)?,
        includes: matches
            .get_many::<String>("include")
            .map(|values| values.cloned().collect())
            .unwrap_or_default(),
        excludes: matches
            .get_many::<String>("exclude")
            .map(|values| values.cloned().collect())
            .unwrap_or_default(),
        options: SyncOptions {
            delete: matches.get_flag("delete") || delete_excluded,
            delete_excluded,
            allow_empty: matches.get_flag("allow-empty"),
            allow_large_delete: matches.get_flag("allow-large-delete"),
        },
        dry_run: matches.get_flag("dry-run"),
        force: matches.get_flag("force"),
        rebind_host_path: matches.get_flag("rebind-host-path"),
        update_rules: matches.get_flag("update-rules"),
        json: matches
            .get_one::<String>("format")
            .is_some_and(|value| value == "json"),
    };
    sync(request, access)
}

fn sync(request: SyncRequest, access: &Access) -> CliResult<()> {
    let canonical = canonical_source(&request.source)?;
    let identity = platform_identity(&canonical)?;
    let mut lb = open_existing(&request.lockbox, access)?;
    lb.set_workload_profile(WorkloadProfile::BulkImport);

    let profiles = load_profiles(&lb)?;
    let existing = profiles
        .iter()
        .find(|profile| profile.destination == request.destination.as_str());
    validate_destination_overlap(
        &profiles,
        &request.destination,
        existing.map(|p| p.id.as_str()),
    )?;
    validate_profile(
        existing,
        &canonical,
        identity.as_deref(),
        &request.includes,
        &request.excludes,
        &request,
    )?;
    let profile_changed = existing.is_none_or(|profile| {
        profile.source != canonical.display().to_string()
            || profile.identity.as_deref() != identity.as_deref()
            || profile.includes != request.includes
            || profile.excludes != request.excludes
    });
    if let Some(profile) = existing {
        if profile.source != canonical.display().to_string()
            || profile.identity.as_deref() != identity.as_deref()
        {
            println!("Rebinding synchronization source:");
            println!("  old: {}", profile.source);
            println!("  new: {}", canonical.display());
        }
    }

    let source_entries = walk_source(&canonical, &request.includes, &request.excludes)?;
    if source_entries.files.is_empty() && request.options.delete && !request.options.allow_empty {
        return Err(cli_error(
            "source directory is empty; pass --allow-empty to permit deletion",
        ));
    }
    let plan = build_plan(&lb, &request, &source_entries)?;
    validate_large_delete(
        &plan,
        request.options,
        destination_file_count(&lb, &request.destination)?,
    )?;
    print_plan(&plan, &canonical, &request)?;

    if request.dry_run {
        if !request.json {
            println!("Dry run: no lockbox changes were committed.");
        }
        return Ok(());
    }
    if plan.additions.is_empty()
        && plan.replacements.is_empty()
        && plan.removals.is_empty()
        && !profile_changed
    {
        if !request.json {
            println!(
                "{} is already synchronized with {}.",
                request.destination,
                canonical.display()
            );
        }
        return Ok(());
    }
    if !request.force && !confirm("Apply this synchronization? Type y or yes to confirm: ")? {
        println!("Synchronization cancelled.");
        return Ok(());
    }

    verify_source_snapshot(&plan)?;
    for path in removal_roots(&plan.removals, &lb) {
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
    verify_source_snapshot(&plan)?;

    let profile = SyncProfile {
        id: existing
            .map(|profile| profile.id.clone())
            .unwrap_or_else(|| profile_id(&canonical, &request.destination)),
        source: canonical.display().to_string(),
        destination: request.destination.to_string(),
        includes: request.includes.clone(),
        excludes: request.excludes.clone(),
        identity,
    };
    store_profile(&mut lb, &profile)?;
    lb.commit()?;
    if !request.json {
        println!(
            "Synchronized {} to {}: {} added, {} replaced, {} removed, {} unchanged.",
            canonical.display(),
            request.destination,
            plan.additions.len(),
            plan.replacements.len(),
            plan.removals.len(),
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
            "sync source is not a directory: {}",
            source.display()
        )));
    }
    if canonical.parent().is_none() {
        return Err(cli_error("refusing to synchronize a filesystem root"));
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
                    "symbolic links are not supported by sync: {}",
                    path.display()
                )));
            }
            if kind.is_dir() {
                directories.insert(relative);
                visit(root, &path, includes, excludes, files, directories)?;
            } else if kind.is_file() {
                if included(&relative, includes) {
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

fn build_plan(lb: &Lockbox, request: &SyncRequest, source: &SourceSnapshot) -> CliResult<SyncPlan> {
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
    let mut plan = SyncPlan {
        additions: Vec::new(),
        replacements: Vec::new(),
        removals: Vec::new(),
        unchanged: 0,
        directories: Vec::new(),
    };
    for relative in &source.directories {
        let destination = join_destination(&request.destination, relative)?;
        match archive.remove(relative) {
            Some(entry) if entry.kind != LockboxEntryKind::Directory => {
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
            None => plan.additions.push(SyncAddition {
                source: host_path.clone(),
                destination,
                fingerprint: host_fingerprint.clone(),
            }),
            Some(entry) if entry.kind == LockboxEntryKind::Directory => {
                let directory = entry.path.clone();
                plan.removals.push(directory.clone());
                archive.retain(|_, candidate| !candidate.path.is_descendant_of(&directory));
                plan.additions.push(SyncAddition {
                    source: host_path.clone(),
                    destination,
                    fingerprint: host_fingerprint.clone(),
                });
            }
            Some(entry) if entry.kind != LockboxEntryKind::File => {
                plan.removals.push(entry.path);
                plan.additions.push(SyncAddition {
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
                    plan.replacements.push(SyncReplacement {
                        source: host_path.clone(),
                        destination,
                        fingerprint: host_fingerprint.clone(),
                    });
                }
            }
        }
    }
    if request.options.delete {
        for (relative, entry) in archive {
            if entry.kind == LockboxEntryKind::Directory {
                continue;
            }
            if (!included(&relative, &request.includes) || excluded(&relative, &request.excludes))
                && !request.options.delete_excluded
            {
                continue;
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

fn verify_source_snapshot(plan: &SyncPlan) -> CliResult<()> {
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
                "source file changed during synchronization: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn load_profiles(lb: &Lockbox) -> CliResult<Vec<SyncProfile>> {
    let mut profiles = Vec::new();
    for (name, _) in lb.list_variables()? {
        if !name.as_str().starts_with(PROFILE_PREFIX) {
            continue;
        }
        let Some(value) = lb.get_variable(&name)? else {
            continue;
        };
        profiles.push(profile_from_json(&value)?);
    }
    Ok(profiles)
}

fn store_profile(lb: &mut Lockbox, profile: &SyncProfile) -> CliResult<()> {
    let name = VariableName::new(format!("{PROFILE_PREFIX}P{}", profile.id))?;
    let encoded = json!({
        "version": 1,
        "id": profile.id,
        "source": profile.source,
            "destination": profile.destination,
            "includes": profile.includes,
            "excludes": profile.excludes,
        "symlinks": "reject",
        "identity": profile.identity,
    })
    .to_string();
    lb.set_variable(&name, &encoded)?;
    Ok(())
}

fn profile_from_json(text: &str) -> CliResult<SyncProfile> {
    let value: Value = serde_json::from_str(text)
        .map_err(|err| cli_error(format!("invalid sync profile: {err}")))?;
    let strings = |name: &str| {
        value[name]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(SyncProfile {
        id: json_string(&value, "id")?,
        source: json_string(&value, "source")?,
        destination: json_string(&value, "destination")?,
        includes: strings("includes"),
        excludes: strings("excludes"),
        identity: value["identity"].as_str().map(str::to_string),
    })
}

fn json_string(value: &Value, field: &str) -> CliResult<String> {
    value[field]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| cli_error(format!("sync profile is missing {field}")))
}

fn validate_profile(
    profile: Option<&SyncProfile>,
    source: &Path,
    identity: Option<&str>,
    includes: &[String],
    excludes: &[String],
    request: &SyncRequest,
) -> CliResult<()> {
    let Some(profile) = profile else {
        return Ok(());
    };
    let source_changed =
        profile.source != source.display().to_string() || profile.identity.as_deref() != identity;
    if source_changed && !request.rebind_host_path {
        return Err(cli_error(format!(
            "host source does not match the stored synchronization profile (stored: {}; current: {}); pass --rebind-host-path",
            profile.source,
            source.display()
        )));
    }
    if (profile.includes != includes || profile.excludes != excludes) && !request.update_rules {
        return Err(cli_error(
            "include/exclude rules differ from the stored synchronization profile; pass --update-rules",
        ));
    }
    Ok(())
}

fn validate_destination_overlap(
    profiles: &[SyncProfile],
    destination: &LockboxPath,
    current_id: Option<&str>,
) -> CliResult<()> {
    for profile in profiles {
        if current_id == Some(profile.id.as_str()) {
            continue;
        }
        let other = LockboxPath::new(&profile.destination)?;
        if destination == &other
            || destination.is_descendant_of(&other)
            || other.is_descendant_of(destination)
        {
            return Err(cli_error(format!(
                "synchronization destination overlaps stored profile {} at {}",
                profile.id, profile.destination
            )));
        }
    }
    Ok(())
}

fn validate_large_delete(
    plan: &SyncPlan,
    options: SyncOptions,
    destination_files: usize,
) -> CliResult<()> {
    if options.delete
        && !options.allow_large_delete
        && destination_files > 0
        && plan.removals.len() * 100 > destination_files * LARGE_DELETE_PERCENT
    {
        return Err(cli_error(format!(
            "sync would delete {} of {} destination files; pass --allow-large-delete",
            plan.removals.len(),
            destination_files
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

fn print_plan(plan: &SyncPlan, source: &Path, request: &SyncRequest) -> CliResult<()> {
    if request.json {
        println!(
            "{}",
            json!({
                "source": source.display().to_string(),
                "destination": request.destination.to_string(),
                "add": plan.additions.iter().map(|v| v.destination.to_string()).collect::<Vec<_>>(),
                "replace": plan.replacements.iter().map(|v| v.destination.to_string()).collect::<Vec<_>>(),
                "remove": plan.removals.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "unchanged": plan.unchanged,
                "dry_run": request.dry_run,
            })
        );
        return Ok(());
    }
    println!("Synchronization plan for {}", request.destination);
    println!();
    println!("  source:    {}", source.display());
    println!("  add:       {} files", plan.additions.len());
    println!("  replace:   {} files", plan.replacements.len());
    println!("  remove:    {} files", plan.removals.len());
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
            "confirmation requires a terminal; inspect with --dry-run, then pass --force",
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

fn profile_id(source: &Path, destination: &LockboxPath) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_os_str().as_encoded_bytes());
    hasher.update([0]);
    hasher.update(destination.as_str().as_bytes());
    hex_digest(hasher.finalize().as_slice())[..24].to_string()
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

fn excluded(path: &str, rules: &[String]) -> bool {
    rules.iter().any(|rule| {
        let rule = rule.trim_start_matches("./").trim_end_matches('/');
        path == rule
            || path.starts_with(&format!("{rule}/"))
            || (!rule.contains('/')
                && glob_match_component(rule, path.rsplit('/').next().unwrap_or(path)))
            || glob_match_parts(
                &rule.split('/').collect::<Vec<_>>(),
                &path.split('/').collect::<Vec<_>>(),
            )
    })
}

fn included(path: &str, rules: &[String]) -> bool {
    rules.is_empty()
        || rules.iter().any(|rule| {
            let rule = rule.trim_start_matches("./").trim_end_matches('/');
            path == rule
                || path.starts_with(&format!("{rule}/"))
                || (!rule.contains('/')
                    && glob_match_component(rule, path.rsplit('/').next().unwrap_or(path)))
                || glob_match_parts(
                    &rule.split('/').collect::<Vec<_>>(),
                    &path.split('/').collect::<Vec<_>>(),
                )
        })
}

fn glob_match_parts(pattern: &[&str], text: &[&str]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if pattern[0] == "**" {
        return glob_match_parts(&pattern[1..], text)
            || (!text.is_empty() && glob_match_parts(pattern, &text[1..]));
    }
    !text.is_empty()
        && glob_match_component(pattern[0], text[0])
        && glob_match_parts(&pattern[1..], &text[1..])
}

fn glob_match_component(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut p, mut t, mut star, mut matched) = (0, 0, None, 0);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            matched = t;
            p += 1;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            matched += 1;
            t = matched;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
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
