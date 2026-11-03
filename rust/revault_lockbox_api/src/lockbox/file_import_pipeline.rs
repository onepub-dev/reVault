use super::metric_timer::MetricTimer as Instant;
use std::borrow::Cow;
type FrameEncoder<'a> = dyn FnMut(&[u8]) -> crate::compression::FrameEncoding + 'a;

use crate::page_buffer::{zeroize_bytes, ZeroizingBytes};

use crate::compression::{encode_frame_outcome, FrameEncoding};
use crate::compression_frame_manifest::CompressionFrameSlice;
use crate::crypto::strong_checksum;
use crate::lockbox_path::LockboxPath;

enum FrameInput<'a> {
    Borrowed(&'a [u8]),
    Owned(ZeroizingBytes),
}

impl FrameInput<'_> {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Borrowed(bytes) => bytes,
            Self::Owned(bytes) => bytes.as_slice(),
        }
    }

    fn into_raw(self) -> ZeroizingBytes {
        match self {
            Self::Borrowed(bytes) => ZeroizingBytes::new(bytes.to_vec()),
            Self::Owned(bytes) => bytes,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct CompressionFrameWrite<'a> {
    pub(super) path: &'a LockboxPath,
    pub(super) permissions: u32,
    pub(super) total_len: u64,
    pub(super) file_offset: u64,
    pub(super) data: &'a [u8],
}

pub(super) enum PreparedFrameIntegrity {
    LegacyChecksum([u8; 32]),
}

impl PreparedFrameIntegrity {
    pub(super) fn legacy_checksum(&self) -> crate::Result<[u8; 32]> {
        match self {
            Self::LegacyChecksum(digest) => Ok(*digest),
        }
    }
}

pub(super) struct PreparedCompressionFrame {
    pub(super) compression: u8,
    pub(super) compression_frame_len: u64,
    pub(super) compressed_len: u64,
    pub(super) integrity: PreparedFrameIntegrity,
    pub(super) slices: Vec<CompressionFrameSlice>,
    pub(super) stored: ZeroizingBytes,
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
        zeroize_bytes(&mut self.data);
    }
}

pub(super) struct ParallelCompressionResult {
    pub(super) index: usize,
    pub(super) frame: PreparedCompressionFrame,
}

#[derive(Clone, Copy)]
pub(super) struct FileImportPipeline {
    zstd_level: i32,
    compression: Option<crate::Compression>,
    jobs: usize,
    sensitive: bool,
}

impl FileImportPipeline {
    pub(super) fn new(zstd_level: i32, jobs: usize) -> Self {
        Self {
            zstd_level,
            compression: None,
            jobs: jobs.max(1),
            sensitive: false,
        }
    }

    pub(super) fn with_compression(mut self, compression: Option<crate::Compression>) -> Self {
        self.compression = compression;
        self
    }

