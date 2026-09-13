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

pub(super) enum PreparedFrameIntegrity {
    LegacyChecksum([u8; 32]),
    NativeBlocksPending,
}

impl PreparedFrameIntegrity {
    pub(super) fn legacy_checksum(&self) -> crate::Result<[u8; 32]> {
        match self {
            Self::LegacyChecksum(digest) => Ok(*digest),
            Self::NativeBlocksPending => Err(crate::Error::CorruptRecord),
        }
    }
}

pub(super) struct PreparedCompressionFrame {
    pub(super) compression: u8,
    pub(super) compression_frame_len: u64,
    pub(super) compressed_len: u64,
    pub(super) integrity: PreparedFrameIntegrity,
    pub(super) slices: Vec<CompressionFrameSlice>,
    pub(super) stored: Zeroizing<Vec<u8>>,
    pub(super) prepare_nanos: u128,
}

#[cfg(any(test, feature = "native-block-layout"))]
impl PreparedCompressionFrame {
    /// Seal the pipeline's already-compressed bytes for the common page layout.
    /// No data decompression or second compression pass is permitted here.
    pub(super) fn encode_native_page(
        &self,
        identity: crate::file_format::indexed_frame::block_page::PageIdentity,
        frame_id: u64,
        key: &[u8],
    ) -> crate::Result<crate::file_format::indexed_frame::block_page::EncodedBlockPage> {
        if self.compressed_len != self.stored.len() as u64 {
            return Err(crate::Error::CorruptRecord);
        }
        crate::file_format::indexed_frame::block_page::EncodedBlockPage::prepare(
            identity,
            frame_id,
            self.compression,
            self.compression_frame_len,
            &self.stored,
            self.slices.clone(),
            key,
        )
    }
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
    compression: Option<crate::Compression>,
    jobs: usize,
    native_blocks: bool,
}

impl FileImportPipeline {
    pub(super) fn new(zstd_level: i32, jobs: usize) -> Self {
        Self {
            zstd_level,
            compression: None,
            jobs: jobs.max(1),
            native_blocks: false,
        }
    }

    pub(super) fn with_compression(mut self, compression: Option<crate::Compression>) -> Self {
        self.compression = compression;
        self
    }

