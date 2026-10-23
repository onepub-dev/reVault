//! Filesystem publication for both current and historical range recovery.
use super::context::{cli_error, CliResult};
use super::output::{print_records, OutputFormat};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(super) struct DirectoryWriter {
    root: PathBuf,
    parts: BTreeMap<(u64, u64), u64>,
    keep: bool,
}

impl DirectoryWriter {
    pub(super) fn new(root: &Path) -> CliResult<Self> {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(root)?; // Exclusive: existing directories and symlinks are refused.
        Ok(Self {
            root: root.to_owned(),
            parts: BTreeMap::new(),
            keep: false,
        })
    }

    pub(super) fn range(&mut self, id: u64, offset: u64, data: &[u8]) -> CliResult<()> {
        let end = offset
            .checked_add(data.len() as u64)
            .ok_or_else(|| cli_error("recovery range overflow"))?;
        if id == 0
            || data.is_empty()
            || self.parts.iter().any(|(&(other_id, start), &len)| {
                other_id == id && start < end && offset < start + len
            })
        {
            return Err(cli_error("invalid or overlapping recovered ranges"));
        }
        let directory = self.root.join(format!("file-{id}"));
        if !directory.exists() {
            private_directory(&directory)?;
        }
        let mut file = private_file(&directory.join(part_name(offset)))?;
        file.write_all(data)?;
        file.sync_all()?;
        self.parts.insert((id, offset), data.len() as u64);
        Ok(())
    }

    pub(super) fn finish(mut self, mut manifest: Value) -> CliResult<()> {
        if manifest["recovery_directory_protocol"] != 1 {
            return Err(cli_error("unsupported recovery directory protocol"));
        }
        let files = manifest["files"]
            .as_array_mut()
            .ok_or_else(|| cli_error("missing recovery file manifest"))?;
        let mut seen = BTreeSet::new();
        for file in files {
            let id = number(file, "id")?;
            let size = number(file, "size")?;
            if id == 0 || !seen.insert(id) || !file["path"].is_string() {
                return Err(cli_error("invalid recovery file identity"));
            }
            let ranges = file["ranges"]
                .as_array()
                .ok_or_else(|| cli_error("missing recovered ranges"))?;
            let missing = file["missing_ranges"]
                .as_array()
                .ok_or_else(|| cli_error("missing recovery gap information"))?;
            let mut coverage = Vec::new();
            let mut expected = BTreeMap::new();
            for range in ranges {
                let start = number(range, "offset")?;
                let len = number(range, "length")?;
                if len == 0 || expected.insert((id, start), len).is_some() {
                    return Err(cli_error("invalid recovery ranges"));
                }
                coverage.push((start, len));
            }
            let actual: BTreeMap<_, _> = self
                .parts
                .iter()
                .filter(|((other, _), _)| *other == id)
                .map(|(key, value)| (*key, *value))
                .collect();
            if actual != expected {
                return Err(cli_error("recovered bytes do not match the manifest"));
            }
            if number(file, "recovered_bytes")? != actual.values().sum::<u64>() {
                return Err(cli_error("recovered byte count disagrees with fragments"));
            }
            for gap in missing {
                if !gap["reason"].is_string() {
                    return Err(cli_error("missing recovery gap reason"));
                }
                coverage.push((number(gap, "offset")?, number(gap, "length")?));
            }
            coverage.sort_unstable();
            let mut cursor = 0;
            for (start, len) in coverage {
                if start != cursor || (len == 0 && size != 0) {
                    return Err(cli_error("recovery ranges and gaps overlap or omit data"));
                }
                cursor = cursor
                    .checked_add(len)
                    .ok_or_else(|| cli_error("recovery coverage overflow"))?;
            }
            if cursor != size {
                return Err(cli_error(
                    "recovery coverage does not match declared file length",
                ));
            }
            let complete = missing.is_empty();
            let status = if complete {
                "complete"
            } else if actual.is_empty() {
                "unrecoverable"
            } else {
                "partial"
            };
            if file["status"] != status {
                return Err(cli_error("recovery status disagrees with coverage"));
            }
            let directory = self.root.join(format!("file-{id}"));
            if complete {
                if !directory.exists() {
                    private_directory(&directory)?;
                }
                let mut output = private_file(&directory.join("complete.bin"))?;
                for &(_, offset) in actual.keys() {
                    let part = directory.join(part_name(offset));
                    std::io::copy(&mut File::open(&part)?, &mut output)?;
                    fs::remove_file(part)?;
                }
                output.sync_all()?;
                file["output"] = Value::String(format!("file-{id}/complete.bin"));
            } else {
                let original_path = file["path"].as_str().unwrap().to_owned();
                for range in file["ranges"].as_array_mut().unwrap() {
                    let offset = number(range, "offset")?;
                    let fallback = format!("file-{id}/{}", part_name(offset));
                    let output = match named_fragment_path(&self.root, &original_path, offset) {
                        Ok(relative) => match fs::hard_link(
                            self.root.join(&fallback),
                            self.root.join(&relative),
                        ) {
                            Ok(()) => {
                                fs::remove_file(self.root.join(&fallback))?;
                                relative.to_string_lossy().replace('\\', "/")
                            }
                            Err(error) => {
                                range["filename_fallback_reason"] = Value::String(format!(
                                    "original filename could not be published safely: {error}"
                                ));
                                fallback
                            }
                        },
                        Err(reason) => {
                            range["filename_fallback_reason"] = Value::String(reason);
                            fallback
                        }
                    };
                    range["output"] = Value::String(output);
                }
                if directory.exists() && fs::read_dir(&directory)?.next().is_none() {
                    fs::remove_dir(&directory)?;
                }
            }
        }
        if self.parts.keys().any(|(id, _)| !seen.contains(id)) {
            return Err(cli_error("unlisted recovered file"));
        }
        manifest["notice"] = Value::String("Complete files belong in the recovered Lockbox. This sidecar preserves incomplete-file fragments at their original offsets; consult missing_ranges before use. Original paths are metadata, never output paths. Recoverable variables, forms, directories and symlinks remain in the recovered Lockbox.".to_owned());
        let mut output = private_file(&self.root.join("manifest.json"))?;
        serde_json::to_writer_pretty(&mut output, &manifest)?;
        output.write_all(b"\n")?;
        output.sync_all()?;
        self.keep = true;
        Ok(())
    }
}

