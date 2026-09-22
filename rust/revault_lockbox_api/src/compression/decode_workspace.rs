//! Scratch allocation reuse limited to one synchronous content-stream operation.
use crate::page_buffer::ZeroizingBytes;
use crate::{Error, Result};
use std::cell::RefCell;
use zstd_complete::decoding::{
    errors::FrameDecoderError, DecoderWorkspaceError, StaticDecoderWorkspace,
};

const WINDOW: usize = super::MAX_DECOMPRESSED_COMPRESSION_FRAME_BYTES as usize;

thread_local! {
    static SCRATCH: RefCell<Option<ZeroizingBytes>> = const { RefCell::new(None) };
}

pub(crate) fn scoped<R>(operation: impl FnOnce() -> R) -> R {
    struct Scope(bool);
    impl Drop for Scope {
        fn drop(&mut self) {
            if self.0 {
                // The last scope wipes the full allocation, including scratch
                // history, on success, error, or unwinding through a visitor.
                SCRATCH.with(|scratch| {
                    scratch.borrow_mut().take();
                });
            }
        }
    }
    let owns = SCRATCH.with(|scratch| {
        let mut scratch = scratch.borrow_mut();
        if scratch.is_some() {
            false
        } else {
            *scratch = Some(ZeroizingBytes::new(Vec::new()));
            true
        }
    });
    let _scope = Scope(owns);
    operation()
}

