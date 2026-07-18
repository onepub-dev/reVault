use super::Lockbox;
use crate::constants::DEFAULT_DIRECTORY_PERMISSIONS;
use crate::lockbox_path::LockboxPath;
use crate::node_kind::NodeKind;
use crate::security::validate_permissions;
use crate::toc_entry::TocEntry;
use crate::{Error, Result};

impl<State> Lockbox<State> {
    /// Create a directory entry.
    ///
    /// When `create_parents` is true, missing parent directories are created
    /// first. The root directory `/` is implicit and cannot be created.
    pub fn create_dir(&mut self, path: &LockboxPath, create_parents: bool) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        if self.live_entry(&path).is_some() {
            return Err(Error::AlreadyExists(path.to_string()));
        }
        if create_parents {
            for directory in missing_directory_chain(&path)? {
                if let Some(entry) = self.live_entry(&directory) {
                    if entry.node_kind != NodeKind::Directory {
                        return Err(Error::InvalidOperation(format!(
                            "{} is not a directory",
                            directory.as_str()
                        )));
                    }
                    continue;
                }
                self.insert_directory_entry(directory, DEFAULT_DIRECTORY_PERMISSIONS);
            }
            return Ok(());
        }
        self.ensure_parent_directory(&path)?;
        self.insert_directory_entry(path, DEFAULT_DIRECTORY_PERMISSIONS);
        Ok(())
    }

    /// Creates every missing parent directory required by `path`.
    ///
    /// The path itself is not created.
    pub fn create_parent_dirs_for(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let Some(parent) = path.parent()? else {
            return Ok(());
        };
        if self.is_dir(&parent) {
            return Ok(());
        }
        match self.create_dir(&parent, true) {
            Ok(()) | Err(Error::AlreadyExists(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Remove an empty directory entry.
    pub fn remove_dir(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let entry = self
            .live_entry(&path)
            .cloned()
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        if entry.node_kind != NodeKind::Directory {
            return Err(Error::InvalidOperation(format!(
                "{} is not a directory",
                path.as_str()
            )));
        }
        if self.directory_has_children(&path) {
            return Err(Error::InvalidOperation(format!(
                "directory is not empty: {}",
                path.as_str()
            )));
        }
        self.remove_toc_entry(&path)?;
        Ok(())
    }

    /// Remove a directory entry and all descendants.
    pub fn remove_dir_recursive(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let entry = self
            .live_entry(&path)
            .cloned()
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        if entry.node_kind != NodeKind::Directory {
            return Err(Error::InvalidOperation(format!(
                "{} is not a directory",
                path.as_str()
            )));
        }
        let mut paths = self
            .toc_entries
            .values()
            .filter(|entry| {
                !entry.deleted && (entry.path == path || entry.path.is_descendant_of(&path))
            })
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        paths.sort_by_key(|path| std::cmp::Reverse(path.as_str().len()));
        for path in paths {
            self.remove_toc_entry(&path)?;
        }
        Ok(())
    }

    /// Change stored Unix-style permission bits on a file, symlink, or directory.
    pub fn set_permissions(&mut self, path: &LockboxPath, permissions: u32) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        let permissions = validate_permissions(permissions)?;
        let entry = self
            .toc_entries
            .get_mut(path.as_str())
            .filter(|entry| !entry.deleted)
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        entry.permissions = permissions;
        self.mark_toc_dirty(&path);
        Ok(())
    }

    /// Delete a file or symlink from the lockbox.
    ///
    /// Returns `Error::InvalidPath` for directory-only paths, `Error::NotFound`
    /// if `path` does not name an existing entry, and storage errors if pending data
    /// must be flushed before deletion.
    pub fn delete(&mut self, path: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let path = path.file_path()?;
        if self.should_discard_file_pages_after_flush()
            && self.pending_small_files.contains_key(path.as_str())
        {
            self.flush_bulk_small_file_packer()?;
        }
        self.remove_pending_small_file(&path);
        let old = self
            .toc_entries
            .get(path.as_str())
            .filter(|entry| !entry.deleted)
            .cloned()
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        if old.node_kind == NodeKind::Directory {
            return Err(Error::InvalidOperation(format!(
                "{} is a directory; use remove_dir or remove_dir_recursive",
                path.as_str()
            )));
        }
        self.remove_toc_entry(&path)?;
        Ok(())
    }

    /// Rename one file/symlink or a directory prefix.
    ///
    /// Returns `Error::InvalidPath` for unsafe file paths, self-nested
    /// directory moves, or generated destination paths that are not valid
    /// lockbox file paths. Returns `Error::NotFound` when the source file or
    /// directory prefix does not exist. Existing destination entries are
    /// replaced by the rename.
    pub fn rename(&mut self, from: &LockboxPath, to: &LockboxPath) -> Result<()>
    where
        State: crate::WritableLockboxState,
    {
        let from_path = from.file_path()?;
        let to_path = to.file_path()?;
        let entry = self
            .live_entry(&from_path)
            .cloned()
            .ok_or_else(|| Error::NotFound(from_path.to_string()))?;
        if entry.node_kind == NodeKind::Directory {
            self.rename_directory_entry(&from_path, &to_path)?;
            return Ok(());
        }
        self.rename_toc_entry(&from_path, &to_path)?;
        Ok(())
    }

    pub(crate) fn ensure_parent_directory(&self, path: &LockboxPath) -> Result<()> {
        let Some(parent) = path.parent()? else {
            return Ok(());
        };
        match self.live_entry(&parent) {
            Some(entry) if entry.node_kind == NodeKind::Directory => Ok(()),
            Some(_) => Err(Error::InvalidOperation(format!(
                "{} is not a directory",
                parent.as_str()
            ))),
            None => Err(Error::NotFound(parent.to_string())),
        }
    }

    fn rename_directory_entry(
        &mut self,
        from_path: &LockboxPath,
        to_path: &LockboxPath,
    ) -> Result<()> {
        if from_path == to_path {
            return Ok(());
        }
        if to_path.is_descendant_of(from_path) {
            return Err(Error::InvalidPath(to_path.to_string()));
        }
        self.ensure_parent_directory(to_path)?;
        if self.live_entry(to_path).is_some() {
            return Err(Error::AlreadyExists(to_path.to_string()));
        }

        let entries = self
            .toc_entries
            .values()
            .filter(|entry| {
                !entry.deleted
                    && (entry.path == *from_path || entry.path.is_descendant_of(from_path))
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut moves = Vec::with_capacity(entries.len());
        for entry in entries {
            let new_path = if entry.path == *from_path {
                to_path.clone()
            } else {
                let suffix = &entry.path.as_str()[from_path.as_str().len()..];
                LockboxPath::from_api(&format!("{}{}", to_path.as_str(), suffix), false)?
            };
            if let Some(existing) = self.live_entry(&new_path) {
                if existing.path != entry.path {
                    return Err(Error::AlreadyExists(new_path.to_string()));
                }
            }
            moves.push((entry.path, new_path));
        }
        moves.sort_by_key(|(from, _)| from.as_str().len());
        for (from, to) in moves {
            self.rename_toc_entry(&from, &to)?;
        }
        Ok(())
    }

    fn rename_toc_entry(&mut self, from_path: &LockboxPath, to_path: &LockboxPath) -> Result<()> {
        if from_path == to_path {
            return Ok(());
        }
        self.ensure_parent_directory(to_path)?;

        if self.should_discard_file_pages_after_flush()
            && (self.pending_small_files.contains_key(from_path.as_str())
                || self.pending_small_files.contains_key(to_path.as_str()))
        {
            self.flush_bulk_small_file_packer()?;
        }

        if let Some(old_target) = self
            .toc_entries
            .get(to_path.as_str())
            .filter(|entry| !entry.deleted)
            .cloned()
        {
            if old_target.node_kind == NodeKind::Directory {
                return Err(Error::InvalidOperation(format!(
                    "{} is a directory",
                    to_path.as_str()
                )));
            }
            self.pending_symlinks.remove(to_path.as_str());
            self.free_entry_slots(old_target)?;
            self.toc_entries.remove(to_path.as_str());
        }

        let mut entry = self
            .toc_entries
            .remove(from_path.as_str())
            .filter(|entry| !entry.deleted)
            .ok_or_else(|| Error::NotFound(from_path.to_string()))?;

        if let Some(pending) = self.remove_pending_small_file(from_path) {
            self.insert_pending_small_file(
                to_path.clone(),
                crate::file_chunk::PendingFileChunk {
                    path: to_path.clone(),
                    ..pending
                },
            );
        }
        if let Some(target) = self.pending_symlinks.remove(from_path.as_str()) {
            self.pending_symlinks.insert(to_path.clone(), target);
        } else if entry.node_kind == crate::node_kind::NodeKind::Symlink {
            let target = self.symlink_target_for_entry(&entry)?;
            self.free_entry_slots(entry.clone())?;
            self.pending_symlinks.insert(to_path.clone(), target);
        }

        entry.path = to_path.clone();
        self.toc_entries.insert(entry.path.clone(), entry);
        self.mark_toc_dirty(from_path);
        self.mark_toc_dirty(to_path);
        Ok(())
    }

    fn live_entry(&self, path: &LockboxPath) -> Option<&TocEntry> {
        self.toc_entries
            .get(path.as_str())
            .filter(|entry| !entry.deleted)
    }

    fn insert_directory_entry(&mut self, path: LockboxPath, permissions: u32) {
        self.toc_entries.insert(
            path.clone(),
            TocEntry {
                path: path.clone(),
                len: 0,
                record_offset: 0,
                record_len: 0,
                record_object_id: 0,
                deleted: false,
                node_kind: NodeKind::Directory,
                permissions,
                chunks: Vec::new(),
            },
        );
        self.mark_toc_dirty(&path);
    }

    fn directory_has_children(&self, path: &LockboxPath) -> bool {
        self.toc_entries
            .values()
            .any(|entry| !entry.deleted && entry.path.is_descendant_of(path))
    }

    fn remove_toc_entry(&mut self, path: &LockboxPath) -> Result<()> {
        self.pending_symlinks.remove(path.as_str());
        self.remove_pending_small_file(path);
        let old = self
            .toc_entries
            .remove(path.as_str())
            .filter(|entry| !entry.deleted)
            .ok_or_else(|| Error::NotFound(path.to_string()))?;
        self.free_entry_slots(old)?;
        self.mark_toc_dirty(path);
        Ok(())
    }
}

fn missing_directory_chain(path: &LockboxPath) -> Result<Vec<LockboxPath>> {
    let mut chain = Vec::new();
    let mut current = Some(path.clone());
    while let Some(path) = current {
        chain.push(path.clone());
        current = path.parent()?;
    }
    chain.reverse();
    Ok(chain)
}
