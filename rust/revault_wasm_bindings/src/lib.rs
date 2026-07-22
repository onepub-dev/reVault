#![deny(missing_docs)]
//! WebAssembly components for the hosted reVault JavaScript binding.
//!
//! The crate provides browser-safe lockbox operations and a dispatcher that
//! validates calls delegated to an explicit native host. WebAssembly cannot
//! provide the locked and guarded memory used by native builds, so callers must
//! opt in to weakened allocation before creating secrets.
//!
//! See the [repository README](https://github.com/onepub-dev/reVault#readme) for
//! installation, the security model, and complete examples.

use js_sys::{Array, Function, Uint8Array};
use revault_lockbox_api::{
    ListOptions, Lockbox, LockboxEntry, LockboxEntryKind, LockboxPath, SecretString,
    VariableSensitivity,
};
use wasm_bindgen::prelude::*;

/// Explicitly permits or rejects the weakened secure-memory implementation
/// required by WebAssembly runtimes.
///
/// The default is `false`, so callers must acknowledge that browsers cannot
/// provide locked pages, guard pages, or dump/fork exclusion before creating
/// keys or lockboxes.
#[wasm_bindgen]
pub fn set_weakened_allocation_allowed(allowed: bool) {
    revault_page_api::set_weakened_allocation_allowed(allowed);
}

/// Returns whether the caller has explicitly enabled weakened secure memory.
#[wasm_bindgen]
pub fn weakened_allocation_allowed() -> bool {
    revault_page_api::weakened_allocation_allowed()
}

/// WebAssembly-side dispatcher used by the full hosted API. Browser-only
/// lockbox methods can remain self-contained, while OS-backed vault, agent and
/// keyring operations are supplied by an explicit host adapter.
#[wasm_bindgen]
pub struct Runtime {
    calls: u32,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(constructor)]
    /// Creates a dispatcher with no recorded calls.
    pub fn new() -> Runtime {
        Runtime { calls: 0 }
    }

    /// Validates an operation name and records one hosted dispatch.
    ///
    /// Unknown names are rejected before control reaches the native host.
    pub fn before_call(&mut self, operation: &str) -> Result<(), JsValue> {
        let known = include_str!("../../../bindings/e2e/operations.tsv")
            .lines()
            .skip(1)
            .filter_map(|line| line.split('\t').next())
            .any(|name| name == operation);
        if !known {
            return Err(JsValue::from_str(&format!(
                "unknown reVault operation: {operation}"
            )));
        }
        self.calls = self.calls.saturating_add(1);
        Ok(())
    }

    #[wasm_bindgen(getter)]
    /// Returns the number of successfully validated hosted calls.
    pub fn calls(&self) -> u32 {
        self.calls
    }
}

#[wasm_bindgen]
/// In-memory encrypted lockbox exposed to JavaScript.
pub struct WasmLockbox(Lockbox);

#[wasm_bindgen]
/// Hybrid contact key pair used to wrap and unwrap lockbox content keys.
pub struct WasmContactKey(revault_lockbox_api::ContactKeyPair);

#[wasm_bindgen]
/// Metadata for one file, directory, or symbolic link in a lockbox.
pub struct WasmEntry {
    path: String,
    kind: String,
    length: u64,
    permissions: u32,
}

#[wasm_bindgen]
impl WasmEntry {
    #[wasm_bindgen(getter)]
    /// Returns the absolute lockbox path.
    pub fn path(&self) -> String {
        self.path.clone()
    }
    #[wasm_bindgen(getter)]
    /// Returns `file`, `directory`, or `symlink`.
    pub fn kind(&self) -> String {
        self.kind.clone()
    }
    #[wasm_bindgen(getter)]
    /// Returns the logical file length in bytes.
    pub fn length(&self) -> u64 {
        self.length
    }
    #[wasm_bindgen(getter)]
    /// Returns the stored Unix permission bits.
    pub fn permissions(&self) -> u32 {
        self.permissions
    }
}

#[wasm_bindgen]
/// Name and sensitivity metadata for a lockbox variable.
pub struct WasmVariable {
    name: String,
    sensitivity: String,
}

#[wasm_bindgen]
/// Metadata for one password or contact access slot.
pub struct WasmKeySlot {
    id: u64,
    protection: String,
    algorithm: String,
}

