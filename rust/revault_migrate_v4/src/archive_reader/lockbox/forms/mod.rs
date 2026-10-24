use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use super::Lockbox;
use crate::archive_reader::form::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue,
};
use crate::archive_reader::form_btree::{
    decode_form_node_secure, definition_key, encode_form_internal, encode_form_leaf_secure,
    form_child_groups, form_entries_from_maps, form_leaf_groups, record_key, FormChild, FormEntry,
    FormEntryValue, FormInternal, FormLeaf, FormNode, FormTreeNode,
};
use crate::archive_reader::free_slot::FreeSlot;
use crate::archive_reader::incremental_btree::{IncrementalBTree, LeafRewrite};
use crate::archive_reader::page::{PageObject, PageObjectKind};
use crate::archive_reader::page_cache::SecurePageAppend;
use crate::archive_reader::secret_vec::SecureVec;
use crate::archive_reader::{
    crypto::derive_page_content_key, Error, LockboxPath, Result, SecretString,
};
use zeroize::Zeroize;

type FormTreeDecodeResult = (
    BTreeMap<String, FormDefinition>,
    BTreeMap<LockboxPath, FormRecord>,
    FormTreeNode,
    Vec<FormLeaf>,
);

mod definitions;
mod fields;
mod persistence;
mod records;
