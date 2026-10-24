use std::collections::BTreeMap;
use std::sync::Arc;

use crate::checked::{read_u16_le, read_u32_le};
use crate::constants::DEFAULT_METADATA_MAX_PAGE_BODY_BYTES;
use crate::file_format::page_commitment::PageCommitment;
use crate::page_tree::{AuthenticatedPageTreeChildren, PageTreeChild, PageTreeLayout};
use crate::secret_vec::{secure_read_access, SecureVec};
use crate::security::{validate_variable_name, validate_variable_value_ref};
use crate::{Error, Result, SecretString, VariableName, VariableSensitivity};

const VARIABLE_NODE_VERSION: u8 = 2;
const VARIABLE_LEAF: u8 = 1;
const VARIABLE_INTERNAL: u8 = 2;
const VARIABLE_NODE_PREFIX_BYTES: usize = 2;
const ENTRY_COUNT_BYTES: usize = 4;
const CHILD_COUNT_BYTES: usize = 4;
const VARIABLE_PLAIN_PREFIX: &str = ".plain/";
const VARIABLE_SECRET_PREFIX: &str = ".secret/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariableEntry {
    pub(crate) name: String,
    pub(crate) value: VariableValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VariableValue {
    Normal(String, Option<String>),
    Secret(Arc<SecretString>, Option<String>),
}

impl VariableValue {
    pub(crate) fn description(&self) -> &Option<String> {
        match self {
            Self::Normal(_, description) | Self::Secret(_, description) => description,
        }
    }
    pub(crate) fn set_description(&mut self, text: Option<String>) {
        match self {
            Self::Normal(_, description) | Self::Secret(_, description) => *description = text,
        }
    }

    pub(crate) fn sensitivity(&self) -> VariableSensitivity {
        match self {
            Self::Normal(_, _) => VariableSensitivity::Normal,
            Self::Secret(_, _) => VariableSensitivity::Secret,
        }
    }

    pub(crate) fn with_plaintext<R>(&self, f: impl FnOnce(&str) -> R) -> Result<R> {
        match self {
            Self::Normal(value, _) => Ok(f(value)),
            Self::Secret(value, _) => value.with_str(f).map_err(Into::into),
        }
    }
}

