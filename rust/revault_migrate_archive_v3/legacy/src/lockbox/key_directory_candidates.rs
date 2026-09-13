use std::cmp::Reverse;

use crate::file_format::read_header;
use crate::key_directory::{
    best_key_directory, read_key_directory, read_key_directory_via_page_cache,
    scan_key_directories, DecodedKeyDirectory,
};
use crate::storage::{Storage, StorageBackend};
use crate::{Error, Result};

/// Key-directory copies discovered in preferred opening order.
pub(crate) struct KeyDirectoryCandidates {
    directories: Vec<DecodedKeyDirectory>,
}

impl KeyDirectoryCandidates {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut directories = Vec::new();
        if let Ok(header) = read_header(bytes) {
            let lockbox_id = header.lockbox_id;
            if let Ok(directory) =
                read_key_directory(bytes, header.key_directory_offset, Some(lockbox_id))
            {
                directories.push(directory);
            }
            directories.extend(scan_key_directories(bytes, Some(lockbox_id)));
        } else {
            directories.extend(scan_key_directories(bytes, None));
        }
        Self::ranked(directories)
    }

    pub(crate) fn from_storage(storage: &StorageBackend) -> Result<Self> {
        let header_bytes = storage.read_at(0, crate::constants::HEADER_LEN)?;
        let mut directories = Vec::new();
        if let Ok(header) = read_header(&header_bytes) {
            let lockbox_id = header.lockbox_id;
            if let Ok(directory) = read_key_directory_via_page_cache(
                storage,
                header.key_directory_offset,
                Some(lockbox_id),
            ) {
                directories.push(directory);
            }
            if directories.is_empty() {
                directories.extend(scan_key_directories(&storage.read_all()?, Some(lockbox_id)));
            }
        } else {
            directories.extend(scan_key_directories(&storage.read_all()?, None));
        }
        Self::ranked(directories)
    }

    pub(crate) fn into_ranked(self) -> Vec<DecodedKeyDirectory> {
        self.directories
    }

    fn ranked(mut directories: Vec<DecodedKeyDirectory>) -> Result<Self> {
        let best = best_key_directory(directories.clone()).ok_or(Error::CorruptHeader)?;
        directories.sort_by_key(|directory| {
            (
                Reverse(directory.lockbox_id == best.lockbox_id),
                Reverse(directory.generation),
                directory.copy_index,
            )
        });
        Ok(Self { directories })
    }
}
