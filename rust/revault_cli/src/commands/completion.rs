//! Dynamic completion follows these rules:
//!
//! - At `lbx <TAB>`, offer commands, global options, and `.lbox` files.
//! - Never offer unrelated host files or directories as lockboxes.
//! - Keep relative and nested lockbox path navigation working.
//! - After a lockbox is selected, offer only valid commands and options.
//! - Host source arguments complete host filesystem paths.
//! - Archive destination arguments complete archive directories.
//! - Archive file arguments complete archive entries.
//! - Variable and form values are offered only by their respective commands.
//! - Prefer the selected lockbox for archive values; fall back to all cached
//!   lockboxes only when the command line does not identify one.

use clap::ArgMatches;
use clap_complete::engine::CompletionCandidate;
use revault_lockbox_api::{ListOptions, LockboxPath, SecretString};
use revault_vault_api::{
    default_vault_path, get_vault_unlock_key, list as list_cached_lockboxes, local_vault,
    ReadOnlyVaultDirectory,
};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::context::CliResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl CompletionShell {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            "elvish" => Some(Self::Elvish),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::PowerShell => "powershell",
            Self::Elvish => "elvish",
        }
    }
}

pub(crate) fn run_matches(matches: &ArgMatches) -> CliResult<()> {
    let Some((subcommand, submatches)) = matches.subcommand() else {
        return Err(super::context::cli_error(
            "completion requires generate, install, or uninstall",
        ));
    };
    match subcommand {
        "generate" => generate_matches(submatches),
        "install" => install_matches(submatches),
        "uninstall" => uninstall_matches(submatches),
        other => Err(super::context::cli_error(format!(
            "unknown completion command: {other}"
        ))),
    }
}

fn generate_matches(matches: &ArgMatches) -> CliResult<()> {
    let shell = shell_from_matches(matches)?;
    let script = registration_script(shell)?;
    if let Some(path) = matches.get_one::<String>("output") {
        write_script(Path::new(path), &script)?;
    } else {
        print!("{script}");
    }
    Ok(())
}

fn install_matches(matches: &ArgMatches) -> CliResult<()> {
    let shell = shell_from_matches(matches)?;
    let explicit_path = matches.get_one::<String>("path").map(PathBuf::from);
    let path = explicit_path
        .clone()
        .unwrap_or(standard_install_path(shell)?);
    let script = registration_script(shell)?;
    if shell == CompletionShell::PowerShell && explicit_path.is_none() {
        install_powershell_profile_block(&path, &script)?;
    } else {
        write_script(&path, &script)?;
    }
    eprintln!(
        "Installed {} completion at {}. Source it from your shell configuration if needed.",
        shell.name(),
        path.display()
    );
    Ok(())
}

