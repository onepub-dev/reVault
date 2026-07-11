use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::server_log::server_log_destination;

const UNIT_PATH: &str = "/etc/systemd/system/revault_key_server.service";
const INSTALL_BINARY_PATH: &str = "/usr/local/bin/revault_key_server";
const CONFIG_DIR: &str = "/etc/revault";
pub const CONFIG_PATH: &str = "/etc/revault/key-server.toml";
const LEGACY_CONFIG_PATH: &str = "/etc/lockbox/key-server.toml";
const STATE_DIR: &str = "/var/lib/revault-key-server";
const CACHE_DIR: &str = "/var/cache/revault-key-server";
const LOG_DIR: &str = "/var/log/revault-key-server";
const LOG_FILE: &str = "/var/log/revault-key-server/server.log";
const USER: &str = "revault-publish";

pub fn install_systemd(force_config: bool) -> Result<(), Box<dyn std::error::Error>> {
    require_root("install")?;
    let user_created = ensure_user()?;
    if user_created {
        println!("created service account: {USER}");
    }
    fs::create_dir_all(CONFIG_DIR)?;
    fs::create_dir_all(STATE_DIR)?;
    fs::create_dir_all(CACHE_DIR)?;
    fs::create_dir_all(LOG_DIR)?;
    set_dir_permissions(CONFIG_DIR, 0o755)?;
    set_dir_permissions(STATE_DIR, 0o750)?;
    set_dir_permissions(CACHE_DIR, 0o750)?;
    set_dir_permissions(LOG_DIR, 0o750)?;
    chown_path(STATE_DIR)?;
    chown_path(CACHE_DIR)?;
    chown_path(LOG_DIR)?;
    install_binary()?;
    if force_config || !Path::new(CONFIG_PATH).exists() {
        fs::write(CONFIG_PATH, default_config())?;
    }
    set_file_permissions(CONFIG_PATH, 0o640)?;
    chown_config()?;
    fs::write(UNIT_PATH, unit_file(INSTALL_BINARY_PATH))?;
    run("systemctl", &["daemon-reload"])?;
    run("systemctl", &["enable", "revault_key_server.service"])?;
    run("systemctl", &["reset-failed", "revault_key_server.service"])?;
    if let Err(err) = run("systemctl", &["restart", "revault_key_server.service"]) {
        return Err(format!(
            "{err}\nRun `sudo {INSTALL_BINARY_PATH} doctor` for an English diagnostic, then inspect logs with:\n  sudo journalctl -u revault_key_server -n 50 --no-pager"
        )
        .into());
    }
    println!("installed revault_key_server systemd service");
    Ok(())
}

fn install_binary() -> Result<(), Box<dyn std::error::Error>> {
    let current = std::env::current_exe()?;
    let target = Path::new(INSTALL_BINARY_PATH);
    if current != target {
        fs::copy(&current, target).map_err(|err| {
            format!(
                "could not install server executable at {}: {err}",
                target.display()
            )
        })?;
    }
    set_file_permissions(INSTALL_BINARY_PATH, 0o755)?;
    Ok(())
}

pub fn uninstall_systemd(purge_data: bool) -> Result<(), Box<dyn std::error::Error>> {
    let command = if purge_data {
        "uninstall --purge-data"
    } else {
        "uninstall"
    };
    require_root(command)?;
    let _ = run("systemctl", &["stop", "revault_key_server.service"]);
    let _ = run("systemctl", &["disable", "revault_key_server.service"]);
    if Path::new(UNIT_PATH).exists() {
        fs::remove_file(UNIT_PATH)?;
    }
    run("systemctl", &["daemon-reload"])?;
    if purge_data {
        let _ = fs::remove_dir_all(STATE_DIR);
        let _ = fs::remove_dir_all(CACHE_DIR);
        let _ = fs::remove_dir_all(LOG_DIR);
        let _ = fs::remove_file(CONFIG_PATH);
    }
    println!("uninstalled revault_key_server systemd service");
    Ok(())
}

pub fn start_systemd() -> Result<(), Box<dyn std::error::Error>> {
    require_root("start")?;
    run("systemctl", &["reset-failed", "revault_key_server.service"])?;
    run("systemctl", &["start", "revault_key_server.service"])?;
    println!("reVault key server started");
    Ok(())
}

pub fn stop_systemd() -> Result<(), Box<dyn std::error::Error>> {
    require_root("stop")?;
    run("systemctl", &["stop", "revault_key_server.service"])?;
    println!("reVault key server stopped");
    Ok(())
}

