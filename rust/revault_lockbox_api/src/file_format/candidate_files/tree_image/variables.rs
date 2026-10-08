//! Bounded selected-variable adapter. Value bytes never enter index entries;
//! selected metadata and the physical graph jointly authorize segment reads.
use super::*;
use crate::file_format::secure_segments;
use crate::secret_vec::SecureVec;
use crate::{SecretString, VariableName, VariableSensitivity};
use std::sync::Arc;

impl<S: Storage> TreeImage<S> {
    pub fn get_variable(&self, name: &VariableName) -> Result<Option<String>> {
        let Some(variable) = self
            .image
            .catalogue
            .variables
            .iter()
            .find(|v| &v.name == name)
        else {
            return Ok(None);
        };
        if variable.layout.sensitivity == VariableSensitivity::Secret {
            return Err(Error::InvalidOperation(
                "variable is secret; use secret access".into(),
            ));
        }
        let bytes = self.read_variable(variable)?;
        bytes.with_bytes(|bytes| {
            Ok(Some(
                std::str::from_utf8(bytes)
                    .map_err(|_| Error::CorruptRecord)?
                    .to_owned(),
            ))
        })?
    }
    pub fn with_secret_variable<R>(
        &self,
        name: &VariableName,
        f: impl FnOnce(&SecretString) -> R,
    ) -> Result<Option<R>> {
        let Some(variable) = self
            .image
            .catalogue
            .variables
            .iter()
            .find(|v| &v.name == name)
        else {
            return Ok(None);
        };
        if variable.layout.sensitivity != VariableSensitivity::Secret {
            return Err(Error::InvalidOperation("variable is not secret".into()));
        }
        let value = Arc::new(SecretString::from_secure_vec(self.read_variable(variable)?));
        Ok(Some(f(&value)))
    }
    fn read_variable(
        &self,
        variable: &super::super::dense_catalogue::Variable,
    ) -> Result<SecureVec> {
        variable.layout.read(
            &self.image.storage,
            self.image.anchor.archive,
            self.image.value_key.as_ref().ok_or(Error::CorruptRecord)?,
            self.image.anchor.sealed_len,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn set_variable(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    name: &VariableName,
    value: &str,
) -> Result<bool> {
    crate::security::validate_variable_value_ref(value)?;
    let bytes = SecureVec::try_from_slice(value.as_bytes())?;
    change(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        name,
        Some((
            VariableSensitivity::Normal,
            secure_segments::Source::Bytes(&bytes),
            value.len(),
        )),
    )
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn set_secret_variable(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    name: &VariableName,
    value: &SecretString,
) -> Result<bool> {
    let length = value.with_str(|text| -> Result<usize> {
        crate::security::validate_variable_value_ref(text)?;
        Ok(text.len())
    })??;
    change(
        storage,
        archive,
        mode,
        authority,
        signer,
        key,
        name,
        Some((
            VariableSensitivity::Secret,
            secure_segments::Source::Secret(value),
            length,
        )),
    )
}
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn delete_variable(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    name: &VariableName,
) -> Result<bool> {
    change(storage, archive, mode, authority, signer, key, name, None)
}

/// Atomic name changes over selected metadata. Guarded content authentication
/// remains mandatory, but value identity, revision and segment placement do not
/// change. Callers hold exclusive writer access throughout this transaction.
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn move_variables(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    moves: &[(VariableName, VariableName)],
) -> Result<bool> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    let sources: BTreeSet<_> = moves.iter().map(|(source, _)| source.clone()).collect();
    if sources.len() != moves.len() {
        return Err(Error::InvalidInput(
            "a variable cannot be moved more than once".into(),
        ));
    }
    let existing: BTreeSet<_> = opened
        .image
        .catalogue
        .variables
        .iter()
        .map(|v| v.name.clone())
        .collect();
    let mut destinations = BTreeSet::new();
    for (source, destination) in moves {
        if !existing.contains(source) {
            return Err(Error::NotFound(format!("variable {source}")));
        }
        if !destinations.insert(destination.clone())
            || (source != destination
                && existing.contains(destination)
                && !sources.contains(destination))
        {
            return Err(Error::AlreadyExists(destination.to_string()));
        }
    }
    let final_names: BTreeSet<_> = existing
        .iter()
        .filter(|name| !sources.contains(*name))
        .chain(destinations.iter())
        .map(|name| name.as_str())
        .collect();
    // Check every canonical ancestor, as the persisted typed validator does.
    for name in &final_names {
        for (offset, _) in name.match_indices('/').skip(1) {
            if final_names.contains(&name[..offset]) {
                return Err(Error::AlreadyExists(format!(
                    "variable namespace conflict at {name}"
                )));
            }
        }
    }
    if moves
        .iter()
        .all(|(source, destination)| source == destination)
    {
        return Ok(false);
    }
    let mapping: BTreeMap<_, _> = moves.iter().cloned().collect();
    for variable in &mut opened.image.catalogue.variables {
        if let Some(destination) = mapping.get(&variable.name) {
            variable.name = destination.clone();
        }
    }
    let records = opened.image.catalogue.tree_records()?;
    drop(opened);
    tree::rewrite_records(storage, archive, mode, authority, signer, key, records)
}

#[allow(clippy::too_many_arguments)]
fn change(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    name: &VariableName,
    value: Option<(VariableSensitivity, secure_segments::Source<'_>, usize)>,
) -> Result<bool> {
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    let anchor = opened.image.anchor.clone();
    shared::prepare(&anchor, authority, signer)?;
    // Even a no-change request cannot attest to a damaged selected generation.
    opened.image.verify_all()?;
    let old = opened
        .image
        .catalogue
        .variables
        .iter()
        .position(|v| &v.name == name);
    let mut retired = Vec::new();
    let (id, revision) = if let Some(index) = old {
        let existing = &opened.image.catalogue.variables[index];
        if let Some((sensitivity, bytes, length)) = &value {
            if existing.layout.sensitivity == VariableSensitivity::Secret
                && *sensitivity == VariableSensitivity::Normal
            {
                return Err(Error::InvalidOperation(
                    "variable is secret; delete and recreate to change sensitivity".into(),
                ));
            }
            if existing.layout.sensitivity == *sensitivity && existing.layout.length == *length {
                let prior = opened.read_variable(existing)?;
                if prior.with_bytes(|prior| bytes.with_bytes(|bytes| prior == bytes))?? {
                    return Ok(false);
                }
            }
        }
        retired = existing.layout.extents.clone();
        (
            existing.layout.id,
            if value.is_some() {
                existing
                    .layout
                    .revision
                    .checked_add(1)
                    .ok_or(Error::CorruptRecord)?
            } else {
                existing.layout.revision
            },
        )
    } else {
        if value.is_none() {
            return Ok(false);
        }
        let mut id = [0; 16];
        getrandom::fill(&mut id).map_err(|e| Error::Io(e.to_string()))?;
        (id, 1)
    };
    if value.is_some() {
        for existing in &opened.image.catalogue.variables {
            let parent =
                |a: &str, b: &str| b.strip_prefix(a).is_some_and(|tail| tail.starts_with('/'));
            if parent(existing.name.as_str(), name.as_str())
                || parent(name.as_str(), existing.name.as_str())
            {
                return Err(Error::AlreadyExists(name.as_str().into()));
            }
        }
    }
    let mut pages = Vec::new();
    let encoded = if let Some((sensitivity, bytes, _)) = value {
        let content_key = match opened.image.value_key.take() {
            Some(content_key) => content_key,
            None => super::super::dense_image::value_key(mode, key)?,
        };
        let encoded = secure_segments::encode_source(
            bytes,
            archive,
            mode,
            &content_key,
            id,
            revision,
            sensitivity,
        )?;
        pages = encoded.pages;
        Some(encoded.layout)
    } else {
        None
    };
    let mut catalogue = opened.image.catalogue;
    if let Some(index) = old {
        catalogue.variables.remove(index);
    }
    let updated = encoded.map(|layout| {
        catalogue
            .variables
            .push(super::super::dense_catalogue::Variable {
                name: name.clone(),
                layout,
            });
        catalogue.variables.len() - 1
    });
    let records = catalogue.tree_records()?;
    let plan = tree::SecurePayloadPlan {
        base: shared::commitment(&anchor)?,
        retired,
        bytes: pages,
        rebind: Box::new(move |extents| {
            if let Some(index) = updated {
                catalogue.variables[index].layout.rebind(extents)?;
            } else if !extents.is_empty() {
                return Err(Error::CorruptRecord);
            }
            catalogue.tree_records()
        }),
    };
    tree::rewrite_secure_payload_records(
        storage, archive, mode, authority, signer, key, records, plan,
    )
}

/// Salvage only the currently selected variable membership. A stable source is
/// required; caller stages callbacks until success. Damaged values are named,
/// never replaced with older revisions or values from unselected metadata.
pub(in crate::file_format::candidate_files) fn salvage_variables(
    storage: &impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    key: Option<&[u8]>,
    mut sink: impl FnMut(&VariableName, VariableSensitivity, &SecretString) -> Result<()>,
) -> Result<Vec<VariableName>> {
    let codec = Codec::shared_packed(archive, mode, key)?;
    let (catalogue, tree) = Catalogue::with_tree(&codec, |visitor| {
        Tree::salvage_visit(storage, archive, mode, authority, key, visitor)
    })?;
    let value_key = super::super::dense_image::value_key(mode, key)?;
    let mut unavailable = Vec::new();
    for variable in catalogue.variables {
        match variable
            .layout
            .read(storage, archive, &value_key, tree.anchor.sealed_len)
        {
            Ok(bytes) => {
                let value = Arc::new(SecretString::from_secure_vec(bytes));
                sink(&variable.name, variable.layout.sensitivity, &value)?;
            }
            Err(Error::CorruptRecord | Error::CorruptHeader | Error::Truncated) => {
                unavailable.push(variable.name)
            }
            Err(error) => return Err(error),
        }
    }
    Ok(unavailable)
}
