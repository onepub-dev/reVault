//! Optional alias records. Existing vault records and the structure discriminator
//! are deliberately unchanged so older readers can preserve this collection.

use super::*;

const ROOT: &str = "/lockbox_aliases";
const EXTENSION: &str = ".lbla";

/// A local Vault shorthand pointing to a stable lockbox identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockboxAlias {
    /// Case-sensitive alias name (ASCII letters, digits, underscore or hyphen).
    pub name: String,
    /// Identity of the target; its current path lives in the known-lockbox record.
    pub lockbox_id: LockboxId,
}

fn record_path(name: &str) -> Result<LockboxPath> {
    validate_record_name(name)?;
    if name.len() > 128 {
        return Err(Error::InvalidInput(
            "lockbox alias exceeds 128 bytes".into(),
        ));
    }
    LockboxPath::new(format!("{ROOT}/{name}{EXTENSION}"))
}

fn encode(id: LockboxId) -> Vec<u8> {
    let mut bytes = b"LBLA".to_vec();
    put_u16(&mut bytes, 1);
    bytes.extend_from_slice(id.as_bytes());
    bytes
}

fn decode(name: String, bytes: &[u8]) -> Result<LockboxAlias> {
    record_path(&name)?;
    let mut reader = BinaryReader::new(bytes);
    if reader.bytes(4)? != b"LBLA" || reader.u16()? != 1 {
        return Err(Error::CorruptVaultRecord(
            "unsupported lockbox alias record".into(),
        ));
    }
    let id = reader
        .bytes(16)?
        .try_into()
        .map_err(|_| Error::CorruptVaultRecord("invalid lockbox alias identity".into()))?;
    reader.finish()?;
    Ok(LockboxAlias {
        name,
        lockbox_id: LockboxId::from_bytes(id),
    })
}

fn resolve(alias: LockboxAlias, known: Vec<KnownLockbox>) -> Result<KnownLockbox> {
    let mut matches = known
        .into_iter()
        .filter(|entry| entry.lockbox_id == alias.lockbox_id);
    let target = matches.next().ok_or_else(|| {
        Error::NotFound(format!(
            "alias {} has no remembered lockbox path; remember the lockbox again",
            alias.name
        ))
    })?;
    if matches.next().is_some() {
        return Err(Error::InvalidInput(format!(
            "alias {} has ambiguous remembered paths",
            alias.name
        )));
    }
    Ok(target)
}

impl VaultDirectory {
    /// Set or replace an alias. The target must already be remembered by this Vault.
    /// Repeating an unchanged mapping performs no write.
    pub fn set_lockbox_alias(&self, name: &str, id: LockboxId) -> Result<()> {
        let path = record_path(name)?;
        if !self
            .list_known_lockboxes()?
            .iter()
            .any(|entry| entry.lockbox_id == id)
        {
            return Err(Error::NotFound(
                "alias target is not a remembered lockbox".into(),
            ));
        }
        let bytes = encode(id);
        if self.lockbox.borrow().stat(&path).is_some() && self.get_record(&path)? == bytes {
            return Ok(());
        }
        self.put_record_replace(&path, &bytes)
    }

    /// List aliases without opening their target lockboxes.
    pub fn list_lockbox_aliases(&self) -> Result<Vec<LockboxAlias>> {
        let mut out = Vec::new();
        for name in self.list_record_names(ROOT, EXTENSION)? {
            out.push(decode(
                name.clone(),
                &self.get_record(&record_path(&name)?)?,
            )?);
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    /// Remove only the alias; never remove its target archive or remembered path.
    pub fn remove_lockbox_alias(&self, name: &str) -> Result<()> {
        self.delete_record_if_exists(&record_path(name)?)
    }

    /// Resolve an alias to its stable identity and currently remembered path.
    /// Callers must verify the identity when opening the target file.
    pub fn resolve_lockbox_alias(&self, name: &str) -> Result<KnownLockbox> {
        let path = record_path(name)?;
        if self.lockbox.borrow().stat(&path).is_none() {
            return Err(Error::NotFound(format!("unknown lockbox alias: {name}")));
        }
        resolve(
            decode(name.to_string(), &self.get_record(&path)?)?,
            self.list_known_lockboxes()?,
        )
    }
}

impl ReadOnlyVaultDirectory {
    /// Read alias names and target identities without loading profile keys.
    pub fn list_lockbox_aliases(&self) -> Result<Vec<LockboxAlias>> {
        let mut out = Vec::new();
        for name in list_read_only_record_names(&self.lockbox, ROOT, EXTENSION)? {
            let bytes = self.lockbox.borrow().get_file(&record_path(&name)?)?;
            out.push(decode(name, &bytes)?);
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    /// Resolve an alias without loading profile keys or opening the target.
    pub fn resolve_lockbox_alias(&self, name: &str) -> Result<KnownLockbox> {
        let path = record_path(name)?;
        if self.lockbox.borrow().stat(&path).is_none() {
            return Err(Error::NotFound(format!("unknown lockbox alias: {name}")));
        }
        let bytes = self.lockbox.borrow().get_file(&path)?;
        resolve(
            decode(name.to_string(), &bytes)?,
            self.list_known_lockboxes()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_record_is_strict_and_names_cannot_escape_collection() {
        let id = LockboxId::from_bytes([7; 16]);
        let bytes = encode(id);
        assert_eq!(decode("dev".into(), &bytes).unwrap().lockbox_id, id);
        for length in 0..bytes.len() {
            assert!(decode("dev".into(), &bytes[..length]).is_err());
        }
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(decode("dev".into(), &extended).is_err());
        let mut future = bytes;
        future[4] = 9;
        assert!(decode("dev".into(), &future).is_err());
        for name in ["", "../dev", "a@dev", "dev/name", "dev.name", "développeur"] {
            assert!(record_path(name).is_err());
        }
        assert!(record_path(&"a".repeat(129)).is_err());
        assert!(record_path("DEV_01-local").is_ok());
    }
}