pub fn print_status() -> Result<(), Box<dyn std::error::Error>> {
    let installed = Path::new(UNIT_PATH).exists();
    let enabled = systemctl_value(&["is-enabled", "revault_key_server.service"])
        .unwrap_or_else(|| "not available".to_string());
    let active = systemctl_value(&["is-active", "revault_key_server.service"])
        .unwrap_or_else(|| "not available".to_string());
    let result = systemctl_show("Result");
    let exec_status = systemctl_show("ExecMainStatus");
    let exec_start = systemctl_show("ExecStart");

    println!("reVault key server doctor");
    println!();
    println!("Service");
    println!("  Installed: {}", yes_no(installed));
    println!("  Enabled at boot: {}", human_enabled(&enabled));
    println!("  Current state: {}", human_active(&active));
    println!("  Unit: {UNIT_PATH}");

    println!();
    println!("Configuration");
    println!("  Path: {CONFIG_PATH}");
    println!("  Present: {}", yes_no(Path::new(CONFIG_PATH).exists()));
    println!(
        "  Service account can read it: {}",
        service_can_read_path(CONFIG_PATH)
    );
    println!("  State directory: {STATE_DIR}");
    println!(
        "  State directory present: {}",
        yes_no(Path::new(STATE_DIR).exists())
    );
    println!(
        "  Service account can write state: {}",
        service_can_write_path(STATE_DIR)
    );
    println!("  Log file: {LOG_FILE}");
    println!(
        "  Service account can write logs: {}",
        service_can_write_path(LOG_DIR)
    );
    println!("  Foreground logging: {}", server_log_destination());
    if Path::new(LEGACY_CONFIG_PATH).exists() {
        println!();
        println!("Migration warning");
        println!("  Legacy configuration found at {LEGACY_CONFIG_PATH}.");
        println!("  The current installer uses {CONFIG_PATH}; migrate settings before restarting.");
    }

    if installed {
        println!();
        println!("Startup diagnostics");
        println!("  Systemd result: {result}");
        println!("  Process exit status: {exec_status}");
        if result == "exit-code" && exec_status == "203" {
            println!(
                "  Problem: systemd could not execute the configured server binary (203/EXEC)."
            );
            println!(
                "  Check that the executable exists and is executable by the service account."
            );
        } else if active == "failed" {
            println!("  Problem: the service failed during startup.");
            println!("  Inspect recent details with: sudo journalctl -u revault_key_server -n 50");
        }
        if !exec_start.is_empty() {
            println!("  Configured start command: {exec_start}");
        }
    }
    std::io::stdout().flush()?;
    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "YES"
    } else {
        "NO"
    }
}

fn human_enabled(value: &str) -> &str {
    match value {
        "enabled" => "YES",
        "disabled" => "NO",
        other => other,
    }
}

fn human_active(value: &str) -> &str {
    match value {
        "active" => "RUNNING",
        "inactive" => "STOPPED",
        "failed" => "FAILED",
        other => other,
    }
}

fn systemctl_show(property: &str) -> String {
    systemctl_value(&[
        "show",
        "revault_key_server.service",
        &format!("--property={property}"),
        "--value",
    ])
    .unwrap_or_else(|| "not available".to_string())
}

fn service_can_read_path(path: &str) -> &'static str {
    service_path_access("-r", path)
}

fn service_can_write_path(path: &str) -> &'static str {
    service_path_access("-w", path)
}

fn service_path_access(flag: &str, path: &str) -> &'static str {
    let output = Command::new("runuser")
        .args(["-u", USER, "--", "test", flag, path])
        .output();
    match output {
        Ok(output) if output.status.success() => "YES",
        Ok(output) => {
            let detail = String::from_utf8_lossy(&output.stderr);
            if detail.contains("may only be used by root") {
                "RUN DOCTOR WITH SUDO"
            } else {
                "NO"
            }
        }
        Err(_) => "UNAVAILABLE",
    }
}

fn require_root(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    if unsafe { libc_geteuid() } != 0 {
        let executable = std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "revault_key_server".to_string());
        return Err(format!(
            "`{command}` requires administrator privileges. Run:\n  {}",
            sudo_command(&executable, command)
        )
        .into());
    }
    Ok(())
}

fn sudo_command(executable: &str, command: &str) -> String {
    format!("sudo {executable} {command}")
}

#[cfg(unix)]
unsafe fn libc_geteuid() -> u32 {
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    unsafe { geteuid() }
}

#[cfg(not(unix))]
unsafe fn libc_geteuid() -> u32 {
    1
}

fn ensure_user() -> Result<bool, Box<dyn std::error::Error>> {
    // `id` writes "no such user" to stderr when the account is absent. That
    // is the normal first-install path, not an error, so capture the result
    // instead of leaking the diagnostic to the administrator.
    if Command::new("id")
        .arg("-u")
        .arg(USER)
        .output()?
        .status
        .success()
    {
        return Ok(false);
    }
    run(
        "useradd",
        &[
            "--system",
            "--user-group",
            "--home-dir",
            STATE_DIR,
            "--shell",
            "/usr/sbin/nologin",
            USER,
        ],
    )?;
    Ok(true)
}

