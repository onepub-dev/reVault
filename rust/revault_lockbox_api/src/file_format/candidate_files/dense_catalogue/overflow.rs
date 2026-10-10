//! Typed filesystem records for the experimental shared tree. File fragments
//! are separate bounded records; no file descriptor grows with its payload.
use super::*;
use crate::file_format::authenticated_index::{Entry, EntryRef};
use crate::file_format::metadata_budget as memory;
use crate::file_format::publication_anchor::shared::tree::Tree;
use std::collections::BTreeMap;
const FILE_RECORD: u8 = 1;
const FRAGMENT_RECORD: u8 = 2;
const PACK_RECORD: u8 = 3;
const NODE_RECORD: u8 = 4;
const FORMAT_RECORD: u8 = 5;
const VARIABLE_RECORD: u8 = 6;
const MAX_NODES: usize = 100_000;

impl Catalogue {
    /// Account for materialized filesystem metadata and its temporary joins and
    /// coverage vectors, including worst-case geometric vector capacity slack.
    pub(super) fn tree_budget(&self) -> Result<memory::Budget> {
        let mut budget = memory::Budget::new(memory::TYPED_BYTES);
        for file in &self.files {
            budget.take(memory::FILE)?;
            budget.paths(file.path.len())?;
            for _ in &file.fragments {
                budget.take(memory::FRAGMENT)?;
            }
        }
        for _ in &self.packs {
            budget.take(memory::PACK)?;
        }
        for node in &self.nodes {
            budget.take(memory::FILE)?;
            budget.paths(node.path.len())?;
            budget.paths(node.target.as_ref().map_or(0, |target| target.len()))?;
        }
        for variable in &self.variables {
            budget.take(memory::VARIABLE)?;
            budget.paths(variable.name.as_str().len())?;
        }
        self.forms.admit_tree_budget(&mut budget)?;
        Ok(budget)
    }

