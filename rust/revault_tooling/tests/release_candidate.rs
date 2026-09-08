//! Exercise the public release CLI with real local Git state and a Rust GitHub
//! subprocess fixture. Remote CI cannot be created by a local regression test.
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
use tempfile::{tempdir, TempDir};
fn mock() -> &'static Path {
    static MOCK: OnceLock<(TempDir, PathBuf)> = OnceLock::new();
    &MOCK
        .get_or_init(|| {
            let dir = tempdir().unwrap();
            let exe = dir
                .path()
                .join(format!("gh{}", std::env::consts::EXE_SUFFIX));
            assert!(Command::new("rustc")
                .args([
                    "--edition",
                    "2021",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mock_gh.rs"),
                    "-o"
                ])
                .arg(&exe)
                .status()
                .unwrap()
                .success());
            (dir, exe)
        })
        .1
}
struct Fixture {
    dir: TempDir,
    sha: String,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempdir().unwrap();
        let root = dir.path();
        git(root, &["init"]);
        git(root, &["config", "user.name", "Release Test"]);
        git(root, &["config", "user.email", "release@example.invalid"]);
        fs::write(root.join("source"), "reviewed source").unwrap();
        git(root, &["add", "source"]);
        git(root, &["commit", "-m", "Reviewed source"]);
        let sha = git(root, &["rev-parse", "HEAD"]);
        let artifacts =
            json!([{"id":1,"name":"native-linux","digest":format!("sha256:{}","b".repeat(64))}]);
        write(
            root,
            "candidate.json",
            json!({"schema":1,"repository":"org/repo","run_id":42,"source_sha":sha,"source_ref":"release-candidates/test","scope":"all","cli_version":"1.2.3","bindings_version":"2.3.4","artifacts":artifacts}),
        );
        write(
            root,
            "artifacts.json",
            json!({"artifacts":[{"id":1,"name":"native-linux","digest":format!("sha256:{}","b".repeat(64)),"expired":false}]}),
        );
        write(
            root,
            "run.json",
            json!({"conclusion":"success","status":"completed","event":"workflow_dispatch","path":".github/workflows/release-candidate.yml","head_sha":sha}),
        );
        write(root, "runs.json", json!({"workflow_runs":[]}));
        write(root, "ref.json", json!({"object":{"sha":sha}}));
        // Fixture transport state is deliberately outside Git's tracked source state.
        fs::write(root.join(".git/info/exclude"), "*\n").unwrap();
        write(
            root,
            ".git/revault-release-candidate.json",
            json!({"repository":"org/repo","candidate":42,"promotion":null}),
        );
        Self { dir, sha }
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        let mut paths = vec![mock().parent().unwrap().to_path_buf()];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
        Command::new(env!("CARGO_BIN_EXE_revault-tool"))
            .args([if args.first() == Some(&"candidate") {
                "internal"
            } else {
                "release"
            }])
            .args(args)
            .current_dir(self.dir.path())
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("RELEASE_FIXTURE", self.dir.path())
            .env("GITHUB_SHA", &self.sha)
            .output()
            .unwrap()
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.dir.path().join("calls")).unwrap_or_default()
    }
    fn runs(&self, conclusion: Value, status: &str) {
        write(
            self.dir.path(),
            "runs.json",
            json!({"workflow_runs":[{"id":99,"display_title":"publish all 42","head_sha":self.sha,"head_branch":"release-candidates/test","conclusion":conclusion,"status":status}]}),
        );
    }
}
fn git(root: &Path, args: &[&str]) -> String {
    let r = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    String::from_utf8(r.stdout).unwrap().trim().into()
}
fn write(root: &Path, name: &str, v: Value) {
    fs::write(root.join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
fn succeeds(result: std::process::Output) {
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
fn remembered_candidate_resumes_failed_publication_without_preparing() {
    let f = Fixture::new();
    f.runs(json!("failure"), "completed");
    succeeds(f.run(&["publish"]));
    let calls = f.calls();
    assert!(calls.contains("run rerun 99 --repo org/repo --failed"));
    assert!(calls.contains("run watch 99"));
    assert!(!calls.contains("POST"));
    assert!(!calls.contains("prepare"));
    let state: Value = serde_json::from_slice(
        &fs::read(f.dir.path().join(".git/revault-release-candidate.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(state["promotion"], 99);
}
#[test]
fn published_candidate_is_idempotent() {
    let f = Fixture::new();
    f.runs(json!("success"), "completed");
    succeeds(f.run(&["publish"]));
    assert!(!f.calls().contains("rerun"));
    assert!(!f.calls().contains("watch"));
    assert!(!f.calls().contains("POST"));
}
#[test]
fn failed_or_expired_candidates_never_publish() {
    for expired in [false, true] {
        let f = Fixture::new();
        if expired {
            write(
                f.dir.path(),
                "artifacts.json",
                json!({"artifacts":[{"name":"native-linux","expired":true}]}),
            );
        } else {
            let mut r: Value =
                serde_json::from_slice(&fs::read(f.dir.path().join("run.json")).unwrap()).unwrap();
            r["conclusion"] = json!("failure");
            write(f.dir.path(), "run.json", r);
        }
        assert!(!f.run(&["publish"]).status.success());
        assert!(!f.calls().contains("POST"));
        assert!(!f.calls().contains("watch"));
    }
}
#[test]
fn changed_checkout_requires_explicit_old_candidate() {
    let f = Fixture::new();
    f.runs(json!("success"), "completed");
    fs::write(f.dir.path().join("source"), "new source").unwrap();
    assert!(!f.run(&["publish"]).status.success());
    succeeds(f.run(&["publish", "--candidate", "42"]));
}
#[test]
fn changed_artifact_or_branch_refuses_publication() {
    for field in ["artifact", "branch"] {
        let f = Fixture::new();
        if field == "artifact" {
            write(f.dir.path(), "artifacts.json", json!({"artifacts":[]}));
        } else {
            write(
                f.dir.path(),
                "ref.json",
                json!({"object":{"sha":"b".repeat(40)}}),
            );
        }
        assert!(!f.run(&["publish"]).status.success());
        assert!(!f.calls().contains("POST"));
    }
}
#[test]
fn status_prints_recoverable_candidate_and_publish_command() {
    let f = Fixture::new();
    let r = f.run(&["status"]);
    assert!(r.status.success());
    let text = String::from_utf8(r.stdout).unwrap();
    assert!(text.contains("Candidate: 42"));
    assert!(text.contains("release publish --candidate 42"));
}

#[test]
fn new_publication_dispatches_only_the_frozen_candidate() {
    let f = Fixture::new();
    write(
        f.dir.path(),
        "dispatch-response.json",
        json!({"workflow_runs":[{"id":99,"display_title":"publish all 42","head_sha":f.sha,"head_branch":"release-candidates/test","status":"queued","conclusion":null}]}),
    );
    succeeds(f.run(&["publish"]));
    let body: Value =
        serde_json::from_slice(&fs::read(f.dir.path().join("dispatch.json")).unwrap()).unwrap();
    assert_eq!(body["ref"], "release-candidates/test");
    assert_eq!(body["inputs"]["candidate"], "42");
    assert_eq!(body["inputs"]["mode"], "publish");
    assert_eq!(f.calls().matches("--method POST").count(), 1);
    assert!(f.calls().contains("run watch 99"));
}

#[test]
fn coordinator_preserves_trusted_publication_workflow_entry_points() {
    for (action, workflow) in [
        ("promote-cli", "revault_cli-v-release.yml"),
        ("promote-bindings", "bindings-native-release.yml"),
    ] {
        let f = Fixture::new();
        write(
            f.dir.path(),
            "dispatch-response.json",
            json!({"workflow_runs":[{"id":99,"display_title":"promote candidate 42","head_sha":f.sha,"status":"queued","conclusion":null}]}),
        );
        succeeds(f.run(&[
            "candidate",
            action,
            "--candidate",
            "42",
            "--scope",
            "all",
            "--cli-version",
            "1.2.3",
            "--bindings-version",
            "2.3.4",
        ]));
        assert!(f
            .calls()
            .contains(&format!("actions/workflows/{workflow}/dispatches")));
        let body: Value =
            serde_json::from_slice(&fs::read(f.dir.path().join("dispatch.json")).unwrap()).unwrap();
        assert_eq!(body["inputs"]["promotion_run_id"], "42");
        assert_eq!(body["ref"], "release-candidates/test");
        assert!(f.calls().contains("run watch 99"));
    }
}

#[test]
fn status_watch_reattaches_without_dispatching() {
    let f = Fixture::new();
    succeeds(f.run(&["status", "--watch"]));
    let calls = f.calls();
    assert!(calls.contains("run watch 42"));
    assert!(!calls.contains("POST"));
    assert!(!calls.contains("rerun"));
}