#[derive(Debug)]
pub(crate) enum VariableNode {
    Leaf(Vec<VariableEntry>),
    Internal(Vec<VariableChild>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariableChild {
    pub(crate) first_name: String,
    pub(crate) offset: u64,
    pub(crate) commitment: PageCommitment,
}

impl VariableChild {
    fn encoded_len(&self) -> usize {
        PageTreeChild {
            first_key: self.first_name.clone(),
            offset: self.offset,
        }
        .encoded_len()
            + 40
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VariableLeaf {
    pub(crate) offset: u64,
    pub(crate) commitment: PageCommitment,
    pub(crate) entries: Vec<VariableEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct VariableInternal {
    pub(crate) offset: u64,
    pub(crate) commitment: PageCommitment,
    pub(crate) children: Vec<VariableTreeNode>,
}

#[derive(Debug, Clone)]
pub(crate) enum VariableTreeNode {
    Leaf(VariableLeaf),
    Internal(VariableInternal),
}

impl VariableTreeNode {
    pub(crate) fn commitment(&self) -> PageCommitment {
        match self {
            Self::Leaf(leaf) => leaf.commitment,
            Self::Internal(internal) => internal.commitment,
        }
    }
    pub(crate) fn offset(&self) -> u64 {
        match self {
            Self::Leaf(leaf) => leaf.offset,
            Self::Internal(internal) => internal.offset,
        }
    }

    pub(crate) fn first_name(&self) -> &str {
        match self {
            Self::Leaf(leaf) => leaf
                .entries
                .first()
                .map(|entry| entry.name.as_str())
                .unwrap_or(""),
            Self::Internal(internal) => internal
                .children
                .first()
                .map(VariableTreeNode::first_name)
                .unwrap_or(""),
        }
    }

    pub(crate) fn collect_leaves(&self, leaves: &mut Vec<VariableLeaf>) {
        match self {
            Self::Leaf(leaf) => leaves.push(leaf.clone()),
            Self::Internal(internal) => {
                for child in &internal.children {
                    child.collect_leaves(leaves);
                }
            }
        }
    }
}

pub(crate) fn variable_entries_from_map(
    variables: &BTreeMap<VariableName, VariableValue>,
) -> Vec<VariableEntry> {
    let mut entries = variables
        .iter()
        .map(|(name, value)| VariableEntry {
            name: internal_variable_name(name.as_str(), value.sensitivity()),
            value: value.clone(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

pub(crate) fn encode_variable_leaf_secure(entries: &[VariableEntry]) -> Result<SecureVec> {
    let mut out = SecureVec::new();
    out.try_extend_from_slice(&[VARIABLE_NODE_VERSION, VARIABLE_LEAF])?;
    out.try_extend_from_slice(&(entries.len() as u32).to_le_bytes())?;
    for entry in entries {
        out.try_extend_from_slice(&(entry.name.len() as u16).to_le_bytes())?;
        out.try_extend_from_slice(entry.name.as_bytes())?;
        out.try_extend_from_slice(&[sensitivity_tag(entry.value.sensitivity())])?;
        put_description(&mut out, entry.value.description())?;
        match &entry.value {
            VariableValue::Normal(value, _) => {
                out.try_extend_from_slice(&(value.len() as u32).to_le_bytes())?;
                out.try_extend_from_slice(value.as_bytes())?;
            }
            VariableValue::Secret(value, _) => {
                let value_len = value.with_str(str::len)?;
                out.try_extend_from_slice(&(value_len as u32).to_le_bytes())?;
                value.append_to_secure_vec(&mut out)?;
            }
        }
    }
    if out.len() > DEFAULT_METADATA_MAX_PAGE_BODY_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "variable leaf exceeds maximum page size".to_string(),
        ));
    }
    Ok(out)
}

pub(crate) fn encode_variable_internal(children: &[VariableChild]) -> Result<Vec<u8>> {
    let routing_children = children
        .iter()
        .map(|child| PageTreeChild {
            first_key: child.first_name.clone(),
            offset: child.offset,
        })
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    out.push(VARIABLE_NODE_VERSION);
    out.push(VARIABLE_INTERNAL);
    out.extend_from_slice(
        &AuthenticatedPageTreeChildren::new(
            routing_children,
            children.iter().map(|child| child.commitment).collect(),
        )
        .encode()?,
    );
    if out.len() > DEFAULT_METADATA_MAX_PAGE_BODY_BYTES {
        return Err(Error::SecurityLimitExceeded(
            "variable internal node exceeds maximum page size".to_string(),
        ));
    }
    Ok(out)
}

pub(crate) fn variable_leaf_groups(entries: &[VariableEntry]) -> Result<Vec<&[VariableEntry]>> {
    let mut groups = Vec::new();
    let mut start = 0usize;
    while start < entries.len() {
        let class = internal_variable_class(&entries[start].name)?;
        let mut end = start + 1;
        while end < entries.len() && internal_variable_class(&entries[end].name)? == class {
            end += 1;
        }
        groups.extend(
            PageTreeLayout::new(
                VARIABLE_NODE_PREFIX_BYTES + ENTRY_COUNT_BYTES,
                "variable entry",
            )
            .groups(&entries[start..end], variable_entry_encoded_len)?,
        );
        start = end;
    }
    Ok(groups)
}

pub(crate) fn variable_child_groups(children: &[VariableChild]) -> Result<Vec<&[VariableChild]>> {
    PageTreeLayout::new(
        VARIABLE_NODE_PREFIX_BYTES + CHILD_COUNT_BYTES + 4,
        "variable child",
    )
    .groups(children, VariableChild::encoded_len)
}

pub(crate) fn decode_variable_node_secure(payload: &SecureVec) -> Result<VariableNode> {
    let parsed = secure_read_access(|access| {
        payload.with_bytes_in(access, |payload| {
            if payload.len() < VARIABLE_NODE_PREFIX_BYTES {
                return Err(Error::CorruptRecord);
            }
            match payload[0] {
                VARIABLE_NODE_VERSION => match payload[1] {
                    VARIABLE_LEAF => decode_variable_leaf_secure_metadata(payload),
                    VARIABLE_INTERNAL => Ok(SecureVariableNodeMetadata::Internal(
                        AuthenticatedPageTreeChildren::decode(&payload[2..], |key| {
                            validate_internal_variable_name(key)
                        })?
                        .into_pairs()
                        .map(|(child, commitment)| VariableChild {
                            first_name: child.first_key,
                            offset: child.offset,
                            commitment,
                        })
                        .collect(),
                    )),
                    _ => Err(Error::CorruptRecord),
                },
                _ => Err(Error::CorruptRecord),
            }
        })
    })??;
    match parsed {
        SecureVariableNodeMetadata::Internal(children) => Ok(VariableNode::Internal(children)),
        SecureVariableNodeMetadata::Leaf(parsed_entries) => {
            let mut entries = Vec::with_capacity(parsed_entries.len());
            for entry in parsed_entries {
                let value = match entry.value {
                    ParsedVariableValue::Normal(value) => {
                        VariableValue::Normal(value, entry.description.clone())
                    }
                    ParsedVariableValue::Secret { offset, len } => {
                        let bytes = payload.try_clone_range(offset, len)?;
                        VariableValue::Secret(
                            Arc::new(SecretString::from_secure_vec(bytes)),
                            entry.description.clone(),
                        )
                    }
                };
                entries.push(VariableEntry {
                    name: entry.name,
                    value,
                });
            }
            Ok(VariableNode::Leaf(entries))
        }
    }
}

enum SecureVariableNodeMetadata {
    Leaf(Vec<ParsedVariableEntry>),
    Internal(Vec<VariableChild>),
}

struct ParsedVariableEntry {
    name: String,
    description: Option<String>,
    value: ParsedVariableValue,
}

enum ParsedVariableValue {
    Normal(String),
    Secret { offset: usize, len: usize },
}

fn decode_variable_leaf_secure_metadata(payload: &[u8]) -> Result<SecureVariableNodeMetadata> {
    if payload.len() < VARIABLE_NODE_PREFIX_BYTES + ENTRY_COUNT_BYTES {
        return Err(Error::CorruptRecord);
    }
    let mut offset = VARIABLE_NODE_PREFIX_BYTES;
    let count = read_u32_le(&payload[offset..offset + 4])? as usize;
    offset += ENTRY_COUNT_BYTES;
    if count > (payload.len() - VARIABLE_NODE_PREFIX_BYTES - ENTRY_COUNT_BYTES) / 6 {
        return Err(Error::CorruptRecord);
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        if offset + 2 > payload.len() {
            return Err(Error::CorruptRecord);
        }
        let name_len = read_u16_le(&payload[offset..offset + 2])? as usize;
        offset += 2;
        if offset + name_len > payload.len() {
            return Err(Error::CorruptRecord);
        }
        let stored_name = std::str::from_utf8(&payload[offset..offset + name_len])
            .map_err(|_| Error::CorruptRecord)?;
        validate_internal_variable_name(stored_name)?;
        offset += name_len;
        if offset + 1 > payload.len() {
            return Err(Error::CorruptRecord);
        }
        let sensitivity = sensitivity_from_tag(payload[offset])?;
        offset += 1;
        if offset + 4 > payload.len() {
            return Err(Error::CorruptRecord);
        }
        let description = read_description(payload, &mut offset)?;
        let value_len = read_u32_le(
            payload
                .get(offset..offset + 4)
                .ok_or(Error::CorruptRecord)?,
        )? as usize;
        offset += 4;
        if offset + value_len > payload.len() {
            return Err(Error::CorruptRecord);
        }
        let value = std::str::from_utf8(&payload[offset..offset + value_len])
            .map_err(|_| Error::CorruptRecord)?;
        validate_variable_value_ref(value)?;
        let (name, sensitivity) = public_variable_name_and_sensitivity(stored_name, sensitivity)?;
        let value = match sensitivity {
            VariableSensitivity::Normal => ParsedVariableValue::Normal(value.to_string()),
            VariableSensitivity::Secret => ParsedVariableValue::Secret {
                offset,
                len: value_len,
            },
        };
        offset += value_len;
        entries.push(ParsedVariableEntry {
            name,
            value,
            description,
        });
    }
    if offset != payload.len() {
        return Err(Error::CorruptRecord);
    }
    for pair in entries.windows(2) {
        if pair[0].name >= pair[1].name {
            return Err(Error::CorruptRecord);
        }
    }
    Ok(SecureVariableNodeMetadata::Leaf(entries))
}

fn variable_entry_encoded_len(entry: &VariableEntry) -> usize {
    2 + entry.name.len()
        + 4
        + entry.value.description().as_ref().map_or(0, String::len)
        + 1
        + 4
        + entry
            .value
            .with_plaintext(str::len)
            .unwrap_or(DEFAULT_METADATA_MAX_PAGE_BODY_BYTES)
}

fn sensitivity_tag(sensitivity: VariableSensitivity) -> u8 {
    match sensitivity {
        VariableSensitivity::Normal => 0,
        VariableSensitivity::Secret => 1,
    }
}

fn sensitivity_from_tag(tag: u8) -> Result<VariableSensitivity> {
    match tag {
        0 => Ok(VariableSensitivity::Normal),
        1 => Ok(VariableSensitivity::Secret),
        _ => Err(Error::CorruptRecord),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableStorageClass {
    Plain,
    Secret,
    Legacy,
}

fn internal_variable_name(name: &str, sensitivity: VariableSensitivity) -> String {
    match sensitivity {
        VariableSensitivity::Normal => format!("{VARIABLE_PLAIN_PREFIX}{name}"),
        VariableSensitivity::Secret => format!("{VARIABLE_SECRET_PREFIX}{name}"),
    }
}

fn internal_variable_class(name: &str) -> Result<VariableStorageClass> {
    if name.starts_with(VARIABLE_PLAIN_PREFIX) {
        Ok(VariableStorageClass::Plain)
    } else if name.starts_with(VARIABLE_SECRET_PREFIX) {
        Ok(VariableStorageClass::Secret)
    } else {
        validate_variable_name(name)?;
        Ok(VariableStorageClass::Legacy)
    }
}

fn validate_internal_variable_name(name: &str) -> Result<()> {
    match internal_variable_class(name)? {
        VariableStorageClass::Plain => {
            validate_variable_name(&name[VARIABLE_PLAIN_PREFIX.len()..])?;
        }
        VariableStorageClass::Secret => {
            validate_variable_name(&name[VARIABLE_SECRET_PREFIX.len()..])?;
        }
        VariableStorageClass::Legacy => {}
    }
    Ok(())
}

fn public_variable_name_and_sensitivity(
    name: &str,
    encoded_sensitivity: VariableSensitivity,
) -> Result<(String, VariableSensitivity)> {
    if let Some(name) = name.strip_prefix(VARIABLE_PLAIN_PREFIX) {
        let name = VariableName::new(name)?;
        if encoded_sensitivity != VariableSensitivity::Normal {
            return Err(Error::CorruptRecord);
        }
        return Ok((name.as_str().to_string(), VariableSensitivity::Normal));
    }
    if let Some(name) = name.strip_prefix(VARIABLE_SECRET_PREFIX) {
        let name = VariableName::new(name)?;
        if encoded_sensitivity != VariableSensitivity::Secret {
            return Err(Error::CorruptRecord);
        }
        return Ok((name.as_str().to_string(), VariableSensitivity::Secret));
    }
    Ok((
        VariableName::new(name)?.as_str().to_string(),
        encoded_sensitivity,
    ))
}
pub(crate) fn put_description(out: &mut SecureVec, description: &Option<String>) -> Result<()> {
    match description {
        None => {
            out.try_extend_from_slice(&u32::MAX.to_le_bytes())?;
            Ok(())
        }
        Some(text) => {
            validate_variable_value_ref(text)?;
            out.try_extend_from_slice(&(text.len() as u32).to_le_bytes())?;
            out.try_extend_from_slice(text.as_bytes())?;
            Ok(())
        }
    }
}
pub(crate) fn read_description(payload: &[u8], offset: &mut usize) -> Result<Option<String>> {
    let len = read_u32_le(
        payload
            .get(*offset..*offset + 4)
            .ok_or(Error::CorruptRecord)?,
    )?;
    *offset += 4;
    if len == u32::MAX {
        return Ok(None);
    }
    let end = offset
        .checked_add(len as usize)
        .ok_or(Error::CorruptRecord)?;
    let text = std::str::from_utf8(payload.get(*offset..end).ok_or(Error::CorruptRecord)?)
        .map_err(|_| Error::CorruptRecord)?;
    validate_variable_value_ref(text)?;
    *offset = end;
    Ok(Some(text.to_owned()))
}
