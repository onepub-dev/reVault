#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
// This crate is a generated-style C boundary: exported functions accept
// caller-owned pointers and mirror the flat declarations in revault_api.h.
#![allow(
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_cast
)]

//! Small, deliberately boring ABI shared by the Dart, Java and Python clients.
//!
//! The ABI owns Rust objects behind opaque pointers. Byte buffers returned by
//! this module must be released with [`buffer_free`]. Secrets supplied
//! to the API are copied into the core's zeroizing secure allocations before
//! the call returns. Every returned buffer is wiped by [`buffer_free`]; secret
//! getters additionally use opaque handles so language facades can constrain
//! plaintext copies to a callback scope.

use revault_lockbox_api::Result as LockboxResult;

// flatc 25.2.10 predates Rust 2024's explicit unsafe-block requirement. Keep
// that compatibility allowance confined to generated private transport code.
#[allow(missing_docs, unsafe_op_in_unsafe_fn, unused_imports, clippy::all)]
mod bindings_flatbuffers {
    include!("generated/revault_bindings_generated.rs");
}
use bindings_flatbuffers::revault::internal as bindings_transport;
use revault_lockbox_api::{
    ContactKeyPair, ContactPublicKey, ContactWrappedKey, ContentStreamOptions, ContentStreamOrder,
    ExtractPolicy, FormDefinition, FormFieldDefinition, FormFieldKind, FormRecord, FormTypeId,
    FormValue, ListOptions, Lockbox, LockboxEntry, LockboxEntryKind, LockboxKeySlot,
    LockboxKeySlotAlgorithm, LockboxKeySlotProtection, LockboxOpen, LockboxPath, LockboxProtection,
    OwnerSigningKeyPair, OwnerSigningPublicKey, SecretString, VariableName,
};
use std::cell::RefCell;
use std::ffi::{c_char, c_void, CString};
use std::ptr;
use zeroize::Zeroize;

#[repr(C)]
/// Caller-owned bytes returned through the stable C ABI.
pub struct RevaultBuffer {
    /// Pointer to the first byte, or null for an empty or failed result.
    pub ptr: *mut u8,
    /// Number of readable bytes starting at `ptr`.
    pub len: usize,
}

/// Opaque owner for secret bytes returned across the native boundary.
enum SecretHandle {
    String(SecretString),
}

impl SecretHandle {
    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> revault_lockbox_api::Result<R> {
        match self {
            Self::String(value) => value.with_bytes(f).map_err(Into::into),
        }
    }
}

/// Major version of the stable native ABI exposed by this library.
#[no_mangle]
pub extern "C" fn api_abi_version() -> u32 {
    3
}

type LockboxHandle = Lockbox;
type VaultDirectoryHandle = revault_vault_api::VaultDirectory;
type ReadOnlyVaultDirectoryHandle = revault_vault_api::ReadOnlyVaultDirectory;
type ContactWrappedKeyHandle = ContactWrappedKey;
type LocalVaultHandle = revault_vault_api::LocalVault;
type SecretActivityHandle = revault_vault_api::SecretActivityGuard;

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("").unwrap());
}

fn set_error(error: impl std::fmt::Display) {
    let message = error.to_string().replace('\0', "\\0");
    LAST_ERROR.with(|slot| *slot.borrow_mut() = CString::new(message).unwrap());
}

fn clear_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = CString::new("").unwrap());
}

unsafe fn input<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    (!ptr.is_null()).then(|| unsafe { std::slice::from_raw_parts(ptr, len) })
}

unsafe fn input_str<'a>(ptr: *const c_char, len: usize) -> Option<&'a str> {
    let bytes = unsafe { input(ptr.cast(), len) }?;
    std::str::from_utf8(bytes).ok()
}

fn buffer(bytes: Vec<u8>) -> RevaultBuffer {
    let mut bytes = bytes.into_boxed_slice();
    let result = RevaultBuffer {
        ptr: bytes.as_mut_ptr(),
        len: bytes.len(),
    };
    std::mem::forget(bytes);
    result
}

macro_rules! flatbuffer_buffer {
    ($value:expr) => {{
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let root = $value.pack(&mut builder);
        builder.finish(root, None);
        clear_error();
        buffer(builder.finished_data().to_vec())
    }};
}

fn lockbox_entry_transport(entry: &LockboxEntry) -> bindings_transport::LockboxEntryT {
    bindings_transport::LockboxEntryT {
        path: Some(entry.path.as_str().to_string()),
        kind: match entry.kind {
            LockboxEntryKind::File => bindings_transport::LockboxEntryKind::FILE,
            LockboxEntryKind::Symlink => bindings_transport::LockboxEntryKind::SYMLINK,
            LockboxEntryKind::Directory => bindings_transport::LockboxEntryKind::DIRECTORY,
        },
        length: entry.len,
        permissions: entry.permissions,
    }
}

fn form_definition_transport(definition: &FormDefinition) -> bindings_transport::FormDefinitionT {
    bindings_transport::FormDefinitionT {
        type_id: Some(definition.type_id.as_str().to_string()),
        alias: Some(definition.alias.clone()),
        revision: definition.revision as u32,
        name: Some(definition.name.clone()),
        description: Some(definition.description.clone()),
        fields: Some(
            definition
                .fields
                .iter()
                .map(|field| bindings_transport::FormFieldT {
                    id: Some(field.id.clone()),
                    label: Some(field.label.clone()),
                    kind: Some(format!("{:?}", field.kind).to_ascii_lowercase()),
                    required: field.required,
                })
                .collect(),
        ),
    }
}

fn form_value_transport(
    value: &revault_lockbox_api::FormFieldValue,
) -> bindings_transport::FormValueT {
    let text = match &value.value {
        FormValue::Normal(text) => text.clone(),
        FormValue::Secret(_) => String::new(),
    };
    bindings_transport::FormValueT {
        field_id: Some(value.field_id.clone()),
        label: Some(value.captured_label.clone()),
        kind: Some(format!("{:?}", value.kind).to_ascii_lowercase()),
        value: Some(text),
        secret: value.value.is_secret(),
    }
}

fn form_record_transport(record: &FormRecord) -> bindings_transport::FormRecordT {
    bindings_transport::FormRecordT {
        path: Some(record.path.as_str().to_string()),
        name: Some(record.name.clone()),
        type_id: Some(record.type_id.as_str().to_string()),
        definition_alias: Some(record.definition_alias.clone()),
        definition_revision: record.definition_revision as u32,
        values: Some(record.values.iter().map(form_value_transport).collect()),
    }
}

fn form_definition_list_transport(
    values: &[FormDefinition],
) -> bindings_transport::FormDefinitionListT {
    bindings_transport::FormDefinitionListT {
        values: Some(values.iter().map(form_definition_transport).collect()),
    }
}

fn form_record_list_transport(values: &[FormRecord]) -> bindings_transport::FormRecordListT {
    bindings_transport::FormRecordListT {
        values: Some(values.iter().map(form_record_transport).collect()),
    }
}

fn form_kind(value: &str) -> Option<FormFieldKind> {
    Some(match value.to_ascii_lowercase().as_str() {
        "text" => FormFieldKind::Text,
        "secret" => FormFieldKind::Secret,
        "url" => FormFieldKind::Url,
        "email" => FormFieldKind::Email,
        "date" => FormFieldKind::Date,
        "month" => FormFieldKind::Month,
        "notes" => FormFieldKind::Notes,
        "number" => FormFieldKind::Number,
        _ => return None,
    })
}

fn form_fields_from_transport(bytes: &[u8]) -> Result<Vec<FormFieldDefinition>, String> {
    let fields = flatbuffers::root::<bindings_transport::FormFieldList<'_>>(bytes)
        .map_err(|error| format!("invalid form fields FlatBuffer: {error}"))?
        .unpack();
    fields
        .values
        .unwrap_or_default()
        .into_iter()
        .map(|field| {
            Ok(FormFieldDefinition {
                id: field.id.unwrap_or_default(),
                label: field.label.unwrap_or_default(),
                kind: form_kind(field.kind.as_deref().unwrap_or_default())
                    .ok_or_else(|| "invalid form field kind".to_string())?,
                required: field.required,
            })
        })
        .collect()
}

fn path_moves_from_transport(bytes: &[u8]) -> Result<Vec<(LockboxPath, LockboxPath)>, String> {
    flatbuffers::root::<bindings_transport::PathMoveList<'_>>(bytes)
        .map_err(|error| format!("invalid path moves FlatBuffer: {error}"))?
        .unpack()
        .values
        .unwrap_or_default()
        .into_iter()
        .map(|value| {
            Ok((
                LockboxPath::new(value.source.unwrap_or_default())
                    .map_err(|error| error.to_string())?,
                LockboxPath::new(value.destination.unwrap_or_default())
                    .map_err(|error| error.to_string())?,
            ))
        })
        .collect()
}

fn string_moves_from_transport(bytes: &[u8]) -> Result<Vec<(String, String)>, String> {
    Ok(
        flatbuffers::root::<bindings_transport::PathMoveList<'_>>(bytes)
            .map_err(|error| format!("invalid moves FlatBuffer: {error}"))?
            .unpack()
            .values
            .unwrap_or_default()
            .into_iter()
            .map(|value| {
                (
                    value.source.unwrap_or_default(),
                    value.destination.unwrap_or_default(),
                )
            })
            .collect(),
    )
}

fn recovery_transport(
    report: &revault_lockbox_api::RecoveryReport,
) -> bindings_transport::RecoveryReportT {
    bindings_transport::RecoveryReportT {
        intact_files: Some(
            report
                .intact_files
                .iter()
                .map(lockbox_entry_transport)
                .collect(),
        ),
        intact_file_count: report.intact_file_count as u64,
        partial_files: report.partial_files as u64,
        corrupt_records: report.corrupt_records as u64,
        toc_recovered: report.toc_recovered,
        variables_recovered: report.variables_recovered,
        variable_count: report.variable_count as u64,
        forms_recovered: report.forms_recovered,
        form_definition_count: report.form_definition_count as u64,
        form_record_count: report.form_record_count as u64,
    }
}

fn key_slot_transport(slot: &LockboxKeySlot) -> bindings_transport::KeySlotT {
    bindings_transport::KeySlotT {
        id: slot.id,
        protection: Some(
            match slot.protection {
                LockboxKeySlotProtection::Password => "password",
                LockboxKeySlotProtection::Contact => "contact",
                _ => "unknown",
            }
            .to_string(),
        ),
        algorithm: Some(
            match slot.algorithm {
                LockboxKeySlotAlgorithm::Argon2idChaCha20Poly1305 => "argon2id+chacha20-poly1305",
                LockboxKeySlotAlgorithm::X25519MlKem768ChaCha20Poly1305 => {
                    "x25519+ml-kem-768+chacha20-poly1305"
                }
                _ => "unknown",
            }
            .to_string(),
        ),
    }
}

fn cache_stats_transport(
    stats: revault_lockbox_api::CacheStats,
) -> bindings_transport::CacheStatsT {
    bindings_transport::CacheStatsT {
        limit_bytes: stats.limit_bytes as u64,
        used_bytes: stats.used_bytes as u64,
        entries: stats.entries as u64,
        hits: stats.hits as u64,
        misses: stats.misses as u64,
    }
}

fn import_stats_transport(
    stats: revault_lockbox_api::ImportStats,
) -> bindings_transport::ImportStatsT {
    bindings_transport::ImportStatsT {
        host_stat_nanos: Some(stats.host_stat_nanos.to_string()),
        host_read_nanos: Some(stats.host_read_nanos.to_string()),
        frame_prepare_nanos: Some(stats.frame_prepare_nanos.to_string()),
        page_write_nanos: Some(stats.page_write_nanos.to_string()),
    }
}

fn page_inspection_transport(
    page: &revault_lockbox_api::PageInspection,
) -> bindings_transport::PageInspectionT {
    bindings_transport::PageInspectionT {
        offset: page.offset,
        page_id: page.page_id,
        sequence: page.sequence,
        page_size: page.page_size as u64,
        encrypted_body_len: page.encrypted_body_len as u64,
        unused_bytes: page.unused_bytes as u64,
        object_count: page.object_count as u64,
        objects: Some(
            page.objects
                .iter()
                .map(|object| bindings_transport::PageObjectT {
                    id: object.id,
                    kind: Some(object.kind.to_string()),
                    payload_len: object.payload_len as u64,
                })
                .collect(),
        ),
    }
}

fn file_inspection_transport(
    value: &revault_lockbox_api::LockboxFileInspection,
) -> bindings_transport::FileInspectionT {
    bindings_transport::FileInspectionT {
        lockbox_id: Some(value.lockbox_id.as_bytes().to_vec()),
        header_readable: value.header_readable,
        key_directory_generation: value.key_directory_generation,
        key_directory_copy_count: value.key_directory_copy_count as u64,
        owner_signed: value.owner_signed,
        key_slots: Some(value.key_slots.iter().map(key_slot_transport).collect()),
    }
}

