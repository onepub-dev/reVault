use revault_browser_protocol::{
    validation::{identifier, validate_origin},
    Error,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

/// Locally approved mapping. Never accepted from browser messages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    /// Exact canonical HTTPS origin.
    pub origin: String,
    /// Stable application identifier signed by the server.
    pub application_id: String,
    /// Locally entered display name.
    pub application_name: String,
    /// Locally selected canonical lockbox identifier.
    pub lockbox_id: String,
    /// Canonical local path, never supplied by a website.
    pub lockbox_path: PathBuf,
    /// Local profile used to unwrap the content key.
    pub profile: String,
    /// Pinned Ed25519 server signing key, confirmed out of band.
    pub server_signing_key: [u8; 32],
    /// Specific installed browser extension instance identity.
    pub extension_id: String,
    /// Local explanation of the credentials being delegated.
    pub disclosure: String,
}
impl Pairing {
    pub(super) fn validate(&self) -> Result<(), Error> {
        validate_origin(&self.origin)?;
        if ![
            &self.application_id,
            &self.lockbox_id,
            &self.profile,
            &self.extension_id,
        ]
        .into_iter()
        .all(|s| identifier(s))
            || self.application_name.is_empty()
            || self.application_name.len() > 128
            || self.disclosure.is_empty()
            || self.disclosure.len() > 512
            || self
                .application_name
                .chars()
                .chain(self.disclosure.chars())
                .any(|c| c.is_control())
            || !self.lockbox_path.is_absolute()
        {
            return Err(Error::InvalidRequest);
        }
        Ok(())
    }
    pub(super) fn id(&self) -> String {
        pair_id(
            &self.origin,
            &self.application_id,
            &self.lockbox_id,
            &self.extension_id,
        )
    }
    pub(super) fn prompt(&self) -> String {
        format!("{}\n{}\nLockbox: {}\nPermission: unlock the ENTIRE lockbox\n{}\nServer signing key: {}\n\nThe server can read all contents after approval. Expiry does not revoke a delivered key.",
            self.application_name, self.origin, self.lockbox_id, self.disclosure, crate::encode_hex(&self.server_signing_key))
    }
}
pub(super) fn pair_id(origin: &str, app: &str, lockbox: &str, extension: &str) -> String {
    digest(&[
        origin.as_bytes(),
        app.as_bytes(),
        lockbox.as_bytes(),
        extension.as_bytes(),
    ])
}
pub(super) fn digest(fields: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for field in fields {
        hash.update((field.len() as u64).to_be_bytes());
        hash.update(field);
    }
    crate::encode_hex(&hash.finalize())
}
/// Location of browser pairings and replay tombstones, containing no secrets.
pub fn state_directory() -> Result<PathBuf, Error> {
    Ok(crate::default_vault_dir()
        .map_err(|_| Error::VaultLocked)?
        .join("browser-delegation"))
}
pub(super) fn secure_directory(path: &Path) -> Result<(), Error> {
    fs::DirBuilder::new()
        .recursive(true)
        .create(path)
        .map_err(|_| Error::Internal)?;
    let meta = fs::symlink_metadata(path).map_err(|_| Error::Internal)?;
    // SAFETY: geteuid has no preconditions and cannot fail.
    if !meta.is_dir() || meta.uid() != unsafe { libc::geteuid() } {
        return Err(Error::Internal);
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| Error::Internal)
}
pub(super) fn read_pair(root: &Path, id: &str) -> Result<Pairing, Error> {
    let path = root.join(format!("{id}.json"));
    let meta = fs::symlink_metadata(&path).map_err(|_| Error::UnpairedRecipient)?;
    // SAFETY: geteuid has no preconditions and cannot fail.
    let uid = unsafe { libc::geteuid() };
    if !meta.is_file() || meta.len() > 8192 || meta.mode() & 0o077 != 0 || meta.uid() != uid {
        return Err(Error::UnpairedRecipient);
    }
    let pair: Pairing =
        serde_json::from_slice(&fs::read(path).map_err(|_| Error::UnpairedRecipient)?)
            .map_err(|_| Error::UnpairedRecipient)?;
    pair.validate()?;
    if pair.id() != id {
        return Err(Error::UnpairedRecipient);
    }
    Ok(pair)
}
/// Pair with a trusted native dialog. Existing identities require revocation first.
pub fn pair(pair: Pairing) -> Result<String, Error> {
    pair.validate()?;
    let id = revault_lockbox_api::vault_integration::VaultOpen::read_lockbox_id(&pair.lockbox_path)
        .map_err(|_| Error::LockboxUnavailable)?;
    if id.to_string() != pair.lockbox_id
        || fs::canonicalize(&pair.lockbox_path).map_err(|_| Error::LockboxUnavailable)?
            != pair.lockbox_path
    {
        return Err(Error::LockboxUnavailable);
    }
    let root = state_directory()?;
    secure_directory(&root)?;
    super::approval::approve(&format!("Pair reVault with this server?\n\n{}\n\nConfirm the signing key through an independent trusted channel before approving.", pair.prompt()), &std::sync::atomic::AtomicBool::new(false), super::now()? + 120)?;
    let id = pair.id();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(format!("{id}.json")))
        .map_err(|_| Error::UnpairedRecipient)?;
    file.write_all(&serde_json::to_vec(&pair).map_err(|_| Error::Internal)?)
        .map_err(|_| Error::Internal)?;
    file.sync_all().map_err(|_| Error::Internal)?;
    Ok(id)
}
/// Revoke a website/server/browser pairing. Tombstones remain to prevent replay.
pub fn revoke(id: &str) -> Result<(), Error> {
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::InvalidRequest);
    }
    match fs::remove_file(state_directory()?.join(format!("{id}.json"))) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(Error::Internal),
    }
}
/// List local pairing metadata for inspection and revocation.
pub fn list_pairings() -> Result<Vec<(String, Pairing)>, Error> {
    let root = state_directory()?;
    secure_directory(&root)?;
    let mut result = Vec::new();
    for entry in fs::read_dir(&root).map_err(|_| Error::Internal)? {
        let entry = entry.map_err(|_| Error::Internal)?;
        let name = entry.file_name();
        if let Some(id) = name.to_str().and_then(|s| s.strip_suffix(".json")) {
            result.push((id.to_owned(), read_pair(&root, id)?));
        }
    }
    Ok(result)
}
