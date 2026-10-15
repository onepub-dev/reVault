use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use super::Lockbox;
use crate::vault_archive_reader::form::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue,
};
use crate::vault_archive_reader::form_btree::{
    decode_form_node_secure, definition_key, encode_form_internal, encode_form_leaf_secure,
    form_child_groups, form_entries_from_maps, form_leaf_groups, record_key, FormChild, FormEntry,
    FormEntryValue, FormInternal, FormLeaf, FormNode, FormTreeNode,
};
use crate::vault_archive_reader::free_slot::FreeSlot;
use crate::vault_archive_reader::incremental_btree::{IncrementalBTree, LeafRewrite};
use crate::vault_archive_reader::page::{page_size_for_objects, PageObject, PageObjectKind};
use crate::vault_archive_reader::page_cache::SecurePageAppend;
use crate::vault_archive_reader::secret_vec::SecureVec;
use crate::vault_archive_reader::{
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
