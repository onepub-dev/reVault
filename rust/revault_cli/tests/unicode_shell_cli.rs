//! Actual native-shell argument and CLI completion-protocol coverage.
//! Run explicitly with REVAULT_UNICODE_SHELLS=bash,zsh,fish,powershell,elvish.
//! Linux and macOS CI cover all five shells.
//! Windows CI covers PowerShell, Git Bash and Elvish; Zsh/Fish are Unix-only.
//! Both backend vectors and sourced registrations are tested. Registered
//! providers are invoked without rendering an interactive TAB menu; Zsh's
//! presentation callback records the actual provider's candidate handoff.
mod common;

use common::{agent_socket_dir, CommandTestExt, TestTempDir};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct ShellFixture {
    root: TestTempDir,
    shell: String,
}

impl ShellFixture {
    fn locale() -> OsString {
        std::env::var_os("REVAULT_UNICODE_LOCALE").unwrap_or_else(|| "C.UTF-8".into())
    }

    fn configure(&self, command: &mut Command) {
        let mut paths = vec![PathBuf::from(env!("CARGO_BIN_EXE_lockbox"))
            .parent()
            .unwrap()
            .to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        command
            .current_dir(self.root.path())
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("LOCKBOX_VAULT_DIR", self.root.path().join("vault"))
            .env(
                "LOCKBOX_SESSION_AGENT_DIR",
                agent_socket_dir(&self.root.path().join("agent")),
            )
            .env(
                "LOCKBOX_SESSION_AGENT_LOG",
                self.root.path().join("agent.log"),
            )
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env(
                "LOCKBOX_VAULT_PASSWORD",
                "synthetic unicode shell vault password",
            )
            .env("LOCKBOX_KEY", "synthetic-unicode-shell-content-key")
            .env("LC_ALL", Self::locale())
            .env_remove("LOCKBOX_PASSWORD")
            .env_remove("LOCKBOX_KEY_FILE")
            .env_remove("COMPLETE");
    }

    fn direct(&self, args: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
        self.configure(&mut command);
        command.args(args);
        let output = command.test_output().unwrap();
        assert!(
            output.status.success(),
            "{} {args:?}: {output:?}",
            self.shell
        );
        output
    }

    fn shell_executable(&self) -> OsString {
        let key = format!("REVAULT_UNICODE_{}", self.shell.to_uppercase());
        std::env::var_os(key).unwrap_or_else(|| {
            if self.shell == "powershell" {
                "pwsh".into()
            } else {
                self.shell.clone().into()
            }
        })
    }

    fn report_environment(&self) {
        let executable = self.shell_executable();
        let mut version = Command::new(&executable);
        self.configure(&mut version);
        version.arg("--version");
        let output = version.test_output().unwrap();
        assert!(
            output.status.success(),
            "shell version {executable:?}: {output:?}"
        );
        eprintln!("Native shell evidence: os={} arch={} shell={} executable={executable:?} LC_ALL={:?} inherited_LANG={:?}\n{}{}",
            std::env::consts::OS, std::env::consts::ARCH, self.shell, Self::locale(), std::env::var_os("LANG"),
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        if self.shell == "powershell" {
            let mut environment = Command::new(executable);
            self.configure(&mut environment);
            environment.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                "[System.Runtime.InteropServices.RuntimeInformation]::OSDescription; [System.Globalization.CultureInfo]::CurrentCulture.Name; [Console]::InputEncoding.WebName; [Console]::OutputEncoding.WebName"]);
            let output = environment.test_output().unwrap();
            assert!(
                output.status.success(),
                "PowerShell environment: {output:?}"
            );
            eprintln!(
                "PowerShell OS / culture / input encoding / output encoding:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }

    fn shell_command(&self, cli: &str) -> Command {
        let binary = if cfg!(windows) {
            "lockbox.exe"
        } else {
            "lockbox"
        };
        let native = format!("{binary} {cli}");
        self.shell_program(&native)
    }

    fn shell_program(&self, native: &str) -> Command {
        let mut command = Command::new(self.shell_executable());
        self.configure(&mut command);
        let zsh_native = std::env::var_os("REVAULT_UNICODE_ZSH_MODULE_PATH").map(|path| {
            command.env("REVAULT_UNICODE_ZSH_MODULE_PATH", path);
            format!("module_path=(\"$REVAULT_UNICODE_ZSH_MODULE_PATH\" $module_path)\n{native}")
        });
        match self.shell.as_str() {
            "bash" => {
                command.args(["--noprofile", "--norc", "-c", native]);
            }
            "zsh" => {
                command.args(["-f", "-c", zsh_native.as_deref().unwrap_or(native)]);
            }
            "fish" => {
                command.args(["--no-config", "-c", native]);
            }
            "powershell" => {
                command.args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    &format!("{native}; exit $LASTEXITCODE"),
                ]);
            }
            "elvish" => {
                command.args(["-c", native]);
            }
            other => panic!("undeclared shell: {other}"),
        }
        command
    }

    fn registered_completion(&self, registration: &[u8], prefix: &str, expected: &str) {
        // These generated product artifacts are the same registrations a user
        // installs. The snippets below invoke each shell's registered provider.
        let filename = if self.shell == "powershell" {
            "registered.ps1"
        } else {
            "registered.completion"
        };
        std::fs::write(self.root.path().join(filename), registration).unwrap();
        let program = match self.shell.as_str() {
            "bash" => format!(r#"source ./registered.completion
COMP_WORDS=(lockbox {prefix})
COMP_CWORD=1
COMP_TYPE=9
registration=$(complete -p lockbox)
provider=${{registration#* -F }}
provider=${{provider%% *}}
"$provider" lockbox {prefix} lockbox
printf '%s\n' "${{COMPREPLY[@]}}""#),
            "fish" => format!("source ./registered.completion; complete -C 'lockbox {prefix}'"),
            "powershell" => format!(". ./registered.ps1; (TabExpansion2 'lockbox {prefix}' {}).CompletionMatches | ForEach-Object {{ $_.CompletionText }}", "lockbox ".len() + prefix.encode_utf16().count()),
            "zsh" => format!(r#"autoload -Uz compinit
compinit -D
source ./registered.completion
_describe() {{ local array_name=$3; print -rl -- "${{(@P)array_name}}"; }}
words=(lockbox {prefix})
CURRENT=2
${{_comps[lockbox]}}"#),
            "elvish" => {
                self.registered_elvish_completion(registration, prefix, expected);
                return;
            }
            other => panic!("undeclared shell: {other}"),
        };
        let output = self.shell_program(&program).test_output().unwrap();
        assert!(
            output.status.success(),
            "{} registered completion: {output:?}",
            self.shell
        );
        self.assert_candidates(&output.stdout, prefix, expected);
        eprintln!(
            "Registered completion passed: {} {prefix:?} -> {expected:?}",
            self.shell
        );
    }

    fn registered_elvish_completion(&self, registration: &[u8], prefix: &str, expected: &str) {
        // Elvish only creates edit: when stdin is a terminal. Give it an
        // isolated PTY/console, invoke its installed callback from RC, and exit
        // before the editor reads any input. No mocked edit namespace is used.
        let mut rc = String::from_utf8(registration.to_vec()).unwrap();
        rc.push_str(&format!("\n$edit:completion:arg-completer[lockbox] lockbox {prefix} | each {{|candidate| echo $candidate }}\nexit 0\n"));
        std::fs::write(self.root.path().join("registered.elv"), rc).unwrap();
        let mut command = Command::new(self.shell_executable());
        self.configure(&mut command);
        command
            .args(["-rc", "registered.elv"])
            .env("XDG_STATE_HOME", self.root.path().join("elvish-state"))
            .env("XDG_CONFIG_HOME", self.root.path().join("elvish-config"));
        let output = output_with_terminal_stdin(&mut command);
        assert!(
            output.status.success(),
            "Elvish registered completion: {output:?}"
        );
        self.assert_candidates(&output.stdout, prefix, expected);
        eprintln!("Registered completion passed: elvish {prefix:?} -> {expected:?}");
    }

    fn assert_candidates(&self, output: &[u8], words: &str, expected: &str) {
        let text = String::from_utf8_lossy(output);
        let values: Vec<_> = text
            .lines()
            .map(|line| line.split(['\t', ':']).next().unwrap().trim())
            .collect();
        if self.shell == "fish" {
            // Fish filters completed values against the original input again.
            // Preserve those exact typed bytes in its presentation; persistence
            // and alias resolution must still use the canonical NFC identity.
            let raw_prefix = words.split_whitespace().last().unwrap();
            let candidate = values.iter().copied().find(|value| {
                value.starts_with(raw_prefix)
                    && revault_vault_api::normalize_lockbox_alias_prefix(value) == expected
            }).unwrap_or_else(|| panic!("fish: {words}: expected raw prefix {raw_prefix:?} and NFC {expected:?}, got {text:?}"));
            let selector = if candidate.starts_with("a@") {
                candidate.to_string()
            } else {
                format!("a@{candidate}")
            };
            let canonical_selector = if expected.starts_with("a@") {
                expected.to_string()
            } else {
                format!("a@{expected}")
            };
            let inserted = self.through_shell(&format!("{selector} variable get VALUE"));
            let canonical = self.direct(&[&canonical_selector, "variable", "get", "VALUE"]);
            assert_eq!(
                inserted.stdout, canonical.stdout,
                "completed Fish candidate must resolve the same persisted bytes"
            );
            return;
        }
        assert!(
            values.contains(&expected),
            "{}: {words}: expected {expected:?}, got {text:?}",
            self.shell
        );
    }

    fn through_shell(&self, cli: &str) -> Output {
        let output = self.shell_command(cli).test_output().unwrap();
        assert!(output.status.success(), "{}: {cli}: {output:?}", self.shell);
        output
    }

    fn complete(&self, words: &str, index: usize, expected: &str) {
        // Invoke the same CLI endpoint and vectors that generated registration
        // scripts invoke, through the actual shell, with unquoted Unicode input.
        let output = self
            .shell_command(&format!("-- lockbox {words}"))
            .env("COMPLETE", &self.shell)
            .env("_CLAP_COMPLETE_INDEX", index.to_string())
            .env("_CLAP_COMPLETE_COMP_TYPE", "9")
            .env("_CLAP_COMPLETE_SPACE", "true")
            .env("_CLAP_IFS", "\n")
            .test_output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} completion: {output:?}",
            self.shell
        );
        assert!(
            output.stderr.is_empty(),
            "{} completion: {output:?}",
            self.shell
        );
        self.assert_candidates(&output.stdout, words, expected);
    }

    fn aliases(&self) -> Vec<String> {
        let output = self.direct(&["vault", "lockbox", "alias", "list", "--format", "json"]);
        // The existing public print_records contract emits this exact sentinel
        // for zero rows even in JSON mode. Every nonempty row remains strict JSON.
        if output.stdout == b"empty\n" {
            return Vec::new();
        }
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| {
                let row: serde_json::Value = serde_json::from_str(line).unwrap();
                row["alias"].as_str().unwrap().to_string()
            })
            .collect()
    }
}

struct TerminalInput {
    input: std::fs::File,
    #[cfg(unix)]
    _master: std::fs::File,
    #[cfg(windows)]
    allocated_console: bool,
}

impl TerminalInput {
    fn new() -> Self {
        #[cfg(unix)]
        {
            use std::os::fd::FromRawFd;
            let mut master = -1;
            let mut slave = -1;
            // openpty initializes two new owned descriptors; convert each once.
            let result = unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            assert_eq!(result, 0, "openpty: {}", std::io::Error::last_os_error());
            Self {
                input: unsafe { std::fs::File::from_raw_fd(slave) },
                _master: unsafe { std::fs::File::from_raw_fd(master) },
            }
        }
        #[cfg(windows)]
        {
            // A CI test process may lack a console. An existing console remains
            // attached; only a console allocated here is released by Drop.
            let allocated_console =
                unsafe { windows_sys::Win32::System::Console::AllocConsole() } != 0;
            Self {
                input: std::fs::File::open("CONIN$").expect("native console stdin for Elvish"),
                allocated_console,
            }
        }
    }
}

#[cfg(windows)]
impl Drop for TerminalInput {
    fn drop(&mut self) {
        if self.allocated_console {
            unsafe {
                windows_sys::Win32::System::Console::FreeConsole();
            }
        }
    }
}

fn output_with_terminal_stdin(command: &mut Command) -> Output {
    use std::io::{Read, Seek};
    use std::time::{Duration, Instant};
    let terminal = TerminalInput::new();
    let mut stdout = tempfile::tempfile().unwrap();
    let mut stderr = tempfile::tempfile().unwrap();
    command
        .stdin(Stdio::from(terminal.input.try_clone().unwrap()))
        .stdout(Stdio::from(stdout.try_clone().unwrap()))
        .stderr(Stdio::from(stderr.try_clone().unwrap()));
    let mut child = command.spawn().unwrap();
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            stderr.rewind().unwrap();
            let mut diagnostic = String::new();
            stderr.read_to_string(&mut diagnostic).unwrap();
            panic!("registered completion exceeded 30s: {diagnostic}");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    stdout.rewind().unwrap();
    stderr.rewind().unwrap();
    let mut out = Vec::new();
    let mut err = Vec::new();
    stdout.read_to_end(&mut out).unwrap();
    stderr.read_to_end(&mut err).unwrap();
    Output {
        status,
        stdout: out,
        stderr: err,
    }
}

#[test]
#[ignore = "requires the explicit native-shell matrix; exercised on Linux and Windows in CI"]
fn unicode_alias_roundtrips_and_completion_through_native_shells() {
    let shells = std::env::var("REVAULT_UNICODE_SHELLS")
        .expect("set the required shell matrix; missing shells must fail");
    assert!(!shells.is_empty());
    for shell in shells.split(',') {
        let fixture = ShellFixture {
            root: TestTempDir::new("unicode-shell"),
            shell: shell.to_string(),
        };
        fixture.report_environment();
        fixture.through_shell("vault init");
        let registration = fixture.through_shell(&format!("completion generate --shell {shell}"));
        assert!(!registration.stdout.is_empty());
        // All aliases are intentionally unquoted in actual shell command text.
        // Include decomposed Latin, Indic marks, CJK, Greek, Arabic digits, and
        // allowed ASCII separators; expected names are canonical NFC spellings.
        let names = [
            ("cafe\u{301}-équipe", "café-équipe"),
            ("कक्षा", "कक्षा"),
            ("東京", "東京"),
            ("_δοκιμή-٢.1", "_δοκιμή-٢.1"),
            ("١٢", "١٢"),
        ];
        for (index, (input, canonical)) in names.iter().enumerate() {
            let archive = format!("./case-{index}.lbox");
            fixture.through_shell(&format!("{archive} create --alias {input}"));
            let content = format!("persisted-shell-payload-{index}");
            fixture.through_shell(&format!("a@{input} variable set VALUE {content}"));
            let stored = fixture.direct(&[&format!("a@{canonical}"), "variable", "get", "VALUE"]);
            assert_eq!(stored.stdout, format!("{content}\n").as_bytes());
            assert_eq!(fixture.aliases(), vec![canonical.to_string()]);
            // Both explicit selectors and alias operands normalize prefixes.
            fixture.complete(&format!("a@{input}"), 1, &format!("a@{canonical}"));
            fixture.complete(&format!("vault lockbox alias remove {input}"), 5, canonical);
            if index == 0 {
                fixture.complete("a@cafe\u{301}", 1, "a@café-équipe");
                fixture.complete("vault lockbox alias remove cafe\u{301}", 5, "café-équipe");
                fixture.complete("vault lockbox alias set cafe\u{301}", 5, "café-équipe");
                fixture.complete("./new.lbox create --alias cafe\u{301}", 4, "café-équipe");
                fixture.registered_completion(&registration.stdout, "a@caf", "a@café-équipe");
                fixture.registered_completion(
                    &registration.stdout,
                    "a@cafe\u{301}",
                    "a@café-équipe",
                );
            }
            fixture.through_shell(&format!("vault lockbox alias remove {input}"));
            assert!(fixture.aliases().is_empty());
            let still_stored = fixture.direct(&[&archive, "variable", "get", "VALUE"]);
            assert_eq!(still_stored.stdout, stored.stdout);
        }
    }
}