    pub(super) fn with_native_blocks(mut self, native_blocks: bool) -> Self {
        self.native_blocks = native_blocks;
        self
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
        let (compression, stored) = match self.compression {
            Some(compression) => crate::compression::encode_with_compression(&payload, compression),
            None => encode_compression_frame_with_level(&payload, self.zstd_level),
        };
        payload.zeroize();
        let stored = Zeroizing::new(stored);
        let integrity = if self.native_blocks {
            PreparedFrameIntegrity::NativeBlocksPending
        } else {
            PreparedFrameIntegrity::LegacyChecksum(strong_checksum(stored.as_slice()))
        };
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
    use crate::creation_options::FormatMode;
    use crate::file_format::indexed_frame::block_page::{PageIdentity, Reader};
    use crate::{
        Compression, EncryptionMode, LockboxFormatOptions, LockboxId, SigningMode, SizePadding,
    };

    #[test]
    fn prepared_native_pages_reject_inconsistent_lengths_codecs_and_identity() {
        let path = LockboxPath::new("/file").unwrap();
        let mode = FormatMode::new(LockboxFormatOptions {
            encryption: EncryptionMode::ChaCha20Poly1305,
            signing: SigningMode::Owner,
            compression: Compression::default(),
            size_padding: SizePadding::None,
        });
        let identity = PageIdentity {
            archive: LockboxId::from_bytes([71; 16]),
            mode,
            page_id: 41,
            sequence: 43,
        };
        let mut frame = FileImportPipeline::new(3, 1)
            .with_compression(Some(Compression::default()))
            .prepare(&[CompressionFrameWrite {
                path: &path,
                permissions: 0o640,
                total_len: 32768,
                file_offset: 0,
                data: &[37; 32768],
            }]);
        assert!(frame.encode_native_page(identity, 31, &[53; 32]).is_ok());
        assert!(frame.encode_native_page(identity, 31, &[53; 31]).is_ok());
        assert!(frame.encode_native_page(identity, 0, &[53; 32]).is_err());
        assert!(frame
            .encode_native_page(
                PageIdentity {
                    page_id: 0,
                    ..identity
                },
                31,
                &[53; 32]
            )
            .is_err());
        assert!(frame
            .encode_native_page(
                PageIdentity {
                    mode: FormatMode(0),
                    ..identity
                },
                31,
                &[53; 32]
            )
            .is_err());
        frame.compressed_len += 1;
        assert!(frame.encode_native_page(identity, 31, &[53; 32]).is_err());
        frame.compressed_len -= 1;
        let codec = frame.compression;
        frame.compression = 255;
        assert!(frame.encode_native_page(identity, 31, &[53; 32]).is_err());
        frame.compression = codec;
        frame.compression_frame_len = 4 * 1024 * 1024 + 1;
        assert!(frame.encode_native_page(identity, 31, &[53; 32]).is_err());
    }

    #[test]
    fn prepared_native_pages_preserve_serial_and_parallel_pipeline_output_in_all_modes() {
        let mut state = 0xabcdef0123456789u64;
        let random = (0..65539)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect::<Vec<_>>();
        let inputs = [Vec::new(), vec![37; 65539], random];
        let paths = [
            LockboxPath::new("/first").unwrap(),
            LockboxPath::new("/second").unwrap(),
        ];
        let key = [53; 32];
        for encryption in [EncryptionMode::None, EncryptionMode::ChaCha20Poly1305] {
            for signing in [SigningMode::None, SigningMode::Owner] {
                for size_padding in [SizePadding::Default, SizePadding::None] {
                    for compression in [Compression::None, Compression::default()] {
                        let mode = FormatMode::new(LockboxFormatOptions {
                            encryption,
                            signing,
                            size_padding,
                            compression,
                        });
                        let identity = PageIdentity {
                            archive: LockboxId::from_bytes([71; 16]),
                            mode,
                            page_id: 41,
                            sequence: 43,
                        };
                        for jobs in [1, 3] {
                            let pipeline = FileImportPipeline::new(3, jobs)
                                .with_compression(Some(compression))
                                .with_native_blocks(true);
                            let batches = inputs
                                .iter()
                                .map(|input| {
                                    let split = input.len() / 2;
                                    [&input[..split], &input[split..]]
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, data)| CompressionFrameWrite {
                                            path: &paths[i],
                                            permissions: 0o640,
                                            total_len: data.len() as u64,
                                            file_offset: 0,
                                            data,
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .collect::<Vec<_>>();
                            let prepared = pipeline.prepare_batches(&batches);
                            let legacy =
                                pipeline.with_native_blocks(false).prepare_batches(&batches);
                            for (native, legacy) in prepared.iter().zip(&legacy) {
                                assert!(matches!(
                                    &native.integrity,
                                    PreparedFrameIntegrity::NativeBlocksPending
                                ));
                                assert!(native.integrity.legacy_checksum().is_err());
                                assert_eq!(
                                    legacy.integrity.legacy_checksum().unwrap(),
                                    strong_checksum(&legacy.stored)
                                );
                                assert_eq!(
                                    &*native.stored, &*legacy.stored,
                                    "layout selection must not change codec bytes"
                                );
                            }
                            assert_eq!(
                                prepared[2].compression,
                                crate::compression::COMPRESSION_NONE,
                                "incompressible input must retain raw fallback"
                            );
                            if compression != Compression::None {
                                assert_eq!(
                                    prepared[1].compression,
                                    crate::compression::COMPRESSION_ZSTD
                                );
                            }
                            for (frame, input) in prepared.iter().zip(&inputs) {
                                // Independent test oracle, not a second writer pass.
                                let (codec, encoded) =
                                    crate::compression::encode_with_compression(input, compression);
                                assert_eq!(frame.compression, codec);
                                assert_eq!(&*frame.stored, &encoded);
                                let page = frame.encode_native_page(identity, 31, &key).unwrap();
                                let descriptor = page.descriptor();
                                assert_eq!(descriptor.compression, codec);
                                assert_eq!(descriptor.logical_len, input.len() as u64);
                                assert_eq!(descriptor.stored_len, encoded.len() as u64);
                                let page_len = page.bytes().len();
                                let storage =
                                    crate::storage::StorageBackend::memory(page.bytes().to_vec());
                                let reader =
                                    Reader::open(&storage, 0, identity, descriptor, page_len, &key)
                                        .unwrap();
                                assert_eq!(reader.read(0..input.len() as u64).unwrap(), *input);
                                let retry = frame.encode_native_page(identity, 31, &key).unwrap();
                                assert_ne!(
                                    retry.descriptor().salt,
                                    descriptor.salt,
                                    "retry must derive a fresh block key"
                                );
                                assert_eq!(
                                    &*frame.stored, &encoded,
                                    "protection must not mutate prepared codec bytes"
                                );
                            }
                            // Streaming worker jobs feed the same prepared-page boundary.
                            let result = pipeline.prepare_parallel_job(ParallelCompressionJob {
                                index: 7,
                                path: paths[0].clone(),
                                permissions: 0o640,
                                total_len: inputs[1].len() as u64,
                                file_offset: 0,
                                data: inputs[1].clone(),
                            });
                            assert_eq!(result.index, 7);
                            assert!(matches!(
                                &result.frame.integrity,
                                PreparedFrameIntegrity::NativeBlocksPending
                            ));
                            let page = result.frame.encode_native_page(identity, 31, &key).unwrap();
                            let descriptor = page.descriptor();
                            let page_len = page.bytes().len();
                            let storage =
                                crate::storage::StorageBackend::memory(page.bytes().to_vec());
                            assert_eq!(
                                Reader::open(&storage, 0, identity, descriptor, page_len, &key)
                                    .unwrap()
                                    .read(0..inputs[1].len() as u64)
                                    .unwrap(),
                                inputs[1]
                            );
                        }
                    }
                }
            }
        }
    }
}
