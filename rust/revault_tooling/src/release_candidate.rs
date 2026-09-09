//! A release is a successful CI run plus its immutable artifacts, never a local test result.
use crate::Result;
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{IsTerminal, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const WORKFLOW: &str = "release-candidate.yml";
const MANIFEST: &str = "release-candidate";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ValueEnum, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Cli,
    Bindings,
    All,
}
impl Scope {
    fn name(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Bindings => "bindings",
            Self::All => "all",
        }
    }
}
#[derive(Args)]
#[command(
    long_about = "Prepare a release candidate without publishing. Requires a clean checkout, Git, Cargo and an authenticated GitHub CLI. Updates versions, dependencies and release notes, commits the release, pushes an isolated candidate ref, and waits for CI. Prints and remembers the candidate ID. Use release publish to promote it. See rust/revault_tooling/RELEASE.txt."
)]
pub struct Prepare {
    #[arg(value_enum, default_value = "all")]
    scope: Scope,
    /// CLI version: an exact version, patch, minor or major. Prompts when omitted.
    #[arg(long)]
    cli_version: Option<String>,
    /// Binding version: an exact version, patch, minor or major. Prompts when omitted.
    #[arg(long)]
    bindings_version: Option<String>,
    /// Version for a single-target release (cli or bindings).
    #[arg(long, conflicts_with_all = ["cli_version", "bindings_version"])]
    version: Option<String>,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
}
#[derive(Args)]
pub struct Selection {
    /// Successful prepare run ID. Defaults to this checkout's remembered candidate.
    #[arg(long)]
    pub candidate: Option<u64>,
    /// Print complete job logs instead of the condensed failure excerpts.
    #[arg(long)]
    pub full: bool,
    #[arg(long, default_value = ".")]
    pub repository: PathBuf,
}
#[derive(Args)]
pub struct StatusSelection {
    #[command(flatten)]
    pub selection: Selection,
    /// Reattach to live progress without starting another workflow.
    #[arg(long)]
    pub watch: bool,
}
#[derive(Args)]
pub struct Ci {
    #[arg(value_parser = ["seal", "verify", "tags", "promote-cli", "promote-bindings"])]
    action: String,
    #[arg(long)]
    candidate: Option<u64>,
    #[arg(long, value_enum, default_value = "all")]
    scope: Scope,
    #[arg(long, default_value = "")]
    cli_version: String,
    #[arg(long, default_value = "")]
    bindings_version: String,
    #[arg(long, default_value = ".")]
    repository: PathBuf,
    #[arg(long, default_value = "candidate.json")]
    output: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Artifact {
    id: u64,
    name: String,
    digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Candidate {
    schema: u32,
    repository: String,
    run_id: u64,
    source_sha: String,
    source_ref: String,
    scope: Scope,
    cli_version: String,
    bindings_version: String,
    artifacts: Vec<Artifact>,
}
#[derive(Serialize, Deserialize)]
struct Remembered {
    repository: String,
    candidate: u64,
    promotion: Option<u64>,
}
struct GitHub {
    root: PathBuf,
    repo: String,
}
impl GitHub {
    fn new(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let repo = output(
            &root,
            "gh",
            &[
                "repo",
                "view",
                "--json",
                "nameWithOwner",
                "--jq",
                ".nameWithOwner",
            ],
        )?;
        Ok(Self { root, repo })
    }
    fn api(&self, suffix: &str) -> Result<Value> {
        Ok(serde_json::from_str(&output(
            &self.root,
            "gh",
            &["api", &format!("repos/{}/{}", self.repo, suffix)],
        )?)?)
    }
    fn post(&self, suffix: &str, body: Value) -> Result<()> {
        let mut child = Command::new("gh")
            .current_dir(&self.root)
            .args([
                "api",
                "--method",
                "POST",
                &format!("repos/{}/{}", self.repo, suffix),
                "--input",
                "-",
            ])
            .stdin(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("missing gh stdin")?
            .write_all(serde_json::to_string(&body)?.as_bytes())?;
        if !child.wait()?.success() {
            return Err("GitHub request failed".into());
        }
        Ok(())
    }
    fn artifacts(&self, run: u64) -> Result<Vec<Artifact>> {
        let mut found = Vec::new();
        for page in 1.. {
            let v = self.api(&format!(
                "actions/runs/{run}/artifacts?per_page=100&page={page}"
            ))?;
            let rows = v["artifacts"].as_array().ok_or("missing artifacts")?;
            for row in rows {
                if row["name"] == MANIFEST {
                    continue;
                }
                if row["expired"] != false {
                    return Err("Candidate artifacts expired; prepare a new candidate".into());
                }
                let digest = string(row, "digest")?;
                if !digest.starts_with("sha256:") || digest.len() != 71 {
                    return Err("Artifact lacks a SHA-256 digest".into());
                }
                found.push(Artifact {
                    id: row["id"].as_u64().ok_or("invalid artifact ID")?,
                    name: string(row, "name")?,
                    digest,
                });
            }
            if rows.len() < 100 {
                break;
            }
        }
        found.sort_by_key(|a| a.id);
        Ok(found)
    }
    fn load(&self, id: u64) -> Result<Candidate> {
        let run = self.api(&format!("actions/runs/{id}"))?;
        if run["conclusion"] != "success"
            || run["event"] != "workflow_dispatch"
            || run["path"] != format!(".github/workflows/{WORKFLOW}")
        {
            return Err("Candidate must be a successful release prepare workflow run".into());
        }
        let temp = tempfile::tempdir()?;
        run_command(
            &self.root,
            "gh",
            &[
                "run",
                "download",
                &id.to_string(),
                "--repo",
                &self.repo,
                "--name",
                MANIFEST,
                "--dir",
                &temp.path().to_string_lossy(),
            ],
        )?;
        let candidate: Candidate =
            serde_json::from_slice(&fs::read(temp.path().join("candidate.json"))?)?;
        validate(
            &candidate,
            &self.repo,
            id,
            &string(&run, "head_sha")?,
            &self.artifacts(id)?,
        )?;
        Ok(candidate)
    }
    fn dispatch(&self, source_ref: &str, inputs: Value, title: &str) -> Result<u64> {
        // A unique immutable branch plus mode-specific run title disambiguates concurrent dispatches.
        self.post(
            &format!("actions/workflows/{WORKFLOW}/dispatches"),
            json!({"ref":source_ref,"inputs":inputs}),
        )?;
        for _ in 0..60 {
            let runs = self.api(&format!(
                "actions/workflows/{WORKFLOW}/runs?event=workflow_dispatch&per_page=100"
            ))?;
            if let Some(run) = runs["workflow_runs"]
                .as_array()
                .ok_or("missing runs")?
                .iter()
                .find(|r| r["head_branch"] == source_ref && r["display_title"] == title)
            {
                return run["id"].as_u64().ok_or_else(|| "invalid run ID".into());
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        Err("Dispatch accepted but run ID was not found. Use release status to recover it; do not dispatch again blindly.".into())
    }
    fn watch(&self, id: u64) -> Result<()> {
        let interrupted = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&interrupted);
        ctrlc::set_handler(move || {
            signal.store(true, Ordering::Relaxed);
        })?;
        let started = std::time::Instant::now();
        loop {
            let result = (|| -> Result<Value> {
                let run = self.watch_api(&format!("actions/runs/{id}"), &interrupted)?;
                let mut jobs = Vec::new();
                for page in 1.. {
                    let value = self.watch_api(
                        &format!("actions/runs/{id}/jobs?per_page=100&page={page}"),
                        &interrupted,
                    )?;
                    let rows = value["jobs"].as_array().ok_or("Missing jobs")?;
                    jobs.extend(rows.iter().cloned());
                    if rows.len() < 100 {
                        break;
                    }
                }
                if std::io::stdout().is_terminal() {
                    print!("\x1b[2J\x1b[H");
                }
                let complete = jobs.iter().filter(|j| j["status"] == "completed").count();
                println!(
                    "CI {id}: {} / {} — {complete}/{} jobs complete — watching {}m {}s",
                    run["status"],
                    run["conclusion"],
                    jobs.len(),
                    started.elapsed().as_secs() / 60,
                    started.elapsed().as_secs() % 60
                );
                println!("https://github.com/{}/actions/runs/{id}", self.repo);
                println!("Ctrl-C stops watching only; remote CI continues.\nCancel: revault-tool release cancel --candidate {id}\nFailure logs: revault-tool release logs --candidate {id}\n");
                for job in &jobs {
                    println!("{}", job_progress(job));
                }
                std::io::stdout().flush()?;
                Ok(run)
            })();
            if interrupted.load(Ordering::Relaxed) {
                return Err(format!("Stopped watching. Remote CI has not been cancelled.\nCancel: revault-tool release cancel --candidate {id}\nResume: revault-tool release status --candidate {id} --watch").into());
            }
            let run = result.map_err(|error| format!("Cannot refresh CI: {error}. Remote CI has not been cancelled.\nCancel: revault-tool release cancel --candidate {id}"))?;
            if run["status"] == "completed" {
                if run["conclusion"] == "success" {
                    return Ok(());
                }
                self.report_run_failure(id);
                return Err(format!("CI run {id} finished with {}. Read errors: revault-tool release logs --candidate {id}", run["conclusion"]).into());
            }
            for _ in 0..100 {
                if interrupted.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if interrupted.load(Ordering::Relaxed) {
                return Err(format!("Stopped watching. Remote CI has not been cancelled.\nCancel: revault-tool release cancel --candidate {id}\nResume: revault-tool release status --candidate {id} --watch").into());
            }
        }
    }

    fn watch_api(&self, suffix: &str, interrupted: &AtomicBool) -> Result<Value> {
        let stdout = tempfile::tempfile()?;
        let stderr = tempfile::tempfile()?;
        let mut child = Command::new("gh")
            .current_dir(&self.root)
            .args(["api", &format!("repos/{}/{}", self.repo, suffix)])
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?)
            .spawn()?;
        let started = std::time::Instant::now();
        loop {
            if interrupted.load(Ordering::Relaxed) || started.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                let _ = child.wait();
                return Err("CI status request interrupted or timed out".into());
            }
            if let Some(status) = child.try_wait()? {
                use std::io::{Read, Seek, SeekFrom};
                let mut file = if status.success() { stdout } else { stderr };
                file.seek(SeekFrom::Start(0))?;
                let mut text = String::new();
                file.read_to_string(&mut text)?;
                if !status.success() {
                    return Err(text.into());
                }
                return Ok(serde_json::from_str(&text)?);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn report_run_failure(&self, id: u64) {
        let result = (|| -> Result<()> {
            let value = self.api(&format!("actions/runs/{id}/jobs?per_page=100"))?;
            let jobs = value["jobs"].as_array().ok_or("missing jobs")?;
            eprintln!("CI run {id} failed or was cancelled:");
            for job in jobs.iter().filter(|job| {
                matches!(
                    job["conclusion"].as_str(),
                    Some("failure" | "cancelled" | "timed_out")
                )
            }) {
                let name = job["name"].as_str().unwrap_or("unknown job");
                let job_id = job["id"].as_u64().unwrap_or_default();
                eprintln!(
                    "  - {name} ({})",
                    job["conclusion"].as_str().unwrap_or("unknown")
                );
                if let Some(steps) = job["steps"].as_array() {
                    for step in steps.iter().filter(|step| step["conclusion"] == "failure") {
                        eprintln!(
                            "      step failed: {}",
                            step["name"].as_str().unwrap_or("unknown")
                        );
                    }
                }
                eprintln!(
                    "      https://github.com/{}/actions/runs/{id}/job/{job_id}",
                    self.repo
                );
            }
            eprintln!("  Logs: gh run view {id} --repo {} --log-failed", self.repo);
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("Unable to retrieve CI failure details: {error}");
            eprintln!("  Run: https://github.com/{}/actions/runs/{id}", self.repo);
        }
    }
    fn state_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(output(
            &self.root,
            "git",
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "revault-release-candidate.json",
            ],
        )?))
    }
    fn remember(&self, id: u64, promotion: Option<u64>) -> Result<()> {
        let path = self.state_path()?;
        let mut temp = tempfile::NamedTempFile::new_in(path.parent().ok_or("invalid state path")?)?;
        serde_json::to_writer_pretty(
            &mut temp,
            &Remembered {
                repository: self.repo.clone(),
                candidate: id,
                promotion,
            },
        )?;
        temp.persist(path)?;
        Ok(())
    }
    fn remembered(&self) -> Result<Option<Remembered>> {
        let path = self.state_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let state: Remembered = serde_json::from_slice(&fs::read(path)?)?;
        if state.repository != self.repo {
            return Err(
                "Remembered candidate belongs to another repository; supply --candidate".into(),
            );
        }
        Ok(Some(state))
    }
}
// GitHub's job timestamps are UTC RFC3339 values, with whole seconds.
fn github_seconds(timestamp: &str) -> Option<u64> {
    let (date, clock) = timestamp.strip_suffix('Z')?.split_once('T')?;
    let date: Vec<u64> = date
        .split('-')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    let clock: Vec<u64> = clock
        .split(':')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()
        .ok()?;
    if date.len() != 3 || clock.len() != 3 {
        return None;
    }
    let (year, month, day) = (date[0], date[1], date[2]);
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || clock[0] > 23
        || clock[1] > 59
        || clock[2] > 59
    {
        return None;
    }
    let leap = |y| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let months = [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day == 0 || day > months[(month - 1) as usize] {
        return None;
    }
    let days: u64 = (1970..year)
        .map(|y| if leap(y) { 366 } else { 365 })
        .sum::<u64>()
        + months[..(month - 1) as usize].iter().sum::<u64>()
        + day
        - 1;
    Some(days * 86400 + clock[0] * 3600 + clock[1] * 60 + clock[2])
}

fn job_progress(job: &Value) -> String {
    let name = job["name"].as_str().unwrap_or("Unknown job");
    let status = job["status"].as_str().unwrap_or("unknown");
    let conclusion = job["conclusion"].as_str().unwrap_or(status);
    let marker = match conclusion {
        "success" => "✓",
        "failure" | "timed_out" => "✗",
        "skipped" | "cancelled" => "-",
        _ => "*",
    };
    let duration = job["started_at"]
        .as_str()
        .zip(job["completed_at"].as_str())
        .and_then(|(start, end)| github_seconds(end)?.checked_sub(github_seconds(start)?))
        .map(|seconds| format!(" in {}m{:02}s", seconds / 60, seconds % 60))
        .unwrap_or_default();
    let label = if conclusion == "success" {
        "complete"
    } else {
        conclusion
    };
    let mut line = format!("{marker} {name} — {label}{duration} (ID {})", job["id"]);
    if let Some(steps) = job["steps"].as_array() {
        for step in steps.iter().filter(|step| {
            step["conclusion"] == "failure"
                || (status != "completed" && step["status"] == "in_progress")
        }) {
            line.push_str(&format!(
                "\n    {}: {}",
                step["name"].as_str().unwrap_or("step"),
                step["conclusion"].as_str().unwrap_or("running")
            ));
        }
    }
    line
}

fn string(v: &Value, key: &str) -> Result<String> {
    Ok(v[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}"))?
        .to_owned())
}
fn output(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    let result = Command::new(program)
        .current_dir(root)
        .args(args)
        .output()?;
    if !result.status.success() {
        return Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}
fn run_command(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    if !Command::new(program)
        .current_dir(root)
        .args(args)
        .status()?
        .success()
    {
        return Err(format!("{program} {} failed", args.join(" ")).into());
    }
    Ok(())
}
fn validate(c: &Candidate, repo: &str, id: u64, sha: &str, artifacts: &[Artifact]) -> Result<()> {
    if c.schema != 1
        || c.repository != repo
        || c.run_id != id
        || c.source_sha != sha
        || !valid_sha(sha)
    {
        return Err("Candidate identity does not match the CI run".into());
    }
    if c.artifacts.is_empty() || c.artifacts != artifacts {
        return Err("Candidate artifacts are missing or changed; prepare a new candidate".into());
    }
    if !c.source_ref.starts_with("release-candidates/") {
        return Err("Invalid candidate ref".into());
    }
    if c.scope != Scope::Bindings {
        version(&c.cli_version)?;
    }
    if c.scope != Scope::Cli {
        version(&c.bindings_version)?;
    }
    Ok(())
}
fn valid_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit())
}
fn version(s: &str) -> Result<()> {
    if s.split('.').count() != 3
        || !s.split('.').all(|p| {
            !p.is_empty()
                && p.bytes().all(|c| c.is_ascii_digit())
                && (p == "0" || !p.starts_with('0'))
        })
    {
        return Err("Release versions must be stable MAJOR.MINOR.PATCH versions".into());
    }
    Ok(())
}
fn patch(s: &str) -> Result<String> {
    version(s)?;
    let (base, n) = s.rsplit_once('.').ok_or("invalid version")?;
    Ok(format!(
        "{base}.{}",
        n.parse::<u64>()?.checked_add(1).ok_or("version overflow")?
    ))
}
fn display(c: &Candidate) {
    println!("Candidate: {}\nScope: {}\nCommit: {}\nCLI: {}\nBindings: {}\nCI: https://github.com/{}/actions/runs/{}\nPublish:\n  revault-tool release publish --candidate {}", c.run_id,c.scope.name(),c.source_sha,c.cli_version,c.bindings_version,c.repository,c.run_id,c.run_id);
}

pub fn prepare(mut args: Prepare) -> Result<()> {
    if let Some(version) = args.version.take() {
        match args.scope {
            Scope::Cli => args.cli_version = Some(version),
            Scope::Bindings => args.bindings_version = Some(version),
            Scope::All => {
                return Err(
                    "For prepare all, use --cli-version and --bindings-version separately".into(),
                )
            }
        }
    }
    if (args.scope == Scope::Cli && args.bindings_version.is_some())
        || (args.scope == Scope::Bindings && args.cli_version.is_some())
    {
        return Err("The version option must match the selected release target".into());
    }

    let gh = GitHub::new(&args.repository)?;
    let workflow = gh.api(&format!("actions/workflows/{WORKFLOW}"))?;
    if workflow["state"] != "active" {
        return Err(
            "Release candidate workflow must be active on the default branch before preparation"
                .into(),
        );
    }
    if !output(&gh.root, "git", &["status", "--porcelain"])?.is_empty() {
        return Err("Commit or stash working tree changes before preparing a release".into());
    }
    run_command(&gh.root, "git", &["fetch", "origin", "--tags"])?;
    if args.scope != Scope::Bindings && args.cli_version.is_none() {
        let current = current_version(&gh.root, Scope::Cli)?;
        let released = latest_released_version(&gh.root, "revault_cli-v", &current)?;
        args.cli_version = Some(prompt_version("CLI", &current, &released)?);
    }
    if args.scope != Scope::Cli && args.bindings_version.is_none() {
        let current = current_version(&gh.root, Scope::Bindings)?;
        let released = latest_released_version(&gh.root, "revault-api-v", &current)?;
        args.bindings_version = Some(prompt_version("bindings", &current, &released)?);
    }
    let (cli, bindings) = prepare_versions(
        &gh.root,
        args.scope,
        args.cli_version,
        args.bindings_version,
    )?;
    let sha = output(&gh.root, "git", &["rev-parse", "HEAD"])?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let source_ref = format!("release-candidates/{}-{nonce}", &sha[..12]);
    // Deliberately not a release tag: prepare never triggers publication.
    run_command(
        &gh.root,
        "git",
        &["push", "origin", &format!("{sha}:refs/heads/{source_ref}")],
    )?;
    let title = format!("prepare {} {source_ref}", args.scope.name());
    let id=gh.dispatch(&source_ref,json!({"mode":"prepare","scope":args.scope.name(),"cli_version":cli,"bindings_version":bindings,"candidate":"","request":source_ref}),&title)?;
    gh.remember(id, None)?;
    println!("Remembered candidate {id}. Preparation publishes no packages or release tags.");
    gh.watch(id)?;
    let candidate = gh.load(id)?;
    display(&candidate);
    Ok(())
}

fn selected_run(gh: &GitHub, args: &Selection) -> Result<u64> {
    Ok(args
        .candidate
        .or(gh
            .remembered()?
            .and_then(|s| s.promotion.or(Some(s.candidate))))
        .ok_or("No remembered release. Supply --candidate <run-id>.")?)
}

pub fn cancel(args: Selection) -> Result<()> {
    let gh = GitHub::new(&args.repository)?;
    let id = selected_run(&gh, &args)?;
    let run = gh.api(&format!("actions/runs/{id}"))?;
    if run["status"] == "completed" {
        println!("CI run {id} has already completed ({})", run["conclusion"]);
        return Ok(());
    }
    run_command(
        &gh.root,
        "gh",
        &["run", "cancel", &id.to_string(), "--repo", &gh.repo],
    )?;
    println!("Cancellation requested for CI run {id}; runners may take a moment to stop.");
    Ok(())
}

pub fn logs(args: Selection) -> Result<()> {
    let gh = GitHub::new(&args.repository)?;
    let id = selected_run(&gh, &args)?;
    println!(
        "Failure logs for CI run {id}: https://github.com/{}/actions/runs/{id}",
        gh.repo
    );
    let mut found = false;
    let mut unavailable = false;
    for page in 1.. {
        let value = gh.api(&format!("actions/runs/{id}/jobs?per_page=100&page={page}"))?;
        let jobs = value["jobs"].as_array().ok_or("Missing jobs")?;
        for job in jobs {
            if !matches!(
                job["conclusion"].as_str(),
                Some("failure" | "timed_out" | "cancelled")
            ) {
                continue;
            }
            found = true;
            let job_id = job["id"].as_u64().ok_or("Missing job ID")?;
            println!(
                "\n{} ({})\nhttps://github.com/{}/actions/runs/{id}/job/{job_id}",
                job["name"], job["conclusion"], gh.repo
            );
            // The per-job endpoint is available as soon as that job finishes,
            // unlike gh run view --log-failed, which waits for the entire run.
            match output(
                &gh.root,
                "gh",
                &[
                    "api",
                    &format!("repos/{}/actions/jobs/{job_id}/logs", gh.repo),
                ],
            ) {
                Ok(log) if !log.trim().is_empty() => {
                    if args.full {
                        println!("{log}");
                    } else {
                        println!("{}", failure_excerpt(&log));
                        println!("Use --full to print the complete job log.");
                    }
                }
                result => {
                    let reason = result
                        .err()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "GitHub returned an empty log".into());
                    eprintln!("Logs unavailable for job {job_id}: {reason}. Retry release logs or open the job link.");
                    unavailable = true;
                }
            }
        }
        if jobs.len() < 100 {
            break;
        }
    }
    if !found {
        println!("No completed failed jobs yet. GitHub's API does not stream unfinished job logs.\nLive runner logs: https://github.com/{}/actions/runs/{id}", gh.repo);
    }
    if unavailable {
        return Err("Some job logs were unavailable; available logs are printed above".into());
    }
    Ok(())
}

fn failure_excerpt(log: &str) -> String {
    const MAX_LINES: usize = 160;
    let lines: Vec<&str> = log.lines().collect();
    let interesting: Vec<&str> = lines
        .iter()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("##[error]")
                || lower.contains("error:")
                || lower.contains("failed")
                || lower.contains("failure")
                || lower.contains("panic")
                || lower.contains("panicked")
                || lower.contains("assertion")
                || lower.contains("test result:")
                || lower.contains("exit code")
                || lower.contains("timed out")
        })
        .copied()
        .take(MAX_LINES)
        .collect();
    if !interesting.is_empty() {
        return interesting.join("\n");
    }
    lines
        .iter()
        .rev()
        .take(MAX_LINES)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn status(options: StatusSelection) -> Result<()> {
    let args = options.selection;
    if let Err(error) = crate::release_status::report(&args.repository) {
        println!("Registry status unavailable: {error}");
    }
    let gh = GitHub::new(&args.repository)?;
    let remembered = gh.remembered()?;
    let id = args.candidate.or(remembered.as_ref().map(|s| s.candidate));
    if options.watch {
        let active = remembered
            .as_ref()
            .filter(|s| Some(s.candidate) == id)
            .and_then(|s| s.promotion)
            .or(id)
            .ok_or("No remembered release to watch")?;
        gh.watch(active)?;
    }

    if let Some(id) = id {
        let run = gh.api(&format!("actions/runs/{id}"))?;
        println!("Candidate {id}: {} / {}", run["status"], run["conclusion"]);
        if run["conclusion"] == "success" {
            display(&gh.load(id)?);
        }
        if let Some(state) = gh.remembered()?.filter(|s| s.candidate == id) {
            if let Some(p) = state.promotion {
                let publication = gh.api(&format!("actions/runs/{p}"))?;
                for page in 1.. {
                    let jobs =
                        gh.api(&format!("actions/runs/{p}/jobs?per_page=100&page={page}"))?;
                    let rows = jobs["jobs"].as_array().ok_or("Missing publication jobs")?;
                    for job in rows {
                        println!(
                            "  {}: {} / {}",
                            job["name"], job["status"], job["conclusion"]
                        );
                    }
                    if rows.len() < 100 {
                        break;
                    }
                }

                println!(
                    "Candidate publication: {} / {}\nhttps://github.com/{}/actions/runs/{p}",
                    publication["status"], publication["conclusion"], gh.repo
                );
            }
        }
    } else {
        let runs = gh.api(&format!(
            "actions/workflows/{WORKFLOW}/runs?event=workflow_dispatch&per_page=100"
        ))?;
        for run in runs["workflow_runs"].as_array().ok_or("missing runs")? {
            if run["display_title"]
                .as_str()
                .is_some_and(|s| s.starts_with("prepare "))
            {
                println!(
                    "{}  {}  {}  {}",
                    run["id"], run["conclusion"], run["head_sha"], run["html_url"]
                );
            }
        }
        println!(
            "Select a candidate explicitly: revault-tool release publish --candidate <run-id>"
        );
    }
    Ok(())
}

pub fn publish(args: Selection) -> Result<()> {
    let gh = GitHub::new(&args.repository)?;
    let remembered = gh.remembered()?;
    let id = args
        .candidate
        .or(remembered.as_ref().map(|s| s.candidate))
        .ok_or("No remembered candidate. Run release status and supply --candidate <run-id>.")?;
    let c = gh.load(id)?;
    if args.candidate.is_none()
        && (!output(&gh.root, "git", &["status", "--porcelain"])?.is_empty()
            || output(&gh.root, "git", &["rev-parse", "HEAD"])? != c.source_sha)
    {
        return Err("Checkout changed since preparation. Prepare again, or explicitly select the old candidate with --candidate.".into());
    }
    let remote = gh.api(&format!("git/ref/heads/{}", c.source_ref))?;
    if remote["object"]["sha"] != c.source_sha {
        return Err("Candidate branch moved; prepare a new candidate".into());
    }
    display(&c);
    // Recover the same publication run on any machine, including after an interrupted dispatch.
    let title = format!("publish {} {id}", c.scope.name());
    let runs = gh.api(&format!(
        "actions/workflows/{WORKFLOW}/runs?event=workflow_dispatch&per_page=100"
    ))?;
    let previous = runs["workflow_runs"]
        .as_array()
        .ok_or("missing runs")?
        .iter()
        .find(|r| r["display_title"] == title && r["head_sha"] == c.source_sha);
    let run_id = if let Some(run) = previous {
        let run_id = run["id"].as_u64().ok_or("invalid run ID")?;
        if run["conclusion"] == "success" {
            println!("Candidate {id} is already published.");
            gh.remember(id, Some(run_id))?;
            return Ok(());
        }
        if run["status"] == "completed" {
            run_command(
                &gh.root,
                "gh",
                &[
                    "run",
                    "rerun",
                    &run_id.to_string(),
                    "--repo",
                    &gh.repo,
                    "--failed",
                ],
            )?;
        }
        run_id
    } else {
        gh.dispatch(&c.source_ref,json!({"mode":"publish","scope":c.scope.name(),"cli_version":c.cli_version,"bindings_version":c.bindings_version,"candidate":id.to_string(),"request":id.to_string()}),&title)?
    };
    gh.remember(id, Some(run_id))?;
    gh.watch(run_id)?;
    println!("Published candidate {id}; retained CI artifacts were promoted without repeating the preflight.");
    Ok(())
}

pub fn ci(args: Ci) -> Result<()> {
    let gh = GitHub::new(&args.repository)?;
    if args.action == "seal" {
        let id = std::env::var("GITHUB_RUN_ID")?.parse()?;
        let source_sha = std::env::var("GITHUB_SHA")?;
        let c = Candidate {
            schema: 1,
            repository: gh.repo.clone(),
            run_id: id,
            source_sha: source_sha.clone(),
            source_ref: std::env::var("GITHUB_REF_NAME")?,
            scope: args.scope,
            cli_version: args.cli_version,
            bindings_version: args.bindings_version,
            artifacts: gh.artifacts(id)?,
        };
        validate(&c, &gh.repo, id, &source_sha, &c.artifacts)?;
        fs::write(args.output, serde_json::to_vec_pretty(&c)?)?;
    } else {
        let c = gh.load(args.candidate.ok_or("--candidate is required")?)?;
        if c.source_sha != std::env::var("GITHUB_SHA")?
            || c.scope != args.scope
            || c.cli_version != args.cli_version
            || c.bindings_version != args.bindings_version
        {
            return Err("Publication inputs differ from the validated candidate".into());
        }
        if matches!(args.action.as_str(), "promote-cli" | "promote-bindings") {
            let cli = args.action == "promote-cli";
            let workflow = if cli {
                "revault_cli-v-release.yml"
            } else {
                "bindings-native-release.yml"
            };
            let title = format!("promote candidate {}", c.run_id);
            let runs = gh.api(&format!(
                "actions/workflows/{workflow}/runs?event=workflow_dispatch&per_page=100"
            ))?;
            let existing = runs["workflow_runs"]
                .as_array()
                .ok_or("missing runs")?
                .iter()
                .find(|r| r["display_title"] == title && r["head_sha"] == c.source_sha);
            let id = if let Some(run) = existing {
                let id = run["id"].as_u64().ok_or("missing run ID")?;
                if run["conclusion"] == "success" {
                    return Ok(());
                }
                if run["status"] == "completed" {
                    run_command(
                        &gh.root,
                        "gh",
                        &[
                            "run",
                            "rerun",
                            &id.to_string(),
                            "--repo",
                            &gh.repo,
                            "--failed",
                        ],
                    )?;
                }
                id
            } else {
                let inputs = if cli {
                    json!({"candidate":true,"version":c.cli_version,"promotion_run_id":c.run_id.to_string()})
                } else {
                    json!({"version":c.bindings_version,"publish":true,"targets":"npm,python,maven,nuget,dart,ruby,lua,rust,git,homebrew","promotion_run_id":c.run_id.to_string(),"promotion_source_sha":c.source_sha})
                };
                gh.post(
                    &format!("actions/workflows/{workflow}/dispatches"),
                    json!({"ref":c.source_ref,"inputs":inputs}),
                )?;
                let mut id = None;
                for _ in 0..60 {
                    let runs = gh.api(&format!(
                        "actions/workflows/{workflow}/runs?event=workflow_dispatch&per_page=100"
                    ))?;
                    if let Some(run) = runs["workflow_runs"]
                        .as_array()
                        .ok_or("missing runs")?
                        .iter()
                        .find(|r| r["display_title"] == title && r["head_sha"] == c.source_sha)
                    {
                        id = run["id"].as_u64();
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(2));
                }
                id.ok_or("Publication dispatch accepted, but run not found; retry to recover")?
            };
            return gh.watch(id);
        }
        if args.action == "tags" {
            for tag in [
                if c.scope != Scope::Bindings {
                    Some(format!("revault_cli-v{}", c.cli_version))
                } else {
                    None
                },
                if c.scope != Scope::Cli {
                    Some(format!("revault-api-v{}", c.bindings_version))
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            {
                // API creation with GITHUB_TOKEN does not trigger the tag-push release workflows.
                let existing = Command::new("gh")
                    .current_dir(&gh.root)
                    .args(["api", &format!("repos/{}/git/ref/tags/{tag}", gh.repo)])
                    .output()?;
                if existing.status.success() {
                    let value: Value = serde_json::from_slice(&existing.stdout)?;
                    if value["object"]["type"] != "commit" || value["object"]["sha"] != c.source_sha
                    {
                        return Err(format!("Tag {tag} already refers to another object").into());
                    }
                } else {
                    gh.post(
                        "git/refs",
                        json!({"ref":format!("refs/tags/{tag}"),"sha":c.source_sha}),
                    )?;
                }
            }
        }
    }
    Ok(())
}

// Version preparation is kept separate from publication. Existing migration-format
// dependencies without a local path must keep their pinned historical versions.
fn prepare_versions(
    root: &Path,
    scope: Scope,
    cli: Option<String>,
    bindings: Option<String>,
) -> Result<(String, String)> {
    let metadata: Value = serde_json::from_str(&output(
        root,
        "cargo",
        &[
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            "rust/Cargo.toml",
        ],
    )?)?;
    let packages = metadata["packages"].as_array().ok_or("missing packages")?;
    let cli_package = packages
        .iter()
        .find(|p| p["name"] == "revault_cli")
        .ok_or("missing CLI package")?;
    let cli = if scope != Scope::Bindings {
        next_version(root, "revault_cli-v", &string(cli_package, "version")?, cli)?
    } else {
        String::new()
    };
    let dart = fs::read_to_string(root.join("bindings/dart/pubspec.yaml"))?;
    let current = dart
        .lines()
        .find_map(|l| l.strip_prefix("version: "))
        .ok_or("missing Dart version")?;
    let bindings = if scope != Scope::Cli {
        next_version(root, "revault-api-v", current, bindings)?
    } else {
        String::new()
    };
    if !cli.is_empty() {
        version(&cli)?;
    }
    if !bindings.is_empty() {
        version(&bindings)?;
    }
    let mut versions = BTreeMap::new();
    for p in packages {
        if p["publish"].as_array().is_some_and(|v| v.is_empty()) {
            continue;
        }
        let name = string(p, "name")?;
        if !crate::release::CLI_PUBLISH_PACKAGES.contains(&name.as_str())
            && !["revault_bindings", "revault_wasm_bindings"].contains(&name.as_str())
        {
            continue;
        }
        if scope == Scope::Cli && !crate::release::CLI_PUBLISH_PACKAGES.contains(&name.as_str()) {
            continue;
        }
        // CLI publishes its complete dependency chain; bindings publish their API dependencies.
        if scope == Scope::Bindings
            && ![
                "revault_page_api",
                "revault_lockbox_api",
                "revault_vault_api",
                "revault_bindings",
                "revault_wasm_bindings",
            ]
            .contains(&name.as_str())
        {
            continue;
        }
        versions.insert(
            name.clone(),
            if name == "revault_cli" {
                cli.clone()
            } else {
                patch(&string(p, "version")?)?
            },
        );
    }
    let tracked = output(root, "git", &["ls-files", "*Cargo.toml"])?;
    for path in tracked.lines() {
        let full = root.join(path);
        let mut doc = fs::read_to_string(&full)?.parse::<toml_edit::DocumentMut>()?;
        if let Some(name) = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        {
            if let Some(v) = versions.get(name) {
                doc["package"]["version"] = toml_edit::value(v);
            }
        }
        update_dependencies(doc.as_table_mut(), &versions);
        fs::write(full, doc.to_string())?;
    }
    if !bindings.is_empty() {
        fs::write(
            root.join("bindings/dart/pubspec.yaml"),
            dart.replacen(
                &format!("version: {current}"),
                &format!("version: {bindings}"),
                1,
            ),
        )?;
    }
    if !bindings.is_empty() {
        let generated = root.join("bindings/dart/lib/src/version/version.g.dart");
        if generated.exists() {
            fs::write(generated,format!("/// GENERATED BY revault-tool release prepare. Do not modify.\n/// Package version.\nString packageVersion = '{bindings}';\n"))?;
        }
    }
    // These are separate Cargo workspaces, each with its own dependency lock.
    for manifest in [
        "rust/Cargo.toml",
        "bindings/rust/Cargo.toml",
        "rust/fuzz/Cargo.toml",
    ] {
        output(
            root,
            "cargo",
            &[
                "metadata",
                "--format-version",
                "1",
                "--manifest-path",
                manifest,
            ],
        )?;
    }
    let prefix = if scope == Scope::Bindings {
        "revault-api-v*"
    } else {
        "revault_cli-v*"
    };
    let tags = output(
        root,
        "git",
        &["tag", "--list", prefix, "--sort=-version:refname"],
    )?;
    let range = tags
        .lines()
        .next()
        .map(|t| format!("{t}..HEAD"))
        .unwrap_or_else(|| "HEAD".into());
    let notes = output(root, "git", &["log", &range, "--format=- %s"])?;
    if !bindings.is_empty() {
        let changelog = root.join("bindings/dart/CHANGELOG.md");
        let previous = match fs::read_to_string(&changelog) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };
        let header = format!("## {bindings}");
        if !previous
            .lines()
            .any(|line| line.trim_start_matches('#').trim() == bindings)
        {
            fs::write(&changelog, format!("{header}\n\n{notes}\n\n{previous}"))?;
        }
        run_command(root, "git", &["add", "bindings/dart/CHANGELOG.md"])?;
    }
    fs::create_dir_all(root.join("release-notes"))?;
    fs::write(
        root.join("release-notes/candidate.txt"),
        format!(
            "Release {}\nCLI: {cli}\nBindings: {bindings}\n\n{notes}\n",
            scope.name()
        ),
    )?;
    run_command(root, "git", &["add", "--update"])?;
    run_command(
        root,
        "git",
        &[
            "add",
            "release-notes/candidate.txt",
            "rust/Cargo.lock",
            "bindings/rust/Cargo.lock",
            "rust/fuzz/Cargo.lock",
        ],
    )?;
    run_command(
        root,
        "git",
        &[
            "commit",
            "-m",
            &format!(
                "Prepare {} release: CLI {cli}, bindings {bindings}",
                scope.name()
            ),
        ],
    )?;
    Ok((cli, bindings))
}
fn current_version(root: &Path, scope: Scope) -> Result<String> {
    match scope {
        Scope::Cli => {
            let metadata: Value = serde_json::from_str(&output(
                root,
                "cargo",
                &[
                    "metadata",
                    "--no-deps",
                    "--format-version",
                    "1",
                    "--manifest-path",
                    "rust/Cargo.toml",
                ],
            )?)?;
            metadata["packages"]
                .as_array()
                .and_then(|packages| {
                    packages
                        .iter()
                        .find(|package| package["name"] == "revault_cli")
                })
                .map(|package| string(package, "version"))
                .ok_or_else(|| "missing CLI package".to_owned())?
        }
        Scope::Bindings => {
            let dart = fs::read_to_string(root.join("bindings/dart/pubspec.yaml"))?;
            dart.lines()
                .find_map(|line| line.strip_prefix("version: "))
                .map(str::to_owned)
                .ok_or_else(|| "missing Dart version".into())
        }
        Scope::All => Err("a release target is required".into()),
    }
}

fn prompt_version(target: &str, current: &str, released: &str) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        return Err(format!(
            "Choose the {target} version explicitly: --{}-version patch|minor|major|MAJOR.MINOR.PATCH",
            if target == "CLI" { "cli" } else { "bindings" }
        )
        .into());
    }
    let next_patch = patch(released)?;
    let (major, minor, _) = version_tuple(released)?;
    let next_minor = format!(
        "{major}.{}.0",
        minor.checked_add(1).ok_or("Version overflow")?
    );
    let next_major = format!("{}.0.0", major.checked_add(1).ok_or("Version overflow")?);
    println!("{target} versions:\n  Current: {current}\n  Latest released: {released}");
    loop {
        println!("  1) Patch  -> {next_patch}");
        println!("  2) Minor  -> {next_minor}");
        println!("  3) Major  -> {next_major}");
        println!("  4) Retain current version -> {current}");
        println!("  5) Enter an exact MAJOR.MINOR.PATCH version");
        print!("Select {target} version [1]: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input)? == 0 {
            return Err("Version selection cancelled".into());
        }
        let input = input.trim();
        let input = if input.is_empty() { "1" } else { input };
        match input {
            "1" => return Ok("patch".into()),
            "2" => return Ok("minor".into()),
            "3" => return Ok("major".into()),
            "4" => return Ok(current.to_owned()),
            "5" => {
                print!("Enter exact {target} version: ");
                std::io::stdout().flush()?;
                let mut exact = String::new();
                if std::io::stdin().read_line(&mut exact)? == 0 {
                    return Err("Version selection cancelled".into());
                }
                let exact = exact.trim();
                if version(exact).is_ok() {
                    return Ok(exact.to_owned());
                }
            }
            _ => {}
        }
        eprintln!("Select 1, 2, 3, 4, or 5; exact versions must look like 1.2.3.");
    }
}

fn next_version(
    root: &Path,
    prefix: &str,
    current: &str,
    requested: Option<String>,
) -> Result<String> {
    let highest = latest_released_version(root, prefix, current)?;
    let (major, minor, _) = version_tuple(&highest)?;
    let next = match requested.as_deref().unwrap_or("patch") {
        "patch" => patch(&highest)?,
        "minor" => format!(
            "{major}.{}.0",
            minor.checked_add(1).ok_or("Version overflow")?
        ),
        "major" => format!("{}.0.0", major.checked_add(1).ok_or("Version overflow")?),
        exact => exact.to_owned(),
    };
    if version_tuple(&next)? <= version_tuple(&highest)? {
        return Err(format!("Version {next} must exceed {highest}").into());
    }
    Ok(next)
}

fn latest_released_version(root: &Path, prefix: &str, current: &str) -> Result<String> {
    version_tuple(current)?;
    let tags = output(root, "git", &["tag", "--list", &format!("{prefix}*")])?;
    let mut highest: Option<String> = None;
    for tag in tags.lines() {
        if let Some(released) = tag.strip_prefix(prefix) {
            let Ok(released_tuple) = version_tuple(released) else {
                continue;
            };
            let is_newer = highest
                .as_deref()
                .map(|known| released_tuple > version_tuple(known).expect("validated version"))
                .unwrap_or(true);
            if is_newer {
                highest = Some(released.to_owned());
            }
        }
    }
    Ok(highest.unwrap_or_else(|| current.to_owned()))
}

fn version_tuple(s: &str) -> Result<(u64, u64, u64)> {
    version(s)?;
    let n = s
        .split('.')
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((n[0], n[1], n[2]))
}
fn update_dependencies(table: &mut toml_edit::Table, versions: &BTreeMap<String, String>) {
    for (key, item) in table.iter_mut() {
        if matches!(
            key.get(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        ) {
            if let Some(deps) = item.as_table_mut() {
                for (name, dep) in deps.iter_mut() {
                    if let Some(t) = dep.as_inline_table_mut() {
                        if t.contains_key("path") {
                            let package = t
                                .get("package")
                                .and_then(|v| v.as_str())
                                .unwrap_or(name.get());
                            if let Some(v) = versions.get(package) {
                                t.insert("version", v.as_str().into());
                            }
                        }
                    } else if let Some(t) = dep.as_table_mut() {
                        if t.contains_key("path") {
                            let package = t
                                .get("package")
                                .and_then(|v| v.as_str())
                                .unwrap_or(name.get());
                            if let Some(v) = versions.get(package) {
                                t["version"] = toml_edit::value(v);
                            }
                        }
                    }
                }
            }
        } else if let Some(t) = item.as_table_mut() {
            update_dependencies(t, versions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate() -> Candidate {
        Candidate {
            schema: 1,
            repository: "org/repo".into(),
            run_id: 42,
            source_sha: "a".repeat(40),
            source_ref: "release-candidates/a-1".into(),
            scope: Scope::All,
            cli_version: "1.2.3".into(),
            bindings_version: "2.3.4".into(),
            artifacts: vec![Artifact {
                id: 1,
                name: "native-linux".into(),
                digest: format!("sha256:{}", "b".repeat(64)),
            }],
        }
    }
    #[test]
    fn completed_jobs_collapse_with_elapsed_time_across_midnight() {
        let job = json!({"id":7,"name":"CLI build","status":"completed","conclusion":"success",
            "started_at":"2026-09-08T23:58:00Z","completed_at":"2026-09-09T00:03:43Z",
            "steps":[{"name":"Build artifacts","status":"completed","conclusion":"success"}]});
        assert_eq!(job_progress(&job), "✓ CLI build — complete in 5m43s (ID 7)");
        assert_eq!(
            github_seconds("2024-03-01T00:00:00Z").unwrap()
                - github_seconds("2024-02-28T00:00:00Z").unwrap(),
            172800
        );
        assert!(github_seconds("invalid").is_none());
    }

    #[test]
    fn failure_excerpt_hides_unrelated_job_output() {
        let log =
            "compile lots of crates\nnormal output\nerror: password profile test failed\nfinished";
        assert_eq!(failure_excerpt(log), "error: password profile test failed");
    }

    #[test]
    fn failure_excerpt_falls_back_to_log_tail() {
        let log = "first\nsecond\nlast";
        assert_eq!(failure_excerpt(log), log);
    }

    #[test]
    fn promotion_refuses_missing_replaced_or_foreign_artifacts() {
        let c = candidate();
        assert!(validate(&c, &c.repository, 42, &c.source_sha, &c.artifacts).is_ok());
        assert!(validate(&c, "other/repo", 42, &c.source_sha, &c.artifacts).is_err());
        assert!(validate(&c, &c.repository, 43, &c.source_sha, &c.artifacts).is_err());
        assert!(validate(&c, &c.repository, 42, &"b".repeat(40), &c.artifacts).is_err());
        assert!(validate(&c, &c.repository, 42, &c.source_sha, &[]).is_err());
        let mut a = c.artifacts.clone();
        a[0].digest.push('a');
        assert!(validate(&c, &c.repository, 42, &c.source_sha, &a).is_err());
    }
    #[test]
    fn dependency_bumps_preserve_historical_migrations() {
        let mut doc = r#"[dependencies]
current = { package = "api", path = "../api", version = "1.0.0" }
legacy = { package = "api", version = "=0.1.0" }
[target.'cfg(windows)'.dependencies.api]
path = "../api"
version = "1.0.0"
"#
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
        update_dependencies(
            doc.as_table_mut(),
            &BTreeMap::from([("api".into(), "1.0.1".into())]),
        );
        assert_eq!(
            doc["dependencies"]["current"]["version"].as_str(),
            Some("1.0.1")
        );
        assert_eq!(
            doc["dependencies"]["legacy"]["version"].as_str(),
            Some("=0.1.0")
        );
        assert!(doc.to_string().contains("version = \"1.0.1\""));
    }
    #[test]
    fn version_preparation_updates_path_dependencies_and_commits_reviewable_notes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for path in [
            "rust/revault_cli/src",
            "rust/revault_page_api/src",
            "bindings/rust/src",
            "bindings/dart",
            "rust/fuzz/src",
        ] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        fs::write(
            root.join("rust/Cargo.toml"),
            "[workspace]\nmembers = [\"revault_cli\",\"revault_page_api\"]\nresolver = \"2\"\n",
        )
        .unwrap();
        fs::write(root.join("rust/revault_cli/Cargo.toml"),"[package]\nname = \"revault_cli\"\nversion = \"1.0.0\"\nedition = \"2021\"\n[dependencies]\nrevault_page_api = { path = \"../revault_page_api\", version = \"1.0.0\" }\n").unwrap();
        fs::write(
            root.join("rust/revault_page_api/Cargo.toml"),
            "[package]\nname = \"revault_page_api\"\nversion = \"1.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(root.join("bindings/rust/Cargo.toml"),"[package]\nname = \"revault-api\"\nversion = \"1.0.0\"\nedition = \"2021\"\n[dependencies]\nrevault_page_api = { path = \"../../rust/revault_page_api\", version = \"1.0.0\" }\n").unwrap();
        for p in [
            "rust/revault_page_api/src/lib.rs",
            "bindings/rust/src/lib.rs",
        ] {
            fs::write(root.join(p), "").unwrap();
        }
        fs::write(root.join("rust/revault_cli/src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            root.join("bindings/dart/pubspec.yaml"),
            "name: revault_api\nversion: 2.0.0\n",
        )
        .unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "release@example.invalid"],
            vec!["config", "user.name", "Release Test"],
            vec!["add", "."],
            vec!["commit", "-m", "Implement feature"],
            vec!["tag", "revault-api-v2.0.5"],
        ] {
            output(root, "git", &args).unwrap();
        }
        fs::write(root.join("rust/fuzz/Cargo.toml"), "[workspace]\n[package]\nname = \"fuzz-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[dependencies]\nrevault_page_api = { path = \"../revault_page_api\", version = \"1.0.0\" }\n").unwrap();
        fs::write(root.join("rust/fuzz/src/lib.rs"), "").unwrap();
        output(
            root,
            "cargo",
            &[
                "metadata",
                "--format-version",
                "1",
                "--manifest-path",
                "rust/fuzz/Cargo.toml",
            ],
        )
        .unwrap();
        output(root, "git", &["add", "rust/fuzz"]).unwrap();
        output(
            root,
            "git",
            &["commit", "-m", "Add independent fuzz workspace"],
        )
        .unwrap();
        let (cli, bindings) = prepare_versions(root, Scope::All, None, None).unwrap();
        output(
            root,
            "cargo",
            &[
                "check",
                "--locked",
                "--offline",
                "--target-dir",
                &tempfile::tempdir().unwrap().path().to_string_lossy(),
                "--manifest-path",
                "rust/fuzz/Cargo.toml",
            ],
        )
        .unwrap();

        assert_eq!((cli.as_str(), bindings.as_str()), ("1.0.1", "2.0.6"));
        assert!(fs::read_to_string(root.join("bindings/dart/CHANGELOG.md"))
            .unwrap()
            .starts_with("## 2.0.6\n"));
        let manifest = fs::read_to_string(root.join("rust/revault_cli/Cargo.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(
            manifest["dependencies"]["revault_page_api"]["version"].as_str(),
            Some("1.0.1")
        );
        assert!(fs::read_to_string(root.join("release-notes/candidate.txt"))
            .unwrap()
            .contains("Implement feature"));
        assert!(output(root, "git", &["status", "--porcelain"])
            .unwrap()
            .is_empty());
        assert!(next_version(root, "revault-api-v", "2.0.0", Some("2.0.4".into())).is_err());
        assert_eq!(
            next_version(root, "revault-api-v", "2.0.0", Some("minor".into())).unwrap(),
            "2.1.0"
        );
        assert_eq!(
            next_version(root, "revault-api-v", "2.0.6", Some("patch".into())).unwrap(),
            "2.0.6"
        );
        assert_eq!(
            next_version(root, "revault-api-v", "2.0.0", Some("major".into())).unwrap(),
            "3.0.0"
        );
        assert_eq!(
            next_version(root, "revault-api-v", "2.0.0", Some("4.5.6".into())).unwrap(),
            "4.5.6"
        );
    }
    #[test]
    fn stable_version_validation() {
        assert_eq!(patch("1.2.3").unwrap(), "1.2.4");
        for s in ["1.2", "1.2.3-beta", "1.2.03", "../1.2.3", "1.2.3\n"] {
            assert!(version(s).is_err(), "{s}");
        }
    }
}