#[wasm_bindgen]
impl WasmKeySlot {
    #[wasm_bindgen(getter)]
    /// Returns the stable key-slot identifier.
    pub fn id(&self) -> u64 {
        self.id
    }
    #[wasm_bindgen(getter)]
    /// Returns the slot protection type, such as `password` or `contact`.
    pub fn protection(&self) -> String {
        self.protection.clone()
    }
    #[wasm_bindgen(getter)]
    /// Returns the cryptographic algorithm name stored by the slot.
    pub fn algorithm(&self) -> String {
        self.algorithm.clone()
    }
}

#[wasm_bindgen]
impl WasmVariable {
    #[wasm_bindgen(getter)]
    /// Returns the variable name.
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    /// Returns `normal` or `secret`.
    pub fn sensitivity(&self) -> String {
        self.sensitivity.clone()
    }
}

#[wasm_bindgen]
impl WasmContactKey {
    #[wasm_bindgen(constructor)]
    /// Generates a new contact key pair from the runtime random source.
    pub fn generate() -> Result<WasmContactKey, JsValue> {
        revault_lockbox_api::ContactKeyPair::generate()
            .map(WasmContactKey)
            .map_err(error)
    }

    /// Imports a contact key pair from its encrypted private-key record bytes.
    pub fn from_private_record(record: &[u8]) -> Result<WasmContactKey, JsValue> {
        let record = revault_lockbox_api::SecretVec::try_from_slice(record)
            .map_err(|value| JsValue::from_str(&value.to_string()))?;
        revault_lockbox_api::ContactKeyPair::from_private_key_record(record)
            .map(WasmContactKey)
            .map_err(error)
    }

    /// Exports the public key bytes safe to share with a sender.
    pub fn public_key(&self) -> Vec<u8> {
        self.0.public_key().to_bytes()
    }

    /// Exports the private-key record.
    ///
    /// The returned JavaScript bytes contain secret material and should be
    /// persisted only in an appropriately protected vault.
    pub fn private_record(&self) -> Result<Vec<u8>, JsValue> {
        self.0
            .private_key_record()
            .map_err(error)
            .and_then(|value| {
                value
                    .with_bytes(|bytes| bytes.to_vec())
                    .map_err(|value| JsValue::from_str(&value.to_string()))
            })
    }
}

#[wasm_bindgen]
/// Encodes bytes as lowercase hexadecimal text.
pub fn encode_hex(bytes: &[u8]) -> String {
    revault_vault_api::encode_hex(bytes)
}

