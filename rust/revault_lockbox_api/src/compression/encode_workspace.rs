//! Encoder state lives within a write; only wiped backing buffers are pooled.
use crate::page_buffer::{zeroize_bytes, ZeroizingBytes};
use std::cell::RefCell;
use zstd_complete::encoding::{CompressionLevel, EncoderWorkspace, StaticEncoderWorkspace};

const MAX_RETAINED: usize = 8 * 1024 * 1024;
struct Buffers {
    scratch: ZeroizingBytes,
    output: ZeroizingBytes,
}
thread_local! { static POOL: RefCell<Option<Buffers>> = const { RefCell::new(None) }; }
struct Lease(Option<Buffers>);
impl Lease {
    fn acquire(required: usize, output_size: usize) -> Self {
        let cached = POOL
            .try_with(|slot| slot.try_borrow_mut().ok().and_then(|mut slot| slot.take()))
            .ok()
            .flatten();
        let buffers = cached
            .filter(|b| b.scratch.len() == required && b.output.len() == output_size)
            .unwrap_or_else(|| Buffers {
                scratch: ZeroizingBytes::new(vec![0; required]),
                output: ZeroizingBytes::new(vec![0; output_size]),
            });
        Self(Some(buffers))
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut buffers = self.0.take();
        if let Some(buffers) = &mut buffers {
            // Encoder borrows have ended. Wipe all capacity before these bytes
            // become idle reusable storage, including on error or unwinding.
            zeroize_bytes(&mut buffers.scratch);
            zeroize_bytes(&mut buffers.output);
            if buffers
                .scratch
                .capacity()
                .saturating_add(buffers.output.capacity())
                > MAX_RETAINED
            {
                return;
            }
        }
        let _ = POOL.try_with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut() {
                if slot.is_none() {
                    *slot = buffers.take();
                }
            }
        });
        // Reentry/TLS teardown can prevent caching; normal zeroizing Drop then
        // releases the allocation. No borrowing failure can bypass cleanup.
    }
}

/// Input properties only: the caller may refill its buffer during encoding.
pub(crate) struct EncoderFrame {
    len: usize,
    incompressible: bool,
}
impl EncoderFrame {
    pub(crate) fn new(input: &[u8], probe: bool) -> Self {
        Self {
            len: input.len(),
            incompressible: !probe
                || !(64 * 1024..=super::MAX_DECOMPRESSED_COMPRESSION_FRAME_BYTES as usize)
                    .contains(&input.len())
                || super::looks_incompressible(input),
        }
    }
}

