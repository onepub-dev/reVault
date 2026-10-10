mod common;
use common::CommandTestExt;

use common::TestTempDir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(bin: &str, args: &[&str], vault_dir: &std::path::Path) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", vault_dir)
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .env_remove("COMPLETE")
        .test_output()
        .unwrap()
}

const COMPLETION_BINARIES: [&str; 4] = ["lockbox", "lbx", "lbxv", "lbxx"];
const COMPLETION_SHELLS: [&str; 5] = ["bash", "zsh", "fish", "powershell", "elvish"];

fn run_in_home(bin: &str, args: &[String], vault_dir: &Path, home: &Path) -> std::process::Output {
    Command::new(bin)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", vault_dir)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env_remove("LOCKBOX_VAULT_PASSWORD")
        .env_remove("COMPLETE")
        .test_output()
        .unwrap()
}

fn default_completion_paths(shell: &str, home: &Path) -> Vec<PathBuf> {
    if shell == "powershell" {
        let profile_dir = if cfg!(windows) {
            home.join("Documents/PowerShell")
        } else {
            home.join(".config/powershell")
        };
        return vec![profile_dir.join("Microsoft.PowerShell_profile.ps1")];
    }

    COMPLETION_BINARIES
        .iter()
        .map(|binary| match shell {
            "bash" => home
                .join(".local/share/bash-completion/completions")
                .join(binary),
            "zsh" => home
                .join(".local/share/zsh/site-functions")
                .join(format!("_{binary}")),
            "fish" => home
                .join(".config/fish/completions")
                .join(format!("{binary}.fish")),
            "elvish" => home
                .join(".config/elvish/lib")
                .join(format!("{binary}.elv")),
            _ => unreachable!("all supported shells are listed above"),
        })
        .collect()
}

fn completion_command(subcommand: &str, shell: &str, path: Option<&Path>) -> Vec<String> {
    let mut args = vec![
        "completion".to_string(),
        subcommand.to_string(),
        "--shell".to_string(),
        shell.to_string(),
    ];
    if let Some(path) = path {
        args.push("--path".to_string());
        args.push(path.to_string_lossy().into_owned());
    }
    args
}

fn assert_all_binary_registrations(script: &str, shell: &str) {
    for binary in COMPLETION_BINARIES {
        assert_shell_registration(script, shell, binary);
    }
}

fn assert_shell_registration(script: &str, shell: &str, binary: &str) {
    let token = match shell {
        "bash" => format!("-F _clap_complete_{binary} {binary}"),
        "zsh" => format!("compdef _clap_dynamic_completer_{binary} {binary}"),
        "fish" => format!("complete --keep-order --exclusive --command {binary} "),
        "powershell" => format!("-CommandName {binary} -ScriptBlock"),
        "elvish" => format!("arg-completer[{binary}]"),
        _ => unreachable!("all supported shells are listed above"),
    };
    assert!(
        script.contains(&token),
        "{shell} completion omitted registration for {binary} ({token}): {script}"
    );
}