fn uninstall_matches(matches: &ArgMatches) -> CliResult<()> {
    let shell = shell_from_matches(matches)?;
    let explicit_path = matches.get_one::<String>("path").map(PathBuf::from);
    let path = explicit_path
        .clone()
        .unwrap_or(standard_install_path(shell)?);
    if shell == CompletionShell::PowerShell && explicit_path.is_none() {
        uninstall_powershell_profile_block(&path)?;
        eprintln!(
            "Removed {} completion from {}.",
            shell.name(),
            path.display()
        );
        return Ok(());
    }
    match fs::remove_file(&path) {
        Ok(()) => eprintln!(
            "Removed {} completion from {}.",
            shell.name(),
            path.display()
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

fn shell_from_matches(matches: &ArgMatches) -> CliResult<CompletionShell> {
    if let Some(value) = matches.get_one::<String>("shell") {
        return CompletionShell::parse(value).ok_or_else(|| {
            super::context::cli_error("shell must be bash, zsh, fish, powershell, or elvish")
        });
    }
    detect_shell().ok_or_else(|| {
        super::context::cli_error(
            "cannot detect the current shell; pass --shell bash|zsh|fish|powershell|elvish",
        )
    })
}

pub(crate) fn detect_shell() -> Option<CompletionShell> {
    detect_shell_from(
        cfg!(windows),
        env::var_os("SHELL").as_deref(),
        env::var_os("PSModulePath").as_deref(),
        env::var_os("ComSpec").as_deref(),
    )
}

fn detect_shell_from(
    windows: bool,
    shell: Option<&OsStr>,
    ps_module_path: Option<&OsStr>,
    comspec: Option<&OsStr>,
) -> Option<CompletionShell> {
    if windows {
        if ps_module_path.is_some() {
            return Some(CompletionShell::PowerShell);
        }
        return comspec
            .and_then(|value| Path::new(&value).file_stem().map(|name| name.to_owned()))
            .and_then(|name| CompletionShell::parse(&name.to_string_lossy()));
    }
    shell
        .and_then(|value| Path::new(&value).file_stem().map(|name| name.to_owned()))
        .and_then(|name| CompletionShell::parse(&name.to_string_lossy()))
}

fn registration_script(shell: CompletionShell) -> CliResult<String> {
    let executable = env::current_exe()?;
    let output = Command::new(&executable)
        .env("COMPLETE", shell.name())
        .output()?;
    if !output.status.success() {
        return Err(super::context::cli_error(format!(
            "completion generator exited with {}",
            output.status
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| super::context::cli_error("completion generator returned non-UTF-8 output"))
}

fn standard_install_path(shell: CompletionShell) -> CliResult<PathBuf> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| super::context::cli_error("home directory is unavailable"))?;
    let binary = binary_name();
    Ok(match shell {
        CompletionShell::Bash => home
            .join(".local/share/bash-completion/completions")
            .join(&binary),
        CompletionShell::Zsh => home
            .join(".local/share/zsh/site-functions")
            .join(format!("_{binary}")),
        CompletionShell::Fish => home
            .join(".config/fish/completions")
            .join(format!("{binary}.fish")),
        CompletionShell::Elvish => home
            .join(".config/elvish/lib")
            .join(format!("{binary}.elv")),
        CompletionShell::PowerShell if cfg!(windows) => home
            .join("Documents/PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
        CompletionShell::PowerShell => home
            .join(".config/powershell")
            .join("Microsoft.PowerShell_profile.ps1"),
    })
}

fn write_script(path: &Path, script: &str) -> CliResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| super::context::cli_error("completion output path has no parent"))?;
    fs::create_dir_all(parent)?;
    fs::write(path, script)?;
    Ok(())
}

const POWERSHELL_BLOCK_START: &str = "# BEGIN revault dynamic completion";
const POWERSHELL_BLOCK_END: &str = "# END revault dynamic completion";

fn install_powershell_profile_block(path: &Path, script: &str) -> CliResult<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut profile = remove_managed_block(&existing);
    if !profile.is_empty() && !profile.ends_with('\n') {
        profile.push('\n');
    }
    profile.push_str(POWERSHELL_BLOCK_START);
    profile.push('\n');
    profile.push_str(script);
    if !script.ends_with('\n') {
        profile.push('\n');
    }
    profile.push_str(POWERSHELL_BLOCK_END);
    profile.push('\n');
    write_script(path, &profile)
}

fn uninstall_powershell_profile_block(path: &Path) -> CliResult<()> {
    let existing = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    write_script(path, &remove_managed_block(&existing))
}

fn remove_managed_block(profile: &str) -> String {
    let Some(start) = profile.find(POWERSHELL_BLOCK_START) else {
        return profile.to_string();
    };
    let Some(relative_end) = profile[start..].find(POWERSHELL_BLOCK_END) else {
        return profile.to_string();
    };
    let mut end = start + relative_end + POWERSHELL_BLOCK_END.len();
    if profile.as_bytes().get(end) == Some(&b'\n') {
        end += 1;
    }
    let mut output = String::with_capacity(profile.len() - (end - start));
    output.push_str(&profile[..start]);
    output.push_str(&profile[end..]);
    output
}

fn binary_name() -> String {
    env::args_os()
        .next()
        .and_then(|value| Path::new(&value).file_stem().map(|value| value.to_owned()))
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| value == "lbx")
        .unwrap_or_else(|| "lockbox".to_string())
}

fn current_prefix(current: &OsStr) -> Option<&str> {
    current.to_str()
}

fn candidates(
    current: &OsStr,
    values: impl IntoIterator<Item = String>,
) -> Vec<CompletionCandidate> {
    let Some(current) = current_prefix(current) else {
        return Vec::new();
    };
    values
        .into_iter()
        .filter(|value| value.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}

pub(crate) fn lockbox_path_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    let value = Path::new(current);
    let (prefix, file_prefix) = match value.file_name() {
        Some(name)
            if !current
                .as_encoded_bytes()
                .ends_with(std::path::MAIN_SEPARATOR_STR.as_bytes()) =>
        {
            (value.parent().unwrap_or_else(|| Path::new("")), name)
        }
        _ => (value, OsStr::new("")),
    };
    let search_root = if prefix.is_absolute() {
        prefix.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(prefix)
    };
    let mut values = fs::read_dir(search_root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(&*file_prefix.to_string_lossy())
        })
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if file_type.is_file() && is_lockbox_path(&entry.path()) {
                return Some(prefix.join(entry.file_name()));
            }
            if file_type.is_dir() && directory_contains_lockbox(&entry.path()) {
                let mut path = prefix.join(entry.file_name());
                path.push("");
                return Some(path);
            }
            None
        })
        .map(|path| CompletionCandidate::new(path.into_os_string()))
        .collect::<Vec<_>>();
    values.sort();
    values
}

