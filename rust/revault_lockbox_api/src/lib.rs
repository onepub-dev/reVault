#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Core encrypted lockbox storage engine.
//!
//! `revault_lockbox_api` owns the portable `.lbox` file format and the in-memory API
//! for storing files, symlinks, variable values, and key slots. It does not
//! know about a user's local vault or open-cache agent; those are implemented in
//! `revault_vault_api`.
//!
//! Start with [`Lockbox::create_in_memory`] for an in-memory archive or
//! [`Lockbox::create_file`] for a file-backed archive. Add password or contact
//! key slots before sharing the resulting `.lbox` file. Secret variables and
//! secret form fields are exposed only through callback-scoped secure values.
//!
//! # Example
//!
//! ```
//! use revault_lockbox_api::{
//!     Lockbox, LockboxPath, LockboxProtection, OwnerSigningKeyPair, SecretString,
//! };
//!
//! let password = SecretString::try_from_slice(b"correct horse")?;
//! let signing_key = OwnerSigningKeyPair::generate()?;
//! let mut lockbox = Lockbox::create_in_memory(
//!     LockboxProtection::Password(&password),
//!     &signing_key,
//! )?;
//! lockbox.add_file(&LockboxPath::new("/notes.txt")?, b"encrypted", false)?;
//! lockbox.commit()?;
//! let encrypted_archive = lockbox.try_to_bytes()?;
//! assert!(!encrypted_archive.is_empty());
//! # Ok::<(), revault_lockbox_api::Error>(())
//! ```
//!
//! See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
//! for installation, security guidance, and end-to-end examples.

mod checked;
mod compression;
mod constants;
mod error;
mod fast_hash;
mod file_format;
mod index;
mod keys;
mod lockbox;
mod model;
mod paths;
mod scan;
mod security;
mod storage;
mod toc;
#[cfg(feature = "vault-integration")]
pub mod vault_integration;

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod compression_regression_tests;
#[cfg(test)]
mod fixture_artifact_tests;

pub(crate) use file_format::{
    commit_auth, commit_root, key_directory, page, page_buffer, page_inspection, page_scanner,
    payload,
};
pub(crate) use keys::{crypto, key_derivation, key_slot, key_wrap, secret_vec, signing};
pub(crate) use model::{
    compression_frame_manifest, entry, extract_policy, file_chunk, form, list_options, lockbox_id,
    node_kind, page_object_packer, record, recovery_report, recovery_report_options, variable_name,
    variable_sensitivity,
};
pub(crate) use paths::{host_path, lockbox_path};
pub(crate) use storage::{cache_options, file_lock, free_index, free_slot, page_cache};
pub(crate) use toc::{form_btree, page_tree, toc_btree, toc_codec, toc_entry, variable_btree};

pub use cache_options::{CacheLimit, CacheStats, LockboxOptions, WorkerPolicy, WorkloadProfile};
pub use entry::{LockboxEntry, LockboxEntryKind};
pub use error::{ArtifactKind, Error, Result};
pub use extract_policy::ExtractPolicy;
pub use file_format::header::{probe_lockbox_format_version, LOCKBOX_FORMAT_VERSION};
#[doc(hidden)]
pub use file_lock::{lock_path_for, FileLockScope, ScopedFileLock};
pub use form::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue,
};
pub use key_slot::{
    LockboxKeySlot, LockboxKeySlotAlgorithm, LockboxKeySlotProtection, MAX_KEY_SLOT_NAME_BYTES,
};
pub use key_wrap::{ContactKeyPair, ContactPublicKey, ContactWrappedKey};
pub use list_options::ListOptions;
pub use lockbox::{
    ContentChunk, ContentStreamOptions, ContentStreamOrder, ImportStats, Lockbox,
    LockboxFileInspection, LockboxFileMut, LockboxFileReader, LockboxInspector, LockboxOpen,
    LockboxOwnerInspection, LockboxProtection, OpenFileOptions, ReadOnly, RecoveryScanner,
    VariableValueRef, Writable, WritableLockboxState,
};
pub use lockbox_id::LockboxId;
pub use lockbox_path::LockboxPath;
pub use page_inspection::{PageInspection, PageObjectInspection};
pub use recovery_report::RecoveryReport;
pub use recovery_report_options::RecoveryReportOptions;
pub use secret_vec::{SecretString, SecretVec};
pub use signing::{OwnerSigningKeyPair, OwnerSigningPublicKey};
pub use variable_name::{VariableName, VariableNamePattern};
pub use variable_sensitivity::VariableSensitivity;