#[test]
fn completion_default_install_refreshes_and_uninstalls_all_commands() {
    let bin = env!("CARGO_BIN_EXE_lbx");
    let lbxv = env!("CARGO_BIN_EXE_lbxv");
    let lbxx = env!("CARGO_BIN_EXE_lbxx");
    let temp = TestTempDir::new("completion-all-defaults");
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let vault_dir = temp.path().join("vault");

    for shell in COMPLETION_SHELLS {
        let paths = default_completion_paths(shell, &home);
        let powershell_profile = paths.first().unwrap();
        if shell == "powershell" {
            fs::create_dir_all(powershell_profile.parent().unwrap()).unwrap();
            fs::write(
                powershell_profile,
                "# user profile before\nWrite-Output user\n",
            )
            .unwrap();
        }

        let install_args = completion_command("install", shell, None);
        let installed = run_in_home(bin, &install_args, &vault_dir, &home);
        assert!(installed.status.success(), "{shell}: {installed:?}");

        if shell == "powershell" {
            let first = fs::read_to_string(powershell_profile).unwrap();
            assert!(first.starts_with("# user profile before\nWrite-Output user\n"));
            assert_eq!(
                first.matches("# BEGIN revault dynamic completion").count(),
                1
            );
            assert_eq!(first.matches("# END revault dynamic completion").count(), 1);
            assert_all_binary_registrations(&first, shell);

            let repeated = run_in_home(lbxv, &install_args, &vault_dir, &home);
            assert!(repeated.status.success(), "{shell}: {repeated:?}");
            assert_eq!(fs::read_to_string(powershell_profile).unwrap(), first);

            fs::write(powershell_profile, first.replace("lbxv", "stale-lbxv")).unwrap();
            let refreshed = run_in_home(lbxx, &install_args, &vault_dir, &home);
            assert!(refreshed.status.success(), "{shell}: {refreshed:?}");
            let refreshed = fs::read_to_string(powershell_profile).unwrap();
            assert!(!refreshed.contains("stale-lbxv"));
            assert_eq!(
                refreshed
                    .matches("# BEGIN revault dynamic completion")
                    .count(),
                1
            );
            assert_eq!(
                refreshed
                    .matches("# END revault dynamic completion")
                    .count(),
                1
            );
            assert!(refreshed.starts_with("# user profile before\nWrite-Output user\n"));
            assert_all_binary_registrations(&refreshed, shell);

            let uninstall_args = completion_command("uninstall", shell, None);
            let uninstalled = run_in_home(bin, &uninstall_args, &vault_dir, &home);
            assert!(uninstalled.status.success(), "{shell}: {uninstalled:?}");
            assert_eq!(
                fs::read_to_string(powershell_profile).unwrap(),
                "# user profile before\nWrite-Output user\n"
            );
            let repeated = run_in_home(bin, &uninstall_args, &vault_dir, &home);
            assert!(repeated.status.success(), "{shell}: {repeated:?}");
            assert_eq!(
                fs::read_to_string(powershell_profile).unwrap(),
                "# user profile before\nWrite-Output user\n"
            );
            continue;
        }

        let first_contents: Vec<_> = paths
            .iter()
            .map(|path| fs::read_to_string(path).unwrap())
            .collect();
        for (binary, script) in COMPLETION_BINARIES.iter().zip(&first_contents) {
            assert_shell_registration(script, shell, binary);
        }

        let repeated = run_in_home(lbxv, &install_args, &vault_dir, &home);
        assert!(repeated.status.success(), "{shell}: {repeated:?}");
        for (path, script) in paths.iter().zip(&first_contents) {
            assert_eq!(
                fs::read_to_string(path).unwrap(),
                *script,
                "{shell}: {path:?}"
            );
            fs::write(path, "outdated completion content\n").unwrap();
        }

        let refreshed = run_in_home(lbxx, &install_args, &vault_dir, &home);
        assert!(refreshed.status.success(), "{shell}: {refreshed:?}");
        for (path, binary) in paths.iter().zip(COMPLETION_BINARIES) {
            let script = fs::read_to_string(path).unwrap();
            assert!(!script.contains("outdated completion content"));
            assert_shell_registration(&script, shell, binary);
        }

        let uninstall_args = completion_command("uninstall", shell, None);
        let uninstalled = run_in_home(bin, &uninstall_args, &vault_dir, &home);
        assert!(uninstalled.status.success(), "{shell}: {uninstalled:?}");
        assert!(
            paths.iter().all(|path| !path.exists()),
            "{shell}: {paths:?}"
        );
        let repeated = run_in_home(bin, &uninstall_args, &vault_dir, &home);
        assert!(repeated.status.success(), "{shell}: {repeated:?}");
        assert!(
            paths.iter().all(|path| !path.exists()),
            "{shell}: {paths:?}"
        );
    }
}

#[test]
fn completion_generate_and_explicit_path_cover_all_commands_for_every_shell() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-all-explicit");
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let vault_dir = temp.path().join("vault");

    for shell in COMPLETION_SHELLS {
        let generated_path = temp
            .path()
            .join("generated")
            .join(format!("{shell}.script"));
        let generate_args = vec![
            "completion".to_string(),
            "generate".to_string(),
            "--shell".to_string(),
            shell.to_string(),
            "--output".to_string(),
            generated_path.to_string_lossy().into_owned(),
        ];
        let generated = run_in_home(bin, &generate_args, &vault_dir, &home);
        assert!(generated.status.success(), "{shell}: {generated:?}");
        let generated_script = fs::read_to_string(&generated_path).unwrap();
        assert_all_binary_registrations(&generated_script, shell);

        let explicit_path = temp.path().join("explicit").join(format!("{shell}.script"));
        let install_args = completion_command("install", shell, Some(&explicit_path));
        let installed = run_in_home(bin, &install_args, &vault_dir, &home);
        assert!(installed.status.success(), "{shell}: {installed:?}");
        assert_eq!(
            fs::read_to_string(&explicit_path).unwrap(),
            generated_script
        );

        let uninstall_args = completion_command("uninstall", shell, Some(&explicit_path));
        let uninstalled = run_in_home(bin, &uninstall_args, &vault_dir, &home);
        assert!(uninstalled.status.success(), "{shell}: {uninstalled:?}");
        assert!(!explicit_path.exists(), "{shell}: {explicit_path:?}");
        let repeated = run_in_home(bin, &uninstall_args, &vault_dir, &home);
        assert!(repeated.status.success(), "{shell}: {repeated:?}");
    }
}

