use std::io::Write;
use std::process::{Command, Output, Stdio};

struct Fixture {
    root: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        Self {
            root: tempfile::Builder::new()
                .prefix("lbx-pw-")
                .tempdir()
                .unwrap(),
        }
    }
    fn command(&self, server: bool, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lockbox"));
        command
            .current_dir(self.root.path())
            .args(args)
            .env_remove("LOCKBOX_KEY")
            .env_remove("LOCKBOX_PASSWORD")
            .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
            .env(
                "LOCKBOX_VAULT_DIR",
                self.root
                    .path()
                    .join(if server { "server-vault" } else { "vault" }),
            )
            .env(
                "LOCKBOX_SESSION_AGENT_DIR",
                self.root.path().join(if server { "sa" } else { "oa" }),
            )
            .env(
                "LOCKBOX_SESSION_AGENT_LOG",
                self.root
                    .path()
                    .join(if server { "sa.log" } else { "oa.log" }),
            );
        if server {
            command.env_remove("LOCKBOX_VAULT_PASSWORD");
        } else {
            command.env("LOCKBOX_VAULT_PASSWORD", "password profile test vault");
        }
        command
    }
    fn run(&self, server: bool, args: &[&str]) -> Output {
        self.command(server, args).output().unwrap()
    }
    fn ok(&self, server: bool, args: &[&str]) -> Vec<u8> {
        let out = self.run(server, args);
        assert!(
            out.status.success(),
            "{args:?}: {} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    }
    fn fail(&self, server: bool, args: &[&str]) {
        assert!(!self.run(server, args).status.success(), "{args:?}");
    }
    fn open_server(&self, file: &str) {
        self.ok(true, &["shared.lbox", "open", "--password-file", file]);
    }
    fn slots(&self) -> usize {
        let output = self.ok(
            false,
            &["shared.lbox", "access", "list", "--format", "json"],
        );
        serde_json::Deserializer::from_slice(&output)
            .into_iter::<serde_json::Value>()
            .map(Result::unwrap)
            .count()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for server in [false, true] {
            let _ = self.run(server, &["session", "stop"]);
        }
    }
}

#[test]
fn password_profile_lifecycle_and_vaultless_server_access() {
    let f = Fixture::new();
    f.ok(false, &["vault", "init"]);
    f.ok(
        false,
        &[
            "vault",
            "profile",
            "create",
            "production-server",
            "--password",
        ],
    );
    let password = f.ok(
        false,
        &["vault", "profile", "password", "production-server"],
    );
    assert_eq!(password.len(), 65);
    assert!(password[..64].iter().all(u8::is_ascii_hexdigit));
    let list = f.ok(false, &["vault", "profile", "list", "--format", "json"]);
    assert!(String::from_utf8_lossy(&list).contains("password"));
    assert!(!list.windows(64).any(|bytes| bytes == &password[..64]));
    f.fail(
        false,
        &[
            "vault",
            "profile",
            "create",
            "production-server",
            "--password",
        ],
    );
    f.fail(false, &["vault", "profile", "create", "production-server"]);
    assert_eq!(
        password,
        f.ok(
            false,
            &["vault", "profile", "password", "production-server"]
        )
    );
    f.fail(false, &["vault", "profile", "password", "default"]);
    f.fail(
        false,
        &[
            "vault",
            "profile",
            "export",
            "server.pub",
            "--name",
            "production-server",
        ],
    );
    f.ok(
        false,
        &[
            "vault",
            "profile",
            "password",
            "production-server",
            "--output",
            "credential",
        ],
    );
    assert_eq!(
        std::fs::read(f.root.path().join("credential")).unwrap(),
        password[..64]
    );
    f.fail(
        false,
        &[
            "vault",
            "profile",
            "password",
            "production-server",
            "--output",
            "credential",
        ],
    );
    f.ok(
        false,
        &[
            "vault",
            "profile",
            "password",
            "production-server",
            "--output",
            "credential",
            "--overwrite",
        ],
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(f.root.path().join("credential"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    f.ok(false, &["shared.lbox", "create"]);
    f.ok(false, &["shared.lbox", "open"]);
    let content = b"server payload\0\xff\n";
    std::fs::write(f.root.path().join("payload"), content).unwrap();
    f.ok(false, &["shared.lbox", "add", "payload"]);
    f.ok(
        false,
        &["shared.lbox", "access", "grant", "production-server"],
    );
    assert_eq!(f.slots(), 2);
    f.ok(
        false,
        &[
            "shared.lbox",
            "access",
            "grant",
            "profile:production-server",
        ],
    );
    assert_eq!(f.slots(), 2);
    f.open_server("credential");
    assert!(!f.root.path().join("server-vault/local-vault.lbox").exists());
    assert_eq!(f.ok(true, &["shared.lbox", "cat", "payload"]), content);
    f.ok(true, &["shared.lbox", "extract", "payload", "extracted"]);
    assert_eq!(
        std::fs::read(f.root.path().join("extracted")).unwrap(),
        content
    );
    f.ok(true, &["shared.lbox", "close"]);
    let mut child = f
        .command(true, &["shared.lbox", "open", "--password-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&password).unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    assert_eq!(f.ok(true, &["shared.lbox", "cat", "payload"]), content);

    f.ok(
        false,
        &[
            "vault",
            "profile",
            "backup",
            "profile.backup",
            "--name",
            "production-server",
        ],
    );
    f.ok(
        false,
        &["vault", "profile", "remove", "production-server", "--force"],
    );
    f.fail(
        false,
        &["vault", "profile", "password", "production-server"],
    );
    f.ok(false, &["vault", "profile", "restore", "profile.backup"]);
    assert_eq!(
        password,
        f.ok(
            false,
            &["vault", "profile", "password", "production-server"]
        )
    );
    f.fail(false, &["vault", "profile", "restore", "profile.backup"]);
    f.ok(
        false,
        &[
            "vault",
            "profile",
            "restore",
            "profile.backup",
            "--overwrite",
        ],
    );
    assert_eq!(
        password,
        f.ok(
            false,
            &["vault", "profile", "password", "production-server"]
        )
    );

    f.ok(
        false,
        &["vault", "profile", "create", "replacement", "--password"],
    );
    f.ok(
        false,
        &[
            "vault",
            "profile",
            "password",
            "replacement",
            "--output",
            "replacement-credential",
        ],
    );
    f.ok(false, &["shared.lbox", "access", "grant", "replacement"]);
    assert_eq!(f.slots(), 3);
    f.ok(
        false,
        &["shared.lbox", "access", "revoke", "production-server"],
    );
    f.ok(false, &["shared.lbox", "open"]);
    assert_eq!(f.slots(), 2);
    f.ok(true, &["shared.lbox", "close"]);
    f.fail(
        true,
        &["shared.lbox", "open", "--password-file", "credential"],
    );
    f.open_server("replacement-credential");
    assert_eq!(f.ok(true, &["shared.lbox", "cat", "payload"]), content);
    assert_eq!(f.ok(false, &["shared.lbox", "cat", "payload"]), content);
    f.ok(
        false,
        &["password-only.lbox", "create", "--for", "replacement"],
    );
    f.fail(
        false,
        &["password-only.lbox", "access", "revoke", "replacement"],
    );
}
