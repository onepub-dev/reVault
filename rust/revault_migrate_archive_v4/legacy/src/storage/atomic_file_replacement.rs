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
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let process_id = std::process::id();
        for attempt in 0..1000u64 {
            let temp_path = parent.join(format!("{stem}-{process_id}-{attempt}.tmp"));
            match OpenOptions::new()
                .read(true)
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

    pub(crate) fn for_compaction(destination: &Path) -> Result<Self> {
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).map_err(|err| Error::Io(err.to_string()))?;
        let nonce = u128::from_le_bytes(nonce);
        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lockbox");
        Ok(Self {
            temp_path: destination.with_file_name(format!(".{file_name}.compact-{nonce:032x}")),
            destination: destination.to_path_buf(),
        })
    }

    pub(crate) fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    pub(crate) fn discard(&self) {
        let _ = fs::remove_file(&self.temp_path);
    }

    pub(crate) fn install(&self) -> Result<()> {
        self.publish()?;
        self.sync_parent()
    }

    pub(crate) fn publish(&self) -> Result<()> {
        match fs::rename(&self.temp_path, &self.destination) {
            Ok(()) => {}
            Err(err) => {
                return Err(Error::Io(format!(
                    "replace {}: {err}",
                    self.destination.display()
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn sync_parent(&self) -> Result<()> {
        #[cfg(unix)]
        {
            let parent = self
                .destination
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let dir = File::open(parent)
                .map_err(|err| Error::Io(format!("open {}: {err}", parent.display())))?;
            dir.sync_data()
                .map_err(|err| Error::Io(format!("sync {}: {err}", parent.display())))?;
        }
        Ok(())
    }
}
