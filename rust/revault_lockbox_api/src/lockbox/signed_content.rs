use super::Lockbox;
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::io::{self, Write};

// Encryption authenticates individual private pages. Without encryption, sign a
// deterministic snapshot as well as the commit coordinates. Length prefixes and
// section tags prevent ambiguous concatenations. Verify before returning a handle
// or allowing recovery cleanup to modify storage.
impl<State> Lockbox<State> {
    pub(super) fn signed_content_digest(&self) -> Result<[u8; 32]> {
        let mut digest = Sha256::new();
        digest.update(b"revault-plaintext-content-v3\0");
        let toc = crate::toc_codec::TocEncoder::new(self.toc_entries.values()).encode();
        field(&mut digest, &toc);
        for entry in self.toc_entries.values().filter(|entry| !entry.deleted) {
            match entry.node_kind {
                crate::node_kind::NodeKind::File => {
                    digest.update(entry.len.to_le_bytes());
                    self.extract_file_to_writer(&entry.path, HashWriter(&mut digest))?;
                }
                crate::node_kind::NodeKind::Symlink => {
                    field(
                        &mut digest,
                        self.get_symlink_target(&entry.path)?.as_str().as_bytes(),
                    );
                }
                crate::node_kind::NodeKind::Directory => {}
            }
        }
        digest.update(b"variables\0");
        for (name, value) in self.clone_all_variable_values()? {
            field(&mut digest, name.as_str().as_bytes());
            digest.update([u8::from(
                value.sensitivity() == crate::VariableSensitivity::Secret,
            )]);
            value.with_plaintext(|text| field(&mut digest, text.as_bytes()))?;
        }
        digest.update(b"forms\0");
        let (definitions, records) = self.clone_all_form_state()?;
        for entry in crate::form_btree::form_entries_from_maps(&definitions, &records) {
            let encoded = crate::form_btree::encode_form_leaf_secure(&[entry])?;
            encoded.with_bytes(|bytes| field(&mut digest, bytes))?;
        }
        digest.update(b"allocation-and-cleanup\0");
        let root = self.read_commit_root_at(self.commit_root_offset)?;
        for offset in [
            root.free_index_root_offset,
            root.post_cleanup_free_index_root_offset,
        ] {
            digest.update(offset.to_le_bytes());
            if offset != 0 {
                let slots = self.read_free_index_slots(offset, 0)?;
                digest.update((slots.len() as u64).to_le_bytes());
                for slot in slots {
                    digest.update(slot.offset.to_le_bytes());
                    digest.update(slot.len.to_le_bytes());
                }
            }
        }
        let mut offset = root.redaction_manifest_offset;
        let mut count = 0;
        while offset != 0 {
            count += 1;
            if count > crate::file_format::redaction_manifest::MAX_REDACTION_PAGES {
                return Err(Error::CorruptRecord);
            }
            let manifest = self.read_redaction_manifest_page_at(offset)?;
            field(
                &mut digest,
                &crate::file_format::redaction_manifest::encode_page(&manifest)?,
            );
            offset = manifest.next_page_offset;
        }
        Ok(digest.finalize().into())
    }

    pub(super) fn verify_signed_content(&self) -> Result<()> {
        if self.format_mode.plaintext() && self.format_mode.signed() && self.sequence != 0 {
            let (auth, _) = self.read_and_verify_commit_auth_at(self.commit_auth_offset)?;
            if auth.content_digest != Some(self.signed_content_digest()?) {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
}

fn field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

struct HashWriter<'a>(&'a mut Sha256);
impl Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn plaintext_signature_rejects_variable_tampering_even_with_recomputed_checksums() {
        let signer = OwnerSigningKeyPair::generate().unwrap();
        let mut lb = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
            compression: Compression::None,
            ..LockboxCreateOptions::new(Encryption::None, Signing::Owner(&signer))
        })
        .unwrap();
        lb.set_variable(
            &VariableName::new("signed").unwrap(),
            "original signed value",
        )
        .unwrap();
        lb.commit().unwrap();
        let mut bytes = lb.try_to_bytes().unwrap();
        let marker = b"original signed value";
        let position = bytes
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();
        let page = (crate::constants::HEADER_LEN..position)
            .rev()
            .find(|offset| {
                bytes.get(*offset..*offset + 8) == Some(crate::page::PAGE_MAGIC.as_slice())
            })
            .unwrap();
        bytes[position] = b'X';
        let body = page + crate::page::PAGE_HEADER_LEN;
        let stored_len =
            u32::from_le_bytes(bytes[page + 44..page + 48].try_into().unwrap()) as usize;
        let checksum = crate::crypto::strong_checksum(&bytes[body + 32..body + stored_len]);
        bytes[body..body + 32].copy_from_slice(&checksum);
        assert!(Lockbox::open_bytes(bytes, LockboxOpen::Unencrypted).is_err());
    }

    #[test]
    fn compaction_preserves_every_persistent_choice() {
        let signer = OwnerSigningKeyPair::generate().unwrap();
        for signing in [Signing::None, Signing::Owner(&signer)] {
            let mut lb = Lockbox::create_in_memory_with_options(LockboxCreateOptions {
                compression: Compression::None,
                ..LockboxCreateOptions::new(Encryption::None, signing)
            })
            .unwrap();
            let path = LockboxPath::new("/file.txt").unwrap();
            lb.add_file(&path, b"before", false).unwrap();
            lb.commit().unwrap();
            lb.add_file(&path, b"after", true).unwrap();
            lb.commit().unwrap();
            let options = lb.format_options();
            lb.compact().unwrap();
            let reopened =
                Lockbox::open_bytes(lb.try_to_bytes().unwrap(), LockboxOpen::Unencrypted).unwrap();
            assert_eq!(reopened.format_options(), options);
            assert_eq!(reopened.get_file(&path).unwrap(), b"after");
        }
    }
}
