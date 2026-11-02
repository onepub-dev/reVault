use super::context::{cli_error, CliResult};
use clap::ArgMatches;
use revault_vault_api::{SecretActivityGuard, SecretActivityKind};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

fn path() -> CliResult<PathBuf> {
    revault_vault_api::agent_configuration_path().ok_or_else(|| {
        cli_error("Cannot determine the configuration path; set LOCKBOX_AGENT_CONFIG.")
    })
}

fn read_config() -> CliResult<String> {
    let mut text = String::new();
    match fs::File::open(path()?) {
        Ok(file) => {
            file.take(1_048_577).read_to_string(&mut text)?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if text.len() > 1_048_576 {
        return Err(cli_error(
            "Configuration exceeds one MiB; edit it manually.",
        ));
    }
    Ok(text)
}

fn assignment(line: &str) -> Option<(&str, &str)> {
    let content = line.split('#').next()?.trim();
    let (key, value) = content
        .split_once('=')
        .or_else(|| content.split_once(':'))?;
    Some((
        key.trim(),
        value.trim().trim_matches('"').trim_matches('\''),
    ))
}

fn bool_value(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => Some(true),
        "false" | "0" | "no" | "off" | "disabled" => Some(false),
        _ => None,
    }
}

fn key(setting: &str) -> &'static str {
    if setting == "prevent-sleep" {
        "agent.prevent_sleep"
    } else {
        "agent.terminate_on_suspend"
    }
}

fn selects(candidate: &str, setting: &str) -> bool {
    candidate == key(setting) || setting == "prevent-sleep" && candidate == "agent.suspend_inhibit"
}

fn edited(text: &str, setting: &str, enabled: bool) -> String {
    let mut result = format!("{} = {enabled}\n", key(setting));
    for line in text.lines() {
        if assignment(line).is_some_and(|(name, _)| selects(name, setting)) {
            if let Some((_, comment)) = line.split_once('#') {
                result.push_str(&format!("#{comment}\n"));
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn warning(setting: &str) -> &'static str {
    if setting == "prevent-sleep" {
        "The machine may sleep during secret operations. Termination-on-suspend is unchanged."
    } else {
        "Secret operations and in-memory secrets may survive suspension. Sleep inhibition is unchanged."
    }
}

fn confirm(setting: &str) -> CliResult<bool> {
    eprintln!("Warning: {} Applying the change stops the Session Agent and closes cached Lockbox sessions.", warning(setting));
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(cli_error(format!("Interactive confirmation is unavailable. To explicitly accept only this change, run `lbx session protection {setting} off --yes`.")));
    }
    eprint!("Remember this setting and continue? [y/N] ");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn save(setting: &str, enabled: bool) -> CliResult<()> {
    let path = path()?;
    if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(cli_error(
            "Configuration is a symlink; set LOCKBOX_AGENT_CONFIG to its real path before editing.",
        ));
    }
    let text = edited(&read_config()?, setting, enabled);
    let root = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    fs::create_dir_all(root)?;
    let mut pending = tempfile::NamedTempFile::new_in(root)?;
    pending.write_all(text.as_bytes())?;
    pending.as_file().sync_all()?;
    revault_vault_api::stop()?;
    pending.persist(&path)?;
    println!(
        "Saved {} in {}. The other protection setting is unchanged.",
        key(setting),
        path.display()
    );
    Ok(())
}

pub(crate) fn run(matches: &ArgMatches) -> CliResult<()> {
    let (setting, sub) = matches
        .subcommand()
        .ok_or_else(|| cli_error("missing protection command"))?;
    if setting == "status" {
        let text = read_config()?;
        println!("Configuration: {}", path()?.display());
        for (setting, variable) in [
            ("prevent-sleep", "LOCKBOX_AGENT_PREVENT_SLEEP"),
            ("terminate-on-suspend", "LOCKBOX_AGENT_TERMINATE_ON_SUSPEND"),
        ] {
            let configured = text
                .lines()
                .filter_map(assignment)
                .filter(|(name, _)| selects(name, setting))
                .filter_map(|(_, value)| bool_value(value))
                .next_back()
                .unwrap_or(true);
            let override_value = std::env::var(variable)
                .ok()
                .and_then(|value| bool_value(&value));
            println!(
                "{setting}: configured={}, next-start={}{}",
                configured,
                override_value.unwrap_or(configured),
                if override_value.is_some() {
                    " (environment override)"
                } else {
                    ""
                }
            );
        }
        println!("Session Agent running: {}. These values apply at startup; an existing agent may have older settings.", revault_vault_api::is_running());
        println!(
            "Cache clearing on suspend active: {}",
            revault_vault_api::suspend_cache_clearing_active()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "unknown".into())
        );
        return Ok(());
    }
    let enabled = sub.get_one::<String>("value").map(String::as_str) == Some("on");
    if !enabled {
        if sub.get_flag("yes") {
            eprintln!("Warning: {} Applying the change stops the Session Agent and closes cached Lockbox sessions.", warning(setting));
        } else if !confirm(setting)? {
            return Err(cli_error("Protection unchanged."));
        }
    }
    save(setting, enabled)
}

pub(crate) fn begin(kind: SecretActivityKind, offer_setup: bool) -> CliResult<SecretActivityGuard> {
    match revault_vault_api::begin_secret_activity(kind) {
        Ok(guard) => Ok(guard),
        Err(error) => {
            if !offer_setup
                || !error
                    .to_string()
                    .contains("requested sleep inhibition unavailable")
            {
                return Err(error.into());
            }
            eprintln!("Sleep inhibition could not start: {error}");
            if !confirm("prevent-sleep")? {
                return Err(cli_error(
                    "Initialization cancelled; configuration unchanged.",
                ));
            }
            save("prevent-sleep", false)?;
            revault_vault_api::begin_secret_activity(kind).map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editing_one_setting_preserves_other_policy_and_unrelated_configuration() {
        let initial = "# user settings\nagent.terminate_on_suspend = true\nagent.prevent_sleep = true # desktop\nagent.suspend_inhibit: yes\npublish.server = 'https://example.test'\n";
        let changed = edited(initial, "prevent-sleep", false);
        assert!(changed.starts_with("agent.prevent_sleep = false\n"));
        assert!(changed.contains("agent.terminate_on_suspend = true"));
        assert!(changed.contains("publish.server = 'https://example.test'"));
        assert!(changed.contains("# desktop"));
        assert!(!changed.contains("agent.suspend_inhibit"));
        assert_eq!(edited(&changed, "prevent-sleep", false), changed);
        let independent = edited(&changed, "terminate-on-suspend", false);
        assert!(independent.contains("agent.prevent_sleep = false"));
        assert!(independent.contains("agent.terminate_on_suspend = false"));
    }
}