    pub(super) fn with_sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    pub(super) fn prepare(self, frames: &[CompressionFrameWrite<'_>]) -> PreparedCompressionFrame {
        self.prepare_with_encoder(frames, &mut |payload| self.encode(payload))
    }

    pub(super) fn encoder_frame(self, input: &[u8]) -> crate::compression::EncoderFrame {
        let probe = matches!(self.compression, Some(crate::Compression::Zstd { level }) if level.get() < 18);
        crate::compression::EncoderFrame::new_sensitive(input, probe, self.sensitive)
    }

    pub(super) fn with_encoder<R>(
        self,
        first: crate::compression::EncoderFrame,
        operation: impl FnOnce(&mut dyn FnMut(&[u8]) -> FrameEncoding) -> R,
    ) -> R {
        crate::compression::with_frame_encoder(
            first,
            self.compression,
            self.zstd_level,
            self.sensitive,
            operation,
        )
    }

    pub(super) fn prepare_with_encoder(
        self,
        frames: &[CompressionFrameWrite<'_>],
        encode: &mut FrameEncoder<'_>,
    ) -> PreparedCompressionFrame {
        let prepare_start = Instant::now();
        let single_frame = frames.len() == 1;
        let capacity = if self.sensitive && !single_frame {
            frames
                .iter()
                .try_fold(0usize, |sum, frame| sum.checked_add(frame.data.len()))
                .expect("bounded packed frame size")
        } else {
            0
        };
        let mut payload =
            crate::page_buffer::SensitiveBytes::new(Vec::with_capacity(capacity), self.sensitive);
        let mut slices = Vec::with_capacity(frames.len());
        for frame in frames {
            let compression_frame_offset = payload.len() as u64;
            if !single_frame {
                payload.extend_from_slice(frame.data);
            }
            slices.push(CompressionFrameSlice {
                path: frame.path.clone(),
                permissions: frame.permissions,
                total_len: frame.total_len,
                file_offset: frame.file_offset,
                compression_frame_offset,
                len: frame.data.len() as u64,
            });
        }
        // A standalone frame can encode its borrowed input directly. Packed
        // frames still assemble one contiguous owned payload.
        let payload = if single_frame {
            Cow::Borrowed(frames[0].data)
        } else {
            Cow::Owned(payload.into_vec())
        };
        self.prepare_payload_with_encoder(payload, slices, prepare_start, encode)
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

    pub(super) fn prepare_parallel_job_with_encoder(
        self,
        mut job: ParallelCompressionJob,
        encode: &mut FrameEncoder<'_>,
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
        let frame = self.prepare_payload_with_encoder(
            Cow::Owned(std::mem::take(&mut job.data)),
            vec![slice],
            prepare_start,
            encode,
        );
        ParallelCompressionResult { index, frame }
    }

    fn encode(self, payload: &[u8]) -> FrameEncoding {
        if self.sensitive {
            return crate::compression::encode_sensitive_outcome(
                payload,
                self.compression,
                self.zstd_level,
            );
        }
        match self.compression {
            Some(compression) => {
                crate::compression::encode_with_compression_outcome(payload, compression)
            }
            None => encode_frame_outcome(payload, self.zstd_level),
        }
    }

    fn prepare_payload_with_encoder(
        self,
        payload: Cow<'_, [u8]>,
        slices: Vec<CompressionFrameSlice>,
        prepare_start: Instant,
        encode: &mut FrameEncoder<'_>,
    ) -> PreparedCompressionFrame {
        // Own input under a wiping guard before calling an encoder that may unwind.
        let payload = match payload {
            Cow::Borrowed(bytes) => FrameInput::Borrowed(bytes),
            Cow::Owned(bytes) => FrameInput::Owned(ZeroizingBytes::new(bytes)),
        };
        let compression_frame_len = payload.as_slice().len() as u64;
        let (compression, stored) = match encode(payload.as_slice()) {
            FrameEncoding::Raw => (crate::compression::COMPRESSION_NONE, payload.into_raw()),
            FrameEncoding::Compressed(stored) => {
                drop(payload);
                (crate::compression::COMPRESSION_ZSTD, stored)
            }
        };
        let integrity = PreparedFrameIntegrity::LegacyChecksum(strong_checksum(stored.as_slice()));
        PreparedCompressionFrame {
            compression,
            compression_frame_len,
            compressed_len: stored.len() as u64,
            integrity,
            slices,
            stored,
            prepare_nanos: prepare_start.elapsed().as_nanos(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Compression;

    #[test]
    fn raw_parallel_frames_transfer_owned_allocation_including_spare_capacity() {
        for compression in [Compression::None, Compression::default()] {
            for len in [0, 31, 128 * 1024] {
                let mut input = Vec::with_capacity(len + 4096);
                let mut random = 0xa1b2_c3d4_u32;
                for _ in 0..len {
                    random ^= random << 13;
                    random ^= random >> 17;
                    random ^= random << 5;
                    input.push(random as u8);
                }
                let expected = input.clone();
                let pointer = input.as_ptr();
                let capacity = input.capacity();
                let pipeline = FileImportPipeline::new(3, 4).with_compression(Some(compression));
                let first = pipeline.encoder_frame(&input);
                let job = ParallelCompressionJob {
                    index: 7,
                    path: LockboxPath::new("/raw").unwrap(),
                    permissions: 0o640,
                    total_len: len as u64 + 19,
                    file_offset: 19,
                    data: input,
                };
                let result = pipeline.with_encoder(first, |encode| {
                    pipeline.prepare_parallel_job_with_encoder(job, encode)
                });
                assert_eq!(result.index, 7);
                let frame = result.frame;
                assert_eq!(frame.compression, crate::compression::COMPRESSION_NONE);
                assert_eq!(frame.stored.as_slice(), expected);
                assert_eq!(frame.stored.as_ptr(), pointer);
                assert_eq!(frame.stored.capacity(), capacity);
                assert_eq!(frame.compression_frame_len, len as u64);
                assert_eq!(frame.compressed_len, len as u64);
                assert_eq!(frame.slices[0].file_offset, 19);
                assert_eq!(
                    frame.integrity.legacy_checksum().unwrap(),
                    strong_checksum(&expected)
                );
            }
        }
    }

    #[test]
    fn raw_borrowed_and_packed_frames_keep_independent_content_and_slice_offsets() {
        let first = b"borrowed short frame";
        let second = b"packed final tail";
        let first_path = LockboxPath::new("/first").unwrap();
        let second_path = LockboxPath::new("/second").unwrap();
        let frames = [
            CompressionFrameWrite {
                path: &first_path,
                permissions: 0o640,
                total_len: first.len() as u64,
                file_offset: 0,
                data: first,
            },
            CompressionFrameWrite {
                path: &second_path,
                permissions: 0o600,
                total_len: second.len() as u64 + 9,
                file_offset: 9,
                data: second,
            },
        ];
        let pipeline = FileImportPipeline::new(3, 1).with_compression(Some(Compression::None));
        let single = pipeline.prepare(&frames[..1]);
        assert_eq!(single.stored.as_slice(), first);
        assert_ne!(single.stored.as_ptr(), first.as_ptr());
        let packed = pipeline.prepare(&frames);
        assert_eq!(packed.compression, crate::compression::COMPRESSION_NONE);
        assert_eq!(
            packed.stored.as_slice(),
            [first.as_slice(), second.as_slice()].concat()
        );
        assert_eq!(packed.slices.len(), 2);
        assert_eq!(packed.slices[0].compression_frame_offset, 0);
        assert_eq!(
            packed.slices[1].compression_frame_offset,
            first.len() as u64
        );
        assert_eq!(packed.slices[1].file_offset, 9);
        assert_eq!(packed.slices[1].permissions, 0o600);
        assert_eq!(first, b"borrowed short frame");
        assert_eq!(second, b"packed final tail");
    }
}
