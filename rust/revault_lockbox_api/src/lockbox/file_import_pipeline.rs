use std::time::Instant;

use zeroize::{Zeroize, Zeroizing};

use crate::compression::encode_compression_frame_with_level;
use crate::compression_frame_manifest::CompressionFrameSlice;
use crate::crypto::strong_checksum;
use crate::lockbox_path::LockboxPath;

#[derive(Clone, Copy)]
pub(super) struct CompressionFrameWrite<'a> {
    pub(super) path: &'a LockboxPath,
    pub(super) permissions: u32,
    pub(super) total_len: u64,
    pub(super) file_offset: u64,
    pub(super) data: &'a [u8],
}

pub(super) struct PreparedCompressionFrame {
    pub(super) compression: u8,
    pub(super) compression_frame_len: u64,
    pub(super) compressed_len: u64,
    pub(super) compression_frame_digest: [u8; 32],
    pub(super) slices: Vec<CompressionFrameSlice>,
    pub(super) stored: Zeroizing<Vec<u8>>,
    pub(super) prepare_nanos: u128,
}

pub(super) struct ParallelCompressionJob {
    pub(super) index: usize,
    pub(super) path: LockboxPath,
    pub(super) permissions: u32,
    pub(super) total_len: u64,
    pub(super) file_offset: u64,
    pub(super) data: Vec<u8>,
}

impl Drop for ParallelCompressionJob {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

pub(super) struct ParallelCompressionResult {
    pub(super) index: usize,
    pub(super) frame: PreparedCompressionFrame,
}

#[derive(Clone, Copy)]
pub(super) struct FileImportPipeline {
    zstd_level: i32,
    jobs: usize,
}

impl FileImportPipeline {
    pub(super) fn new(zstd_level: i32, jobs: usize) -> Self {
        Self {
            zstd_level,
            jobs: jobs.max(1),
        }
    }

    pub(super) fn prepare(self, frames: &[CompressionFrameWrite<'_>]) -> PreparedCompressionFrame {
        let prepare_start = Instant::now();
        let mut payload = Vec::new();
        let mut slices = Vec::with_capacity(frames.len());
        for frame in frames {
            let compression_frame_offset = payload.len() as u64;
            payload.extend_from_slice(frame.data);
            slices.push(CompressionFrameSlice {
                path: frame.path.clone(),
                permissions: frame.permissions,
                total_len: frame.total_len,
                file_offset: frame.file_offset,
                compression_frame_offset,
                len: frame.data.len() as u64,
            });
        }
        self.prepare_payload(payload, slices, prepare_start)
    }

    pub(super) fn prepare_batches(
        self,
        batches: &[Vec<CompressionFrameWrite<'_>>],
    ) -> Vec<PreparedCompressionFrame> {
        if self.jobs <= 1 || batches.len() <= 1 {
            return batches.iter().map(|batch| self.prepare(batch)).collect();
        }

        let worker_count = self.jobs.min(batches.len());
        let next_index = std::sync::atomic::AtomicUsize::new(0);
        let (result_tx, result_rx) =
            std::sync::mpsc::channel::<(usize, PreparedCompressionFrame)>();
        std::thread::scope(|scope| {
            for _ in 0..worker_count {
                let result_tx = result_tx.clone();
                let next_index = &next_index;
                scope.spawn(move || loop {
                    let index = next_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if index >= batches.len() {
                        return;
                    }
                    if result_tx
                        .send((index, self.prepare(&batches[index])))
                        .is_err()
                    {
                        return;
                    }
                });
            }
            drop(result_tx);
        });

        let mut prepared = Vec::with_capacity(batches.len());
        prepared.resize_with(batches.len(), || None);
        for (index, frame) in result_rx {
            prepared[index] = Some(frame);
        }
        prepared
            .into_iter()
            .enumerate()
            .map(|(index, frame)| frame.unwrap_or_else(|| self.prepare(&batches[index])))
            .collect()
    }

    pub(super) fn prepare_parallel_job(
        self,
        mut job: ParallelCompressionJob,
    ) -> ParallelCompressionResult {
        let prepare_start = Instant::now();
        let index = job.index;
        let slice = CompressionFrameSlice {
            path: job.path.clone(),
            permissions: job.permissions,
            total_len: job.total_len,
            file_offset: job.file_offset,
            compression_frame_offset: 0,
            len: job.data.len() as u64,
        };
        let frame = self.prepare_payload(std::mem::take(&mut job.data), vec![slice], prepare_start);
        ParallelCompressionResult { index, frame }
    }

    fn prepare_payload(
        self,
        mut payload: Vec<u8>,
        slices: Vec<CompressionFrameSlice>,
        prepare_start: Instant,
    ) -> PreparedCompressionFrame {
        let compression_frame_len = payload.len() as u64;
        let (compression, stored) = encode_compression_frame_with_level(&payload, self.zstd_level);
        payload.zeroize();
        let stored = Zeroizing::new(stored);
        let compression_frame_digest = strong_checksum(stored.as_slice());
        PreparedCompressionFrame {
            compression,
            compression_frame_len,
            compressed_len: stored.len() as u64,
            compression_frame_digest,
            slices,
            stored,
            prepare_nanos: prepare_start.elapsed().as_nanos(),
        }
    }
}