fn profile_history_transport(
    value: &revault_vault_api::ProfileHistory,
) -> bindings_transport::ProfileHistoryT {
    bindings_transport::ProfileHistoryT {
        name: Some(value.name.clone()),
        active_generation: value.active_generation as u32,
        generations: Some(
            value
                .generations
                .iter()
                .map(|generation| bindings_transport::ProfileGenerationT {
                    index: generation.index as u32,
                    status: Some(format!("{:?}", generation.status).to_ascii_lowercase()),
                    contact_fingerprint: Some(generation.contact_fingerprint.clone()),
                    created_at_unix_ms: generation.created_at_unix_ms,
                    retired_at_unix_ms: generation.retired_at_unix_ms.unwrap_or_default(),
                    has_retired_at: generation.retired_at_unix_ms.is_some(),
                })
                .collect(),
        ),
    }
}

fn known_lockbox_transport(
    value: &revault_vault_api::KnownLockbox,
) -> bindings_transport::KnownLockboxT {
    bindings_transport::KnownLockboxT {
        lockbox_id: Some(value.lockbox_id.as_bytes().to_vec()),
        path: Some(value.path.clone()),
        last_seen_unix_ms: value.last_seen_unix_ms,
    }
}

fn access_slot_label_transport(
    value: &revault_vault_api::AccessSlotLabel,
) -> bindings_transport::AccessSlotLabelT {
    bindings_transport::AccessSlotLabelT {
        lockbox_id: Some(value.lockbox_id.as_bytes().to_vec()),
        slot_id: value.slot_id,
        name: Some(value.name.clone()),
        updated_at_unix_ms: value.updated_at_unix_ms,
    }
}

fn key_slot_list_transport(values: &[LockboxKeySlot]) -> bindings_transport::KeySlotListT {
    bindings_transport::KeySlotListT {
        values: Some(values.iter().map(key_slot_transport).collect()),
    }
}

fn page_inspection_list_transport(
    values: &[revault_lockbox_api::PageInspection],
) -> bindings_transport::PageInspectionListT {
    bindings_transport::PageInspectionListT {
        values: Some(values.iter().map(page_inspection_transport).collect()),
    }
}

fn known_lockbox_list_transport(
    values: &[revault_vault_api::KnownLockbox],
) -> bindings_transport::KnownLockboxListT {
    bindings_transport::KnownLockboxListT {
        values: Some(values.iter().map(known_lockbox_transport).collect()),
    }
}

fn access_slot_label_list_transport(
    values: &[revault_vault_api::AccessSlotLabel],
) -> bindings_transport::AccessSlotLabelListT {
    bindings_transport::AccessSlotLabelListT {
        values: Some(values.iter().map(access_slot_label_transport).collect()),
    }
}

fn contact_list_transport(
    values: &[revault_vault_api::StoredContact],
) -> bindings_transport::ContactListT {
    bindings_transport::ContactListT {
        values: Some(
            values
                .iter()
                .map(|value| bindings_transport::ContactT {
                    name: Some(value.name.clone()),
                    key: Some(value.key.to_bytes()),
                })
                .collect(),
        ),
    }
}

#[no_mangle]
/// Returns the last error.
pub extern "C" fn buffer_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}

#[no_mangle]
/// Returns the last error details.
pub extern "C" fn buffer_last_error_details() -> RevaultBuffer {
    let message = LAST_ERROR.with(|slot| slot.borrow().to_string_lossy().into_owned());
    let guidance = message
        .split_once(". ")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    let words = message.split_whitespace().collect::<Vec<_>>();
    let unsupported = words.first() == Some(&"unsupported") && words.get(2) == Some(&"format");
    let parse_version = |index: usize| {
        words
            .get(index)
            .and_then(|value| {
                value
                    .trim_end_matches(|character: char| !character.is_ascii_digit())
                    .parse()
                    .ok()
            })
            .unwrap_or(0)
    };
    flatbuffer_buffer!(&bindings_transport::ErrorDetailsT {
        category: Some(
            if unsupported {
                "unsupported_format_version"
            } else {
                "native"
            }
            .to_string()
        ),
        artifact_kind: Some(
            if unsupported {
                words.get(1).copied().unwrap_or("")
            } else {
                ""
            }
            .to_string()
        ),
        found_version: if unsupported { parse_version(4) } else { 0 },
        supported_version: if unsupported { parse_version(10) } else { 0 },
        message: Some(message),
        guidance: Some(guidance),
    })
}

#[no_mangle]
/// Returns the supported lockbox format version.
pub extern "C" fn lockbox_format_version() -> u16 {
    clear_error();
    revault_lockbox_api::LOCKBOX_FORMAT_VERSION
}