fn is_lockbox_path(path: &Path) -> bool {
    path.extension() == Some(OsStr::new("lbox"))
}

fn directory_contains_lockbox(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return false;
            };
            (file_type.is_file() && is_lockbox_path(&entry.path()))
                || (file_type.is_dir() && directory_contains_lockbox(&entry.path()))
        })
}

fn read_only_vault() -> Option<ReadOnlyVaultDirectory> {
    let vault_id = default_vault_path().ok()?.to_string_lossy().into_owned();
    let password = SecretString::try_from_env("LOCKBOX_VAULT_PASSWORD")
        .ok()
        .flatten()
        .or_else(|| get_vault_unlock_key(&vault_id).ok().flatten())?;
    ReadOnlyVaultDirectory::open_default(&password).ok()
}

pub(crate) fn profile_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(
        current,
        read_only_vault()
            .and_then(|vault| vault.list_private_key_names().ok())
            .unwrap_or_default(),
    )
}

pub(crate) fn contact_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(
        current,
        read_only_vault()
            .and_then(|vault| vault.list_contact_names().ok())
            .unwrap_or_default(),
    )
}

pub(crate) fn named_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut values = profile_candidates(current);
    values.extend(contact_candidates(current));
    values.sort();
    values.dedup();
    values
}

pub(crate) fn form_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(
        current,
        read_only_vault()
            .and_then(|vault| vault.list_form_aliases().ok())
            .unwrap_or_default(),
    )
}

#[derive(Clone, Copy)]
enum ArchiveCandidateKind {
    Directory,
    Entry,
    Form,
    Variable,
}

/// Returns values from already-cached archives. It never attempts a password,
/// contact key, vault key, or owner-signing key open.
fn cached_archive_candidates(
    current: &OsStr,
    kind: ArchiveCandidateKind,
) -> Vec<CompletionCandidate> {
    let mut values = Vec::new();
    for path in completion_lockbox_paths() {
        let Ok(lockbox) = local_vault().open_lockbox_read_only(&path) else {
            continue;
        };
        match kind {
            ArchiveCandidateKind::Variable => {
                if let Ok(variables) = lockbox.list_variables() {
                    let reveal_hidden = current
                        .to_str()
                        .is_some_and(|prefix| prefix.starts_with("/.") || prefix.starts_with('.'));
                    values.extend(
                        variables
                            .into_iter()
                            .map(|(name, _)| name)
                            .filter(|name| {
                                reveal_hidden || !super::variables::is_hidden_variable(name)
                            })
                            .map(|name| name.to_string()),
                    );
                }
            }
            ArchiveCandidateKind::Form => {
                if let Ok(forms) = lockbox.list_form_definitions() {
                    values.extend(forms.into_iter().map(|form| form.alias));
                }
                if let Ok(records) = lockbox.list_form_records() {
                    values.extend(records.into_iter().map(|record| record.path.to_string()));
                }
            }
            ArchiveCandidateKind::Entry | ArchiveCandidateKind::Directory => {
                if let Ok(path) = LockboxPath::new("/") {
                    let mut options = ListOptions::new(&path);
                    options.recursive = true;
                    if let Ok(entries) = lockbox.list(options) {
                        for entry in entries.filter_map(Result::ok) {
                            let entry_path = entry.path.to_string();
                            if matches!(kind, ArchiveCandidateKind::Entry) {
                                values.push(entry_path.clone());
                            }
                            add_archive_directories(&mut values, &entry_path);
                        }
                    }
                }
            }
        }
    }
    values.sort();
    values.dedup();
    candidates(current, values)
}

