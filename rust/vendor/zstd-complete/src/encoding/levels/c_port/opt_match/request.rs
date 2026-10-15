use crate::encoding::levels::c_port::{
    hash_chain_match::AttachedDictionarySearch, params::CompressionParameters,
};

use super::OptMatchBounds;

#[derive(Clone, Copy)]
pub(in crate::encoding::levels::c_port) struct BtMatchRequest<'a> {
    pub(in crate::encoding::levels::c_port) src: &'a [u8],
    pub(in crate::encoding::levels::c_port) ip: usize,
    pub(in crate::encoding::levels::c_port) block_end: usize,
    pub(in crate::encoding::levels::c_port) rep: [u32; 3],
    pub(in crate::encoding::levels::c_port) ll0: bool,
    pub(in crate::encoding::levels::c_port) length_to_beat: u32,
    pub(in crate::encoding::levels::c_port) params: CompressionParameters,
    pub(in crate::encoding::levels::c_port) bounds: OptMatchBounds,
    pub(in crate::encoding::levels::c_port) attached_dictionary:
        Option<AttachedDictionarySearch<'a>>,
}