pub(super) fn decode(stored: &[u8], expected_len: usize) -> Option<Result<Vec<u8>>> {
    // Metadata and oversized legacy pages keep the ordinary decoder. The
    // bounded allocation is lazy, so raw-only streams allocate no workspace.
    if !(64 * 1024..=WINDOW).contains(&expected_len) {
        return None;
    }
    // This is only an allocation hint: the decoder still validates the whole
    // header and enforces the bound. Non-single-segment frames can advertise a
    // window larger than their output; preserve that without always reserving
    // the maximum. Skippable prefixes retain the ordinary decoder.
    let header = stored.get(..6)?;
    if &header[..4] != super::ZSTD_MAGIC {
        return None;
    }
    let window = if header[4] & 0x20 == 0 {
        let base = 1u64 << (10 + (header[5] >> 3));
        let advertised = base + base / 8 * u64::from(header[5] & 7);
        advertised.max(expected_len as u64)
    } else {
        expected_len as u64
    };
    if window > WINDOW as u64 {
        return None;
    }
    let window = window as usize;
    SCRATCH.with(|slot| {
        let mut slot = slot.borrow_mut();
        let scratch = slot.as_mut()?;
        let result = (|| {
            let required = StaticDecoderWorkspace::required_size(window, 0)
                .map_err(|_| Error::CorruptRecord)?;
            if scratch.len() < required {
                // Replacing the wrapper wipes the old history before freeing
                // it; Vec growth could otherwise leave an unwiped old buffer.
                *scratch = ZeroizingBytes::new(vec![0; required]);
            }
            let mut decoded = ZeroizingBytes::new(vec![0; expected_len]);
            let mut decoder = StaticDecoderWorkspace::new(scratch, window, 0)
                .map_err(|_| Error::CorruptRecord)?;
            match decoder.decode_into(stored, &mut decoded) {
                Ok(len) => {
                    decoded.truncate(len);
                    Ok(Some(std::mem::take(&mut *decoded)))
                }
                // A valid small-output frame can advertise a larger history
                // window. Preserve the existing decoder's acceptance limits.
                Err(DecoderWorkspaceError::Decode(FrameDecoderError::WindowSizeTooBig {
                    ..
                })) => Ok(None),
                Err(_) => Err(Error::CorruptRecord),
            }
        })();
        match result {
            Ok(Some(bytes)) => Some(Ok(bytes)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_decoder_matches_fresh_decoder_for_sizes_corruption_and_concatenation() {
        for len in [65_536, 131_073, 2 * 1024 * 1024, WINDOW] {
            let payload = (0..len)
                .map(|i| ((i * 13 + i / 251) % 251) as u8)
                .collect::<Vec<_>>();
            let encoded = super::super::zstd_encode(&payload, 1);
            let compare = |bytes: &[u8], expected: usize| {
                let fresh = super::super::zstd_decode(bytes, expected as u64);
                let reused = scoped(|| super::super::zstd_decode(bytes, expected as u64));
                match (fresh, reused) {
                    (Ok(left), Ok(right)) => assert_eq!(left, right),
                    (Err(_), Err(_)) => {}
                    other => panic!(
                        "decoder acceptance differs: {:?}",
                        (other.0.is_ok(), other.1.is_ok())
                    ),
                }
            };
            compare(&encoded, len);
            compare(&encoded, len - 1);
            compare(&encoded, len + 1);
            for offset in [0, encoded.len() / 2, encoded.len() - 1] {
                let mut corrupt = encoded.clone();
                corrupt[offset] ^= 0x80;
                compare(&corrupt, len);
                compare(&encoded[..offset], len);
            }
            if len * 2 <= WINDOW {
                let mut concatenated = encoded.clone();
                concatenated.extend_from_slice(&encoded);
                compare(&concatenated, len * 2);
            }
        }
    }

    #[test]
    fn scoped_scratch_reuses_allocation_and_cleans_up_nested_and_panicking_calls() {
        let payload = vec![42; 128 * 1024];
        let encoded = super::super::zstd_encode(&payload, 1);
        assert!(decode(&encoded, payload.len()).is_none());
        scoped(|| {
            assert_eq!(decode(&encoded, payload.len()).unwrap().unwrap(), payload);
            let allocation = SCRATCH.with(|slot| slot.borrow().as_ref().unwrap().as_ptr());
            scoped(|| {
                assert_eq!(decode(&encoded, payload.len()).unwrap().unwrap(), payload);
                assert!(decode(&encoded[..encoded.len() - 1], payload.len())
                    .unwrap()
                    .is_err());
                assert_eq!(decode(&encoded, payload.len()).unwrap().unwrap(), payload);
            });
            assert_eq!(
                SCRATCH.with(|slot| slot.borrow().as_ref().unwrap().as_ptr()),
                allocation
            );
            let previous_len = SCRATCH.with(|slot| slot.borrow().as_ref().unwrap().len());
            let larger = vec![73; WINDOW];
            let larger_encoded = super::super::zstd_encode(&larger, 1);
            assert_eq!(
                decode(&larger_encoded, larger.len()).unwrap().unwrap(),
                larger
            );
            assert!(SCRATCH.with(|slot| slot.borrow().as_ref().unwrap().len()) > previous_len);
            assert_eq!(decode(&encoded, payload.len()).unwrap().unwrap(), payload);
        });
        assert!(SCRATCH.with(|slot| slot.borrow().is_none()));
        assert!(std::panic::catch_unwind(|| scoped(|| {
            decode(&encoded, payload.len()).unwrap().unwrap();
            panic!("visitor failure");
        }))
        .is_err());
        assert!(SCRATCH.with(|slot| slot.borrow().is_none()));
    }

    #[test]
    fn oversized_history_keeps_the_original_decoder_fallback() {
        // A single raw block with a 16 MiB advertised window and no content
        // size. Output is small enough for our workspace, but history is not.
        let payload = vec![19; 64 * 1024];
        let mut encoded = vec![0x28, 0xb5, 0x2f, 0xfd, 0, 0x70];
        let header = ((payload.len() as u32) << 3) | 1;
        encoded.extend_from_slice(&header.to_le_bytes()[..3]);
        encoded.extend_from_slice(&payload);
        scoped(|| {
            assert!(decode(&encoded, payload.len()).is_none());
            assert_eq!(
                super::super::zstd_decode(&encoded, payload.len() as u64).unwrap(),
                payload
            );
        });
    }
}
