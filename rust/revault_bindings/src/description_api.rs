//! Description operations shared by all native facades.
use super::*;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    target: String,
    reference: String,
    operation: String,
    field: Option<String>,
    text: Option<String>,
}

impl Request {
    fn validate(&self) -> LockboxResult<()> {
        if !matches!(
            self.target.as_str(),
            "variable" | "record" | "definition" | "field"
        ) || !matches!(self.operation.as_str(), "get" | "set" | "clear")
            || (self.target == "field") != self.field.is_some()
            || (self.operation == "set") != self.text.is_some()
        {
            return Err(revault_lockbox_api::Error::InvalidInput(
                "invalid description request".into(),
            ));
        }
        Ok(())
    }
    fn get(&self, lockbox: &LockboxHandle) -> LockboxResult<Option<String>> {
        match self.target.as_str() {
            "variable" => lockbox.variable_description(&VariableName::new(&self.reference)?),
            "record" => lockbox.form_record_description(&LockboxPath::new(&self.reference)?),
            "definition" => Ok(Some(
                lockbox
                    .resolve_form_definition(&self.reference)?
                    .description,
            )),
            "field" => {
                lockbox.form_field_description(&self.reference, self.field.as_deref().unwrap())
            }
            _ => unreachable!(),
        }
    }
    fn change(&self, lockbox: &mut LockboxHandle) -> LockboxResult<()> {
        match (self.target.as_str(), self.text.as_deref()) {
            ("variable", Some(text)) => {
                lockbox.set_variable_description(&VariableName::new(&self.reference)?, text)
            }
            ("variable", None) => {
                lockbox.clear_variable_description(&VariableName::new(&self.reference)?)
            }
            ("record", Some(text)) => {
                lockbox.set_form_record_description(&LockboxPath::new(&self.reference)?, text)
            }
            ("record", None) => {
                lockbox.clear_form_record_description(&LockboxPath::new(&self.reference)?)
            }
            ("definition", Some(text)) => lockbox
                .set_form_definition_description(&self.reference, text)
                .map(|_| ()),
            ("definition", None) => lockbox
                .clear_form_definition_description(&self.reference)
                .map(|_| ()),
            ("field", Some(text)) => lockbox
                .set_form_field_description(&self.reference, self.field.as_deref().unwrap(), text)
                .map(|_| ()),
            ("field", None) => lockbox
                .clear_form_field_description(&self.reference, self.field.as_deref().unwrap())
                .map(|_| ()),
            _ => unreachable!(),
        }
    }
}

unsafe fn parse_request(request: *const c_char, len: usize) -> LockboxResult<Request> {
    // Allow JSON escaping of a one-MiB description while bounding parsing work.
    if len > 8 * 1024 * 1024 {
        return Err(revault_lockbox_api::Error::InvalidInput(
            "description request exceeds 8 MiB".into(),
        ));
    }
    let request = unsafe { input_str(request, len) }.ok_or_else(|| {
        revault_lockbox_api::Error::InvalidInput("invalid description request UTF-8".into())
    })?;
    let request: Request = serde_json::from_str(request).map_err(|_| {
        revault_lockbox_api::Error::InvalidInput("invalid description request JSON".into())
    })?;
    request.validate()?;
    Ok(request)
}

fn response(result: LockboxResult<Option<String>>) -> RevaultBuffer {
    match result {
        Ok(value) => {
            clear_error();
            let data = bindings_transport::OptionalStringT {
                present: value.is_some(),
                value: Some(value.unwrap_or_default()),
            };
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let root = data.pack(&mut builder);
            builder.finish(root, None);
            buffer(builder.finished_data().to_vec())
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

/// Get, set, or clear an item's description. Request is UTF-8 JSON with target
/// (variable/record/definition/field), reference, operation (get/set/clear), field
/// (only for field targets), and text (only for set). Returns OptionalString
/// FlatBuffer containing the current description. Writes require a writable
/// handle and a subsequent commit. Release the result with buffer_free.
///
/// # Safety
/// The handle must be live; request must be readable for request_len bytes.
/// No overlapping mutable access to the handle is permitted.
#[no_mangle]
pub unsafe extern "C" fn lockbox_item_description(
    handle: *const c_void,
    request: *const c_char,
    request_len: usize,
) -> RevaultBuffer {
    response((|| {
        let request = unsafe { parse_request(request, request_len) }?;
        if request.operation != "get" {
            let lockbox = unsafe { lockbox_mut(handle.cast_mut()) }.ok_or_else(|| {
                revault_lockbox_api::Error::InvalidOperation(
                    "description write requires a writable lockbox".into(),
                )
            })?;
            request.change(lockbox)?;
        }
        let lockbox = unsafe { lockbox_ref(handle) }.ok_or_else(|| {
            revault_lockbox_api::Error::InvalidInput("null lockbox handle".into())
        })?;
        request.get(lockbox)
    })())
}

/// Operates on reusable vault definition/field descriptions using the same JSON
/// request and OptionalString result as lockbox_item_description. Writes commit
/// immediately. Release the result with buffer_free.
///
/// # Safety
/// The handle must be a live writable VaultDirectory. Request must be readable
/// for request_len bytes. No overlapping access may alias mutable state.
#[no_mangle]
pub unsafe extern "C" fn vault_directory_item_description(
    handle: *const c_void,
    request: *const c_char,
    request_len: usize,
) -> RevaultBuffer {
    response((|| {
        let request = unsafe { parse_request(request, request_len) }?;
        if handle.is_null() || !matches!(request.target.as_str(), "definition" | "field") {
            return Err(revault_lockbox_api::Error::InvalidInput(
                "vault descriptions require a definition or field target and live handle".into(),
            ));
        }
        let vault = unsafe { &*handle.cast::<VaultDirectoryHandle>() };
        if request.operation != "get" {
            match (request.target.as_str(), request.text.as_deref()) {
                ("definition", Some(text)) => {
                    vault.set_form_definition_description(&request.reference, text)?;
                }
                ("definition", None) => {
                    vault.clear_form_definition_description(&request.reference)?;
                }
                ("field", Some(text)) => {
                    vault.set_form_field_description(
                        &request.reference,
                        request.field.as_deref().unwrap(),
                        text,
                    )?;
                }
                ("field", None) => {
                    vault.clear_form_field_description(
                        &request.reference,
                        request.field.as_deref().unwrap(),
                    )?;
                }
                _ => unreachable!(),
            }
        }
        if request.target == "field" {
            vault.form_field_description(&request.reference, request.field.as_deref().unwrap())
        } else {
            Ok(Some(
                vault
                    .resolve_form_definition(&request.reference)?
                    .description,
            ))
        }
    })())
}
