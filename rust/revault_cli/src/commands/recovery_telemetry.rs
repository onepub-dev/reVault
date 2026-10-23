//! Recovery progress and deliberately limited, opt-in diagnostic reports.
use super::context::CliResult;
use serde_json::{json, Value};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

const MAX_EVENTS: usize = 4096;
pub(super) const EVENT_PREFIX: &str = "REVAULT_RECOVERY_EVENT ";

pub(super) struct RecoveryTelemetry {
    file: Option<File>,
    quiet: bool,
    started: Instant,
    state: Mutex<State>,
}

struct State {
    stage: &'static str,
    events: Vec<Value>,
    counts: Value,
    helper_version: Option<String>,
    helper_error: Option<&'static str>,
    dropped_events: u64,
}

impl RecoveryTelemetry {
    pub(super) fn new(path: Option<&Path>, quiet: bool) -> CliResult<Self> {
        let file = path
            .map(|path| {
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                options.open(path)
            })
            .transpose()?;
        Ok(Self {
            file,
            quiet,
            started: Instant::now(),
            state: Mutex::new(State {
                stage: "starting",
                events: Vec::new(),
                counts: Value::Null,
                helper_version: None,
                helper_error: None,
                dropped_events: 0,
            }),
        })
    }

    pub(super) fn event(&self, stage: &'static str, progress: Option<(u64, u64)>) {
        let mut state = self.state.lock().unwrap();
        if stage != "helper_failed" {
            state.stage = stage;
        }
        let mut event = json!({"stage":stage,"elapsed_ms":self.started.elapsed().as_millis()});
        if let Some((completed, total)) = progress {
            event["completed_files"] = json!(completed);
            event["total_files"] = json!(total);
            if completed < total {
                event["entry_id"] = json!(completed + 1);
            }
        }
        if self.file.is_some() {
            if state.events.len() < MAX_EVENTS {
                state.events.push(event);
            } else {
                state.dropped_events += 1;
            }
        }
        if !self.quiet {
            if let Some((completed, total)) = progress {
                eprintln!("Recovery: verifying files {completed}/{total}.");
            } else {
                eprintln!("Recovery: {}.", stage.replace('_', " "));
            }
        }
    }

    pub(super) fn heartbeat(&self) {
        if !self.quiet {
            let state = self.state.lock().unwrap();
            eprintln!(
                "Recovery: {} ({}s elapsed; still working).",
                state.stage.replace('_', " "),
                self.started.elapsed().as_secs()
            );
        }
    }

    /// Accept only fixed stage names and numeric fields, never raw error text.
    pub(super) fn helper_event(&self, line: &str) -> bool {
        let Some(payload) = line.strip_prefix(EVENT_PREFIX) else {
            return false;
        };
        let Ok(value) = serde_json::from_str::<Value>(payload) else {
            return true;
        };
        let stage = match value["stage"].as_str() {
            Some("reading_source") => "reading_source",
            Some("unlocking") => "unlocking",
            Some("scanning_records") => "scanning_records",
            Some("restoring_archive") => "restoring_archive",
            Some("verifying_files") => "verifying_files",
            Some("exporting_archive") => "exporting_archive",
            Some("helper_failed") => "helper_failed",
            _ => return true,
        };
        let progress = value["completed_files"]
            .as_u64()
            .zip(value["total_files"].as_u64())
            .filter(|(done, total)| done <= total);
        {
            let mut state = self.state.lock().unwrap();
            if let Some(version) = value["version"].as_str().filter(|version| {
                version.len() <= 32
                    && version
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || byte == b'.')
            }) {
                state.helper_version = Some(version.to_owned());
            }
            state.helper_error = match value["error_category"].as_str() {
                Some("authentication") => Some("authentication"),
                Some("integrity") => Some("integrity"),
                Some("mutation") => Some("mutation"),
                Some("io") => Some("io"),
                Some("other") => Some("other"),
                _ => state.helper_error,
            };
        }
        self.event(stage, progress);
        true
    }

    pub(super) fn counts(&self, plan: &Value) {
        let files = plan["files"].as_array();
        self.state.lock().unwrap().counts = if plan["membership"] == "verified_commit_toc" {
            json!({"recovered":plan["complete_files"].as_u64(),
                "partial":files.map(|files| files.iter().filter(|file| file["status"] == "partial").count()),
                "not_recovered":files.map(|files| files.iter().filter(|file| file["status"] == "unrecoverable").count())})
        } else {
            Value::Null
        };
    }

    pub(super) fn finish(
        &mut self,
        error: Option<&(dyn std::error::Error + 'static)>,
    ) -> CliResult<()> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };
        let state = self.state.lock().unwrap();
        let category =
            error.map(
                |error| match error.downcast_ref::<revault_lockbox_api::Error>() {
                    Some(revault_lockbox_api::Error::InvalidKey) => "authentication",
                    Some(
                        revault_lockbox_api::Error::CorruptHeader
                        | revault_lockbox_api::Error::CorruptRecord
                        | revault_lockbox_api::Error::Truncated,
                    ) => "integrity",
                    Some(revault_lockbox_api::Error::Io(_)) => "io",
                    _ => "operation_failed",
                },
            );
        let report = json!({"diagnostic_schema":1,"cli_version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
            "helper_version":state.helper_version,"status":if error.is_some() {"failed"} else {"complete"},
            "error_category":state.helper_error.or(category),"last_stage":state.stage,
            "elapsed_ms":self.started.elapsed().as_millis(),"counts":state.counts,
            "events":state.events,"dropped_events":state.dropped_events});
        serde_json::to_writer_pretty(&mut *file, &report)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_events_are_allowlisted_bounded_and_preserve_failure_stage() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("report.json");
        let mut telemetry = RecoveryTelemetry::new(Some(&path), true).unwrap();
        telemetry.helper_event(r#"REVAULT_RECOVERY_EVENT {"stage":"verifying_files","completed_files":2,"total_files":4,"version":"0.1.0","path":"PRIVATE_SENTINEL","error":"PRIVATE_SENTINEL"}"#);
        telemetry.helper_event(
            r#"REVAULT_RECOVERY_EVENT {"stage":"PRIVATE_SENTINEL","version":"PRIVATE_SENTINEL"}"#,
        );
        for _ in 0..MAX_EVENTS {
            telemetry.event("verifying_files", Some((2, 4)));
        }
        telemetry.helper_event(r#"REVAULT_RECOVERY_EVENT {"stage":"helper_failed","error_category":"mutation","error":"PRIVATE_SENTINEL"}"#);
        let error = std::io::Error::other("PRIVATE_SENTINEL");
        telemetry.finish(Some(&error)).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(!text.contains("PRIVATE_SENTINEL"));
        let report: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(report["last_stage"], "verifying_files");
        assert_eq!(report["error_category"], "mutation");
        assert_eq!(report["events"].as_array().unwrap().len(), MAX_EVENTS);
        assert!(report["dropped_events"].as_u64().unwrap() > 0);
    }
}
