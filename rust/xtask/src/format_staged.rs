//! Commit-time formatting. Never stage work the user did not already stage.
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(output.stdout)
}

pub fn run() -> Result<(), String> {
    let root = git(Path::new("."), &["rev-parse", "--show-toplevel"])?;
    let root = String::from_utf8(root).map_err(|e| e.to_string())?;
    format_staged(Path::new(root.trim_end_matches(['\r', '\n'])))
}

fn format_staged(root: &Path) -> Result<(), String> {
    let names = git(
        root,
        &[
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMR",
            "-z",
        ],
    )?;
    let mut changes = Vec::new();
    for name in names.split(|b| *b == 0).filter(|n| n.ends_with(b".rs")) {
        let name = std::str::from_utf8(name).map_err(|_| "Non-UTF-8 Rust path: format manually")?;
        let path = root.join(name);
        if !fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_file()
        {
            return Err(format!("Refusing non-regular Rust file: {name}"));
        }
        let staged = git(root, &["show", &format!(":{name}")])?;
        let working = fs::read(&path).map_err(|e| e.to_string())?;
        if working != staged {
            return Err(format!("{name} has unstaged edits (or checkout filters). Resolve staging before committing; nothing was formatted."));
        }
        // All repository crates use edition 2021. stdin prevents rustfmt from
        // following `mod` declarations into unrelated, unstaged source files.
        let mut child = Command::new("rustfmt")
            .current_dir(path.parent().unwrap())
            .args(["--edition", "2021", "--emit", "stdout"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Cannot start rustfmt: {e}"))?;
        // Write concurrently so large inputs cannot deadlock on output pipes.
        let mut stdin = child.stdin.take().unwrap();
        let input = staged.clone();
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        let written = writer.join().map_err(|_| "rustfmt input thread panicked")?;
        if !output.status.success() {
            return Err(format!(
                "rustfmt failed for {name}: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        written.map_err(|e| e.to_string())?;
        if staged != output.stdout {
            changes.push((name.to_owned(), staged, output.stdout));
        }
    }
    // Validate all files before touching either worktree or index.
    for (name, original, _) in &changes {
        if fs::read(root.join(name)).map_err(|e| e.to_string())? != *original
            || git(root, &["show", &format!(":{name}")])? != *original
        {
            return Err(format!(
                "{name} changed during formatting; retry the commit"
            ));
        }
    }
    for (name, _, formatted) in &changes {
        fs::write(root.join(name), formatted).map_err(|e| e.to_string())?;
    }
    if !changes.is_empty() {
        let mut args = vec!["--literal-pathspecs", "add", "--"];
        args.extend(changes.iter().map(|(name, _, _)| name.as_str()));
        git(root, &args)?;
        eprintln!("Formatted {} staged Rust file(s).", changes.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Repo(std::path::PathBuf);
    impl Repo {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "revault-hook-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            git(&path, &["init", "-q"]).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, contents: &str) {
            fs::write(self.0.join(name), contents).unwrap();
        }
        fn stage(&self, name: &str) {
            git(&self.0, &["--literal-pathspecs", "add", "--", name]).unwrap();
        }
    }
    impl Drop for Repo {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn formats_only_staged_files_and_does_not_follow_modules() {
        let repo = Repo::new();
        repo.write("main file.rs", "mod child;\nfn main(){println!(\"hi\");}\n");
        repo.write("child.rs", "pub fn untouched(){}\n");
        repo.stage("main file.rs");
        format_staged(&repo.0).unwrap();
        let contents = fs::read(repo.0.join("main file.rs")).unwrap();
        assert!(String::from_utf8_lossy(&contents).contains("fn main() {"));
        assert_eq!(git(&repo.0, &["show", ":main file.rs"]).unwrap(), contents);
        assert_eq!(
            fs::read_to_string(repo.0.join("child.rs")).unwrap(),
            "pub fn untouched(){}\n"
        );
        assert!(git(&repo.0, &["show", ":child.rs"]).is_err());
        format_staged(&repo.0).unwrap();
    }

    #[test]
    fn partial_staging_leaves_everything_untouched() {
        let repo = Repo::new();
        for name in ["a.rs", "b.rs"] {
            repo.write(name, "fn main(){}\n");
            repo.stage(name);
        }
        repo.write("b.rs", "fn main(){ /* unfinished */ }\n");
        assert!(format_staged(&repo.0).unwrap_err().contains("unstaged"));
        assert_eq!(
            fs::read_to_string(repo.0.join("a.rs")).unwrap(),
            "fn main(){}\n"
        );
        assert_eq!(git(&repo.0, &["show", ":b.rs"]).unwrap(), b"fn main(){}\n");
        assert!(fs::read_to_string(repo.0.join("b.rs"))
            .unwrap()
            .contains("unfinished"));
    }

    #[test]
    fn syntax_error_does_not_modify_prior_files() {
        let repo = Repo::new();
        repo.write("a.rs", "fn main(){}\n");
        repo.stage("a.rs");
        repo.write("b.rs", "fn {\n");
        repo.stage("b.rs");
        assert!(format_staged(&repo.0).is_err());
        assert_eq!(
            fs::read_to_string(repo.0.join("a.rs")).unwrap(),
            "fn main(){}\n"
        );
    }

    #[test]
    fn no_rust_files_is_a_noop() {
        let repo = Repo::new();
        repo.write("notes.txt", "unchanged");
        repo.stage("notes.txt");
        format_staged(&repo.0).unwrap();
        assert_eq!(git(&repo.0, &["show", ":notes.txt"]).unwrap(), b"unchanged");
    }
}
