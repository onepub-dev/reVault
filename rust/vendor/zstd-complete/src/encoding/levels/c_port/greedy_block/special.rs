use alloc::vec::Vec;

use crate::encoding::block_header::BlockHeader;

use super::GreedyEncodedBlock;
use crate::encoding::levels::c_port::{
    block_policy::should_skip_sequence_build, sequence_store::RepeatOffsets,
};

pub(in crate::encoding::levels::c_port) fn encode_special_block(
    block: &[u8],
    last_block: bool,
    repeat_offsets: RepeatOffsets,
    bytes: &mut crate::workspace::ReusableVec<u8>,
) -> Result<Option<GreedyEncodedBlock>, crate::workspace::CapacityError> {
    if block.is_empty() {
        write_raw_block(last_block, 0, block, bytes)?;
        return Ok(Some(GreedyEncodedBlock {
            bytes: core::mem::take(bytes),
            repeat_offsets,
            new_huffman_table: None,
        }));
    }

    if should_skip_sequence_build(block.len()) {
        write_raw_block(last_block, block.len() as u32, block, bytes)?;
        return Ok(Some(GreedyEncodedBlock {
            bytes: core::mem::take(bytes),
            repeat_offsets,
            new_huffman_table: None,
        }));
    }

    Ok(None)
}

fn write_raw_block(
    last_block: bool,
    block_size: u32,
    data: &[u8],
    output: &mut impl crate::workspace::Growable<u8>,
) -> Result<(), crate::workspace::CapacityError> {
    let header = BlockHeader {
        last_block,
        block_type: crate::blocks::block::BlockType::Raw,
        block_size,
    };
    header.serialize(output)?;
    output.extend_from_slice(data)?;
    Ok(())
}