#[wasm_bindgen]
/// Decodes hexadecimal text, rejecting malformed input.
pub fn decode_hex(value: &str) -> Result<Vec<u8>, JsValue> {
    revault_vault_api::decode_hex(value).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
/// Returns the lockbox file-format version written by this build.
pub fn lockbox_format_version() -> u16 {
    revault_lockbox_api::LOCKBOX_FORMAT_VERSION
}

#[wasm_bindgen]
/// Reads a lockbox file-format version without decrypting the archive.
pub fn probe_lockbox_format_version(bytes: &[u8]) -> Result<u16, JsValue> {
    revault_lockbox_api::probe_lockbox_format_version(bytes).map_err(error)
}

#[wasm_bindgen]
impl WasmLockbox {
    #[wasm_bindgen(constructor)]
    /// Creates an uncommitted lockbox using a raw content key.
    ///
    /// Add an access slot and call [`WasmLockbox::commit`] before sharing it.
    pub fn create(key: &[u8]) -> WasmLockbox {
        WasmLockbox(Lockbox::create(key))
    }

    /// Creates an uncommitted lockbox with a generated key wrapped by a password.
    pub fn create_with_password(password: &str) -> Result<WasmLockbox, JsValue> {
        let password = SecretString::try_from_slice(password.as_bytes())
            .map_err(|value| JsValue::from_str(&value.to_string()))?;
        Lockbox::create_with_password(&password)
            .map(WasmLockbox)
            .map_err(error)
    }

    /// Opens lockbox bytes for writing using a raw content key.
    pub fn open(bytes: &[u8], key: &[u8]) -> Result<WasmLockbox, JsValue> {
        Lockbox::open_bytes_with_key(bytes.to_vec(), key)
            .map(WasmLockbox)
            .map_err(error)
    }

    /// Opens lockbox bytes for writing using a password access slot.
    pub fn open_with_password(bytes: &[u8], password: &str) -> Result<WasmLockbox, JsValue> {
        let password = SecretString::try_from_slice(password.as_bytes())
            .map_err(|value| JsValue::from_str(&value.to_string()))?;
        Lockbox::open_with_password(bytes.to_vec(), &password)
            .map(WasmLockbox)
            .map_err(error)
    }

    /// Adds a file at `path`, optionally replacing an existing file.
    pub fn add_file(&mut self, path: &str, data: &[u8], replace: bool) -> Result<(), JsValue> {
        let path = LockboxPath::new(path).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.0
            .add_file(&path, data, replace)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Adds a file and stores its Unix permission bits.
    pub fn add_file_with_permissions(
        &mut self,
        path: &str,
        data: &[u8],
        permissions: u32,
        replace: bool,
    ) -> Result<(), JsValue> {
        let path = lockbox_path(path)?;
        self.0
            .add_file_with_permissions(&path, data, permissions, replace)
            .map_err(error)
    }

    /// Creates a directory, optionally creating missing ancestors.
    pub fn create_dir(&mut self, path: &str, create_parents: bool) -> Result<(), JsValue> {
        let path = lockbox_path(path)?;
        self.0.create_dir(&path, create_parents).map_err(error)
    }

    /// Creates every missing parent directory of `path`.
    pub fn create_parent_dirs(&mut self, path: &str) -> Result<(), JsValue> {
        self.0
            .create_parent_dirs_for(&lockbox_path(path)?)
            .map_err(error)
    }

    /// Selects an `interactive`, `bulk-import`, `read-mostly`, or `extract-many` profile.
    pub fn set_workload_profile(&mut self, profile: &str) -> Result<(), JsValue> {
        let profile = match profile {
            "interactive" => revault_lockbox_api::WorkloadProfile::Interactive,
            "bulk-import" | "bulk_import" => revault_lockbox_api::WorkloadProfile::BulkImport,
            "read-mostly" | "read_mostly" => revault_lockbox_api::WorkloadProfile::ReadMostly,
            "extract-many" | "extract_many" => revault_lockbox_api::WorkloadProfile::ExtractMany,
            _ => return Err(JsValue::from_str("invalid workload profile")),
        };
        self.0.set_workload_profile(profile);
        Ok(())
    }

    /// Selects the `auto`, `single`, or `threads` worker policy.
    ///
    /// `jobs` is used only by the `threads` policy.
    pub fn set_worker_policy(&mut self, policy: &str, jobs: usize) -> Result<(), JsValue> {
        let policy = match policy {
            "auto" => revault_lockbox_api::WorkerPolicy::Auto,
            "single" => revault_lockbox_api::WorkerPolicy::Single,
            "threads" => revault_lockbox_api::WorkerPolicy::Threads(jobs),
            _ => return Err(JsValue::from_str("invalid worker policy")),
        };
        self.0.set_worker_policy(policy);
        Ok(())
    }

    /// Adds a password access slot and returns its stable id.
    pub fn add_password(&mut self, password: &str) -> Result<u64, JsValue> {
        let password = SecretString::try_from_slice(password.as_bytes())
            .map_err(|value| JsValue::from_str(&value.to_string()))?;
        self.0.add_password(&password).map_err(error)
    }

    /// Adds a contact-recipient access slot and returns its stable id.
    pub fn add_contact(&mut self, public_key: &[u8]) -> Result<u64, JsValue> {
        let key = revault_lockbox_api::ContactPublicKey::from_bytes(public_key).map_err(error)?;
        self.0.add_contact(&key).map_err(error)
    }

    /// Deletes the access slot identified by `id`.
    pub fn delete_key(&mut self, id: u64) -> Result<(), JsValue> {
        self.0.delete_key(id).map_err(error)
    }

    /// Returns metadata for every access slot without exposing wrapped secrets.
    pub fn list_key_slots(&self) -> Array {
        let result = Array::new();
        for slot in self.0.list_key_slots() {
            result.push(&JsValue::from(WasmKeySlot {
                id: slot.id,
                protection: format!("{:?}", slot.protection).to_ascii_lowercase(),
                algorithm: slot.algorithm.as_str().to_string(),
            }));
        }
        result
    }

    /// Removes a directory, including descendants when `recursive` is true.
    pub fn remove_dir(&mut self, path: &str, recursive: bool) -> Result<(), JsValue> {
        let path = lockbox_path(path)?;
        if recursive {
            self.0.remove_dir_recursive(&path)
        } else {
            self.0.remove_dir(&path)
        }
        .map_err(error)
    }

    /// Deletes a file or symbolic link at `path`.
    pub fn delete(&mut self, path: &str) -> Result<(), JsValue> {
        self.0.delete(&lockbox_path(path)?).map_err(error)
    }

    /// Moves one lockbox entry from `from` to `to`.
    pub fn rename(&mut self, from: &str, to: &str) -> Result<(), JsValue> {
        self.0
            .rename(&lockbox_path(from)?, &lockbox_path(to)?)
            .map_err(error)
    }

    /// Adds a symbolic link whose target is another normalized lockbox path.
    pub fn add_symlink(&mut self, path: &str, target: &str, replace: bool) -> Result<(), JsValue> {
        self.0
            .add_symlink(&lockbox_path(path)?, &lockbox_path(target)?, replace)
            .map_err(error)
    }

    /// Returns the stored target of the symbolic link at `path`.
    pub fn get_symlink_target(&self, path: &str) -> Result<String, JsValue> {
        self.0
            .get_symlink_target(&lockbox_path(path)?)
            .map(|value| value.as_str().to_string())
            .map_err(error)
    }

    /// Stores a non-secret UTF-8 variable.
    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        self.0.set_variable(&name, value).map_err(error)
    }

    /// Stores a secret variable in secure memory until it is encrypted.
    pub fn set_secret_variable(&mut self, name: &str, value: &[u8]) -> Result<(), JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        let value = SecretString::try_from_slice(value)
            .map_err(|value| JsValue::from_str(&value.to_string()))?;
        self.0.set_secret_variable(&name, &value).map_err(error)
    }

    /// Calls `callback` with a temporary copy of a secret variable.
    ///
    /// The temporary `Uint8Array` is overwritten immediately after the callback
    /// returns. Retaining a copy inside the callback is the caller's responsibility.
    pub fn with_secret_variable(
        &self,
        name: &str,
        callback: &Function,
    ) -> Result<JsValue, JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        let result = self
            .0
            .with_secret_variable(&name, |value| {
                value
                    .with_bytes(|bytes| {
                        let secret = Uint8Array::from(bytes);
                        let result = callback.call1(&JsValue::UNDEFINED, &secret);
                        for index in 0..secret.length() {
                            secret.set_index(index, 0);
                        }
                        result
                    })
                    .map_err(|value| JsValue::from_str(&value.to_string()))?
            })
            .map_err(error)?;
        match result {
            Some(result) => result,
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// Returns a non-secret variable, or `undefined` when it does not exist.
    pub fn get_variable(&self, name: &str) -> Result<Option<String>, JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        self.0.get_variable(&name).map_err(error)
    }

    /// Lists entries beneath `path`, optionally including all descendants.
    pub fn list(&self, path: &str, recursive: bool) -> Result<Array, JsValue> {
        let path = lockbox_path(path)?;
        let mut options = ListOptions::new(&path);
        options.recursive = recursive;
        let entries = self.0.list(options).map_err(error)?;
        let result = Array::new();
        for entry in entries {
            result.push(&JsValue::from(entry_to_wasm(entry.map_err(error)?)));
        }
        Ok(result)
    }

    /// Returns entry metadata, or JavaScript `null` when `path` is absent.
    pub fn stat(&self, path: &str) -> Result<JsValue, JsValue> {
        let entry = self.0.stat(&lockbox_path(path)?);
        match entry {
            Some(entry) => Ok(JsValue::from(entry_to_wasm(entry))),
            None => Ok(JsValue::NULL),
        }
    }

    /// Lists variable names and sensitivity without returning their values.
    pub fn list_variables(&self) -> Result<Array, JsValue> {
        let result = Array::new();
        for (name, sensitivity) in self.0.list_variables().map_err(error)? {
            result.push(&JsValue::from(WasmVariable {
                name: name.as_str().to_string(),
                sensitivity: sensitivity_name(sensitivity),
            }));
        }
        Ok(result)
    }

    /// Returns `normal`, `secret`, or `undefined` for an absent variable.
    pub fn variable_sensitivity(&self, name: &str) -> Result<Option<String>, JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        Ok(self
            .0
            .variable_sensitivity(&name)
            .map_err(error)?
            .map(sensitivity_name))
    }

    /// Returns the stable 16-byte lockbox identifier.
    pub fn lockbox_id(&self) -> Vec<u8> {
        self.0.lockbox_id().as_bytes().to_vec()
    }

    /// Deletes the variable named `name`.
    pub fn delete_variable(&mut self, name: &str) -> Result<(), JsValue> {
        let name = revault_lockbox_api::VariableName::new(name).map_err(error)?;
        self.0.delete_variable(&name).map_err(error)
    }

    /// Renames one variable atomically.
    pub fn move_variable(&mut self, source: &str, destination: &str) -> Result<(), JsValue> {
        let source = revault_lockbox_api::VariableName::new(source).map_err(error)?;
        let destination = revault_lockbox_api::VariableName::new(destination).map_err(error)?;
        self.0
            .move_variables(&[(source, destination)])
            .map_err(error)
    }

    /// Moves one typed form record to another lockbox path.
    pub fn move_form_record(&mut self, source: &str, destination: &str) -> Result<(), JsValue> {
        self.0
            .move_form_records(&[(lockbox_path(source)?, lockbox_path(destination)?)])
            .map_err(error)
    }

    /// Reads at most `len` file bytes beginning at `offset`.
    pub fn read_range(&self, path: &str, offset: u64, len: u64) -> Result<Vec<u8>, JsValue> {
        self.0
            .read_file_range(&lockbox_path(path)?, offset, len)
            .map_err(error)
    }

    /// Returns whether any entry exists at `path`.
    pub fn exists(&self, path: &str) -> Result<bool, JsValue> {
        Ok(self.0.exists(&lockbox_path(path)?))
    }

    /// Returns whether `path` names a directory.
    pub fn is_dir(&self, path: &str) -> Result<bool, JsValue> {
        Ok(self.0.is_dir(&lockbox_path(path)?))
    }

    /// Returns stored Unix permission bits, or `undefined` if absent.
    pub fn permissions(&self, path: &str) -> Result<Option<u32>, JsValue> {
        Ok(self.0.permissions(&lockbox_path(path)?))
    }

    /// Updates the stored Unix permission bits for an entry.
    pub fn set_permissions(&mut self, path: &str, permissions: u32) -> Result<(), JsValue> {
        self.0
            .set_permissions(&lockbox_path(path)?, permissions)
            .map_err(error)
    }

    /// Decrypts and returns the complete file at `path`.
    pub fn get_file(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        let path = LockboxPath::new(path).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.0
            .get_file(&path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Authenticates and commits all pending mutations.
    pub fn commit(&mut self) -> Result<(), JsValue> {
        self.0
            .commit()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Serializes the committed encrypted lockbox.
    ///
    /// Call [`WasmLockbox::commit`] first when the lockbox has pending changes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0
            .try_to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

fn lockbox_path(value: &str) -> Result<LockboxPath, JsValue> {
    LockboxPath::new(value).map_err(error)
}

fn entry_to_wasm(entry: LockboxEntry) -> WasmEntry {
    WasmEntry {
        path: entry.path.as_str().to_string(),
        kind: match entry.kind {
            LockboxEntryKind::File => "file",
            LockboxEntryKind::Symlink => "symlink",
            LockboxEntryKind::Directory => "directory",
        }
        .to_string(),
        length: entry.len,
        permissions: entry.permissions,
    }
}

fn sensitivity_name(value: VariableSensitivity) -> String {
    match value {
        VariableSensitivity::Normal => "normal",
        VariableSensitivity::Secret => "secret",
    }
    .to_string()
}

fn error(value: revault_lockbox_api::Error) -> JsValue {
    JsValue::from_str(&value.to_string())
}