impl Drop for DirectoryWriter {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn number(value: &Value, field: &str) -> CliResult<u64> {
    value[field]
        .as_u64()
        .ok_or_else(|| cli_error("invalid recovery manifest number"))
}
fn part_name(offset: u64) -> String {
    format!("offset-{offset:020}.bin")
}
fn private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}
fn private_file(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn named_fragment_path(root: &Path, original: &str, offset: u64) -> Result<PathBuf, String> {
    let relative = original.strip_prefix('/').unwrap_or(original);
    let names: Vec<_> = relative.split('/').collect();
    if names.is_empty()
        || names.iter().any(|name| {
            name.is_empty()
                || *name == "."
                || *name == ".."
                || name
                    .chars()
                    .any(|ch| ch.is_control() || "\\<>:\"|?*".contains(ch))
                || name.ends_with(['.', ' '])
                || reserved_host_name(name)
        })
    {
        return Err(
            "original filename cannot be represented safely as a relative host path".to_owned(),
        );
    }
    // Keep named outputs separate from internal numeric staging/fallback files.
    let mut relative = PathBuf::from("files");
    for name in &names[..names.len() - 1] {
        relative.push(name);
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => {
                return Err("original filename collides with a non-directory or symlink".to_owned())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                private_directory(&current).map_err(|error| error.to_string())?
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    relative.push(format!(
        "{}.part-offset-{offset:020}",
        names.last().unwrap()
    ));
    Ok(relative)
}

fn reserved_host_name(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            base.strip_prefix(prefix).is_some_and(|suffix| {
                suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9')
            })
        })
}

pub(super) fn has_partial_files(plan: &Value) -> bool {
    plan["files"]
        .as_array()
        .is_some_and(|files| !files.is_empty())
}

pub(super) fn set_archive_destination(
    plan: &mut Value,
    archive: &Path,
    directory: &Path,
    dry_run: bool,
) -> CliResult<()> {
    plan["output"] = Value::String(std::path::absolute(archive)?.display().to_string());
    plan["recovery_directory"] = if has_partial_files(plan) {
        Value::String(directory.display().to_string())
    } else {
        Value::Null
    };
    plan["dry_run"] = Value::Bool(dry_run);
    Ok(())
}

pub(super) fn print_plan(plan: &Value, format: OutputFormat) -> CliResult<()> {
    let files = plan["files"]
        .as_array()
        .ok_or_else(|| cli_error("invalid recovery plan"))?;
    let known = plan["membership"] != "unavailable";
    let partial = files
        .iter()
        .filter(|file| file["status"] == "partial")
        .count();
    let unrecoverable = files
        .iter()
        .filter(|file| file["status"] == "unrecoverable")
        .count();
    let recovered = plan["complete_files"].as_u64();
    let mut destination = vec![
        vec![
            "output".to_owned(),
            plan["output"].as_str().unwrap_or("").to_owned(),
        ],
        vec![
            "recovery_directory".to_owned(),
            plan["recovery_directory"].as_str().unwrap_or("").to_owned(),
        ],
        vec!["dry_run".to_owned(), plan["dry_run"].to_string()],
        vec![
            "recovered_files".to_owned(),
            recovered.map_or_else(|| "unknown".to_owned(), |count| count.to_string()),
        ],
        vec![
            "partial_files".to_owned(),
            if known {
                partial.to_string()
            } else {
                "unknown".to_owned()
            },
        ],
        vec![
            "not_recovered_files".to_owned(),
            if known {
                unrecoverable.to_string()
            } else {
                "unknown".to_owned()
            },
        ],
        vec![
            "file_membership".to_owned(),
            plan["membership"].as_str().unwrap_or("unknown").to_owned(),
        ],
        vec![
            "coverage_note".to_owned(),
            plan["reason"].as_str().unwrap_or("").to_owned(),
        ],
    ];
    if format == OutputFormat::Table {
        let planned = plan["dry_run"].as_bool().unwrap_or(false);
        if let Some(path) = plan["output"].as_str().filter(|path| !path.is_empty()) {
            println!(
                "{}: {path}",
                if planned {
                    "Planned recovered Lockbox"
                } else {
                    "Recovered Lockbox"
                }
            );
        }
        if let Some(path) = plan["recovery_directory"]
            .as_str()
            .filter(|path| !path.is_empty())
        {
            println!(
                "{}: {path}",
                if planned {
                    "Planned partial-file directory"
                } else {
                    "Partial-file directory"
                }
            );
        }
        destination.retain(|row| row[0] != "output" && row[0] != "recovery_directory");
    }
    print_records(&["field", "value"], destination, format)?;
    if format == OutputFormat::Table {
        if let Some(recovered) = recovered.filter(|_| known) {
            println!("Recovery summary: {recovered} recovered, {partial} partial, {unrecoverable} not recovered.");
        } else {
            println!("Recovery summary: unknown; authenticated file membership was unavailable.");
        }
    }
    if format != OutputFormat::Table {
        return print_records(
            &["field", "value"],
            vec![vec!["file_recovery_plan".to_owned(), plan.to_string()]],
            format,
        );
    }
    if files.is_empty() {
        return Ok(());
    }
    let rows = files
        .iter()
        .map(|file| {
            vec![
                file["path"].as_str().unwrap_or("").to_owned(),
                file["status"].as_str().unwrap_or("").to_owned(),
                file["recovered_bytes"].to_string(),
                file["size"].to_string(),
                file["missing_ranges"].to_string(),
            ]
        })
        .collect();
    print_records(
        &[
            "path",
            "status",
            "recovered bytes",
            "file bytes",
            "missing ranges",
        ],
        rows,
        format,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn partial_fragments_preserve_beginning_middle_and_end_gaps() {
        for gap in 0..3u64 {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("recovery");
            let mut writer = DirectoryWriter::new(&root).unwrap();
            let pieces = [b"abc", b"def", b"ghi"];
            let mut ranges = Vec::new();
            for index in 0..3u64 {
                if index != gap {
                    writer.range(1, index * 3, pieces[index as usize]).unwrap();
                    ranges.push(json!({"offset":index*3,"length":3}));
                }
            }
            writer.finish(json!({"recovery_directory_protocol":1,"files":[{
                "id":1,"path":"/original","size":9,"status":"partial","recovered_bytes":6,
                "ranges":ranges,"missing_ranges":[{"offset":gap*3,"length":3,"reason":"unreadable frame"}]
            }]})).unwrap();
            assert!(!root.join("file-1/complete.bin").exists());
            let manifest: Value =
                serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
            for index in 0..3u64 {
                let range = manifest["files"][0]["ranges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|range| range["offset"] == index * 3);
                if index == gap {
                    assert!(range.is_none());
                } else {
                    assert_eq!(
                        fs::read(root.join(range.unwrap()["output"].as_str().unwrap())).unwrap(),
                        pieces[index as usize]
                    );
                }
            }
            assert_eq!(manifest["files"][0]["missing_ranges"][0]["offset"], gap * 3);
        }
    }

    #[test]
    fn partial_names_preserve_hierarchy_and_unsafe_paths_have_explicit_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let mut writer = DirectoryWriter::new(&root).unwrap();
        let mut files = Vec::new();
        for (index, path) in ["/a/model.obj", "/b/model.obj", "../../escape", "/NUL.obj"]
            .iter()
            .enumerate()
        {
            let id = index as u64 + 1;
            writer.range(id, 0, b"abc").unwrap();
            files.push(json!({"id":id,"path":path,"size":4,"status":"partial","recovered_bytes":3,
                "ranges":[{"offset":0,"length":3}],"missing_ranges":[{"offset":3,"length":1,"reason":"missing"}]}));
        }
        writer
            .finish(json!({"recovery_directory_protocol":1,"files":files}))
            .unwrap();
        let manifest: Value =
            serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(
            manifest["files"][0]["ranges"][0]["output"],
            "files/a/model.obj.part-offset-00000000000000000000"
        );
        assert_eq!(
            manifest["files"][1]["ranges"][0]["output"],
            "files/b/model.obj.part-offset-00000000000000000000"
        );
        assert!(manifest["files"][2]["ranges"][0]["filename_fallback_reason"].is_string());
        assert!(manifest["files"][3]["ranges"][0]["filename_fallback_reason"].is_string());
        for file in manifest["files"].as_array().unwrap() {
            assert_eq!(
                fs::read(root.join(file["ranges"][0]["output"].as_str().unwrap())).unwrap(),
                b"abc"
            );
        }
        assert!(!temp.path().join("escape").exists());
        #[cfg(unix)]
        {
            let second = temp.path().join("symlink-test");
            let mut writer = DirectoryWriter::new(&second).unwrap();
            writer.range(1, 0, b"abc").unwrap();
            std::os::unix::fs::symlink(temp.path(), second.join("files")).unwrap();
            writer.finish(json!({"recovery_directory_protocol":1,"files":[{
                "id":1,"path":"/escape","size":4,"status":"partial","recovered_bytes":3,
                "ranges":[{"offset":0,"length":3}],"missing_ranges":[{"offset":3,"length":1,"reason":"missing"}]
            }]})).unwrap();
            assert!(!temp
                .path()
                .join("escape.part-offset-00000000000000000000")
                .exists());
        }
    }

    #[test]
    fn output_names_are_numeric_and_empty_files_remain_complete() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let mut writer = DirectoryWriter::new(&root).unwrap();
        writer.range(1, 0, b"known bytes").unwrap();
        writer.finish(json!({"recovery_directory_protocol":1,"files":[
            {"id":1,"path":"../../outside","size":11,"status":"complete","recovered_bytes":11,"ranges":[{"offset":0,"length":11}],"missing_ranges":[]},
            {"id":2,"path":"/empty","size":0,"status":"complete","recovered_bytes":0,"ranges":[],"missing_ranges":[]}
        ]})).unwrap();
        assert_eq!(
            fs::read(root.join("file-1/complete.bin")).unwrap(),
            b"known bytes"
        );
        assert_eq!(fs::read(root.join("file-2/complete.bin")).unwrap(), b"");
        assert!(!temp.path().join("outside").exists());
        assert!(DirectoryWriter::new(&root).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::{symlink, PermissionsExt};
            assert_eq!(
                fs::metadata(&root).unwrap().permissions().mode() & 0o777,
                0o700
            );
            assert_eq!(
                fs::metadata(root.join("file-1/complete.bin"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
            let link = temp.path().join("link");
            symlink(&root, &link).unwrap();
            assert!(DirectoryWriter::new(&link).is_err());
        }
    }

    #[test]
    fn overlapping_or_misdescribed_fragments_are_not_published() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("recovery");
        let mut writer = DirectoryWriter::new(&root).unwrap();
        writer.range(1, 0, b"abc").unwrap();
        assert!(writer.range(1, 2, b"wrong").is_err());
        assert!(writer
            .finish(json!({"recovery_directory_protocol":1,"files":[{
                "id":1,"path":"/bad","size":3,"status":"complete","recovered_bytes":2,
                "ranges":[{"offset":0,"length":3}],"missing_ranges":[]
            }]}))
            .is_err());
        assert!(!root.exists());
    }
}