pub(crate) fn with_encoder<R>(
    first_frame: EncoderFrame,
    compression: Option<crate::Compression>,
    default_level: i32,
    operation: impl FnOnce(&mut dyn FnMut(&[u8]) -> (u8, Vec<u8>)) -> R,
) -> R {
    let mut fallback = |input: &[u8]| match compression {
        Some(compression) => super::encode_with_compression(input, compression),
        None => super::encode_compression_frame_with_level(input, default_level),
    };
    let Some(crate::Compression::Zstd { level }) = compression else {
        return operation(&mut fallback);
    };
    if !(64 * 1024..=super::MAX_DECOMPRESSED_COMPRESSION_FRAME_BYTES as usize)
        .contains(&first_frame.len)
        || first_frame.incompressible
    {
        return operation(&mut fallback);
    }
    // High-level workspace output can differ from the existing numeric path.
    // Preserve that path while evaluating reuse for the lower levels.
    if level.get() >= 18 {
        return operation(&mut fallback);
    }
    let level = CompressionLevel::new(level.get()).expect("validated compression level");
    let Ok(required) = EncoderWorkspace::required_size(level, first_frame.len) else {
        return operation(&mut fallback);
    };
    let Ok(output_size) = EncoderWorkspace::required_output_size(first_frame.len) else {
        return operation(&mut fallback);
    };
    if required.saturating_add(output_size) > MAX_RETAINED {
        return operation(&mut fallback);
    }
    let mut lease = Lease::acquire(required, output_size);
    let Buffers { scratch, output } = lease.0.as_mut().expect("lease owns buffers");
    let Ok(mut encoder) = StaticEncoderWorkspace::new(scratch, level, first_frame.len) else {
        return operation(&mut fallback);
    };
    // Neither backing buffer grows or shrinks while the encoder borrows it.
    // Lease wipes all capacity after the operation before pooling, including
    // rejected blocks and prior frames, on success, error, and unwinding.
    operation(&mut |input| {
        // Match the ordinary encoder's exact input-size parameter selection.
        // The final short frame retains the established numeric path.
        if input.len() != first_frame.len || super::looks_incompressible(input) {
            return fallback(input);
        }
        match encoder.encode_into(input, output) {
            Ok(encoded) if encoded.len() < input.len() => {
                (super::COMPRESSION_ZSTD, encoded.to_vec())
            }
            Ok(_) => (super::COMPRESSION_NONE, input.to_vec()),
            Err(_) => fallback(input),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pooled_buffers_are_wiped_and_reused_after_success_error_and_unwind() {
        POOL.with(|slot| slot.borrow_mut().take());
        let input = vec![73; 65536];
        let mut pointers = None;
        for exit in 0..3 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_encoder(
                    EncoderFrame::new(&input, true),
                    Some(crate::Compression::default()),
                    3,
                    |encode| {
                        let encoded = encode(&input);
                        assert_eq!(
                            super::super::decode_compression_frame(
                                encoded.0,
                                &encoded.1,
                                input.len() as u64
                            )
                            .unwrap(),
                            input
                        );
                        match exit {
                            0 => Ok(()),
                            1 => Err(()),
                            _ => panic!("injected callback failure"),
                        }
                    },
                )
            }));
            match exit {
                0 => assert_eq!(result.unwrap(), Ok(())),
                1 => assert_eq!(result.unwrap(), Err(())),
                _ => assert!(result.is_err()),
            }
            POOL.with(|slot| {
                let slot = slot.borrow();
                let buffers = slot.as_ref().unwrap();
                assert!(buffers.scratch.iter().all(|b| *b == 0));
                assert!(buffers.output.iter().all(|b| *b == 0));
                assert!(buffers.scratch.capacity() + buffers.output.capacity() <= MAX_RETAINED);
                let current = (buffers.scratch.as_ptr(), buffers.output.as_ptr());
                if let Some(previous) = pointers {
                    assert_eq!(previous, current);
                }
                pointers = Some(current);
            });
        }
        POOL.with(|slot| slot.borrow_mut().take());
        drop(Lease::acquire(MAX_RETAINED + 1, 0));
        POOL.with(|slot| {
            assert!(
                slot.borrow().is_none(),
                "over-budget allocations cannot be retained"
            )
        });
    }

    #[test]
    fn pool_reentry_and_unavailable_borrow_preserve_encoding_and_cleanup() {
        POOL.with(|slot| slot.borrow_mut().take());
        let input = vec![61; 65536];
        with_encoder(
            EncoderFrame::new(&input, true),
            Some(crate::Compression::default()),
            3,
            |outer| {
                let first = outer(&input);
                with_encoder(
                    EncoderFrame::new(&input, true),
                    Some(crate::Compression::default()),
                    3,
                    |inner| assert_eq!(inner(&input), first),
                );
                assert_eq!(outer(&input), first);
            },
        );
        POOL.with(|slot| {
            let mut held = slot.borrow_mut();
            held.take();
            with_encoder(
                EncoderFrame::new(&input, true),
                Some(crate::Compression::default()),
                3,
                |encode| {
                    let encoded = encode(&input);
                    assert_eq!(
                        super::super::decode_compression_frame(
                            encoded.0,
                            &encoded.1,
                            input.len() as u64
                        )
                        .unwrap(),
                        input
                    );
                },
            );
            assert!(held.is_none());
        });
    }

    #[test]
    fn full_sized_frames_preserve_numeric_bytes_across_reuse() {
        let pattern: Vec<u8> = (0..2 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        let repeated = vec![37; pattern.len()];
        for level in [1, 3, 6] {
            let compression = crate::Compression::Zstd {
                level: crate::ZstdLevel::new(level).unwrap(),
            };
            for _ in 0..2 {
                with_encoder(
                    EncoderFrame::new(&pattern, true),
                    Some(compression),
                    3,
                    |encode| {
                        for input in [&pattern[..], &repeated[..], &pattern[..]] {
                            let actual = encode(input);
                            assert_eq!(
                                actual,
                                super::super::encode_with_compression(input, compression)
                            );
                            assert_eq!(
                                super::super::decode_compression_frame(
                                    actual.0,
                                    &actual.1,
                                    input.len() as u64
                                )
                                .unwrap(),
                                input
                            );
                        }
                    },
                );
            }
        }
    }

    #[test]
    fn retained_state_matches_fresh_numeric_encoding_including_size_changes() {
        let mut state = 17u64;
        let mixed: Vec<u8> = (0..131073)
            .map(|i| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                if i % 131072 < 65536 {
                    (i % 19) as u8
                } else {
                    state as u8
                }
            })
            .collect();
        let repetitive = vec![5; mixed.len()];
        let short = &mixed[..65536];
        for level in 1..=22 {
            let compression = crate::Compression::Zstd {
                level: crate::ZstdLevel::new(level).unwrap(),
            };
            with_encoder(
                EncoderFrame::new(&mixed, true),
                Some(compression),
                1,
                |encode| {
                    for (frame, input) in [&mixed[..], &repetitive[..], short, &mixed[..], &[]]
                        .into_iter()
                        .enumerate()
                    {
                        let result = encode(input);
                        let decoded = super::super::decode_compression_frame(
                            result.0,
                            &result.1,
                            input.len() as u64,
                        )
                        .unwrap();
                        assert!(
                            decoded == input,
                            "round trip differs: level {level}, frame {frame}"
                        );
                        let fresh = super::super::encode_with_compression(input, compression);
                        assert!(result == fresh, "encoded bytes differ: level {level}, frame {frame}, reused {} bytes, fresh {} bytes", result.1.len(), fresh.1.len());
                    }
                },
            );
        }
    }
}