fn run(command: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if detail.is_empty() {
            format!("exit status {}", output.status)
        } else {
            detail
        };
        return Err(format!("{command} failed: {detail}").into());
    }
    Ok(())
}

fn systemctl_value(args: &[&str]) -> Option<String> {
    let output = Command::new("systemctl").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn chown_path(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    chown_owner_group(&format!("{USER}:{USER}"), path)
}

fn chown_config() -> Result<(), Box<dyn std::error::Error>> {
    chown_owner_group(&format!("root:{USER}"), CONFIG_PATH)
}

fn chown_owner_group(owner_group: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    run("chown", &[owner_group, path])
}

#[cfg(unix)]
fn set_dir_permissions(path: &str, mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_dir_permissions(_path: &str, _mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(unix)]
fn set_file_permissions(path: &str, mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_file_permissions(_path: &str, _mode: u32) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn default_config() -> &'static str {
    "bind_addr = \"0.0.0.0:8089\"\n\
state_dir = \"/var/lib/revault-key-server\"\n\
server_id = 0\n\
cluster_id = \"default\"\n\
public_url = \"https://keypublish.revault.onepub.dev/v1/publish\"\n\
topology_version = 1\n\
origin_epoch = 1\n\
verification_ttl_seconds = 1800\n\
default_receive_ttl_seconds = 7200\n\
max_receive_ttl_seconds = 7200\n\
max_payload_bytes = 8192\n\
max_receives_per_publish = 8\n\
rate_limit_per_minute = 120\n\
rate_limit_burst = 40\n\
smtp_host = \"smtp.gmail.com\"\n\
smtp_port = 587\n\
smtp_username = \"\"\n\
smtp_password = \"\"\n\
smtp_from = \"\"\n\
smtp_tls = \"starttls\"\n\
smtp_timeout_seconds = 30\n\
verification_email_subject = \"Verify your reVault publish\"\n\
verification_email_template = \"Verify {email} for this reVault publish:\\n\\n{verification_url}\\n\\nThis link expires in 30 minutes.\"\n\
verification_email_rate_limit_per_hour = 5\n\
verification_email_ip_rate_limit_per_hour = 30\n\
\n\
[[topology_server]]\n\
id = 0\n\
url = \"https://keypublish.revault.onepub.dev/v1/publish\"\n\
status = \"active\"\n\
\n\
[[route]]\n\
owner = 0\n\
primary = 0\n\
failover = []\n"
}

fn unit_file(binary: &str) -> String {
    format!(
        "[Unit]
Description=reVault Key Rendezvous Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={USER}
Group={USER}
ExecStart={binary} run --config {CONFIG_PATH}
Restart=always
RestartSec=2
Environment=REVAULT_KEY_SERVER_LOG={LOG_FILE}
StandardOutput=append:{LOG_FILE}
StandardError=append:{LOG_FILE}
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
PrivateDevices=true
RestrictSUIDSGID=true
LockPersonality=true
MemoryDenyWriteExecute=true
ReadWritePaths={STATE_DIR} {CACHE_DIR} {LOG_DIR}
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
"
    )
}

#[cfg(test)]
mod tests {
    use super::{default_config, sudo_command, unit_file, CONFIG_PATH, LOG_FILE};

    #[test]
    fn privileged_command_uses_the_actual_binary_path() {
        assert_eq!(
            sudo_command("/home/alice/.cargo/bin/revault_key_server", "install"),
            "sudo /home/alice/.cargo/bin/revault_key_server install"
        );
    }

    #[test]
    fn unit_runs_from_config_and_restarts_on_boot_failures() {
        let unit = unit_file("/usr/local/bin/revault_key_server");
        assert!(unit.contains("ExecStart=/usr/local/bin/revault_key_server run --config "));
        assert!(unit.contains(CONFIG_PATH));
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains(&format!("Environment=REVAULT_KEY_SERVER_LOG={LOG_FILE}")));
        assert!(unit.contains(&format!("StandardOutput=append:{LOG_FILE}")));
        assert!(unit.contains(&format!("StandardError=append:{LOG_FILE}")));
        assert!(unit.contains("WantedBy=multi-user.target"));
        assert!(!unit.contains("--state-dir"));
    }

    #[test]
    fn default_config_includes_public_single_server_topology() {
        let config = default_config();
        assert!(config.contains("server_id = 0"));
        assert!(
            config.contains("public_url = \"https://keypublish.revault.onepub.dev/v1/publish\"")
        );
        assert!(config.contains("[[topology_server]]"));
        assert!(config.contains("url = \"https://keypublish.revault.onepub.dev/v1/publish\""));
        assert!(config.contains("[[route]]"));
        assert!(config.contains("primary = 0"));
    }
}