    /// Fresh filesystem export only. Produce one private entry at a time in
    /// namespace/key order, retaining only a file-reference permutation for the
    /// object-ID ordered fragment namespace. No complete serialized row copy.
    pub(in crate::file_format::candidate_files) fn fresh_tree_records(
        &self,
    ) -> Result<impl Iterator<Item = Result<Entry>> + '_> {
        if !self.typed || !self.variables.is_empty() || !self.forms.is_empty() {
            return Err(Error::InvalidOperation(
                "fresh filesystem catalogue required".into(),
            ));
        }
        self.tree_budget()?;
        nodes::validate_bounded(&self.files, &self.nodes, MAX_NODES)?;
        let mut by_id: Vec<_> = self.files.iter().collect();
        by_id.sort_unstable_by_key(|file| file.info.id);
        if by_id
            .windows(2)
            .any(|pair| pair[0].info.id == pair[1].info.id)
        {
            return Err(Error::CorruptRecord);
        }
        let files = self.files.iter().map(|file| {
            let mut value = Zeroizing::new(file.permissions.to_le_bytes().to_vec());
            value.extend_from_slice(&file.info.encode());
            Entry::new(FILE_RECORD, &file.path, &value)
        });
        let fragments = by_id.into_iter().flat_map(move |file| {
            file.fragments.iter().map(move |fragment| {
                let mut name = file.info.id.to_vec();
                name.extend_from_slice(&fragment.descriptor.ordinal.to_be_bytes());
                let mut value = Zeroizing::new(fragment.descriptor.encode().to_vec());
                value.extend_from_slice(&self.packs[fragment.pack].extent.start.to_le_bytes());
                value.extend_from_slice(&(fragment.relative as u64).to_le_bytes());
                value.extend_from_slice(&fragment.digest);
                Entry::new(FRAGMENT_RECORD, &name, &value)
            })
        });
        let packs = self.packs.iter().map(|pack| {
            let mut value = pack.extent.len.to_le_bytes().to_vec();
            value.extend_from_slice(&pack.extent.digest);
            value.extend_from_slice(&pack.padding_digest);
            Entry::new(PACK_RECORD, &pack.extent.start.to_be_bytes(), &value)
        });
        let nodes = self.nodes.iter().map(|node| {
            let mut value = Zeroizing::new(vec![if node.target.is_some() { 2 } else { 3 }]);
            value.extend_from_slice(&node.permissions.to_le_bytes());
            if let Some(target) = &node.target {
                value.extend_from_slice(target);
            }
            Entry::new(NODE_RECORD, &node.path, &value)
        });
        Ok(files
            .chain(fragments)
            .chain(packs)
            .chain(nodes)
            .chain(std::iter::once_with(|| {
                Entry::new(FORMAT_RECORD, b"format", b"RV4FS001")
            })))
    }
    pub(in crate::file_format::candidate_files) fn dense_body_if_fits(
        &self,
        codec: &Codec,
        sealed: u64,
    ) -> Result<Option<ZeroizingBytes>> {
        if !self.variables.is_empty()
            || !self.forms.is_empty()
            || self.files.len() + self.nodes.len() > MAX_MODEL_FILES
        {
            return Ok(None);
        }
        match self.encode(codec, sealed) {
            Ok(body) => Ok(Some(body)),
            Err(Error::SecurityLimitExceeded(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }
    pub(in crate::file_format::candidate_files) fn set_tree_metadata(
        &mut self,
        entries: &[Metadata],
    ) -> Result<()> {
        let mut budget = self.tree_budget()?;
        let mut bytes = 0usize;
        for metadata in entries {
            bytes = bytes
                .checked_add(metadata.entry.path.as_str().len())
                .ok_or(Error::CorruptRecord)?;
            if metadata.entry.kind != crate::LockboxEntryKind::File {
                budget.take(memory::FILE)?;
                budget.paths(metadata.entry.path.as_str().len())?;
                budget.paths(
                    metadata
                        .target
                        .as_ref()
                        .map_or(0, |target| target.as_str().len()),
                )?;
                bytes = bytes
                    .checked_add(
                        5 + metadata
                            .target
                            .as_ref()
                            .map_or(0, |target| target.as_str().len()),
                    )
                    .ok_or(Error::CorruptRecord)?;
            }
            if bytes > super::super::MAX_PATH_BYTES {
                return Err(Error::SecurityLimitExceeded("typed tree path bytes".into()));
            }
        }
        self.set_metadata_bounded(entries, MAX_NODES)
    }
    pub(in crate::file_format::candidate_files) fn tree_records(&self) -> Result<Vec<Entry>> {
        // All mutation/export writers use the same admission as audited open,
        // before allocating complete rows or authorizing any publication.
        self.tree_budget()?;
        if !self.typed {
            return Err(Error::InvalidOperation(
                "typed filesystem metadata required".into(),
            ));
        }
        nodes::validate_bounded(&self.files, &self.nodes, MAX_NODES)?;
        validate_variables(&self.variables)?;
        validate_form_identities(&self.forms, &self.files, &self.variables)?;
        let form_records = self.forms.encode()?;
        let mut path_bytes = 0usize;
        for len in self
            .files
            .iter()
            .map(|f| f.path.len())
            .chain(
                self.nodes
                    .iter()
                    .map(|n| n.path.len() + 5 + n.target.as_ref().map_or(0, |v| v.len())),
            )
            .chain(self.variables.iter().map(|v| v.name.as_str().len()))
            .chain(form_records.iter().map(|row| row.key.len()))
        {
            path_bytes = path_bytes.checked_add(len).ok_or(Error::CorruptRecord)?;
        }
        if path_bytes > super::super::MAX_PATH_BYTES {
            return Err(Error::SecurityLimitExceeded("typed tree path bytes".into()));
        }
        let version = if !self.forms.is_empty() {
            b"RV4FS003"
        } else if self.variables.is_empty() {
            b"RV4FS001"
        } else {
            b"RV4FS002"
        };
        let mut out = vec![Entry::new(FORMAT_RECORD, b"format", version)?];
        for variable in &self.variables {
            out.push(Entry::new(
                VARIABLE_RECORD,
                variable.name.as_str().as_bytes(),
                &variable.layout.encode_metadata(),
            )?);
        }
        out.extend(form_records);
        for file in &self.files {
            let mut value = Zeroizing::new(file.permissions.to_le_bytes().to_vec());
            value.extend_from_slice(&file.info.encode());
            out.push(Entry::new(FILE_RECORD, &file.path, &value)?);
            for fragment in &file.fragments {
                let mut name = file.info.id.to_vec();
                name.extend_from_slice(&fragment.descriptor.ordinal.to_be_bytes());
                let mut value = Zeroizing::new(fragment.descriptor.encode().to_vec());
                value.extend_from_slice(&self.packs[fragment.pack].extent.start.to_le_bytes());
                value.extend_from_slice(&(fragment.relative as u64).to_le_bytes());
                value.extend_from_slice(&fragment.digest);
                out.push(Entry::new(FRAGMENT_RECORD, &name, &value)?);
            }
        }
        for pack in &self.packs {
            let mut value = pack.extent.len.to_le_bytes().to_vec();
            value.extend_from_slice(&pack.extent.digest);
            value.extend_from_slice(&pack.padding_digest);
            out.push(Entry::new(
                PACK_RECORD,
                &pack.extent.start.to_be_bytes(),
                &value,
            )?);
        }
        for node in &self.nodes {
            let mut value = Zeroizing::new(vec![if node.target.is_some() { 2 } else { 3 }]);
            value.extend_from_slice(&node.permissions.to_le_bytes());
            if let Some(target) = &node.target {
                value.extend_from_slice(target);
            }
            out.push(Entry::new(NODE_RECORD, &node.path, &value)?);
        }
        out.sort_by(|a, b| (a.namespace, a.key.as_slice()).cmp(&(b.namespace, b.key.as_slice())));
        Ok(out)
    }
    /// Decode staged records from one authenticated ownership traversal. Neither
    /// the tree nor the catalogue escapes until both graph and typed validation
    /// succeed; no partial records become observable on a late failure.
    pub(in crate::file_format::candidate_files) fn with_tree(
        codec: &Codec,
        visit: impl FnOnce(&mut dyn FnMut(Entry) -> Result<()>) -> Result<Tree>,
    ) -> Result<(Self, Tree)> {
        Self::with_tree_borrowed(codec, |decode| visit(&mut |entry| decode(entry.as_ref())))
    }

    pub(in crate::file_format::candidate_files) fn with_tree_borrowed(
        codec: &Codec,
        visit: impl FnOnce(&mut dyn FnMut(EntryRef<'_>) -> Result<()>) -> Result<Tree>,
    ) -> Result<(Self, Tree)> {
        let mut files = Vec::new();
        let mut nodes = Vec::new();
        let mut packs = Vec::new();
        let mut fragments = Vec::new();
        let mut path_bytes = 0usize;
        let mut format = 0;
        let mut variables = Vec::new();
        let mut form_rows = Vec::new();
        let mut budget = memory::Budget::new(memory::TYPED_BYTES);
        let tree = visit(&mut |entry| {
            match entry.namespace {
                FILE_RECORD => {
                    budget.take(memory::FILE)?;
                    budget.paths(entry.key.len())?;
                    if entry.value.len() != 76 || files.len() + nodes.len() >= MAX_NODES {
                        return Err(Error::CorruptRecord);
                    }
                    path_bytes = path_bytes
                        .checked_add(entry.key.len())
                        .ok_or(Error::CorruptRecord)?;
                    files.push(File {
                        path: Zeroizing::new(entry.key.to_vec()),
                        permissions: u32::from_le_bytes(entry.value[..4].try_into().unwrap()),
                        info: FileInfo::decode(&entry.value[4..], codec)?,
                        fragments: Vec::new(),
                    });
                }
                FRAGMENT_RECORD => {
                    budget.take(memory::FRAGMENT)?;
                    if entry.key.len() != 24 || entry.value.len() != 112 {
                        return Err(Error::CorruptRecord);
                    }
                    let descriptor = Descriptor::decode(&entry.value[..64])?;
                    let id: [u8; 16] = entry.key[..16].try_into().unwrap();
                    let ordinal = u64::from_be_bytes(entry.key[16..].try_into().unwrap());
                    if descriptor.object != id || descriptor.ordinal != ordinal {
                        return Err(Error::CorruptRecord);
                    }
                    let start = u64::from_le_bytes(entry.value[64..72].try_into().unwrap());
                    let relative = usize::try_from(u64::from_le_bytes(
                        entry.value[72..80].try_into().unwrap(),
                    ))
                    .map_err(|_| Error::CorruptRecord)?;
                    fragments.push((
                        descriptor,
                        start,
                        relative,
                        <[u8; 32]>::try_from(&entry.value[80..]).unwrap(),
                    ));
                }
                PACK_RECORD => {
                    budget.take(memory::PACK)?;
                    if entry.key.len() != 8 || entry.value.len() != 72 {
                        return Err(Error::CorruptRecord);
                    }
                    packs.push(Pack {
                        extent: Extent {
                            start: u64::from_be_bytes(entry.key.try_into().unwrap()),
                            len: u64::from_le_bytes(entry.value[..8].try_into().unwrap()),
                            digest: entry.value[8..40].try_into().unwrap(),
                        },
                        padding_digest: entry.value[40..].try_into().unwrap(),
                        used: 0,
                    });
                }
                NODE_RECORD => {
                    budget.take(memory::FILE)?;
                    budget.paths(entry.key.len())?;
                    budget.paths(entry.value.len().saturating_sub(5))?;
                    if entry.value.len() < 5 || files.len() + nodes.len() >= MAX_NODES {
                        return Err(Error::CorruptRecord);
                    }
                    path_bytes = path_bytes
                        .checked_add(entry.key.len() + entry.value.len())
                        .ok_or(Error::CorruptRecord)?;
                    let target = match entry.value[0] {
                        3 if entry.value.len() == 5 => None,
                        2 if entry.value.len() > 5
                            && entry.value.len() <= 5 + crate::constants::MAX_PATH_BYTES =>
                        {
                            Some(Zeroizing::new(entry.value[5..].to_vec()))
                        }
                        _ => return Err(Error::CorruptRecord),
                    };
                    nodes.push(nodes::Node {
                        path: Zeroizing::new(entry.key.to_vec()),
                        permissions: u32::from_le_bytes(entry.value[1..5].try_into().unwrap()),
                        target,
                    });
                }
                VARIABLE_RECORD => {
                    budget.take(memory::VARIABLE)?;
                    budget.paths(entry.key.len())?;
                    if variables.len() >= MAX_NODES || entry.value.len() > 824 {
                        return Err(Error::CorruptRecord);
                    }
                    path_bytes = path_bytes
                        .checked_add(entry.key.len())
                        .ok_or(Error::CorruptRecord)?;
                    variables.push(entry.to_owned()?);
                }
                7..=10 => {
                    budget.take(memory::FORM)?;
                    budget.paths(entry.key.len())?;
                    let entry = entry.to_owned()?;
                    forms::Forms::admit_row(&entry)?;
                    if form_rows.len() >= 4096 {
                        return Err(Error::CorruptRecord);
                    }
                    path_bytes = path_bytes
                        .checked_add(entry.key.len())
                        .ok_or(Error::CorruptRecord)?;
                    form_rows.push(entry);
                }
                FORMAT_RECORD
                    if format == 0
                        && entry.key == b"format"
                        && matches!(entry.value, b"RV4FS001" | b"RV4FS002" | b"RV4FS003") =>
                {
                    format = if entry.value == b"RV4FS003" {
                        3
                    } else if entry.value == b"RV4FS001" {
                        1
                    } else {
                        2
                    }
                }
                _ => return Err(Error::CorruptRecord),
            }
            if path_bytes > super::super::MAX_PATH_BYTES {
                return Err(Error::SecurityLimitExceeded("typed tree path bytes".into()));
            }
            Ok(())
        })?;
        let variables = variables
            .into_iter()
            .map(|entry| {
                let text = std::str::from_utf8(&entry.key).map_err(|_| Error::CorruptRecord)?;
                let name = crate::VariableName::new(text).map_err(|_| Error::CorruptRecord)?;
                if name.as_str() != text {
                    return Err(Error::CorruptRecord);
                }
                let layout = crate::file_format::secure_segments::Layout::decode_metadata(
                    &entry.value,
                    tree.anchor.mode,
                    tree.anchor.sealed_len,
                )?;
                Ok(Variable { name, layout })
            })
            .collect::<Result<Vec<_>>>()?;
        validate_variables(&variables)?;
        let forms = forms::Forms::decode(form_rows, tree.anchor.mode, tree.anchor.sealed_len)?;
        validate_form_identities(&forms, &files, &variables)?;
        let mut payloads: Vec<_> = packs.iter().map(|pack| pack.extent).collect();
        payloads.extend(
            variables
                .iter()
                .flat_map(|v| v.layout.extents.iter().copied()),
        );
        payloads.extend(
            forms
                .texts()
                .into_iter()
                .flat_map(|text| text.extents.iter().copied()),
        );
        payloads.sort_by_key(|extent| extent.start);
        if format
            != (if !forms.is_empty() {
                3
            } else if variables.is_empty() {
                1
            } else {
                2
            })
            || payloads != tree.graph.payloads()
        {
            return Err(Error::CorruptRecord);
        }
        // Fragment records arrive in authenticated (object, ordinal) order.
        // Stage them densely, then resolve each file once per record instead of
        // allocating/removing a tree-map entry for every fragment. Nothing is
        // exposed until counts, identities, extents and the full catalogue pass.
        let mut ids = BTreeMap::new();
        for (index, file) in files.iter().enumerate() {
            if ids.insert(file.info.id, index).is_some() {
                return Err(Error::CorruptRecord);
            }
        }
        for (descriptor, start, relative, digest) in fragments {
            let index = *ids.get(&descriptor.object).ok_or(Error::CorruptRecord)?;
            let file = &mut files[index];
            let ordinal = file.fragments.len() as u64;
            if descriptor.ordinal != ordinal
                || ordinal >= file.info.count()
                || descriptor.offset != ordinal * u64::from(file.info.unit)
                || u64::from(descriptor.logical_len)
                    != (file.info.len - descriptor.offset).min(u64::from(file.info.unit))
            {
                return Err(Error::CorruptRecord);
            }
            let pack = packs
                .binary_search_by_key(&start, |pack| pack.extent.start)
                .map_err(|_| Error::CorruptRecord)?;
            file.fragments.push(Fragment {
                descriptor,
                pack,
                relative,
                digest,
            });
        }
        if files
            .iter()
            .any(|file| file.fragments.len() as u64 != file.info.count())
        {
            return Err(Error::CorruptRecord);
        }
        nodes::validate_bounded(&files, &nodes, MAX_NODES)?;
        let mut result = Self {
            legacy: false,
            typed: true,
            variables,
            forms,
            files,
            nodes,
            packs,
            vacant: Vec::new(),
        };
        result.validate_fragments(codec, tree.anchor.sealed_len)?;
        Ok((result, tree))
    }
}

// Variables have their own namespace, independent of filesystem names. Reject
// duplicate identities and parent/child name collisions before exposing values.
fn validate_variables(variables: &[Variable]) -> Result<()> {
    if variables.len() > MAX_NODES {
        return Err(Error::CorruptRecord);
    }
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for variable in variables {
        let name = variable.name.as_str();
        if !names.insert(name) || !ids.insert(variable.layout.id) {
            return Err(Error::CorruptRecord);
        }
    }
    for name in &names {
        for (offset, _) in name.match_indices('/').skip(1) {
            if names.contains(&name[..offset]) {
                return Err(Error::CorruptRecord);
            }
        }
    }
    Ok(())
}

fn validate_form_identities(
    forms: &forms::Forms,
    files: &[File],
    variables: &[Variable],
) -> Result<()> {
    forms.validate()?;
    if forms.is_empty() {
        return Ok(());
    }
    let mut ids = BTreeSet::new();
    for id in files
        .iter()
        .map(|f| f.info.id)
        .chain(variables.iter().map(|v| v.layout.id))
        .chain(forms.records.iter().map(|r| r.id))
        .chain(forms.texts().into_iter().map(|t| t.id))
    {
        if !ids.insert(id) {
            return Err(Error::CorruptRecord);
        }
    }
    Ok(())
}
