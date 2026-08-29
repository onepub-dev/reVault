use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Owns the temporary and destination paths for an atomic file replacement.
pub(crate) struct AtomicFileReplacement {
    temp_path: PathBuf,
    destination: PathBuf,
}

impl AtomicFileReplacement {
    pub(crate) fn create_unique(destination: &Path, stem: &str) -> Result<(Self, File)> {
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        let process_id = std::process::id();
        for attempt in 0..1000u64 {
            let temp_path = parent.join(format!("{stem}-{process_id}-{attempt}.tmp"));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
            {
                Ok(file) => {
                    return Ok((
                        Self {
                            temp_path,
                            destination: destination.to_path_buf(),
                        },
                        file,
                    ));
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => {
                    return Err(Error::Io(format!(
                        "create replacement for {}: {err}",
                        destination.display()
                    )));
                }
            }
        }
        Err(Error::Io(format!(
            "unable to create unique replacement for {}",
            destination.display()
        )))
    }

    pub(crate) fn for_compaction(destination: &Path) -> Self {
        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lockbox");
        Self {
            temp_path: destination
                .with_file_name(format!(".{file_name}.compact-{}", std::process::id())),
            destination: destination.to_path_buf(),
        }
    }

    pub(crate) fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    pub(crate) fn discard(&self) {
        let _ = fs::remove_file(&self.temp_path);
    }

    pub(crate) fn install(&self) -> Result<()> {
        match fs::rename(&self.temp_path, &self.destination) {
            Ok(()) => {}
            #[cfg(windows)]
            Err(rename_err) if self.destination.exists() => {
                fs::remove_file(&self.destination).map_err(|remove_err| {
                    Error::Io(format!(
                        "replace {}: remove existing failed after rename error {rename_err}: {remove_err}",
                        self.destination.display()
                    ))
                })?;
                fs::rename(&self.temp_path, &self.destination).map_err(|err| {
                    Error::Io(format!("replace {}: {err}", self.destination.display()))
                })?;
            }
            Err(err) => {
                return Err(Error::Io(format!(
                    "replace {}: {err}",
                    self.destination.display()
                )));
            }
        }
        self.sync_parent()
    }

    fn sync_parent(&self) -> Result<()> {
        #[cfg(unix)]
        {
            let parent = self.destination.parent().unwrap_or_else(|| Path::new("."));
            let dir = File::open(parent)
                .map_err(|err| Error::Io(format!("open {}: {err}", parent.display())))?;
            dir.sync_data()
                .map_err(|err| Error::Io(format!("sync {}: {err}", parent.display())))?;
        }
        Ok(())
    }
}
