#![cfg(target_os = "linux")]
mod common;
use common::{CommandTestExt, TestTempDir};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn command(dir: &Path, args: &[&str], terminal: bool) -> Command {
    use std::os::unix::fs::PermissionsExt;
    let agent_dir = common::agent_socket_dir(&dir.join("agent"));
    fs::set_permissions(&agent_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let binary = std::env::var("REVAULT_HEADLESS_LBX")
        .or_else(|_| {
            option_env!("CARGO_BIN_EXE_lbx")
                .map(str::to_owned)
                .ok_or(std::env::VarError::NotPresent)
        })
        .expect("set REVAULT_HEADLESS_LBX when testing an installed CLI");
    let mut command = if terminal {
        // util-linux script supplies a real terminal to exercise the public
        // confirmation path; no test-only prompt or capability overrides.
        let mut command = Command::new("script");
        let quoted = std::iter::once(binary.as_str())
            .chain(args.iter().copied())
            .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
            .collect::<Vec<_>>()
            .join(" ");
        command.args(["-q", "-e", "-c", &quoted, "/dev/null"]);
        command
    } else {
        let mut command = Command::new(&binary);
        command.args(args);
        command
    };
    command
        .current_dir(dir)
        .env("LOCKBOX_AGENT_CONFIG", dir.join("config.toml"))
        .env("LOCKBOX_VAULT_DIR", dir.join("vault"))
        .env("LOCKBOX_SESSION_AGENT_DIR", agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", dir.join("agent.log"))
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
        .env(
            "LOCKBOX_VAULT_PASSWORD",
            "synthetic headless init test passphrase",
        )
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent/revault-test-session-bus",
        )
        .env(
            "DBUS_SYSTEM_BUS_ADDRESS",
            "unix:path=/nonexistent/revault-test-system-bus",
        )
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("LOCKBOX_AGENT_PREVENT_SLEEP")
        .env_remove("LOCKBOX_AGENT_TERMINATE_ON_SUSPEND")
        .env_remove("LOCKBOX_KEY")
        .env_remove("LOCKBOX_PASSWORD");
    command
}

fn run(dir: &Path, args: &[&str]) -> Output {
    command(dir, args, false).test_output().unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn success(output: Output) -> Output {
    assert!(output.status.success(), "{}", text(&output));
    output
}

fn verify_persistence(dir: &Path) {
    success(run(dir, &["vault", "init", "--verify"]));
    success(run(dir, &["persisted.lbox", "create"]));
    success(run(dir, &["persisted.lbox", "open"]));
    let content = b"headless init persisted bytes\0\xff";
    fs::write(dir.join("input.bin"), content).unwrap();
    success(run(
        dir,
        &["persisted.lbox", "add", "input.bin", "--to", "/payload"],
    ));
    success(run(dir, &["session", "stop"]));
    success(run(dir, &["persisted.lbox", "open"]));
    assert_eq!(
        success(run(dir, &["persisted.lbox", "cat", "/payload"])).stdout,
        content
    );
    success(run(dir, &["session", "stop"]));
}

#[test]
fn noninteractive_defaults_refuse_without_creating_a_vault_or_changing_policy() {
    let temp = TestTempDir::new("headless-init-defaults");
    let dir = temp.path();
    let status = text(&success(run(dir, &["session", "protection", "status"])));
    assert!(
        status.contains("prevent-sleep: configured=true, next-start=true"),
        "{status}"
    );
    assert!(
        status.contains("terminate-on-suspend: configured=true, next-start=true"),
        "{status}"
    );
    let output = run(dir, &["vault", "init"]);
    assert!(!output.status.success(), "{}", text(&output));
    assert!(
        text(&output).contains("requested sleep inhibition unavailable"),
        "{}",
        text(&output)
    );
    assert!(
        text(&output).contains("Interactive confirmation is unavailable"),
        "{}",
        text(&output)
    );
    assert!(!dir.join("config.toml").exists());
    assert!(!dir.join("vault/local-vault.lbox").exists());
    success(run(dir, &["session", "stop"]));
}

#[test]
fn persisted_controls_are_independent_and_explicit_in_automation() {
    let temp = TestTempDir::new("headless-init-controls");
    let dir = temp.path();
    let initial = "# preserved\nagent.prevent_sleep = true\nagent.terminate_on_suspend = true\nagent.idle_timeout_seconds = 321\n";
    fs::write(dir.join("config.toml"), initial).unwrap();
    for setting in ["prevent-sleep", "terminate-on-suspend"] {
        let refused = run(dir, &["session", "protection", setting, "off"]);
        assert!(!refused.status.success());
        assert!(text(&refused).contains("--yes"));
        assert_eq!(
            fs::read_to_string(dir.join("config.toml")).unwrap(),
            initial
        );
    }
    success(run(
        dir,
        &["session", "protection", "prevent-sleep", "off", "--yes"],
    ));
    let status = text(&success(run(dir, &["session", "protection", "status"])));
    assert!(
        status.contains("prevent-sleep: configured=false, next-start=false"),
        "{status}"
    );
    assert!(
        status.contains("terminate-on-suspend: configured=true, next-start=true"),
        "{status}"
    );
    // Sleep opt-out cannot silently waive the separate suspend requirement.
    let refused = run(dir, &["vault", "init"]);
    assert!(!refused.status.success());
    assert!(
        text(&refused).contains("requested suspend protection is unavailable"),
        "{}",
        text(&refused)
    );
    assert!(!dir.join("vault/local-vault.lbox").exists());
    success(run(
        dir,
        &[
            "session",
            "protection",
            "terminate-on-suspend",
            "off",
            "--yes",
        ],
    ));
    success(run(dir, &["vault", "init"]));
    verify_persistence(dir);
    let config = fs::read_to_string(dir.join("config.toml")).unwrap();
    assert!(config.contains("# preserved"));
    assert!(config.contains("agent.idle_timeout_seconds = 321"));
    let overridden = command(dir, &["session", "protection", "status"], false)
        .env("LOCKBOX_AGENT_PREVENT_SLEEP", "true")
        .test_output()
        .unwrap();
    assert!(text(&success(overridden))
        .contains("prevent-sleep: configured=false, next-start=true (environment override)"));
    assert_eq!(fs::read_to_string(dir.join("config.toml")).unwrap(), config);
    success(run(dir, &["session", "protection", "prevent-sleep", "on"]));
    let status = text(&success(run(dir, &["session", "protection", "status"])));
    assert!(status.contains("prevent-sleep: configured=true, next-start=true"));
    assert!(status.contains("terminate-on-suspend: configured=false, next-start=false"));
    success(run(
        dir,
        &["session", "protection", "terminate-on-suspend", "on"],
    ));
    let status = text(&success(run(dir, &["session", "protection", "status"])));
    assert!(status.contains("terminate-on-suspend: configured=true, next-start=true"));
}

#[test]
fn interactive_init_decline_and_default_no_preserve_configuration() {
    for answer in [b"n\n".as_slice(), b"\n".as_slice()] {
        let temp = TestTempDir::new("headless-init-decline");
        let dir = temp.path();
        // Explicit independent choice: this host has no suspend watcher.
        let initial = "agent.terminate_on_suspend = false\n# retain me\n";
        fs::write(dir.join("config.toml"), initial).unwrap();
        let noninteractive = run(dir, &["vault", "init"]);
        assert!(!noninteractive.status.success());
        assert!(
            text(&noninteractive).contains("Interactive confirmation is unavailable"),
            "{}",
            text(&noninteractive)
        );
        assert_eq!(
            fs::read_to_string(dir.join("config.toml")).unwrap(),
            initial
        );
        let output = command(dir, &["vault", "init"], true)
            .test_output_with_input(answer)
            .unwrap();
        assert!(!output.status.success(), "{}", text(&output));
        assert!(
            text(&output).contains("Remember this setting and continue? [y/N]"),
            "{}",
            text(&output)
        );
        assert_eq!(
            fs::read_to_string(dir.join("config.toml")).unwrap(),
            initial
        );
        assert!(!dir.join("vault/local-vault.lbox").exists());
        success(run(dir, &["session", "stop"]));
    }
}

#[test]
fn accepting_sleep_choice_does_not_waive_enabled_termination_protection() {
    let temp = TestTempDir::new("headless-init-independent");
    let dir = temp.path();
    fs::write(
        dir.join("config.toml"),
        "agent.terminate_on_suspend = true\n",
    )
    .unwrap();
    let output = command(dir, &["vault", "init"], true)
        .test_output_with_input(b"y\n")
        .unwrap();
    assert!(!output.status.success(), "{}", text(&output));
    assert!(
        text(&output).contains("Remember this setting and continue? [y/N]"),
        "{}",
        text(&output)
    );
    assert!(
        text(&output).contains("requested suspend protection is unavailable"),
        "{}",
        text(&output)
    );
    assert_eq!(
        fs::read_to_string(dir.join("config.toml")).unwrap(),
        "agent.prevent_sleep = false\nagent.terminate_on_suspend = true\n"
    );
    assert!(!dir.join("vault/local-vault.lbox").exists());
    success(run(dir, &["session", "stop"]));
}

#[test]
fn interactive_init_acceptance_remembers_only_sleep_choice_and_reopens_content() {
    let temp = TestTempDir::new("headless-init-accept");
    let dir = temp.path();
    fs::write(
        dir.join("config.toml"),
        "agent.terminate_on_suspend = false\n# retain me\n",
    )
    .unwrap();
    let output = success(
        command(dir, &["vault", "init"], true)
            .test_output_with_input(b"y\n")
            .unwrap(),
    );
    assert!(text(&output).contains("Remember this setting and continue? [y/N]"));
    assert_eq!(
        fs::read_to_string(dir.join("config.toml")).unwrap(),
        "agent.prevent_sleep = false\nagent.terminate_on_suspend = false\n# retain me\n"
    );
    verify_persistence(dir);
}