fn completion_lockbox_paths() -> Vec<String> {
    selected_lockbox_path()
        .map(|path| vec![path])
        .unwrap_or_else(|| {
            list_cached_lockboxes()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|cached| cached.path)
                .collect()
        })
}

fn selected_lockbox_path() -> Option<String> {
    let words = completion_words();
    let first = words.first()?;
    if is_command_name(first) || first.to_string_lossy().starts_with('-') {
        return None;
    }
    Some(first.to_string_lossy().into_owned())
}

fn is_command_name(value: &OsStr) -> bool {
    matches!(
        value.to_str(),
        Some(
            "create"
                | "open"
                | "close"
                | "add"
                | "mirror"
                | "extract"
                | "cat"
                | "list"
                | "ls"
                | "remove"
                | "rm"
                | "move"
                | "mv"
                | "rename"
                | "variable"
                | "var"
                | "variables"
                | "form"
                | "session"
                | "completion"
                | "migrate"
                | "doctor"
                | "vault"
                | "access"
                | "visualize"
                | "visualise"
                | "keygen"
                | "open-key"
        )
    )
}

fn add_archive_directories(values: &mut Vec<String>, entry: &str) {
    let path = Path::new(entry);
    for parent in path.ancestors().skip(1) {
        let value = parent.to_string_lossy();
        if value.is_empty() {
            continue;
        }
        let value = if value == "/" {
            value.into_owned()
        } else {
            format!("{value}/")
        };
        values.push(value);
    }
}

pub(crate) fn archive_entry_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    cached_archive_candidates(current, ArchiveCandidateKind::Entry)
}

pub(crate) fn archive_directory_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    cached_archive_candidates(current, ArchiveCandidateKind::Directory)
}

pub(crate) fn variable_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    cached_archive_candidates(current, ArchiveCandidateKind::Variable)
}

pub(crate) fn lockbox_form_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    cached_archive_candidates(current, ArchiveCandidateKind::Form)
}

pub(crate) fn archive_value_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    let words = completion_words();
    if words
        .iter()
        .any(|word| matches!(word.to_str(), Some("variable" | "variables" | "var")))
    {
        return variable_candidates(current);
    }
    if words.iter().any(|word| word.to_str() == Some("form")) {
        return lockbox_form_candidates(current);
    }
    archive_entry_candidates(current)
}

pub(crate) fn mirror_project_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut values = Vec::new();
    for path in completion_lockbox_paths() {
        let Ok(lockbox) = local_vault().open_lockbox_read_only(&path) else {
            continue;
        };
        if let Ok(projects) = lockbox.list_mirror_projects() {
            values.extend(projects.into_iter().map(|project| project.name));
        }
    }
    values.sort();
    values.dedup();
    candidates(current, values)
}

pub(crate) fn mirror_entry_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    mirror_path_candidates(current, false)
}

pub(crate) fn mirror_directory_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    mirror_path_candidates(current, true)
}

pub(crate) fn mirror_rule_candidates(current: &OsStr) -> Vec<CompletionCandidate> {
    let words = completion_words();
    let kind = words
        .iter()
        .position(|word| word == "remove" || word == "rm")
        .and_then(|index| words.get(index + 1))
        .and_then(|word| word.to_str());
    let explicit_name = words
        .iter()
        .position(|word| word == "mirror")
        .and_then(|index| words.get(index + 1))
        .filter(|word| !is_mirror_action(word))
        .and_then(|word| word.to_str());
    let mut values = Vec::new();
    for path in completion_lockbox_paths() {
        let Ok(lockbox) = local_vault().open_lockbox_read_only(&path) else {
            continue;
        };
        let Ok(projects) = lockbox.list_mirror_projects() else {
            continue;
        };
        let project = explicit_name
            .and_then(|name| projects.iter().find(|project| project.name == name))
            .or_else(|| (projects.len() == 1).then(|| &projects[0]));
        let Some(project) = project else {
            continue;
        };
        values.extend(match kind {
            Some("include") => project.includes.clone(),
            Some("exclude") => project.excludes.clone(),
            _ => project
                .includes
                .iter()
                .chain(&project.excludes)
                .cloned()
                .collect(),
        });
    }
    values.sort();
    values.dedup();
    candidates(current, values)
}

