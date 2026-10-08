//! Metadata-only record paths; payload context binds stable record identity.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[allow(clippy::too_many_arguments)]
pub(in crate::file_format::candidate_files) fn move_records(
    storage: &mut impl Storage,
    archive: LockboxId,
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
    key: Option<&[u8]>,
    moves: &[(LockboxPath, LockboxPath)],
) -> Result<bool> {
    let moves = moves
        .iter()
        .map(|(source, destination)| Ok((source.file_path()?, destination.file_path()?)))
        .collect::<Result<Vec<_>>>()?;
    let mut opened = TreeImage::open(
        allocation::compaction::View(&*storage),
        archive,
        mode,
        authority,
        key,
    )?;
    shared::prepare(&opened.image.anchor, authority, signer)?;
    opened.image.verify_all()?;
    let sources = moves
        .iter()
        .map(|(source, _)| source)
        .collect::<BTreeSet<_>>();
    if sources.len() != moves.len() {
        return Err(Error::InvalidInput(
            "a form record cannot be moved more than once".into(),
        ));
    }
    let existing = opened
        .image
        .catalogue
        .forms
        .records
        .iter()
        .map(|r| &r.path)
        .collect::<BTreeSet<_>>();
    let mut destinations = BTreeSet::new();
    for (source, destination) in &moves {
        if !existing.contains(source) {
            return Err(Error::NotFound(format!("form record {source}")));
        }
        if !destinations.insert(destination)
            || (source != destination
                && existing.contains(destination)
                && !sources.contains(destination))
        {
            return Err(Error::AlreadyExists(destination.to_string()));
        }
    }
    let mapping = moves
        .iter()
        .filter(|(s, d)| s != d)
        .cloned()
        .collect::<BTreeMap<_, _>>();
    if mapping.is_empty() {
        return Ok(false);
    }
    let base = shared::commitment(&opened.image.anchor)?;
    let mut catalogue = opened.image.catalogue;
    for destination in mapping.values() {
        create_parents(&mut catalogue, destination)?;
    }
    for record in &mut catalogue.forms.records {
        if let Some(destination) = mapping.get(&record.path) {
            record.path = destination.clone();
        }
    }
    let records = catalogue.tree_records()?;
    let plan = tree::PreparedSecurePayloadPlan {
        base,
        retired: Vec::new(),
        pages: PreparedSecurePages::new(0)?,
        payload: Box::new(|_| Err(Error::CorruptRecord)),
        rebind: Box::new(move |extents| {
            if !extents.is_empty() {
                return Err(Error::CorruptRecord);
            }
            catalogue.tree_records()
        }),
    };
    tree::rewrite_prepared_secure_payload_records(
        storage, archive, mode, authority, signer, key, records, plan,
    )
}