#[test]
fn bash_completion_sources_and_registers_all_four_commands() {
    let explicit_bash = std::env::var_os("REVAULT_TEST_BASH");
    let mut candidates = explicit_bash
        .clone()
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if explicit_bash.is_none() {
        // Windows' System32 bash may be a WSL launcher without a distribution.
        // Prefer Git Bash, which can source the generated Windows-host paths.
        if cfg!(windows) {
            for variable in ["ProgramFiles", "ProgramW6432", "LOCALAPPDATA"] {
                if let Some(root) = std::env::var_os(variable) {
                    candidates.push(PathBuf::from(&root).join("Git/bin/bash.exe"));
                    candidates.push(PathBuf::from(root).join("Programs/Git/bin/bash.exe"));
                }
            }
        }
        candidates.push(PathBuf::from("bash"));
    }
    let bash = candidates.into_iter().find(|path| {
        Command::new(path)
            .args(["-c", "test -n \"$BASH_VERSION\""])
            .output()
            .is_ok_and(|output| output.status.success())
    });
    let Some(bash) = bash else {
        assert!(
            explicit_bash.is_none() && std::env::var_os("CI").is_none(),
            "Bash must be runnable in CI or when REVAULT_TEST_BASH is configured"
        );
        eprintln!("Skipping shell registration check: no runnable Bash installation");
        return;
    };

    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-bash-registrations");
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let vault_dir = temp.path().join("vault");
    let path = temp.path().join("lockbox-completions");
    let args = completion_command("install", "bash", Some(&path));
    let installed = run_in_home(bin, &args, &vault_dir, &home);
    assert!(installed.status.success(), "{installed:?}");

    let path = path.to_string_lossy().replace('\\', "/");
    let registrations = Command::new(bash)
        .args([
            "-c",
            "source \"$1\" && complete -p lockbox && complete -p lbx && complete -p lbxv && complete -p lbxx",
            "completion-test",
            &path,
        ])
        .output()
        .unwrap();
    assert!(registrations.status.success(), "{registrations:?}");
    let registrations = String::from_utf8_lossy(&registrations.stdout);
    for binary in COMPLETION_BINARIES {
        assert!(registrations.contains(binary), "{registrations}");
    }
}