fn mirror_path_candidates(current: &OsStr, directories_only: bool) -> Vec<CompletionCandidate> {
    let words = completion_words();
    let explicit_name = words
        .iter()
        .position(|word| word == "mirror")
        .and_then(|index| words.get(index + 1))
        .filter(|word| !is_mirror_action(word))
        .and_then(|word| word.to_str());
    let mut values = Vec::new();
    for path in completion_lockbox_paths() {
        let Ok(lockbox) = local_vault().open_lockbox_read_only(&path) else {
            continue;
        };
        let Ok(projects) = lockbox.list_mirror_projects() else {
            continue;
        };
        let project = explicit_name
            .and_then(|name| projects.iter().find(|project| project.name == name))
            .or_else(|| (projects.len() == 1).then(|| &projects[0]));
        let Some(project) = project else {
            continue;
        };
        let mut options = ListOptions::new(&project.destination);
        options.recursive = true;
        let Ok(entries) = lockbox.list(options) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let relative = entry
                .path
                .as_str()
                .strip_prefix(project.destination.as_str())
                .unwrap_or(entry.path.as_str())
                .trim_start_matches('/');
            if relative.is_empty() {
                continue;
            }
            if !directories_only {
                values.push(relative.to_string());
            }
            add_archive_directories(&mut values, &format!("/{relative}"));
        }
    }
    values = values
        .into_iter()
        .map(|value| value.trim_start_matches('/').to_string())
        .collect();
    values.sort();
    values.dedup();
    candidates(current, values)
}

fn is_mirror_action(value: &OsStr) -> bool {
    matches!(
        value.to_str(),
        Some(
            "create"
                | "projects"
                | "info"
                | "status"
                | "update"
                | "configure"
                | "rebind"
                | "forget"
                | "delete"
                | "rule"
                | "rules"
                | "add"
                | "extract"
                | "cat"
                | "list"
                | "ls"
                | "remove"
                | "rm"
                | "move"
                | "mv"
                | "rename"
        )
    )
}

fn completion_words() -> Vec<std::ffi::OsString> {
    let mut words = env::args_os().skip(1).collect::<Vec<_>>();
    while words.first().is_some_and(|word| word == "--") {
        words.remove(0);
    }
    if words
        .first()
        .and_then(|word| Path::new(word).file_stem())
        .is_some_and(|word| word == "lockbox" || word == "lbx")
    {
        words.remove(0);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_override_accepts_supported_names_and_aliases() {
        assert_eq!(CompletionShell::parse("bash"), Some(CompletionShell::Bash));
        assert_eq!(
            CompletionShell::parse("PWSh"),
            Some(CompletionShell::PowerShell)
        );
        assert_eq!(CompletionShell::parse("unknown"), None);
    }

    #[test]
    fn candidate_filter_is_static_and_secret_free() {
        let values = candidates(OsStr::new("al"), ["alice".to_string(), "bob".to_string()]);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].get_value(), OsStr::new("alice"));
    }

    #[test]
    fn shell_detection_handles_unix_windows_and_ambiguity() {
        assert_eq!(
            detect_shell_from(false, Some(OsStr::new("/bin/zsh")), None, None),
            Some(CompletionShell::Zsh)
        );
        assert_eq!(
            detect_shell_from(true, None, Some(OsStr::new("modules")), None),
            Some(CompletionShell::PowerShell)
        );
        assert_eq!(detect_shell_from(false, None, None, None), None);
        assert_eq!(
            detect_shell_from(false, Some(OsStr::new("/bin/unknown")), None, None),
            None
        );
    }

    #[test]
    fn powershell_managed_block_preserves_user_profile_content() {
        let profile =
            format!("before\n{POWERSHELL_BLOCK_START}\nold\n{POWERSHELL_BLOCK_END}\nafter\n");
        assert_eq!(remove_managed_block(&profile), "before\nafter\n");
        assert_eq!(remove_managed_block("user content\n"), "user content\n");
    }
}
