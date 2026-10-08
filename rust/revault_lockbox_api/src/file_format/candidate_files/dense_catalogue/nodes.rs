//! Typed filesystem metadata for the bounded catalogue experiment. Reuses public
//! path/permission rules; this is not the public mutation or extraction adapter.
use super::*;
use crate::{LockboxEntry, LockboxEntryKind, LockboxPath};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Metadata {
    pub entry: LockboxEntry,
    pub target: Option<LockboxPath>,
}
pub(crate) struct Node {
    pub(super) path: Zeroizing<Vec<u8>>,
    pub(super) permissions: u32,
    pub(super) target: Option<Zeroizing<Vec<u8>>>,
}
pub(super) fn canonical(bytes: &[u8]) -> Result<&str> {
    let path = std::str::from_utf8(bytes).map_err(|_| Error::CorruptRecord)?;
    // Keep the validator's temporary canonical copy wipeable too.
    let checked = Zeroizing::new(
        crate::lockbox_path::canonicalize_stored_path(path, false)
            .map_err(|_| Error::CorruptRecord)?,
    );
    if checked.as_str() != path {
        return Err(Error::CorruptRecord);
    }
    Ok(path)
}
pub(super) fn decode(cursor: &mut Cursor<'_>, files: usize) -> Result<Vec<Node>> {
    let count = cursor.count(MAX_MODEL_FILES.saturating_sub(files))?;
    if count > cursor.0.len() / 8 {
        return Err(Error::CorruptRecord);
    }
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = crate::node_kind::NodeKind::from_u8(cursor.take(1)?[0])?;
        let n = cursor.count(crate::constants::MAX_PATH_BYTES)?;
        let path = Zeroizing::new(cursor.take(n)?.to_vec());
        let permissions = u32::from_le_bytes(cursor.take(4)?.try_into().unwrap());
        let n = cursor.count(crate::constants::MAX_PATH_BYTES)?;
        let target = match kind {
            crate::node_kind::NodeKind::Directory if n == 0 => None,
            crate::node_kind::NodeKind::Symlink if n > 0 => {
                Some(Zeroizing::new(cursor.take(n)?.to_vec()))
            }
            _ => return Err(Error::CorruptRecord),
        };
        nodes.push(Node {
            path,
            permissions,
            target,
        });
    }
    Ok(nodes)
}
pub(super) fn encode(nodes: &[Node], out: &mut Writer) -> Result<()> {
    out.uint(nodes.len() as u64)?;
    for node in nodes {
        out.put(&[if node.target.is_some() { 2 } else { 3 }])?;
        out.uint(node.path.len() as u64)?;
        out.put(&node.path)?;
        out.put(&node.permissions.to_le_bytes())?;
        let target = node.target.as_deref().map(Vec::as_slice).unwrap_or(&[]);
        out.uint(target.len() as u64)?;
        out.put(target)?;
    }
    Ok(())
}
pub(super) fn validate(files: &[File], nodes: &[Node]) -> Result<()> {
    validate_bounded(files, nodes, MAX_MODEL_FILES)
}
pub(super) fn validate_bounded(files: &[File], nodes: &[Node], maximum: usize) -> Result<()> {
    if files.len() + nodes.len() > maximum {
        return Err(Error::CorruptRecord);
    }
    let mut paths = BTreeMap::new();
    for file in files {
        canonical(&file.path)?;
        if paths.insert(file.path.as_slice(), false).is_some() {
            return Err(Error::CorruptRecord);
        }
        crate::security::validate_permissions(file.permissions)
            .map_err(|_| Error::CorruptRecord)?;
    }
    let mut previous: Option<&[u8]> = None;
    for node in nodes {
        canonical(&node.path)?;
        crate::security::validate_permissions(node.permissions)
            .map_err(|_| Error::CorruptRecord)?;
        if previous.is_some_and(|old| old >= node.path.as_slice())
            || paths
                .insert(node.path.as_slice(), node.target.is_none())
                .is_some()
        {
            return Err(Error::CorruptRecord);
        }
        previous = Some(&node.path);
        if let Some(target) = &node.target {
            canonical(target)?;
        }
    }
    for path in paths.keys() {
        let split = path
            .iter()
            .rposition(|byte| *byte == b'/')
            .ok_or(Error::CorruptRecord)?;
        if split != 0 && paths.get(&path[..split]) != Some(&true) {
            return Err(Error::CorruptRecord);
        }
    }
    Ok(())
}
impl Catalogue {
    /// Apply a complete metadata snapshot while preserving every file identity,
    /// fragment binding and payload allocation. No file may be added or omitted.
    pub(in crate::file_format::candidate_files) fn set_filesystem_metadata(
        &mut self,
        entries: &[Metadata],
    ) -> Result<()> {
        self.set_metadata_bounded(entries, MAX_MODEL_FILES)
    }
    pub(super) fn set_metadata_bounded(
        &mut self,
        entries: &[Metadata],
        maximum: usize,
    ) -> Result<()> {
        if entries.len() > maximum {
            return Err(Error::SecurityLimitExceeded(
                "bounded filesystem metadata".into(),
            ));
        }
        let mut seen = BTreeSet::new();
        let mut nodes = Vec::new();
        let mut files = BTreeSet::new();
        for metadata in entries {
            let entry = &metadata.entry;
            let path = entry.path.as_str().as_bytes();
            canonical(path)?;
            crate::security::validate_permissions(entry.permissions)?;
            if !seen.insert(path) {
                return Err(Error::CorruptRecord);
            }
            match entry.kind {
                LockboxEntryKind::File => {
                    if metadata.target.is_some() {
                        return Err(Error::CorruptRecord);
                    }
                    let index = self
                        .files
                        .binary_search_by(|file| file.path.as_slice().cmp(path))
                        .map_err(|_| Error::CorruptRecord)?;
                    if self.files[index].info.len != entry.len {
                        return Err(Error::CorruptRecord);
                    }
                    self.files[index].permissions = entry.permissions;
                    files.insert(index);
                }
                LockboxEntryKind::Directory | LockboxEntryKind::Symlink => {
                    if entry.len != 0
                        || (entry.kind == LockboxEntryKind::Symlink) != metadata.target.is_some()
                    {
                        return Err(Error::CorruptRecord);
                    }
                    nodes.push(Node {
                        path: Zeroizing::new(path.to_vec()),
                        permissions: entry.permissions,
                        target: metadata
                            .target
                            .as_ref()
                            .map(|target| Zeroizing::new(target.as_str().as_bytes().to_vec())),
                    });
                }
            }
        }
        if files.len() != self.files.len() {
            return Err(Error::CorruptRecord);
        }
        nodes.sort_by(|a, b| a.path.cmp(&b.path));
        validate_bounded(&self.files, &nodes, maximum)?;
        self.nodes = nodes;
        self.typed = true;
        Ok(())
    }
    pub(in crate::file_format::candidate_files) fn filesystem_metadata(
        &self,
    ) -> Result<Vec<Metadata>> {
        if !self.typed {
            return Err(Error::InvalidOperation(
                "legacy file-only catalogue has no canonical filesystem metadata".into(),
            ));
        }
        let mut entries = Vec::with_capacity(self.files.len() + self.nodes.len());
        for file in &self.files {
            entries.push(Metadata {
                entry: LockboxEntry {
                    path: LockboxPath::from_stored(canonical(&file.path)?, false)?,
                    kind: LockboxEntryKind::File,
                    len: file.info.len,
                    permissions: file.permissions,
                },
                target: None,
            });
        }
        for node in &self.nodes {
            entries.push(Metadata {
                entry: LockboxEntry {
                    path: LockboxPath::from_stored(canonical(&node.path)?, false)?,
                    kind: if node.target.is_some() {
                        LockboxEntryKind::Symlink
                    } else {
                        LockboxEntryKind::Directory
                    },
                    len: 0,
                    permissions: node.permissions,
                },
                target: node
                    .target
                    .as_ref()
                    .map(|target| LockboxPath::from_stored(canonical(target)?, false))
                    .transpose()?,
            });
        }
        entries.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_node_decoder_refuses_unknown_kinds_and_oversized_counts() {
        let nodes = vec![Node {
            path: Zeroizing::new(b"/link".to_vec()),
            permissions: 0o700,
            target: Some(Zeroizing::new(b"/missing".to_vec())),
        }];
        let mut writer = Writer(ZeroizingBytes::new(Vec::with_capacity(MAX_BODY)));
        encode(&nodes, &mut writer).unwrap();
        let mut cursor = Cursor(&writer.0);
        let decoded = decode(&mut cursor, 0).unwrap();
        validate(&[], &decoded).unwrap();
        assert!(cursor.0.is_empty());
        for kind in [0, 1, 3, 255] {
            let mut bad = writer.0.clone();
            bad[1] = kind;
            assert!(decode(&mut Cursor(&bad), 0).is_err());
        }
        assert!(decode(&mut Cursor(&[0xff; 10]), 0).is_err());
        assert!(decode(&mut Cursor(&writer.0), MAX_MODEL_FILES).is_err());
        let mut invalid = decoded;
        invalid[0].path = Zeroizing::new(b"/missing/link".to_vec());
        assert!(validate(&[], &invalid).is_err());
    }
}