#[test]
fn completion_generation_supports_shell_override_and_install_uninstall() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-install");
    let vault_dir = temp.path().join("vault");
    let output_path = temp.path().join("completion").join("lockbox.bash");

    let generated = run(
        bin,
        &[
            "completion",
            "generate",
            "--shell",
            "bash",
            "--output",
            output_path.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(generated.status.success(), "{generated:?}");
    let script = fs::read_to_string(&output_path).unwrap();
    assert!(script.contains("_clap_complete_lockbox"));
    assert!(script.contains("COMPLETE=\"bash\""));

    let installed = temp.path().join("custom").join("lockbox.fish");
    let output = run(
        bin,
        &[
            "completion",
            "install",
            "--shell",
            "fish",
            "--path",
            installed.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(installed.exists());

    let output = run(
        bin,
        &[
            "completion",
            "uninstall",
            "--shell",
            "fish",
            "--path",
            installed.to_str().unwrap(),
        ],
        &vault_dir,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(!installed.exists());
}

#[test]
fn locked_vault_completion_falls_back_without_prompt_or_diagnostics() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-locked");
    let vault_dir = temp.path().join("vault");

    let output = Command::new(bin)
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "4")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "vault", "profile", "create", ""])
        .test_output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

#[test]
fn dynamic_completion_completes_target_lockboxes_and_add_sources() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-paths");
    let vault_dir = temp.path().join("vault");
    fs::write(temp.path().join("secrets.lbox"), b"test").unwrap();
    fs::write(temp.path().join("README.md"), b"test").unwrap();
    fs::write(temp.path().join("source.txt"), b"test").unwrap();
    fs::create_dir(temp.path().join("source-dir")).unwrap();
    fs::create_dir(temp.path().join("unrelated-dir")).unwrap();

    let target = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", ""])
        .test_output()
        .unwrap();
    assert!(target.status.success(), "{target:?}");
    assert!(
        String::from_utf8_lossy(&target.stdout).contains("secrets.lbox"),
        "{target:?}"
    );
    let target_stdout = String::from_utf8_lossy(&target.stdout);
    assert!(target_stdout.contains("create"), "{target:?}");
    assert!(target_stdout.contains("--help"), "{target:?}");
    assert!(!target_stdout.contains("README.md"), "{target:?}");
    assert!(!target_stdout.contains("source.txt"), "{target:?}");
    assert!(!target_stdout.contains("source-dir"), "{target:?}");
    assert!(!target_stdout.contains("unrelated-dir"), "{target:?}");

    let after_lockbox = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "secrets.lbox", ""])
        .test_output()
        .unwrap();
    assert!(after_lockbox.status.success(), "{after_lockbox:?}");
    let after_lockbox_stdout = String::from_utf8_lossy(&after_lockbox.stdout);
    assert!(after_lockbox_stdout.contains("add"), "{after_lockbox:?}");
    assert!(after_lockbox_stdout.contains("--help"), "{after_lockbox:?}");
    assert!(
        !after_lockbox_stdout.contains("README.md"),
        "{after_lockbox:?}"
    );
    assert!(
        !after_lockbox_stdout.contains("source.txt"),
        "{after_lockbox:?}"
    );

    let source = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "3")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "secrets.lbox", "add", "sou"])
        .test_output()
        .unwrap();
    assert!(source.status.success(), "{source:?}");
    assert!(
        String::from_utf8_lossy(&source.stdout).contains("source.txt"),
        "{source:?}"
    );

    let mirror_action = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "3")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "secrets.lbox", "mirror", ""])
        .test_output()
        .unwrap();
    assert!(mirror_action.status.success(), "{mirror_action:?}");
    assert!(
        String::from_utf8_lossy(&mirror_action.stdout).contains("create"),
        "{mirror_action:?}"
    );
}

#[test]
fn dynamic_completion_navigates_open_lockbox_paths() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-open-paths");
    let vault_dir = temp.path().join("vault");
    let nested = temp.path().join("nested").join("deeper");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("secrets.lbox"), b"test").unwrap();

    let directory = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "nested/d"])
        .test_output()
        .unwrap();
    assert!(directory.status.success(), "{directory:?}");
    assert!(
        String::from_utf8_lossy(&directory.stdout).contains("nested/deeper/"),
        "{directory:?}"
    );

    let lockbox = Command::new(bin)
        .current_dir(temp.path())
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "nested/deeper/sec"])
        .test_output()
        .unwrap();
    assert!(lockbox.status.success(), "{lockbox:?}");
    assert!(
        String::from_utf8_lossy(&lockbox.stdout).contains("nested/deeper/secrets.lbox"),
        "{lockbox:?}"
    );
}

#[test]
fn dynamic_completion_reads_vault_names_without_exposing_signing_material() {
    let bin = env!("CARGO_BIN_EXE_lockbox");
    let temp = TestTempDir::new("completion-dynamic");
    let vault_dir = temp.path().join("vault");
    let agent_dir = temp.path().join("agent");
    let agent_log = temp.path().join("agent.log");
    let password = "completion-test-vault-password";

    let init = Command::new(bin)
        .args(["vault", "init"])
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .test_output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");

    let create = Command::new(bin)
        .args(["vault", "profile", "create", "alice"])
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .test_output()
        .unwrap();
    assert!(create.status.success(), "{create:?}");

    let log_len_before_completion = fs::metadata(&agent_log).map(|m| m.len()).unwrap_or(0);

    let output = Command::new(bin)
        .env("LOCKBOX_VAULT_DIR", &vault_dir)
        .env("LOCKBOX_VAULT_PASSWORD", password)
        .env("LOCKBOX_SESSION_AGENT_DIR", &agent_dir)
        .env("LOCKBOX_SESSION_AGENT_LOG", &agent_log)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "4")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .args(["--", "lockbox", "vault", "profile", "history", "al"])
        .test_output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("alice"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("LBX1SPRV"));
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);

    let agent_log_bytes = fs::read(&agent_log).unwrap_or_default();
    let completion_log = &agent_log_bytes[log_len_before_completion as usize..];
    assert!(
        !String::from_utf8_lossy(completion_log).contains("owner-signing:"),
        "completion must not request an owner-signing key from the agent"
    );
}
