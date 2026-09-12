mod common;
use common::{CommandTestExt, TestTempDir};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(dir: &Path, encrypted: bool, args: &[&str], input: &[u8]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lockbox"));
    cmd.current_dir(dir)
        .args(args)
        .env("LOCKBOX_VAULT_DIR", dir.join("vault"))
        .env("LOCKBOX_VAULT_PASSWORD", "compaction fixture vault")
        .env("LOCKBOX_PLATFORM_SECRET_STORE", "disabled")
        .env(
            "LOCKBOX_SESSION_AGENT_DIR",
            common::agent_socket_dir(&dir.join("agent")),
        )
        .env("LOCKBOX_SESSION_AGENT_LOG", dir.join("agent.log"))
        .env("LOCKBOX_ADD_PROGRESS", "off")
        .env_remove("LOCKBOX_KEY");
    if encrypted {
        cmd.env("LOCKBOX_KEY", "compaction fixture key");
    }
    cmd.test_output_with_input(input).unwrap()
}
fn ok(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
fn noise(len: usize) -> Vec<u8> {
    let mut state = 0x123456789abcdefu64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

#[test]
fn compaction_lifecycle_preserves_bytes_and_reclaims_space_in_every_format_mode() {
    let temp = TestTempDir::new("compact-modes");
    let dir = temp.path();
    ok(run(dir, false, &["vault", "init"], &[]));
    let large = noise(3 * 1024 * 1024);
    fs::write(dir.join("large.bin"), &large).unwrap();
    for encrypted in [false, true] {
        for signed in [false, true] {
            for compressed in [false, true] {
                let archive = format!("{encrypted}-{signed}-{compressed}.lbox");
                let call = |args: &[&str]| ok(run(dir, encrypted, args, &[]));
                call(&[
                    &archive,
                    "create",
                    "--encryption",
                    if encrypted {
                        "chacha20-poly1305"
                    } else {
                        "none"
                    },
                    "--signing",
                    if signed { "owner" } else { "none" },
                    "--compression",
                    if compressed { "zstd" } else { "none" },
                ]);
                call(&[&archive, "doctor", "compact"]);
                let empty = call(&[&archive, "list"]);
                call(&[&archive, "doctor", "compact"]);
                assert_eq!(call(&[&archive, "list"]), empty);
                fs::write(dir.join("keep.txt"), b"original survivor").unwrap();
                call(&[&archive, "add", "keep.txt", "--to", "/keep.txt"]);
                call(&[&archive, "add", "large.bin", "--to", "/discard.bin"]);
                call(&[&archive, "doctor", "compact"]);
                assert_eq!(call(&[&archive, "cat", "/discard.bin"]), large);
                fs::write(dir.join("keep.txt"), b"replacement survivor").unwrap();
                call(&[
                    &archive,
                    "add",
                    "--overwrite",
                    "keep.txt",
                    "--to",
                    "/keep.txt",
                ]);
                call(&[&archive, "remove", "--force", "/discard.bin"]);
                assert!(
                    !run(dir, encrypted, &[&archive, "cat", "/discard.bin"], &[])
                        .status
                        .success()
                );
                let before = fs::metadata(dir.join(&archive)).unwrap().len();
                call(&[&archive, "doctor", "compact"]);
                let after = fs::metadata(dir.join(&archive)).unwrap().len();
                assert!(
                    after + 2 * 1024 * 1024 < before,
                    "{archive}: {before} -> {after}"
                );
                assert_eq!(
                    call(&[&archive, "cat", "/keep.txt"]),
                    b"replacement survivor"
                );
                assert!(
                    !run(dir, encrypted, &[&archive, "cat", "/discard.bin"], &[])
                        .status
                        .success()
                );
                call(&[&archive, "doctor", "--deep"]);
                call(&[&archive, "doctor", "compact"]);
                assert_eq!(fs::metadata(dir.join(&archive)).unwrap().len(), after);
                assert_eq!(
                    call(&[&archive, "cat", "/keep.txt"]),
                    b"replacement survivor"
                );
                let status = String::from_utf8(call(&[&archive, "doctor"])).unwrap();
                assert!(status.contains(if encrypted {
                    "encryption: chacha20-poly1305"
                } else {
                    "encryption: none"
                }));
                assert!(status.contains(if signed {
                    "signing: owner"
                } else {
                    "signing: none"
                }));
                assert!(status.contains(if compressed {
                    "compression: zstd"
                } else {
                    "compression: none"
                }));
            }
        }
    }
}

#[test]
fn compaction_preserves_secret_records_and_mirror_lifecycle() {
    let temp = TestTempDir::new("compact-records");
    let dir = temp.path();
    ok(run(dir, false, &["vault", "init"], &[]));
    let call = |args: &[&str]| ok(run(dir, true, args, &[]));
    call(&["box.lbox", "create"]);
    call(&["box.lbox", "description", "set", "compaction fixture"]);
    ok(run(
        dir,
        true,
        &[
            "box.lbox", "variable", "set", "--secret", "--stdin", "TOKEN",
        ],
        b"variable secret",
    ));
    call(&[
        "box.lbox",
        "form",
        "define",
        "login",
        "--name",
        "Login",
        "--field",
        "password:secret:Password",
    ]);
    call(&[
        "box.lbox", "form", "add", "/account", "--type", "login", "--name", "Account",
    ]);
    ok(run(
        dir,
        true,
        &[
            "box.lbox", "form", "set", "--secret", "--stdin", "/account", "password",
        ],
        b"form secret",
    ));
    fs::create_dir(dir.join("house")).unwrap();
    fs::write(dir.join("house/a"), b"first").unwrap();
    call(&[
        "box.lbox", "mirror", "home", "create", "--from", "house", "--to", "/house",
    ]);
    call(&["box.lbox", "mirror", "home", "update", "--force"]);
    call(&["box.lbox", "mirror", "home", "update"]);
    fs::write(dir.join("house/a"), b"replacement").unwrap();
    fs::write(dir.join("house/b"), b"addition").unwrap();
    call(&["box.lbox", "mirror", "home", "update", "--force"]);
    call(&["box.lbox", "doctor", "compact"]);
    assert_eq!(call(&["box.lbox", "cat", "/house/a"]), b"replacement");
    assert_eq!(call(&["box.lbox", "cat", "/house/b"]), b"addition");
    fs::remove_file(dir.join("house/b")).unwrap();
    call(&["box.lbox", "mirror", "home", "update", "--force"]);
    call(&["box.lbox", "doctor", "compact"]);
    assert!(!run(dir, true, &["box.lbox", "cat", "/house/b"], &[])
        .status
        .success());
    assert_eq!(
        call(&["box.lbox", "variable", "get", "--secret", "TOKEN"]),
        b"variable secret\n"
    );
    assert_eq!(
        call(&["box.lbox", "form", "get", "--secret", "/account", "password"]),
        b"form secret\n"
    );
    assert!(String::from_utf8(call(&["box.lbox", "description", "get"]))
        .unwrap()
        .contains("compaction fixture"));
    call(&["box.lbox", "mirror", "home", "update"]);
    call(&["box.lbox", "doctor", "--deep"]);
    let original = fs::read(dir.join("box.lbox")).unwrap();
    assert!(!run(
        dir,
        true,
        &["--key", "incorrect key", "box.lbox", "doctor", "compact"],
        &[]
    )
    .status
    .success());
    assert_eq!(fs::read(dir.join("box.lbox")).unwrap(), original);
    assert!(!run(dir, true, &["doctor", "compact"], &[]).status.success());
}