#[no_mangle]
/// Determines format version without fully opening it.
pub unsafe extern "C" fn lockbox_probe_format_version(bytes: *const u8, len: usize) -> u16 {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("lockbox bytes pointer is null");
        return 0;
    };
    match revault_lockbox_api::probe_lockbox_format_version(bytes) {
        Ok(version) => {
            clear_error();
            version
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

#[no_mangle]
/// Releases the native resources held by this object.
pub extern "C" fn buffer_free(value: RevaultBuffer) {
    if !value.ptr.is_null() {
        // SAFETY: buffers are only constructed by `buffer` and are freed once.
        unsafe {
            let mut bytes = Box::from_raw(std::ptr::slice_from_raw_parts_mut(value.ptr, value.len));
            bytes.as_mut().zeroize();
            drop(bytes);
        };
    }
}

/// Return the byte length of an opaque secret.
#[no_mangle]
pub unsafe extern "C" fn secret_len(handle: *const c_void, out_len: *mut usize) -> bool {
    // SAFETY: a non-null handle returned by this library points to SecretHandle.
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<SecretHandle>()) })
    else {
        set_error("secret handle is null");
        return false;
    };
    // SAFETY: the caller provides writable storage for one size value.
    let Some(out_len) = (!out_len.is_null()).then(|| unsafe { &mut *out_len }) else {
        set_error("secret length output is null");
        return false;
    };
    match handle.with_bytes(|bytes| bytes.len()) {
        Ok(len) => {
            *out_len = len;
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

/// Copy an opaque secret into a caller-owned mutable buffer.
#[no_mangle]
pub unsafe extern "C" fn secret_copy(
    handle: *const c_void,
    destination: *mut u8,
    destination_len: usize,
) -> bool {
    // SAFETY: a non-null handle returned by this library points to SecretHandle.
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<SecretHandle>()) })
    else {
        set_error("secret handle is null");
        return false;
    };
    if destination_len != 0 && destination.is_null() {
        set_error("secret destination is null");
        return false;
    }
    match handle.with_bytes(|bytes| {
        if bytes.len() != destination_len {
            return false;
        }
        if !bytes.is_empty() {
            // SAFETY: the destination spans exactly destination_len writable bytes.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len()) };
        }
        true
    }) {
        Ok(true) => {
            clear_error();
            true
        }
        Ok(false) => {
            set_error("secret destination length does not match");
            false
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

/// Release an opaque secret and zeroize its secure allocation.
#[no_mangle]
pub unsafe extern "C" fn secret_free(handle: *mut c_void) {
    if !handle.is_null() {
        // SAFETY: ownership of a handle returned by this library transfers once.
        unsafe { drop(Box::from_raw(handle.cast::<SecretHandle>())) };
    }
}

fn optional_secret_output(
    result: revault_lockbox_api::Result<Option<SecretHandle>>,
    output: *mut *mut c_void,
) -> bool {
    // SAFETY: the caller provides storage for one opaque handle pointer.
    let Some(output) = (!output.is_null()).then(|| unsafe { &mut *output }) else {
        set_error("secret output is null");
        return false;
    };
    *output = ptr::null_mut();
    match result {
        Ok(Some(secret)) => {
            *output = Box::into_raw(Box::new(secret)).cast();
            clear_error();
            true
        }
        Ok(None) => {
            clear_error();
            true
        }
        Err(error) => {
            *output = ptr::null_mut();
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Creates a new lockbox.
pub extern "C" fn lockbox_create(key: *const u8, key_len: usize) -> *mut c_void {
    let Some(key) = (unsafe { input(key, key_len) }) else {
        set_error("key pointer is null");
        return ptr::null_mut();
    };
    clear_error();
    Box::into_raw(Box::new(Lockbox::create(key))).cast()
}

#[no_mangle]
/// Creates password.
pub unsafe extern "C" fn lockbox_create_password(password: *const u8, len: usize) -> *mut c_void {
    let Some(password) = (unsafe { input(password, len) }) else {
        set_error("password pointer is null");
        return ptr::null_mut();
    };
    let password = match revault_lockbox_api::SecretString::try_from_slice(password) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    let signing = match OwnerSigningKeyPair::generate() {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match Lockbox::create_in_memory(LockboxProtection::Password(&password), &signing) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

fn lockbox_options(
    cache_mode: *const c_char,
    cache_len: usize,
    cache_bytes: u64,
    workload: *const c_char,
    workload_len: usize,
    worker: *const c_char,
    worker_len: usize,
    jobs: usize,
) -> Result<revault_lockbox_api::LockboxOptions, String> {
    let cache_mode = unsafe { input_str(cache_mode, cache_len) }
        .ok_or_else(|| "invalid cache mode".to_string())?;
    let workload = unsafe { input_str(workload, workload_len) }
        .ok_or_else(|| "invalid workload profile".to_string())?;
    let worker = unsafe { input_str(worker, worker_len) }
        .ok_or_else(|| "invalid worker policy".to_string())?;
    let cache_limit = match cache_mode {
        "auto" => revault_lockbox_api::CacheLimit::Auto,
        "disabled" => revault_lockbox_api::CacheLimit::Disabled,
        "bytes" => revault_lockbox_api::CacheLimit::Bytes(cache_bytes),
        _ => return Err("cache mode must be auto, disabled, or bytes".to_string()),
    };
    let workload_profile = match workload {
        "interactive" => revault_lockbox_api::WorkloadProfile::Interactive,
        "bulk-import" | "bulk_import" => revault_lockbox_api::WorkloadProfile::BulkImport,
        "read-mostly" | "read_mostly" => revault_lockbox_api::WorkloadProfile::ReadMostly,
        "extract-many" | "extract_many" => revault_lockbox_api::WorkloadProfile::ExtractMany,
        _ => return Err("invalid workload profile".to_string()),
    };
    let worker_policy = match worker {
        "auto" => revault_lockbox_api::WorkerPolicy::Auto,
        "single" => revault_lockbox_api::WorkerPolicy::Single,
        "threads" => revault_lockbox_api::WorkerPolicy::Threads(jobs),
        _ => return Err("invalid worker policy".to_string()),
    };
    Ok(revault_lockbox_api::LockboxOptions {
        cache_limit,
        workload_profile,
        worker_policy,
    })
}

#[no_mangle]
/// Creates with options.
pub unsafe extern "C" fn lockbox_create_with_options(
    key: *const u8,
    key_len: usize,
    cache_mode: *const c_char,
    cache_len: usize,
    cache_bytes: u64,
    workload: *const c_char,
    workload_len: usize,
    worker: *const c_char,
    worker_len: usize,
    jobs: usize,
) -> *mut c_void {
    let Some(key) = (unsafe { input(key, key_len) }) else {
        set_error("key pointer is null");
        return ptr::null_mut();
    };
    let options = match lockbox_options(
        cache_mode,
        cache_len,
        cache_bytes,
        workload,
        workload_len,
        worker,
        worker_len,
        jobs,
    ) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match std::panic::catch_unwind(|| Lockbox::create_with_options(key, options)) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(_) => {
            set_error("secure lockbox allocation failed");
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Creates contact.
pub unsafe extern "C" fn lockbox_create_contact(contact: *const c_void) -> *mut c_void {
    let Some(contact) =
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactPublicKey>()) })
    else {
        set_error("contact public key handle is null");
        return ptr::null_mut();
    };
    let signing = match OwnerSigningKeyPair::generate() {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match Lockbox::create_in_memory(
        LockboxProtection::ContactPublicKey {
            name: None,
            contact: contact.clone(),
        },
        &signing,
    ) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Creates with signing key.
pub unsafe extern "C" fn lockbox_create_with_signing_key(
    content_key: *const u8,
    key_len: usize,
    signing_key: *const c_void,
) -> *mut c_void {
    let Some(content_key) = (unsafe { input(content_key, key_len) }) else {
        set_error("content key pointer is null");
        return ptr::null_mut();
    };
    let Some(signing_key) =
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) })
    else {
        set_error("signing key handle is null");
        return ptr::null_mut();
    };
    let content_key = match revault_lockbox_api::SecretVec::try_from_slice(content_key) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match Lockbox::create_in_memory(LockboxProtection::ContentKey(content_key), signing_key) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens an existing lockbox.
pub extern "C" fn lockbox_open(
    archive: *const u8,
    archive_len: usize,
    key: *const u8,
    key_len: usize,
) -> *mut c_void {
    let (Some(archive), Some(key)) = (unsafe { input(archive, archive_len) }, unsafe {
        input(key, key_len)
    }) else {
        set_error("archive or key pointer is null");
        return ptr::null_mut();
    };
    match Lockbox::open_bytes_with_key(archive.to_vec(), key) {
        Ok(lockbox) => {
            clear_error();
            Box::into_raw(Box::new(lockbox)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens with options.
pub unsafe extern "C" fn lockbox_open_with_options(
    archive: *const u8,
    archive_len: usize,
    key: *const u8,
    key_len: usize,
    cache_mode: *const c_char,
    cache_len: usize,
    cache_bytes: u64,
    workload: *const c_char,
    workload_len: usize,
    worker: *const c_char,
    worker_len: usize,
    jobs: usize,
) -> *mut c_void {
    let (Some(archive), Some(key)) = (unsafe { input(archive, archive_len) }, unsafe {
        input(key, key_len)
    }) else {
        set_error("archive or key pointer is null");
        return ptr::null_mut();
    };
    let options = match lockbox_options(
        cache_mode,
        cache_len,
        cache_bytes,
        workload,
        workload_len,
        worker,
        worker_len,
        jobs,
    ) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match Lockbox::open_bytes_with_key_options(archive.to_vec(), key, options) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens password.
pub unsafe extern "C" fn lockbox_open_password(
    bytes: *const u8,
    len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let (Some(bytes), Some(password)) = (unsafe { input(bytes, len) }, unsafe {
        input(password, password_len)
    }) else {
        set_error("invalid lockbox input");
        return ptr::null_mut();
    };
    let password = match revault_lockbox_api::SecretString::try_from_slice(password) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match Lockbox::open_with_password(bytes.to_vec(), &password) {
        Ok(mut value) => {
            let signing = match OwnerSigningKeyPair::generate() {
                Ok(value) => value,
                Err(error) => {
                    set_error(error);
                    return ptr::null_mut();
                }
            };
            value.set_owner_signing_key(signing);
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens contact.
pub unsafe extern "C" fn lockbox_open_contact(
    bytes: *const u8,
    len: usize,
    contact: *const c_void,
) -> *mut c_void {
    let (Some(bytes), Some(contact)) = (
        unsafe { input(bytes, len) },
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactKeyPair>()) }),
    ) else {
        set_error("invalid lockbox input");
        return ptr::null_mut();
    };
    match Lockbox::open_with_contact(bytes.to_vec(), contact) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Adds file.
pub unsafe extern "C" fn lockbox_add_file(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    data: *const u8,
    data_len: usize,
    replace: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(data)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(data, data_len)
    }) else {
        set_error("invalid path or data pointer");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.add_file(&path, data, replace)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Adds file with permissions.
pub unsafe extern "C" fn lockbox_add_file_with_permissions(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    data: *const u8,
    data_len: usize,
    permissions: u32,
    replace: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(data)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(data, data_len)
    }) else {
        set_error("invalid path or data pointer");
        return false;
    };
    match LockboxPath::new(path)
        .and_then(|path| handle.add_file_with_permissions(&path, data, permissions, replace))
    {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns file.
pub unsafe extern "C" fn lockbox_get_file(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path).and_then(|path| handle.get_file(&path)) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Authenticates and publishes the staged changes.
pub unsafe extern "C" fn lockbox_commit(handle: *mut c_void) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    match handle.commit() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Creates dir.
pub unsafe extern "C" fn lockbox_create_dir(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    create_parents: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.create_dir(&path, create_parents)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Removes delete.
pub unsafe extern "C" fn lockbox_delete(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.delete(&path)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Removes dir.
pub unsafe extern "C" fn lockbox_remove_dir(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    recursive: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    let result = LockboxPath::new(path).and_then(|path| {
        if recursive {
            handle.remove_dir_recursive(&path)
        } else {
            handle.remove_dir(&path)
        }
    });
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Creates parent dirs.
pub unsafe extern "C" fn lockbox_create_parent_dirs(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.create_parent_dirs_for(&path)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Extracts file.
pub unsafe extern "C" fn lockbox_extract_file(
    handle: *const c_void,
    source: *const c_char,
    source_len: usize,
    destination: *const c_char,
    destination_len: usize,
    replace: bool,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(source), Some(destination)) = (unsafe { input_str(source, source_len) }, unsafe {
        input_str(destination, destination_len)
    }) else {
        set_error("invalid extraction path");
        return false;
    };
    match LockboxPath::new(source).and_then(|source| {
        handle.extract_file_to(&source, std::path::Path::new(destination), replace)
    }) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Extracts directory.
pub unsafe extern "C" fn lockbox_extract_directory(
    handle: *const c_void,
    destination: *const c_char,
    destination_len: usize,
    max_file_bytes: u64,
    max_total_bytes: u64,
    max_files: usize,
    restore_symlinks: bool,
    restore_permissions: bool,
    overwrite: bool,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(destination) = (unsafe { input_str(destination, destination_len) }) else {
        set_error("invalid extraction destination");
        return false;
    };
    let policy = ExtractPolicy {
        max_file_bytes,
        max_total_bytes,
        max_files,
        restore_symlinks,
        restore_permissions,
        overwrite,
    };
    match handle.extract_to_directory(std::path::Path::new(destination), &policy) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the stream content.
pub unsafe extern "C" fn lockbox_stream_content(
    handle: *const c_void,
    physical: bool,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let order = if physical {
        ContentStreamOrder::Physical
    } else {
        ContentStreamOrder::Logical
    };
    let mut chunks = Vec::new();
    let result = handle.stream_content(ContentStreamOptions { order }, |chunk, reader| {
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|error| revault_lockbox_api::Error::Io(error.to_string()))?;
        chunks.push(bindings_transport::StreamChunkT {
            path: Some(chunk.path.as_str().to_string()),
            file_offset: chunk.file_offset,
            length: chunk.len,
            physical_offset: chunk.physical_offset.unwrap_or_default(),
            sparse: chunk.sparse,
            data: Some(data),
        });
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::StreamChunkListT {
                values: Some(chunks),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns cache statistics for this lockbox.
pub unsafe extern "C" fn lockbox_cache_stats(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    flatbuffer_buffer!(&cache_stats_transport(handle.inspector().cache_stats()))
}

#[no_mangle]
/// Returns import statistics for this lockbox.
pub unsafe extern "C" fn lockbox_import_stats(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    flatbuffer_buffer!(&import_stats_transport(handle.import_stats()))
}

#[no_mangle]
/// Updates import stats.
pub unsafe extern "C" fn lockbox_reset_import_stats(handle: *const c_void) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    handle.reset_import_stats();
    clear_error();
    true
}

#[no_mangle]
/// Inspects file.
pub unsafe extern "C" fn lockbox_inspect_file(
    path: *const c_char,
    path_len: usize,
) -> RevaultBuffer {
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid file path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match Lockbox::inspect_file(path) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&file_inspection_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the page inspection.
pub unsafe extern "C" fn lockbox_page_inspection(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.inspector().inspect_pages() {
        Ok(pages) => {
            clear_error();
            flatbuffer_buffer!(&page_inspection_list_transport(&pages))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the recovery report.
pub unsafe extern "C" fn lockbox_recovery_report(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    flatbuffer_buffer!(&recovery_transport(&handle.inspector().recovery_report()))
}

#[no_mangle]
/// Returns the recovery report render.
pub unsafe extern "C" fn lockbox_recovery_report_render(
    handle: *const c_void,
    verbose: bool,
    max_entries: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let options = revault_lockbox_api::RecoveryReportOptions {
        verbose,
        max_intact_entries: (max_entries != 0).then_some(max_entries),
    };
    clear_error();
    buffer(
        handle
            .inspector()
            .recovery_report()
            .render(&options)
            .into_bytes(),
    )
}

#[no_mangle]
/// Returns the recovery scan path.
pub unsafe extern "C" fn lockbox_recovery_scan_path(
    path: *const c_char,
    path_len: usize,
    key: *const u8,
    key_len: usize,
) -> RevaultBuffer {
    let (Some(path), Some(key)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(key, key_len)
    }) else {
        set_error("invalid recovery input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let report = revault_lockbox_api::RecoveryScanner::scan_path(std::path::Path::new(path), key);
    clear_error();
    flatbuffer_buffer!(&recovery_transport(&report))
}

#[no_mangle]
/// Returns the storage len.
pub unsafe extern "C" fn lockbox_storage_len(handle: *const c_void) -> u64 {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return 0;
    };
    match handle.inspector().storage_len() {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

#[no_mangle]
/// Sets workload profile.
pub unsafe extern "C" fn lockbox_set_workload_profile(
    handle: *mut c_void,
    profile: *const c_char,
    profile_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(profile) = (unsafe { input_str(profile, profile_len) }) else {
        set_error("invalid workload profile");
        return false;
    };
    let value = match profile {
        "interactive" => revault_lockbox_api::WorkloadProfile::Interactive,
        "bulk-import" | "bulk_import" => revault_lockbox_api::WorkloadProfile::BulkImport,
        "read-mostly" | "read_mostly" => revault_lockbox_api::WorkloadProfile::ReadMostly,
        "extract-many" | "extract_many" => revault_lockbox_api::WorkloadProfile::ExtractMany,
        _ => {
            set_error("unknown workload profile");
            return false;
        }
    };
    handle.set_workload_profile(value);
    clear_error();
    true
}

#[no_mangle]
/// Sets worker policy.
pub unsafe extern "C" fn lockbox_set_worker_policy(
    handle: *mut c_void,
    mode: *const c_char,
    mode_len: usize,
    jobs: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(mode) = (unsafe { input_str(mode, mode_len) }) else {
        set_error("invalid worker policy");
        return false;
    };
    let value = match mode {
        "auto" => revault_lockbox_api::WorkerPolicy::Auto,
        "single" => revault_lockbox_api::WorkerPolicy::Single,
        "threads" => revault_lockbox_api::WorkerPolicy::Threads(jobs),
        _ => {
            set_error("worker policy must be auto, single, or threads");
            return false;
        }
    };
    handle.set_worker_policy(value);
    clear_error();
    true
}

#[no_mangle]
/// Returns the runtime options.
pub unsafe extern "C" fn lockbox_runtime_options(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    flatbuffer_buffer!(&bindings_transport::RuntimeOptionsT {
        workload_profile: Some(format!("{:?}", handle.workload_profile()).to_ascii_lowercase()),
        worker_policy: Some(format!("{:?}", handle.worker_policy()).to_ascii_lowercase()),
    })
}

#[no_mangle]
/// Updates rename.
pub unsafe extern "C" fn lockbox_rename(
    handle: *mut c_void,
    from: *const c_char,
    from_len: usize,
    to: *const c_char,
    to_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(from), Some(to)) = (unsafe { input_str(from, from_len) }, unsafe {
        input_str(to, to_len)
    }) else {
        set_error("invalid path");
        return false;
    };
    let result = LockboxPath::new(from)
        .and_then(|from| LockboxPath::new(to).and_then(|to| handle.rename(&from, &to)));
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Lists list.
pub unsafe extern "C" fn lockbox_list(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
    recursive: bool,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let result = LockboxPath::new(path).and_then(|path| {
        let mut options = ListOptions::new(&path);
        options.recursive = recursive;
        handle.list(options)
    });
    match result {
        Ok(entries) => {
            let Ok(entries) = entries.collect::<Result<Vec<_>, _>>() else {
                set_error("failed to decode listing entry");
                return RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                };
            };
            clear_error();
            flatbuffer_buffer!(&bindings_transport::LockboxEntryListT {
                entries: Some(entries.iter().map(lockbox_entry_transport).collect()),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Lists with options.
pub unsafe extern "C" fn lockbox_list_with_options(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
    glob: *const c_char,
    glob_len: usize,
    recursive: bool,
    include_files: bool,
    include_symlinks: bool,
    include_directories: bool,
    limit: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid list path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let glob = if glob.is_null() {
        None
    } else {
        match unsafe { input_str(glob, glob_len) } {
            Some(value) => Some(value.to_string()),
            None => {
                set_error("invalid list glob");
                return RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                };
            }
        }
    };
    let result = LockboxPath::new(path).and_then(|path| {
        let mut options = ListOptions::new(&path);
        options.glob = glob;
        options.recursive = recursive;
        options.include_files = include_files;
        options.include_symlinks = include_symlinks;
        options.include_directories = include_directories;
        options.limit = (limit != 0).then_some(limit);
        handle.list(options)
    });
    match result {
        Ok(entries) => match entries.collect::<Result<Vec<_>, _>>() {
            Ok(entries) => {
                clear_error();
                flatbuffer_buffer!(&bindings_transport::LockboxEntryListT {
                    entries: Some(entries.iter().map(lockbox_entry_transport).collect()),
                })
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns metadata for the selected lockbox entry.
pub unsafe extern "C" fn lockbox_stat(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Ok(path) = LockboxPath::new(path) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.stat(&path) {
        Some(entry) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalLockboxEntryT {
                value: Some(Box::new(lockbox_entry_transport(&entry))),
            })
        }
        None => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalLockboxEntryT { value: None })
        }
    }
}

#[no_mangle]
/// Sets variable.
pub unsafe extern "C" fn lockbox_set_variable(
    handle: *mut c_void,
    name: *const c_char,
    name_len: usize,
    value: *const c_char,
    value_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(name), Some(value)) = (unsafe { input_str(name, name_len) }, unsafe {
        input_str(value, value_len)
    }) else {
        set_error("invalid variable input");
        return false;
    };
    let result = VariableName::new(name).and_then(|name| handle.set_variable(&name, value));
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Sets secret variable.
pub unsafe extern "C" fn lockbox_set_secret_variable(
    handle: *mut c_void,
    name: *const c_char,
    name_len: usize,
    value: *const u8,
    value_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(name), Some(value)) = (unsafe { input_str(name, name_len) }, unsafe {
        input(value, value_len)
    }) else {
        set_error("invalid secret variable input");
        return false;
    };
    let result = VariableName::new(name).and_then(|name| {
        SecretString::try_from_slice(value)
            .map_err(Into::into)
            .and_then(|value| handle.set_secret_variable(&name, &value))
    });
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns variable.
pub unsafe extern "C" fn lockbox_get_variable(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid variable name");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match VariableName::new(name).and_then(|name| handle.get_variable(&name)) {
        Ok(value) => flatbuffer_buffer!(&bindings_transport::OptionalStringT {
            present: value.is_some(),
            value: Some(value.unwrap_or_default()),
        }),
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns secret variable.
pub unsafe extern "C" fn lockbox_get_secret_variable(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    output: *mut *mut c_void,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid variable name");
        return false;
    };
    let result = VariableName::new(name).and_then(|name| {
        handle.with_secret_variable(&name, |value| {
            value
                .try_clone()
                .map(SecretHandle::String)
                .map_err(Into::into)
        })
    });
    optional_secret_output(result.and_then(|value| value.transpose()), output)
}

#[no_mangle]
/// Removes variable.
pub unsafe extern "C" fn lockbox_delete_variable(
    handle: *mut c_void,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid variable name");
        return false;
    };
    match VariableName::new(name).and_then(|name| handle.delete_variable(&name)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Updates variables.
pub unsafe extern "C" fn lockbox_move_variables(
    handle: *mut c_void,
    moves_transport: *const u8,
    moves_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(bytes) = (unsafe { input(moves_transport, moves_len) }) else {
        set_error("path moves pointer is null");
        return false;
    };
    let moves = match string_moves_from_transport(bytes) {
        Ok(values) => values
            .into_iter()
            .map(|(source, destination)| {
                Ok((VariableName::new(source)?, VariableName::new(destination)?))
            })
            .collect::<LockboxResult<Vec<_>>>(),
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match moves.and_then(|values| handle.move_variables(&values)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Lists variables.
pub unsafe extern "C" fn lockbox_list_variables(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_variables() {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::VariableListT {
                values: Some(
                    values
                        .iter()
                        .map(|(name, sensitivity)| bindings_transport::VariableT {
                            name: Some(name.as_str().to_string()),
                            sensitivity: Some(format!("{:?}", sensitivity).to_ascii_lowercase()),
                        })
                        .collect()
                ),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the variable sensitivity.
pub unsafe extern "C" fn lockbox_variable_sensitivity(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid variable name");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match VariableName::new(name).and_then(|name| handle.variable_sensitivity(&name)) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalStringT {
                present: value.is_some(),
                value: Some(
                    value
                        .map(|value| format!("{:?}", value).to_ascii_lowercase())
                        .unwrap_or_default()
                ),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Adds symlink.
pub unsafe extern "C" fn lockbox_add_symlink(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    target: *const c_char,
    target_len: usize,
    replace: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(target)) = (unsafe { input_str(path, path_len) }, unsafe {
        input_str(target, target_len)
    }) else {
        set_error("invalid symlink input");
        return false;
    };
    let result = LockboxPath::new(path).and_then(|path| {
        LockboxPath::new(target).and_then(|target| handle.add_symlink(&path, &target, replace))
    });
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns symlink target.
pub unsafe extern "C" fn lockbox_get_symlink_target(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path).and_then(|path| handle.get_symlink_target(&path)) {
        Ok(target) => {
            clear_error();
            buffer(target.as_str().as_bytes().to_vec())
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the id.
pub unsafe extern "C" fn lockbox_id(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(handle.lockbox_id().as_bytes().to_vec())
}

#[no_mangle]
/// Reports whether exists.
pub unsafe extern "C" fn lockbox_exists(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path) {
        Ok(path) => {
            clear_error();
            handle.exists(&path)
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Reports whether dir.
pub unsafe extern "C" fn lockbox_is_dir(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path) {
        Ok(path) => {
            clear_error();
            handle.is_dir(&path)
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the permissions.
pub unsafe extern "C" fn lockbox_permissions(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> u32 {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return u32::MAX;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return u32::MAX;
    };
    match LockboxPath::new(path) {
        Ok(path) => match handle.permissions(&path) {
            Some(value) => {
                clear_error();
                value
            }
            None => u32::MAX,
        },
        Err(error) => {
            set_error(error);
            u32::MAX
        }
    }
}

#[no_mangle]
/// Sets permissions.
pub unsafe extern "C" fn lockbox_set_permissions(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    permissions: u32,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.set_permissions(&path, permissions)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns range.
pub unsafe extern "C" fn lockbox_read_range(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
    offset: u64,
    len: u64,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path).and_then(|path| handle.read_file_range(&path, offset, len)) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the recovery scan.
pub unsafe extern "C" fn lockbox_recovery_scan(
    bytes: *const u8,
    len: usize,
    key: *const u8,
    key_len: usize,
) -> RevaultBuffer {
    let (Some(bytes), Some(key)) = (unsafe { input(bytes, len) }, unsafe { input(key, key_len) })
    else {
        set_error("invalid recovery input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let report = revault_lockbox_api::RecoveryScanner::scan_bytes(bytes.to_vec(), key);
    clear_error();
    flatbuffer_buffer!(&recovery_transport(&report))
}

#[no_mangle]
/// Returns the recovery salvage.
pub unsafe extern "C" fn lockbox_recovery_salvage(
    bytes: *const u8,
    len: usize,
    key: *const u8,
    key_len: usize,
    signing_key: *const c_void,
) -> *mut c_void {
    let (Some(bytes), Some(key), Some(signing_key)) = (
        unsafe { input(bytes, len) },
        unsafe { input(key, key_len) },
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) }),
    ) else {
        set_error("invalid recovery input");
        return ptr::null_mut();
    };
    match revault_lockbox_api::RecoveryScanner::salvage_bytes(bytes.to_vec(), key, signing_key) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Adds password.
pub unsafe extern "C" fn lockbox_add_password(
    handle: *mut c_void,
    password: *const u8,
    len: usize,
) -> u64 {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return u64::MAX;
    };
    let Some(password) = (unsafe { input(password, len) }) else {
        set_error("password pointer is null");
        return u64::MAX;
    };
    let password = match revault_lockbox_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return u64::MAX;
        }
    };
    match handle.add_password(&password) {
        Ok(id) => {
            clear_error();
            id
        }
        Err(error) => {
            set_error(error);
            u64::MAX
        }
    }
}

#[no_mangle]
/// Adds contact.
pub unsafe extern "C" fn lockbox_add_contact(
    handle: *mut c_void,
    contact: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> u64 {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return u64::MAX;
    };
    let Some(contact) =
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactPublicKey>()) })
    else {
        set_error("contact public key handle is null");
        return u64::MAX;
    };
    let result = if name.is_null() {
        handle.add_contact(contact)
    } else {
        match unsafe { input_str(name, name_len) } {
            Some(name) => handle.add_contact_named(name.to_string(), contact),
            None => {
                set_error("invalid contact name");
                return u64::MAX;
            }
        }
    };
    match result {
        Ok(id) => {
            clear_error();
            id
        }
        Err(error) => {
            set_error(error);
            u64::MAX
        }
    }
}

#[no_mangle]
/// Removes key.
pub unsafe extern "C" fn lockbox_delete_key(handle: *mut c_void, id: u64) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    match handle.delete_key(id) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Lists key slots.
pub unsafe extern "C" fn lockbox_list_key_slots(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    flatbuffer_buffer!(&key_slot_list_transport(&handle.list_key_slots()))
}

#[no_mangle]
/// Sets owner signing key.
pub unsafe extern "C" fn lockbox_set_owner_signing_key(
    handle: *mut c_void,
    key: *const c_void,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(key) = (!key.is_null()).then(|| unsafe { &*(key.cast::<OwnerSigningKeyPair>()) })
    else {
        set_error("signing key handle is null");
        return false;
    };
    match key.try_clone() {
        Ok(key) => {
            handle.set_owner_signing_key(key);
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the owner inspection.
pub unsafe extern "C" fn lockbox_owner_inspection(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.owner_inspection() {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OwnerInspectionT {
                signed: value.signed,
                fingerprint: Some(value.fingerprint.clone().unwrap_or_default()),
                has_fingerprint: value.fingerprint.is_some(),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the define form.
pub unsafe extern "C" fn lockbox_define_form(
    handle: *mut c_void,
    alias: *const c_char,
    alias_len: usize,
    name: *const c_char,
    name_len: usize,
    description: *const c_char,
    description_len: usize,
    fields_transport: *const u8,
    fields_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let (Some(alias), Some(name), Some(description), Some(fields_transport)) = (
        unsafe { input_str(alias, alias_len) },
        unsafe { input_str(name, name_len) },
        unsafe { input_str(description, description_len) },
        unsafe { input(fields_transport, fields_len) },
    ) else {
        set_error("invalid form input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let fields = match form_fields_from_transport(fields_transport) {
        Ok(fields) => fields,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    if fields.is_empty() {
        set_error("form fields must not be empty");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.define_form_with_description(alias, name, description, fields) {
        Ok(definition) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_transport(&definition))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Lists form definitions.
pub unsafe extern "C" fn lockbox_list_form_definitions(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_form_definitions() {
        Ok(definitions) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_list_transport(&definitions))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the resolve form.
pub unsafe extern "C" fn lockbox_resolve_form(
    handle: *const c_void,
    reference: *const c_char,
    reference_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(reference) = (unsafe { input_str(reference, reference_len) }) else {
        set_error("invalid form reference");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.resolve_form_definition(reference) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Lists form revisions.
pub unsafe extern "C" fn lockbox_list_form_revisions(
    handle: *const c_void,
    type_id: *const c_char,
    type_id_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(type_id) = (unsafe { input_str(type_id, type_id_len) }) else {
        set_error("invalid form type id");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let type_id = match FormTypeId::new(type_id) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match handle.list_form_definition_revisions(&type_id) {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Creates form record.
pub unsafe extern "C" fn lockbox_create_form_record(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    type_reference: *const c_char,
    type_len: usize,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let (Some(path), Some(type_reference), Some(name)) = (
        unsafe { input_str(path, path_len) },
        unsafe { input_str(type_reference, type_len) },
        unsafe { input_str(name, name_len) },
    ) else {
        set_error("invalid form record input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path)
        .and_then(|path| handle.create_form_record(&path, type_reference, name))
    {
        Ok(record) => {
            clear_error();
            flatbuffer_buffer!(&form_record_transport(&record))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Sets form field.
pub unsafe extern "C" fn lockbox_set_form_field(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    field: *const c_char,
    field_len: usize,
    value: *const c_char,
    value_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(field), Some(value)) = (
        unsafe { input_str(path, path_len) },
        unsafe { input_str(field, field_len) },
        unsafe { input_str(value, value_len) },
    ) else {
        set_error("invalid form field input");
        return false;
    };
    let result =
        LockboxPath::new(path).and_then(|path| handle.set_form_field_normal(&path, field, value));
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Sets secret form field.
pub unsafe extern "C" fn lockbox_set_secret_form_field(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
    field: *const c_char,
    field_len: usize,
    value: *const u8,
    value_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(field), Some(value)) = (
        unsafe { input_str(path, path_len) },
        unsafe { input_str(field, field_len) },
        unsafe { input(value, value_len) },
    ) else {
        set_error("invalid secret form field input");
        return false;
    };
    let result = LockboxPath::new(path).and_then(|path| {
        SecretString::try_from_slice(value)
            .map_err(Into::into)
            .and_then(|value| handle.set_form_field_secret(&path, field, &value))
    });
    match result {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Lists form records.
pub unsafe extern "C" fn lockbox_list_form_records(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_form_records() {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&form_record_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns form record.
pub unsafe extern "C" fn lockbox_get_form_record(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid form path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path).and_then(|path| handle.get_form_record(&path)) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalFormRecordT {
                value: value.as_ref().map(form_record_transport).map(Box::new),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Removes form record.
pub unsafe extern "C" fn lockbox_delete_form_record(
    handle: *mut c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid form path");
        return false;
    };
    match LockboxPath::new(path).and_then(|path| handle.delete_form_record(&path)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Updates form records.
pub unsafe extern "C" fn lockbox_move_form_records(
    handle: *mut c_void,
    moves_transport: *const u8,
    moves_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &mut *(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let Some(bytes) = (unsafe { input(moves_transport, moves_len) }) else {
        set_error("path moves pointer is null");
        return false;
    };
    let moves = match path_moves_from_transport(bytes) {
        Ok(values) => values,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match handle.move_form_records(&moves) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns form field.
pub unsafe extern "C" fn lockbox_get_form_field(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
    field: *const c_char,
    field_len: usize,
) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let (Some(path), Some(field)) = (unsafe { input_str(path, path_len) }, unsafe {
        input_str(field, field_len)
    }) else {
        set_error("invalid form field input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match LockboxPath::new(path).and_then(|path| handle.get_form_field(&path, field)) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalFormValueT {
                value: value.as_ref().map(form_value_transport).map(Box::new),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns secret form field.
pub unsafe extern "C" fn lockbox_get_secret_form_field(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
    field: *const c_char,
    field_len: usize,
    output: *mut *mut c_void,
) -> bool {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return false;
    };
    let (Some(path), Some(field)) = (unsafe { input_str(path, path_len) }, unsafe {
        input_str(field, field_len)
    }) else {
        set_error("invalid secret form field input");
        return false;
    };
    let result = LockboxPath::new(path)
        .and_then(|path| handle.get_form_field(&path, field))
        .and_then(|value| match value {
            Some(value) => match value.value {
                FormValue::Secret(secret) => secret
                    .as_ref()
                    .try_clone()
                    .map(SecretHandle::String)
                    .map(Some)
                    .map_err(Into::into),
                FormValue::Normal(_) => Err(revault_lockbox_api::Error::InvalidOperation(
                    "form field is not secret".to_string(),
                )),
            },
            None => Ok(None),
        });
    optional_secret_output(result, output)
}

#[no_mangle]
/// Generates generate.
pub extern "C" fn key_contact_generate() -> *mut c_void {
    match ContactKeyPair::generate() {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the from private.
pub unsafe extern "C" fn key_contact_from_private(bytes: *const u8, len: usize) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("private key pointer is null");
        return ptr::null_mut();
    };
    match revault_lockbox_api::SecretVec::try_from_slice(bytes)
        .map_err(revault_lockbox_api::Error::from)
        .and_then(ContactKeyPair::from_private_key_record)
    {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the public.
pub unsafe extern "C" fn key_contact_public(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<ContactKeyPair>()) })
    else {
        set_error("contact key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(handle.public_key().to_bytes())
}

#[no_mangle]
/// Returns the private.
pub unsafe extern "C" fn key_contact_private(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<ContactKeyPair>()) })
    else {
        set_error("contact key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.private_key_record() {
        Ok(bytes) => match bytes.with_bytes(|value| value.to_vec()) {
            Ok(value) => {
                clear_error();
                buffer(value)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the public from bytes.
pub unsafe extern "C" fn key_contact_public_from_bytes(
    bytes: *const u8,
    len: usize,
) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("public key pointer is null");
        return ptr::null_mut();
    };
    match ContactPublicKey::from_bytes(bytes) {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the public free.
pub unsafe extern "C" fn key_contact_public_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<ContactPublicKey>())) };
    }
}

#[no_mangle]
/// Releases the native resources held by this object.
pub unsafe extern "C" fn key_contact_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<ContactKeyPair>())) };
    }
}

#[no_mangle]
/// Encrypts a content key for the selected contact.
pub unsafe extern "C" fn key_contact_encrypt(
    contact: *const c_void,
    content_key: *const u8,
    key_len: usize,
) -> *mut c_void {
    let Some(contact) =
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactPublicKey>()) })
    else {
        set_error("contact public key handle is null");
        return ptr::null_mut();
    };
    let Some(content_key) = (unsafe { input(content_key, key_len) }) else {
        set_error("content key pointer is null");
        return ptr::null_mut();
    };
    match contact.encrypt(content_key) {
        Ok(wrapped) => {
            clear_error();
            Box::into_raw(Box::new(wrapped)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Decrypts a wrapped content key for this contact.
pub unsafe extern "C" fn key_contact_decrypt(
    contact: *const c_void,
    wrapped: *const c_void,
) -> RevaultBuffer {
    let (Some(contact), Some(wrapped)) = (
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactKeyPair>()) }),
        (!wrapped.is_null()).then(|| unsafe { &*(wrapped.cast::<ContactWrappedKeyHandle>()) }),
    ) else {
        set_error("contact key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match contact.decrypt(wrapped) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the wrapped public.
pub unsafe extern "C" fn key_contact_wrapped_public(wrapped: *const c_void) -> RevaultBuffer {
    let Some(wrapped) =
        (!wrapped.is_null()).then(|| unsafe { &*(wrapped.cast::<ContactWrappedKeyHandle>()) })
    else {
        set_error("wrapped key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(wrapped.x25519_ephemeral_public_key().to_vec())
}

#[no_mangle]
/// Returns the wrapped ciphertext.
pub unsafe extern "C" fn key_contact_wrapped_ciphertext(wrapped: *const c_void) -> RevaultBuffer {
    let Some(wrapped) =
        (!wrapped.is_null()).then(|| unsafe { &*(wrapped.cast::<ContactWrappedKeyHandle>()) })
    else {
        set_error("wrapped key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(wrapped.ciphertext_bytes().to_vec())
}

#[no_mangle]
/// Returns the wrapped encrypted.
pub unsafe extern "C" fn key_contact_wrapped_encrypted(wrapped: *const c_void) -> RevaultBuffer {
    let Some(wrapped) =
        (!wrapped.is_null()).then(|| unsafe { &*(wrapped.cast::<ContactWrappedKeyHandle>()) })
    else {
        set_error("wrapped key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(wrapped.encrypted_key().to_vec())
}

#[no_mangle]
/// Returns the wrapped free.
pub unsafe extern "C" fn key_contact_wrapped_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<ContactWrappedKeyHandle>())) };
    }
}

fn key_format(value: *const c_char, len: usize) -> Result<revault_vault_api::KeyFormat, String> {
    let Some(value) = (unsafe { input_str(value, len) }) else {
        return Err("invalid key format".to_string());
    };
    revault_vault_api::KeyFormat::parse(value).map_err(|error| error.to_string())
}

#[no_mangle]
/// Returns the key export private.
pub unsafe extern "C" fn vault_key_export_private(
    key: *const c_void,
    format: *const c_char,
    format_len: usize,
) -> RevaultBuffer {
    let Some(key) = (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactKeyPair>()) }) else {
        set_error("contact key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let format = match key_format(format, format_len) {
        Ok(format) => format,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match revault_vault_api::export_private_key(key, format) {
        Ok(bytes) => match bytes.with_bytes(|value| value.to_vec()) {
            Ok(value) => {
                clear_error();
                buffer(value)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the key export public.
pub unsafe extern "C" fn vault_key_export_public(
    key: *const c_void,
    format: *const c_char,
    format_len: usize,
) -> RevaultBuffer {
    let Some(key) = (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactPublicKey>()) }) else {
        set_error("contact public key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let format = match key_format(format, format_len) {
        Ok(format) => format,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match revault_vault_api::export_public_key(key, format) {
        Ok(value) => {
            clear_error();
            buffer(value)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the key import private.
pub unsafe extern "C" fn vault_key_import_private(bytes: *const u8, len: usize) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("private key pointer is null");
        return ptr::null_mut();
    };
    let secret = match revault_lockbox_api::SecretVec::try_from_slice(bytes) {
        Ok(secret) => secret,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match revault_vault_api::import_private_key(secret) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the key import public.
pub unsafe extern "C" fn vault_key_import_public(bytes: *const u8, len: usize) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("public key pointer is null");
        return ptr::null_mut();
    };
    match revault_vault_api::import_public_key(bytes) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the key fingerprint.
pub unsafe extern "C" fn vault_key_fingerprint(key: *const c_void) -> RevaultBuffer {
    let Some(key) = (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactPublicKey>()) }) else {
        set_error("contact public key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(revault_vault_api::public_key_fingerprint(key))
}

#[no_mangle]
/// Returns the key format hex.
pub unsafe extern "C" fn vault_key_format_hex(bytes: *const u8, len: usize) -> RevaultBuffer {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("fingerprint pointer is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(revault_vault_api::format_fingerprint_hex_pairs(bytes).into_bytes())
}

#[no_mangle]
/// Returns the key decode hex.
pub unsafe extern "C" fn vault_key_decode_hex(text: *const c_char, len: usize) -> RevaultBuffer {
    let Some(text) = (unsafe { input_str(text, len) }) else {
        set_error("fingerprint text is invalid");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::decode_fingerprint_hex(text) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the key format crockford.
pub unsafe extern "C" fn vault_key_format_crockford(bytes: *const u8, len: usize) -> RevaultBuffer {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("fingerprint pointer is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    if bytes.len() < revault_vault_api::FINGERPRINT_CODE_96_LEN {
        set_error("fingerprint must contain at least 96 bits");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    }
    clear_error();
    buffer(revault_vault_api::format_fingerprint_crockford_96(bytes).into_bytes())
}

#[no_mangle]
/// Returns the key format crockford reading.
pub unsafe extern "C" fn vault_key_format_crockford_reading(
    code: *const c_char,
    len: usize,
) -> RevaultBuffer {
    let Some(code) = (unsafe { input_str(code, len) }) else {
        set_error("fingerprint code is invalid");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(revault_vault_api::format_fingerprint_crockford_96_reading(code).into_bytes())
}

#[no_mangle]
/// Returns the key decode crockford.
pub unsafe extern "C" fn vault_key_decode_crockford(
    code: *const c_char,
    len: usize,
) -> RevaultBuffer {
    let Some(code) = (unsafe { input_str(code, len) }) else {
        set_error("fingerprint code is invalid");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::decode_fingerprint_crockford_96(code) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes.to_vec())
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the key hex encode.
pub unsafe extern "C" fn vault_key_hex_encode(bytes: *const u8, len: usize) -> RevaultBuffer {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("byte pointer is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(revault_vault_api::encode_hex(bytes).into_bytes())
}

#[no_mangle]
/// Returns the key hex decode.
pub unsafe extern "C" fn vault_key_hex_decode(text: *const c_char, len: usize) -> RevaultBuffer {
    let Some(text) = (unsafe { input_str(text, len) }) else {
        set_error("hex text is invalid");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::decode_hex(text) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Generates generate.
pub extern "C" fn key_signing_generate() -> *mut c_void {
    match OwnerSigningKeyPair::generate() {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the from private.
pub unsafe extern "C" fn key_signing_from_private(bytes: *const u8, len: usize) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("private signing key pointer is null");
        return ptr::null_mut();
    };
    match revault_lockbox_api::SecretVec::try_from_slice(bytes)
        .map_err(revault_lockbox_api::Error::from)
        .and_then(OwnerSigningKeyPair::from_private_key_record)
    {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the public.
pub unsafe extern "C" fn key_signing_public(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<OwnerSigningKeyPair>()) })
    else {
        set_error("signing key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(handle.public_key().to_bytes())
}

#[no_mangle]
/// Returns the private.
pub unsafe extern "C" fn key_signing_private(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<OwnerSigningKeyPair>()) })
    else {
        set_error("signing key handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.private_key_record() {
        Ok(bytes) => match bytes.with_bytes(|value| value.to_vec()) {
            Ok(value) => {
                clear_error();
                buffer(value)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the public from bytes.
pub unsafe extern "C" fn key_signing_public_from_bytes(
    bytes: *const u8,
    len: usize,
) -> *mut c_void {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("public signing key pointer is null");
        return ptr::null_mut();
    };
    match OwnerSigningPublicKey::from_bytes(bytes) {
        Ok(key) => Box::into_raw(Box::new(key)).cast(),
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the public free.
pub unsafe extern "C" fn key_signing_public_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<OwnerSigningPublicKey>())) };
    }
}

#[no_mangle]
/// Releases the native resources held by this object.
pub unsafe extern "C" fn key_signing_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<OwnerSigningKeyPair>())) };
    }
}

#[no_mangle]
/// Returns the directory open.
pub unsafe extern "C" fn vault_directory_open(
    root: *const c_char,
    root_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let (Some(root), Some(password)) = (unsafe { input_str(root, root_len) }, unsafe {
        input(password, password_len)
    }) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match revault_vault_api::VaultDirectory::open_or_create(root, &password) {
        Ok(vault) => {
            clear_error();
            Box::into_raw(Box::new(vault)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the structure version current.
pub extern "C" fn vault_structure_version_current() -> u32 {
    clear_error();
    revault_vault_api::CURRENT_VAULT_STRUCTURE_VERSION
}

#[no_mangle]
/// Returns the directory probe structure version.
pub unsafe extern "C" fn vault_directory_probe_structure_version(
    root: *const c_char,
    root_len: usize,
    password: *const u8,
    password_len: usize,
) -> u32 {
    let (Some(root), Some(password)) = (unsafe { input_str(root, root_len) }, unsafe {
        vault_password(password, password_len)
    }) else {
        set_error("invalid vault probe input");
        return 0;
    };
    match revault_vault_api::VaultDirectory::probe_structure_version(root, &password) {
        Ok(version) => {
            clear_error();
            version
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

unsafe fn vault_password(
    password: *const u8,
    password_len: usize,
) -> Option<revault_vault_api::SecretString> {
    let bytes = unsafe { input(password, password_len) }?;
    revault_vault_api::SecretString::try_from_slice(bytes).ok()
}

fn vault_handle(value: LockboxResult<VaultDirectoryHandle>) -> *mut c_void {
    match value {
        Ok(vault) => {
            clear_error();
            Box::into_raw(Box::new(vault)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory open or create default.
pub unsafe extern "C" fn vault_directory_open_or_create_default(
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let Some(password) = (unsafe { vault_password(password, password_len) }) else {
        set_error("invalid vault password");
        return ptr::null_mut();
    };
    vault_handle(revault_vault_api::VaultDirectory::open_or_create_default(
        &password,
    ))
}

#[no_mangle]
/// Returns the directory replace default.
pub unsafe extern "C" fn vault_directory_replace_default(
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let Some(password) = (unsafe { vault_password(password, password_len) }) else {
        set_error("invalid vault password");
        return ptr::null_mut();
    };
    vault_handle(revault_vault_api::VaultDirectory::replace_default(
        &password,
    ))
}

#[no_mangle]
/// Returns the directory open or create.
pub unsafe extern "C" fn vault_directory_open_or_create(
    root: *const c_char,
    root_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let (Some(root), Some(password)) = (unsafe { input_str(root, root_len) }, unsafe {
        vault_password(password, password_len)
    }) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    vault_handle(revault_vault_api::VaultDirectory::open_or_create(
        root, &password,
    ))
}

#[no_mangle]
/// Returns the directory replace.
pub unsafe extern "C" fn vault_directory_replace(
    root: *const c_char,
    root_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let (Some(root), Some(password)) = (unsafe { input_str(root, root_len) }, unsafe {
        vault_password(password, password_len)
    }) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    vault_handle(revault_vault_api::VaultDirectory::replace(root, &password))
}

fn vault_bool(value: LockboxResult<()>) -> bool {
    match value {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory change password.
pub unsafe extern "C" fn vault_directory_change_password(
    root: *const c_char,
    root_len: usize,
    old_password: *const u8,
    old_len: usize,
    new_password: *const u8,
    new_len: usize,
) -> bool {
    let (Some(root), Some(old_password), Some(new_password)) = (
        unsafe { input_str(root, root_len) },
        unsafe { vault_password(old_password, old_len) },
        unsafe { vault_password(new_password, new_len) },
    ) else {
        set_error("invalid vault password input");
        return false;
    };
    vault_bool(revault_vault_api::VaultDirectory::change_password(
        root,
        &old_password,
        &new_password,
    ))
}

#[no_mangle]
/// Returns the directory change default password.
pub unsafe extern "C" fn vault_directory_change_default_password(
    old_password: *const u8,
    old_len: usize,
    new_password: *const u8,
    new_len: usize,
) -> bool {
    let (Some(old_password), Some(new_password)) =
        (unsafe { vault_password(old_password, old_len) }, unsafe {
            vault_password(new_password, new_len)
        })
    else {
        set_error("invalid vault password input");
        return false;
    };
    vault_bool(revault_vault_api::VaultDirectory::change_default_password(
        &old_password,
        &new_password,
    ))
}

#[no_mangle]
/// Returns the directory root.
pub unsafe extern "C" fn vault_directory_root(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    clear_error();
    buffer(handle.root().to_string_lossy().as_bytes().to_vec())
}

#[no_mangle]
/// Returns the directory structure version.
pub unsafe extern "C" fn vault_directory_structure_version(handle: *const c_void) -> u32 {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return 0;
    };
    match handle.structure_version() {
        Ok(version) => {
            clear_error();
            version
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

#[no_mangle]
/// Returns the directory list private keys.
pub unsafe extern "C" fn vault_directory_list_private_keys(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_private_keys() {
        Ok(keys) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::StringListT { values: Some(keys) })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

fn string_list_buffer(value: LockboxResult<Vec<String>>) -> RevaultBuffer {
    match value {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::StringListT {
                values: Some(value),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory list private key names.
pub unsafe extern "C" fn vault_directory_list_private_key_names(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(handle.list_private_keys())
}

#[no_mangle]
/// Returns the directory list contact names.
pub unsafe extern "C" fn vault_directory_list_contact_names(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(handle.list_contacts().map(|contacts| {
        contacts
            .into_iter()
            .map(|contact| contact.name)
            .collect::<Vec<_>>()
    }))
}

#[no_mangle]
/// Returns the directory list form aliases.
pub unsafe extern "C" fn vault_directory_list_form_aliases(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(
        handle
            .list_form_definitions()
            .map(|forms| forms.into_iter().map(|form| form.alias).collect::<Vec<_>>()),
    )
}

#[no_mangle]
/// Returns the directory private key exists.
pub unsafe extern "C" fn vault_directory_private_key_exists(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return false;
    };
    match handle.private_key_exists(name) {
        Ok(exists) => {
            clear_error();
            exists
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory delete private key.
pub unsafe extern "C" fn vault_directory_delete_private_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return false;
    };
    match handle.delete_private_key(name) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory store private key.
pub unsafe extern "C" fn vault_directory_store_private_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    key: *const c_void,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return false;
    };
    let Some(key) = (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactKeyPair>()) }) else {
        set_error("contact key handle is null");
        return false;
    };
    match handle.store_private_key(name, key) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory load private key.
pub unsafe extern "C" fn vault_directory_load_private_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return ptr::null_mut();
    };
    match handle.load_private_key(name) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory load private key generation.
pub unsafe extern "C" fn vault_directory_load_private_key_generation(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    index: u16,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return ptr::null_mut();
    };
    match handle.load_private_key_generation(name, index) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory store contact.
pub unsafe extern "C" fn vault_directory_store_contact(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    key: *const c_void,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let (Some(name), Some(key)) = (
        unsafe { input_str(name, name_len) },
        (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactPublicKey>()) }),
    ) else {
        set_error("invalid contact input");
        return false;
    };
    match handle.store_contact(name, key) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory load contact.
pub unsafe extern "C" fn vault_directory_load_contact(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return ptr::null_mut();
    };
    match handle.load_contact(name) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory contact exists.
pub unsafe extern "C" fn vault_directory_contact_exists(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return false;
    };
    match handle.contact_exists(name) {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory delete contact.
pub unsafe extern "C" fn vault_directory_delete_contact(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return false;
    };
    match handle.delete_contact(name) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory list contacts.
pub unsafe extern "C" fn vault_directory_list_contacts(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_contacts() {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&contact_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory store profile email.
pub unsafe extern "C" fn vault_directory_store_profile_email(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    email: *const c_char,
    email_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let (Some(name), Some(email)) = (unsafe { input_str(name, name_len) }, unsafe {
        input_str(email, email_len)
    }) else {
        set_error("invalid profile input");
        return false;
    };
    match handle.store_profile_email(name, email) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory profile email.
pub unsafe extern "C" fn vault_directory_profile_email(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid profile name");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.profile_email(name) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::OptionalStringT {
                present: value.is_some(),
                value: Some(value.unwrap_or_default()),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory store backup.
pub unsafe extern "C" fn vault_directory_store_backup(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    bytes: *const u8,
    len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        set_error("backup pointer is null");
        return false;
    };
    match handle.store_key_directory_backup(id, bytes) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory load backup.
pub unsafe extern "C" fn vault_directory_load_backup(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match handle.load_key_directory_backup(id) {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory backup count.
pub unsafe extern "C" fn vault_directory_backup_count(handle: *const c_void) -> u64 {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return 0;
    };
    match handle.key_directory_backup_count() {
        Ok(count) => {
            clear_error();
            count as u64
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

#[no_mangle]
/// Returns the directory restore private key.
pub unsafe extern "C" fn vault_directory_restore_private_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    key: *const c_void,
    signing_key: *const c_void,
    overwrite: bool,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let (Some(name), Some(key)) = (
        unsafe { input_str(name, name_len) },
        (!key.is_null()).then(|| unsafe { &*(key.cast::<ContactKeyPair>()) }),
    ) else {
        set_error("invalid profile input");
        return false;
    };
    let signing =
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) });
    match handle.restore_private_key(name, key, signing, overwrite) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory load owner signing key.
pub unsafe extern "C" fn vault_directory_load_owner_signing_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid profile name");
        return ptr::null_mut();
    };
    match handle.load_owner_signing_key(name) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory load owner signing key generation.
pub unsafe extern "C" fn vault_directory_load_owner_signing_key_generation(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    index: u16,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid vault name");
        return ptr::null_mut();
    };
    match handle.load_owner_signing_key_generation(name, index) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory store contact signing key.
pub unsafe extern "C" fn vault_directory_store_contact_signing_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
    key: *const c_void,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let (Some(name), Some(key)) = (
        unsafe { input_str(name, name_len) },
        (!key.is_null()).then(|| unsafe { &*(key.cast::<OwnerSigningPublicKey>()) }),
    ) else {
        set_error("invalid signing contact input");
        return false;
    };
    match handle.store_contact_signing_key(name, key) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory load contact signing key.
pub unsafe extern "C" fn vault_directory_load_contact_signing_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> *mut c_void {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid contact name");
        return ptr::null_mut();
    };
    match handle.load_contact_signing_key(name) {
        Ok(key) => {
            clear_error();
            Box::into_raw(Box::new(key)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the directory list profile generations.
pub unsafe extern "C" fn vault_directory_list_profile_generations(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid profile name");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_profile_generations(name) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&profile_history_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory rotate private key.
pub unsafe extern "C" fn vault_directory_rotate_private_key(
    handle: *const c_void,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid profile name");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.rotate_private_key(name) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&profile_history_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory remember lockbox.
pub unsafe extern "C" fn vault_directory_remember_lockbox(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid lockbox path");
        return false;
    };
    match handle.remember_known_lockbox(id, path) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory list known lockboxes.
pub unsafe extern "C" fn vault_directory_list_known_lockboxes(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_known_lockboxes() {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&known_lockbox_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory forget lockbox.
pub unsafe extern "C" fn vault_directory_forget_lockbox(
    handle: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid lockbox path");
        return false;
    };
    match handle.forget_known_lockbox(path) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory remember access slot label.
pub unsafe extern "C" fn vault_directory_remember_access_slot_label(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    slot_id: u64,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid slot label");
        return false;
    };
    match handle.remember_access_slot_label(id, slot_id, name) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory list access slot labels.
pub unsafe extern "C" fn vault_directory_list_access_slot_labels(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match handle.list_access_slot_labels(id) {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&access_slot_label_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory find access slot labels.
pub unsafe extern "C" fn vault_directory_find_access_slot_labels(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    name: *const c_char,
    name_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    let Some(name) = (unsafe { input_str(name, name_len) }) else {
        set_error("invalid slot label");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.find_access_slot_labels(id, name) {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&access_slot_label_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory forget access slot label.
pub unsafe extern "C" fn vault_directory_forget_access_slot_label(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    slot_id: u64,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match handle.forget_access_slot_label(id, slot_id) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory define form.
pub unsafe extern "C" fn vault_directory_define_form(
    handle: *const c_void,
    alias: *const c_char,
    alias_len: usize,
    name: *const c_char,
    name_len: usize,
    description: *const c_char,
    description_len: usize,
    fields_transport: *const u8,
    fields_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let (Some(alias), Some(name), Some(description), Some(fields_transport)) = (
        unsafe { input_str(alias, alias_len) },
        unsafe { input_str(name, name_len) },
        unsafe { input_str(description, description_len) },
        unsafe { input(fields_transport, fields_len) },
    ) else {
        set_error("invalid form input");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let fields = match form_fields_from_transport(fields_transport) {
        Ok(fields) => fields,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    if fields.is_empty() {
        set_error("form fields must not be empty");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.define_form_with_description(alias, name, description, fields) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory resolve form.
pub unsafe extern "C" fn vault_directory_resolve_form(
    handle: *const c_void,
    reference: *const c_char,
    reference_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(reference) = (unsafe { input_str(reference, reference_len) }) else {
        set_error("invalid form reference");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.resolve_form_definition(reference) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_transport(&value))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory list forms.
pub unsafe extern "C" fn vault_directory_list_forms(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_form_definitions() {
        Ok(values) => {
            clear_error();
            flatbuffer_buffer!(&form_definition_list_transport(&values))
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory list form revisions.
pub unsafe extern "C" fn vault_directory_list_form_revisions(
    handle: *const c_void,
    type_id: *const c_char,
    type_id_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let Some(type_id) = (unsafe { input_str(type_id, type_id_len) }) else {
        set_error("invalid form type id");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let type_id = match FormTypeId::new(type_id) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match handle.list_form_definition_revisions(&type_id) {
        Ok(values) => flatbuffer_buffer!(&form_definition_list_transport(&values)),
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory seed forms.
pub unsafe extern "C" fn vault_directory_seed_forms(handle: *const c_void) -> usize {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return 0;
    };
    match handle.seed_default_form_definitions() {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

#[no_mangle]
/// Returns the directory remember password.
pub unsafe extern "C" fn vault_directory_remember_password(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
    password: *const u8,
    password_len: usize,
) -> bool {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    let Some(password) = (unsafe { input(password, password_len) }) else {
        set_error("password pointer is null");
        return false;
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match handle.remember_lockbox_password(id, &password) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the directory remembered password.
pub unsafe extern "C" fn vault_directory_remembered_password(
    handle: *const c_void,
    id: *const u8,
    id_len: usize,
) -> RevaultBuffer {
    let Some(handle) =
        (!handle.is_null()).then(|| unsafe { &*(handle.cast::<VaultDirectoryHandle>()) })
    else {
        set_error("vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match handle.remembered_lockbox_password(id) {
        Ok(Some(value)) => match value.with_bytes(|bytes| bytes.to_vec()) {
            Ok(bytes) => {
                clear_error();
                buffer(bytes)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Ok(None) => {
            clear_error();
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the backup default.
pub unsafe extern "C" fn vault_backup_default(
    path: *const c_char,
    path_len: usize,
    overwrite: bool,
) -> RevaultBuffer {
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid backup path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::backup_default_vault(path, overwrite) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::VaultBackupManifestT {
                format_version: value.format_version as u32,
                created_at_unix_ms: value.created_at_unix_ms,
                vault_file_name: Some(value.vault_file_name),
                vault_size: value.vault_size,
                vault_sha256: Some(value.vault_sha256),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the restore default.
pub unsafe extern "C" fn vault_restore_default(
    path: *const c_char,
    path_len: usize,
    overwrite: bool,
) -> RevaultBuffer {
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid restore path");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::restore_default_vault(path, overwrite) {
        Ok(value) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::VaultBackupManifestT {
                format_version: value.format_version as u32,
                created_at_unix_ms: value.created_at_unix_ms,
                vault_file_name: Some(value.vault_file_name),
                vault_size: value.vault_size,
                vault_sha256: Some(value.vault_sha256),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the directory free.
pub unsafe extern "C" fn vault_directory_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<VaultDirectoryHandle>())) };
    }
}

fn read_only_vault_handle(value: LockboxResult<ReadOnlyVaultDirectoryHandle>) -> *mut c_void {
    match value {
        Ok(vault) => {
            clear_error();
            Box::into_raw(Box::new(vault)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns only open.
pub unsafe extern "C" fn vault_read_only_open(
    root: *const c_char,
    root_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let (Some(root), Some(password)) = (unsafe { input_str(root, root_len) }, unsafe {
        vault_password(password, password_len)
    }) else {
        set_error("invalid read-only vault input");
        return ptr::null_mut();
    };
    read_only_vault_handle(ReadOnlyVaultDirectoryHandle::open(root, &password))
}

#[no_mangle]
/// Returns only open default.
pub unsafe extern "C" fn vault_read_only_open_default(
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let Some(password) = (unsafe { vault_password(password, password_len) }) else {
        set_error("invalid read-only vault password");
        return ptr::null_mut();
    };
    read_only_vault_handle(ReadOnlyVaultDirectoryHandle::open_default(&password))
}

unsafe fn read_only_vault<'a>(handle: *const c_void) -> Option<&'a ReadOnlyVaultDirectoryHandle> {
    (!handle.is_null()).then(|| unsafe { &*(handle.cast::<ReadOnlyVaultDirectoryHandle>()) })
}

#[no_mangle]
/// Returns only list profile names.
pub unsafe extern "C" fn vault_read_only_list_profile_names(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) = (unsafe { read_only_vault(handle) }) else {
        set_error("read-only vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(handle.list_private_key_names())
}

#[no_mangle]
/// Returns only list contact names.
pub unsafe extern "C" fn vault_read_only_list_contact_names(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) = (unsafe { read_only_vault(handle) }) else {
        set_error("read-only vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(handle.list_contact_names())
}

#[no_mangle]
/// Returns only list form aliases.
pub unsafe extern "C" fn vault_read_only_list_form_aliases(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (unsafe { read_only_vault(handle) }) else {
        set_error("read-only vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    string_list_buffer(handle.list_form_aliases())
}

#[no_mangle]
/// Returns only list known lockboxes.
pub unsafe extern "C" fn vault_read_only_list_known_lockboxes(
    handle: *const c_void,
) -> RevaultBuffer {
    let Some(handle) = (unsafe { read_only_vault(handle) }) else {
        set_error("read-only vault handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.list_known_lockboxes() {
        Ok(values) => flatbuffer_buffer!(&known_lockbox_list_transport(&values)),
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns only free.
pub unsafe extern "C" fn vault_read_only_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<ReadOnlyVaultDirectoryHandle>())) };
    }
}

fn lockbox_id_from_bytes(
    bytes: *const u8,
    len: usize,
) -> Result<revault_lockbox_api::LockboxId, String> {
    let Some(bytes) = (unsafe { input(bytes, len) }) else {
        return Err("lockbox id pointer is null".to_string());
    };
    let array: [u8; 16] = bytes
        .try_into()
        .map_err(|_| "lockbox id must be 16 bytes".to_string())?;
    Ok(revault_lockbox_api::LockboxId::from_bytes(array))
}

#[no_mangle]
/// Returns the agent serve.
pub unsafe extern "C" fn vault_agent_serve() -> bool {
    match revault_vault_api::serve_agent() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent verify transport.
pub extern "C" fn vault_agent_verify_transport() -> bool {
    match revault_vault_api::verify_agent_transport_security() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent get.
pub unsafe extern "C" fn vault_agent_get(id: *const u8, id_len: usize) -> RevaultBuffer {
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
    };
    match revault_vault_api::get(id) {
        Ok(Some(key)) => match key.with_bytes(|value| value.to_vec()) {
            Ok(value) => {
                clear_error();
                buffer(value)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Ok(None) => {
            clear_error();
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the agent put.
pub unsafe extern "C" fn vault_agent_put(
    id: *const u8,
    id_len: usize,
    key: *const u8,
    key_len: usize,
) -> bool {
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    let Some(key) = (unsafe { input(key, key_len) }) else {
        set_error("key pointer is null");
        return false;
    };
    match revault_vault_api::put(id, key) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent forget.
pub unsafe extern "C" fn vault_agent_forget(id: *const u8, id_len: usize) -> bool {
    let id = match lockbox_id_from_bytes(id, id_len) {
        Ok(id) => id,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match revault_vault_api::forget(id) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent stop.
pub extern "C" fn vault_agent_stop() -> bool {
    match revault_vault_api::stop() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent start.
pub extern "C" fn vault_agent_start() -> bool {
    match revault_vault_api::start() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent list.
pub extern "C" fn vault_agent_list() -> RevaultBuffer {
    match revault_vault_api::list() {
        Ok(entries) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::AgentEntryListT {
                values: Some(
                    entries
                        .iter()
                        .map(|entry| bindings_transport::AgentEntryT {
                            id: Some(entry.id.clone()),
                            path: Some(entry.path.clone().unwrap_or_default()),
                        })
                        .collect()
                ),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the agent sleep support.
pub extern "C" fn vault_agent_sleep_support() -> RevaultBuffer {
    let support = revault_vault_api::agent_sleep_support();
    clear_error();
    flatbuffer_buffer!(&bindings_transport::SleepSupportT {
        suspend_notifications: support.suspend_notifications,
        sleep_inhibition: support.sleep_inhibition,
        supported: support.supported(),
    })
}

#[no_mangle]
/// Returns the platform status.
pub extern "C" fn vault_platform_status() -> RevaultBuffer {
    match revault_vault_api::platform_secret_store_status() {
        Ok(status) => {
            clear_error();
            flatbuffer_buffer!(&bindings_transport::PlatformStatusT {
                supported: status.supported,
                disabled: status.disabled,
                scope: Some(status.scope.as_str().to_string()),
                backend: Some(status.backend.to_string()),
                item: Some(status.item),
            })
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

fn auto_open_scope(
    value: *const c_char,
    len: usize,
) -> Result<revault_vault_api::AutoOpenScope, String> {
    let Some(value) = (unsafe { input_str(value, len) }) else {
        return Err("invalid auto-open scope".to_string());
    };
    match value {
        "off" => Ok(revault_vault_api::AutoOpenScope::Off),
        "vault" => Ok(revault_vault_api::AutoOpenScope::Vault),
        "lockboxes" => Ok(revault_vault_api::AutoOpenScope::Lockboxes),
        _ => Err("scope must be off, vault, or lockboxes".to_string()),
    }
}

#[no_mangle]
/// Returns the platform set scope.
pub unsafe extern "C" fn vault_platform_set_scope(scope: *const c_char, len: usize) -> bool {
    let scope = match auto_open_scope(scope, len) {
        Ok(scope) => scope,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match revault_vault_api::set_auto_open_scope(scope) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the platform forget password.
pub extern "C" fn vault_platform_forget_password() -> bool {
    match revault_vault_api::forget_platform_vault_password() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the platform enable.
pub extern "C" fn vault_platform_enable() -> bool {
    match revault_vault_api::enable_platform_secret_store() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the platform disable.
pub extern "C" fn vault_platform_disable() -> bool {
    match revault_vault_api::disable_platform_secret_store() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the platform disabled.
pub extern "C" fn vault_platform_disabled() -> bool {
    match revault_vault_api::platform_secret_store_disabled() {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error);
            true
        }
    }
}

#[no_mangle]
/// Returns the platform get password.
pub extern "C" fn vault_platform_get_password() -> RevaultBuffer {
    match revault_vault_api::get_platform_vault_password() {
        Ok(Some(value)) => match value.with_bytes(|bytes| bytes.to_vec()) {
            Ok(bytes) => {
                clear_error();
                buffer(bytes)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Ok(None) => {
            clear_error();
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the default directory.
pub extern "C" fn vault_default_directory() -> RevaultBuffer {
    match revault_vault_api::default_vault_dir() {
        Ok(value) => {
            clear_error();
            buffer(value.to_string_lossy().as_bytes().to_vec())
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the default path.
pub extern "C" fn vault_default_path() -> RevaultBuffer {
    match revault_vault_api::default_vault_path() {
        Ok(value) => {
            clear_error();
            buffer(value.to_string_lossy().as_bytes().to_vec())
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the agent log path.
pub extern "C" fn vault_agent_log_path() -> RevaultBuffer {
    clear_error();
    buffer(
        revault_vault_api::agent_log_path()
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
    )
}

#[no_mangle]
/// Returns the agent log destination.
pub extern "C" fn vault_agent_log_destination() -> RevaultBuffer {
    clear_error();
    buffer(revault_vault_api::agent_log_destination().into_bytes())
}

#[no_mangle]
/// Returns the agent get vault unlock key.
pub unsafe extern "C" fn vault_agent_get_vault_unlock_key(
    vault_id: *const c_char,
    vault_id_len: usize,
) -> RevaultBuffer {
    let Some(vault_id) = (unsafe { input_str(vault_id, vault_id_len) }) else {
        set_error("invalid vault id");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match revault_vault_api::get_vault_unlock_key(vault_id) {
        Ok(Some(value)) => match value.with_bytes(|bytes| bytes.to_vec()) {
            Ok(bytes) => {
                clear_error();
                buffer(bytes)
            }
            Err(error) => {
                set_error(error);
                RevaultBuffer {
                    ptr: ptr::null_mut(),
                    len: 0,
                }
            }
        },
        Ok(None) => {
            clear_error();
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Returns the agent put vault unlock key.
pub unsafe extern "C" fn vault_agent_put_vault_unlock_key(
    vault_id: *const c_char,
    vault_id_len: usize,
    key: *const u8,
    key_len: usize,
    ttl_seconds: u64,
) -> bool {
    let Some(vault_id) = (unsafe { input_str(vault_id, vault_id_len) }) else {
        set_error("invalid vault id");
        return false;
    };
    let Some(key) = (unsafe { input(key, key_len) }) else {
        set_error("key pointer is null");
        return false;
    };
    let key = match revault_vault_api::SecretString::try_from_slice(key) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match revault_vault_api::put_vault_unlock_key(vault_id, key, Some(ttl_seconds)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent forget vault unlock key.
pub unsafe extern "C" fn vault_agent_forget_vault_unlock_key(
    vault_id: *const c_char,
    vault_id_len: usize,
) -> bool {
    let Some(vault_id) = (unsafe { input_str(vault_id, vault_id_len) }) else {
        set_error("invalid vault id");
        return false;
    };
    match revault_vault_api::forget_vault_unlock_key(vault_id) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent get owner signing key.
pub unsafe extern "C" fn vault_agent_get_owner_signing_key(
    vault_id: *const c_char,
    vault_len: usize,
    profile: *const c_char,
    profile_len: usize,
) -> *mut c_void {
    let (Some(vault_id), Some(profile)) = (unsafe { input_str(vault_id, vault_len) }, unsafe {
        input_str(profile, profile_len)
    }) else {
        set_error("invalid owner profile");
        return ptr::null_mut();
    };
    match revault_vault_api::get_owner_signing_key(vault_id, profile) {
        Ok(Some(value)) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Ok(None) => {
            clear_error();
            ptr::null_mut()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the agent put owner signing key.
pub unsafe extern "C" fn vault_agent_put_owner_signing_key(
    vault_id: *const c_char,
    vault_len: usize,
    profile: *const c_char,
    profile_len: usize,
    key: *const c_void,
    ttl_seconds: u64,
) -> bool {
    let (Some(vault_id), Some(profile), Some(key)) = (
        unsafe { input_str(vault_id, vault_len) },
        unsafe { input_str(profile, profile_len) },
        (!key.is_null()).then(|| unsafe { &*(key.cast::<OwnerSigningKeyPair>()) }),
    ) else {
        set_error("invalid owner profile");
        return false;
    };
    let key = match key.try_clone() {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match revault_vault_api::put_owner_signing_key(vault_id, profile, key, Some(ttl_seconds)) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the agent forget owner signing key.
pub unsafe extern "C" fn vault_agent_forget_owner_signing_key(
    vault_id: *const c_char,
    vault_len: usize,
    profile: *const c_char,
    profile_len: usize,
) -> bool {
    let (Some(vault_id), Some(profile)) = (unsafe { input_str(vault_id, vault_len) }, unsafe {
        input_str(profile, profile_len)
    }) else {
        set_error("invalid owner profile");
        return false;
    };
    match revault_vault_api::forget_owner_signing_key(vault_id, profile) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

fn activity_kind(
    value: *const c_char,
    len: usize,
) -> Result<revault_vault_api::SecretActivityKind, String> {
    let Some(value) = (unsafe { input_str(value, len) }) else {
        return Err("invalid activity kind".to_string());
    };
    match value {
        "open" => Ok(revault_vault_api::SecretActivityKind::Open),
        "close" => Ok(revault_vault_api::SecretActivityKind::Close),
        "variables" => Ok(revault_vault_api::SecretActivityKind::Variables),
        "form" => Ok(revault_vault_api::SecretActivityKind::Form),
        "recovery" => Ok(revault_vault_api::SecretActivityKind::Recovery),
        "vault" => Ok(revault_vault_api::SecretActivityKind::Vault),
        _ => Err("unknown activity kind".to_string()),
    }
}

#[no_mangle]
/// Returns the agent begin activity.
pub unsafe extern "C" fn vault_agent_begin_activity(
    kind: *const c_char,
    len: usize,
) -> *mut c_void {
    let kind = match activity_kind(kind, len) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match revault_vault_api::begin_secret_activity(kind) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Returns the agent end activity.
pub unsafe extern "C" fn vault_agent_end_activity(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<SecretActivityHandle>())) };
    }
}

#[no_mangle]
/// Returns the platform put password.
pub unsafe extern "C" fn vault_platform_put_password(password: *const u8, len: usize) -> bool {
    let Some(password) = (unsafe { input(password, len) }) else {
        set_error("password pointer is null");
        return false;
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match revault_vault_api::put_platform_vault_password(&password) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Returns the local.
pub extern "C" fn vault_local() -> *mut c_void {
    Box::into_raw(Box::new(revault_vault_api::local_vault())).cast()
}

#[no_mangle]
/// Creates lockbox password.
pub unsafe extern "C" fn vault_create_lockbox_password(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let (Some(path), Some(password)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(password, password_len)
    }) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match vault.create_lockbox_with_password(path, &password) {
        Ok(lockbox) => {
            clear_error();
            Box::into_raw(Box::new(lockbox)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens lockbox password.
pub unsafe extern "C" fn vault_open_lockbox_password(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_void {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let (Some(path), Some(password)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(password, password_len)
    }) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match vault.open_lockbox_with_password(path, &password) {
        Ok(lockbox) => {
            clear_error();
            Box::into_raw(Box::new(lockbox)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Creates lockbox content key.
pub unsafe extern "C" fn vault_create_lockbox_content_key(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    content_key: *const u8,
    key_len: usize,
    signing_key: *const c_void,
) -> *mut c_void {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let (Some(path), Some(key), Some(signing_key)) = (
        unsafe { input_str(path, path_len) },
        unsafe { input(content_key, key_len) },
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) }),
    ) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let key = match revault_lockbox_api::SecretVec::try_from_slice(key) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match vault.create_lockbox_with_signing_key(
        path,
        LockboxProtection::ContentKey(key),
        signing_key,
    ) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Creates lockbox contact.
pub unsafe extern "C" fn vault_create_lockbox_contact(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    contact: *const c_void,
    name: *const c_char,
    name_len: usize,
    signing_key: *const c_void,
) -> *mut c_void {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let (Some(path), Some(contact), Some(signing_key)) = (
        unsafe { input_str(path, path_len) },
        (!contact.is_null()).then(|| unsafe { &*(contact.cast::<ContactPublicKey>()) }),
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) }),
    ) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let name = if name.is_null() {
        None
    } else {
        match unsafe { input_str(name, name_len) } {
            Some(value) => Some(value.to_string()),
            None => {
                set_error("invalid contact name");
                return ptr::null_mut();
            }
        }
    };
    match vault.create_lockbox_with_signing_key(
        path,
        LockboxProtection::ContactPublicKey {
            name,
            contact: contact.clone(),
        },
        signing_key,
    ) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Opens lockbox content key.
pub unsafe extern "C" fn vault_open_lockbox_content_key(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    content_key: *const u8,
    key_len: usize,
    signing_key: *const c_void,
) -> *mut c_void {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return ptr::null_mut();
    };
    let (Some(path), Some(key), Some(signing_key)) = (
        unsafe { input_str(path, path_len) },
        unsafe { input(content_key, key_len) },
        (!signing_key.is_null()).then(|| unsafe { &*(signing_key.cast::<OwnerSigningKeyPair>()) }),
    ) else {
        set_error("invalid vault input");
        return ptr::null_mut();
    };
    let key = match revault_lockbox_api::SecretVec::try_from_slice(key) {
        Ok(value) => value,
        Err(error) => {
            set_error(error);
            return ptr::null_mut();
        }
    };
    match vault.open_lockbox_with_signing_key(path, LockboxOpen::ContentKey(key), signing_key) {
        Ok(value) => {
            clear_error();
            Box::into_raw(Box::new(value)).cast()
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
/// Stores lockbox password.
pub unsafe extern "C" fn vault_cache_lockbox_password(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
    password: *const u8,
    password_len: usize,
    ttl_seconds: u64,
) -> bool {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let (Some(path), Some(password)) = (unsafe { input_str(path, path_len) }, unsafe {
        input(password, password_len)
    }) else {
        set_error("invalid vault input");
        return false;
    };
    let password = match revault_vault_api::SecretString::try_from_slice(password) {
        Ok(password) => password,
        Err(error) => {
            set_error(error);
            return false;
        }
    };
    match vault.cache_lockbox_password_for_duration(path, &password, ttl_seconds) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Releases the native resources held by lockbox.
pub unsafe extern "C" fn vault_close_lockbox(
    vault: *const c_void,
    path: *const c_char,
    path_len: usize,
) -> bool {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    let Some(path) = (unsafe { input_str(path, path_len) }) else {
        set_error("invalid vault path");
        return false;
    };
    match vault.close_lockbox(path) {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Releases the native resources held by all.
pub unsafe extern "C" fn vault_close_all(vault: *const c_void) -> bool {
    let Some(vault) = (!vault.is_null()).then(|| unsafe { &*(vault.cast::<LocalVaultHandle>()) })
    else {
        set_error("vault handle is null");
        return false;
    };
    match vault.close_all() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[no_mangle]
/// Releases the native resources held by this object.
pub unsafe extern "C" fn vault_free(vault: *mut c_void) {
    if !vault.is_null() {
        unsafe { drop(Box::from_raw(vault.cast::<LocalVaultHandle>())) };
    }
}

#[no_mangle]
/// Returns the to bytes.
pub unsafe extern "C" fn lockbox_to_bytes(handle: *const c_void) -> RevaultBuffer {
    let Some(handle) = (!handle.is_null()).then(|| unsafe { &*(handle.cast::<LockboxHandle>()) })
    else {
        set_error("lockbox handle is null");
        return RevaultBuffer {
            ptr: ptr::null_mut(),
            len: 0,
        };
    };
    match handle.try_to_bytes() {
        Ok(bytes) => {
            clear_error();
            buffer(bytes)
        }
        Err(error) => {
            set_error(error);
            RevaultBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

#[no_mangle]
/// Releases the native resources held by this object.
pub unsafe extern "C" fn lockbox_free(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle.cast::<LockboxHandle>())) };
    }
}

#[no_mangle]
/// Reports whether running.
pub extern "C" fn vault_is_running() -> bool {
    revault_vault_api::is_running()
}

#[no_mangle]
/// Removes all.
pub extern "C" fn vault_forget_all() -> bool {
    match revault_vault_api::forget_all() {
        Ok(()) => {
            clear_error();
            true
        }
        Err(error) => {
            set_error(error);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn c_abi_covers_core_object_operations() {
        let key = [7u8; 32];
        let handle = lockbox_create(key.as_ptr(), key.len());
        assert!(
            !handle.is_null(),
            "{}",
            unsafe { CStr::from_ptr(buffer_last_error()) }.to_string_lossy()
        );
        let path = b"/docs";
        assert!(unsafe { lockbox_create_dir(handle, path.as_ptr().cast(), path.len(), true) });
        let file = b"/docs/readme.txt";
        let contents = b"hello";
        assert!(unsafe {
            lockbox_add_file(
                handle,
                file.as_ptr().cast(),
                file.len(),
                contents.as_ptr(),
                contents.len(),
                false,
            )
        });
        let variable = b"API_KEY";
        let value = b"secret";
        assert!(unsafe {
            lockbox_set_secret_variable(
                handle,
                variable.as_ptr().cast(),
                variable.len(),
                value.as_ptr(),
                value.len(),
            )
        });
        let mut secret = ptr::null_mut();
        assert!(unsafe {
            lockbox_get_secret_variable(
                handle,
                variable.as_ptr().cast(),
                variable.len(),
                &mut secret,
            )
        });
        assert!(!secret.is_null());
        let mut secret_length = 0;
        assert!(unsafe { secret_len(secret, &mut secret_length) });
        assert_eq!(secret_length, value.len());
        let mut secret_copy_bytes = vec![0; secret_length];
        assert!(unsafe {
            secret_copy(
                secret,
                secret_copy_bytes.as_mut_ptr(),
                secret_copy_bytes.len(),
            )
        });
        assert_eq!(secret_copy_bytes, value);
        secret_copy_bytes.fill(0);
        unsafe { secret_free(secret) };

        let missing = b"MISSING";
        secret = usize::MAX as *mut c_void;
        assert!(unsafe {
            lockbox_get_secret_variable(handle, missing.as_ptr().cast(), missing.len(), &mut secret)
        });
        assert!(secret.is_null());
        assert!(unsafe { lockbox_commit(handle) });
        let result = unsafe { lockbox_get_file(handle, file.as_ptr().cast(), file.len()) };
        assert_eq!(
            unsafe { std::slice::from_raw_parts(result.ptr, result.len) },
            contents
        );
        buffer_free(result);
        let listed = unsafe { lockbox_list(handle, b"/".as_ptr().cast(), 1, true) };
        let listed_bytes = unsafe { std::slice::from_raw_parts(listed.ptr, listed.len) };
        let listing =
            flatbuffers::root::<bindings_transport::LockboxEntryList<'_>>(listed_bytes).unwrap();
        assert!(listing
            .entries()
            .unwrap()
            .iter()
            .any(|entry| entry.path() == Some("/docs/readme.txt")));
        buffer_free(listed);
        let stats = unsafe { lockbox_cache_stats(handle) };
        let stats_bytes = unsafe { std::slice::from_raw_parts(stats.ptr, stats.len) };
        flatbuffers::root::<bindings_transport::CacheStats<'_>>(stats_bytes).unwrap();
        buffer_free(stats);

        let missing_path = b"/missing";
        let stat =
            unsafe { lockbox_stat(handle, missing_path.as_ptr().cast(), missing_path.len()) };
        let stat_bytes = unsafe { std::slice::from_raw_parts(stat.ptr, stat.len) };
        assert!(
            flatbuffers::root::<bindings_transport::OptionalLockboxEntry<'_>>(stat_bytes)
                .unwrap()
                .value()
                .is_none()
        );
        buffer_free(stat);

        let record = unsafe {
            lockbox_get_form_record(handle, missing_path.as_ptr().cast(), missing_path.len())
        };
        let record_bytes = unsafe { std::slice::from_raw_parts(record.ptr, record.len) };
        assert!(
            flatbuffers::root::<bindings_transport::OptionalFormRecord<'_>>(record_bytes)
                .unwrap()
                .value()
                .is_none()
        );
        buffer_free(record);

        let field = b"username";
        let value = unsafe {
            lockbox_get_form_field(
                handle,
                missing_path.as_ptr().cast(),
                missing_path.len(),
                field.as_ptr().cast(),
                field.len(),
            )
        };
        let value_bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
        assert!(
            flatbuffers::root::<bindings_transport::OptionalFormValue<'_>>(value_bytes)
                .unwrap()
                .value()
                .is_none()
        );
        buffer_free(value);
        unsafe { lockbox_free(handle) };
    }
}
