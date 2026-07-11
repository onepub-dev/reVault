use super::Lockbox;
use crate::lockbox_path::{glob_matches, validate_glob};
use crate::node_kind::NodeKind;
use crate::{ListOptions, LockboxEntry, LockboxPath, Result};

impl<State> Lockbox<State> {
    /// Return an iterator over entries matching listing options.
    ///
    /// Returns `Error::InvalidPath` if the list root or glob pattern is unsafe.
    /// Iteration returns only table-of-contents metadata. It does not read
    /// symlink page objects; call `get_symlink_target` for symlink targets.
    pub fn list(
        &self,
        options: ListOptions,
    ) -> Result<impl Iterator<Item = Result<LockboxEntry>> + '_> {
        let root = options.path.clone();
        let glob = match &options.glob {
            Some(pattern) => Some(validate_glob(pattern)?),
            None => None,
        };
        let prefix = root.descendant_prefix();
        let mut yielded = 0usize;
        let iter = self.toc_entries.values().filter_map(move |entry| {
            if entry.deleted || !entry.path.is_descendant_of(&root) {
                return None;
            }
            if entry.node_kind == NodeKind::File && !options.include_files {
                return None;
            }
            if entry.node_kind == NodeKind::Symlink && !options.include_symlinks {
                return None;
            }
            if entry.node_kind == NodeKind::Directory && !options.include_directories {
                return None;
            }
            let rest = &entry.path[prefix.len()..];
            if rest.is_empty() {
                return None;
            }
            if !options.recursive && !entry.path.is_direct_child_of(&root) {
                return None;
            }
            if let Some(pattern) = &glob {
                if !glob_matches(pattern, rest) && !glob_matches(pattern, &entry.path) {
                    return None;
                }
            }
            if let Some(limit) = options.limit {
                if yielded >= limit {
                    return None;
                }
            }
            yielded += 1;
            Some(Ok(entry.to_public_entry()))
        });
        Ok(iter)
    }

    /// Return metadata for one file, symlink, or directory.
    pub fn stat(&self, path: &LockboxPath) -> Option<LockboxEntry> {
        let path = path.as_file_path().ok()?;
        self.toc_entries
            .get(path)
            .filter(|e| !e.deleted)
            .map(|entry| entry.to_public_entry())
    }

    /// Return true when `path` names an existing file, symlink, or directory entry.
    pub fn exists(&self, path: &LockboxPath) -> bool {
        let Ok(path) = path.as_file_path() else {
            return false;
        };
        self.toc_entries
            .get(path)
            .filter(|entry| !entry.deleted)
            .is_some()
    }

    /// Return true when `path` names an existing directory entry.
    pub fn is_dir(&self, path: &LockboxPath) -> bool {
        let Ok(path) = path.as_file_path() else {
            return false;
        };
        self.toc_entries
            .get(path)
            .filter(|entry| !entry.deleted)
            .is_some_and(|entry| entry.node_kind == NodeKind::Directory)
    }
}
