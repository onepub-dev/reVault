use alloc::vec::Vec;

use crate::encoding::util::{likely_incompressible, likely_text};

const BEST_SPLIT_CHUNK_SIZE: usize = 8 * 1024;
const BEST_SPLIT_COMPRESSIBLE_RUN_MAX: usize = 128 * 1024;
const BEST_SPLIT_MIN_INCOMPRESSIBLE_RUN_CHUNKS: usize = 1;

pub(super) fn best_block_segment_lengths(data: &[u8]) -> Vec<usize> {
    if data.len() < BEST_SPLIT_CHUNK_SIZE * 2 {
        return alloc::vec![data.len()];
    }

    let chunk_count = data.len().div_ceil(BEST_SPLIT_CHUNK_SIZE);
    let mut incompressible_chunks = Vec::with_capacity(chunk_count);

    for chunk_idx in 0..chunk_count {
        let start = chunk_idx * BEST_SPLIT_CHUNK_SIZE;
        let end = (start + BEST_SPLIT_CHUNK_SIZE).min(data.len());
        let chunk = &data[start..end];
        incompressible_chunks.push(likely_incompressible(chunk) && !likely_text(chunk));
    }

    let mut split_chunks = alloc::vec![false; chunk_count];
    let mut any_split_chunk = false;
    let mut chunk_idx = 0usize;
    while chunk_idx < chunk_count {
        if !incompressible_chunks[chunk_idx] {
            chunk_idx += 1;
            continue;
        }

        let run_start = chunk_idx;
        while chunk_idx < chunk_count && incompressible_chunks[chunk_idx] {
            chunk_idx += 1;
        }
        let run_len = chunk_idx - run_start;
        if run_len >= BEST_SPLIT_MIN_INCOMPRESSIBLE_RUN_CHUNKS {
            for split_chunk in &mut split_chunks[run_start..chunk_idx] {
                *split_chunk = true;
            }
            any_split_chunk = true;
        }
    }

    if !any_split_chunk {
        return alloc::vec![data.len()];
    }

    if split_chunks.iter().all(|split_chunk| *split_chunk) && likely_incompressible(data) {
        return alloc::vec![data.len()];
    }

    let mut segments = Vec::with_capacity(chunk_count);
    let mut run_start_chunk = 0usize;

    for (chunk_idx, split_chunk) in split_chunks.iter().copied().enumerate() {
        if !split_chunk {
            continue;
        }

        push_compressible_run_segments(&mut segments, data, run_start_chunk, chunk_idx);
        let start = chunk_idx * BEST_SPLIT_CHUNK_SIZE;
        let end = (start + BEST_SPLIT_CHUNK_SIZE).min(data.len());
        segments.push(end - start);
        run_start_chunk = chunk_idx + 1;
    }

    push_compressible_run_segments(&mut segments, data, run_start_chunk, chunk_count);

    if segments.is_empty() {
        alloc::vec![data.len()]
    } else {
        segments
    }
}

fn push_compressible_run_segments(
    segments: &mut Vec<usize>,
    data: &[u8],
    start_chunk: usize,
    end_chunk: usize,
) {
    if start_chunk == end_chunk {
        return;
    }

    let mut start = start_chunk * BEST_SPLIT_CHUNK_SIZE;
    let end = (end_chunk * BEST_SPLIT_CHUNK_SIZE).min(data.len());
    while start < end {
        let next_end = (start + BEST_SPLIT_COMPRESSIBLE_RUN_MAX).min(end);
        segments.push(next_end - start);
        start = next_end;
    }
}
