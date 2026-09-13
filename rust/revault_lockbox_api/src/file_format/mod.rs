pub(crate) mod commit_auth;
pub(crate) mod commit_root;
pub(crate) mod current_header;
pub(crate) mod header_v2;
// Common native-layout implementation. The non-default experimental feature
// enables real public-path tests and performance gates before release activation.
#[cfg(any(test, feature = "native-block-layout"))]
pub(crate) mod indexed_frame;
pub(crate) mod key_directory;
pub(crate) mod page;
pub(crate) mod page_buffer;
pub(crate) mod page_inspection;
pub(crate) mod page_scanner;
pub(crate) mod payload;
pub(crate) mod redaction_manifest;

pub(crate) use crate::file_format::current_header::{read_header, write_header};
pub(crate) use crate::file_format::payload::{
    decode_compression_frame_segment_payload_view, decode_symlink_payload, encode_symlink_payload,
};
pub(crate) use crate::index::decode_index_records;
pub(crate) use crate::toc_btree::{
    decode_toc_node, encode_toc_internal, encode_toc_leaf, toc_child_groups, toc_leaf_groups,
    TocChild, TocInternal, TocLeaf, TocNode, TocTreeNode,
};
#[cfg(test)]
pub(crate) use payload::encode_compression_frame_segment_payload;
