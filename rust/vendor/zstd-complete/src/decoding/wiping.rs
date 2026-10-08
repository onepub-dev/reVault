//! Owned decoder byte storage. Caller-owned arenas retain their caller's wipe policy.
use crate::workspace::ReusableVec;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

/// Wipe the whole allocation, including bytes outside the current logical length.
///
/// # Safety
/// `pointer` must permit writes to `capacity` bytes. The allocation remains live
/// and exclusively borrowed until this function and its test observer return.
pub(super) unsafe fn wipe_allocation(pointer: *mut u8, capacity: usize) {
    // Represent spare capacity without asserting that it is initialized. The
    // aligned blocks have no padding; no existing value is read or interpreted.
    // SAFETY: the caller supplies the full exclusive live writable allocation.
    let bytes = unsafe {
        core::slice::from_raw_parts_mut(pointer.cast::<core::mem::MaybeUninit<u8>>(), capacity)
    };
    // SAFETY: MaybeUninit permits every initialized or uninitialized bit pattern.
    // This splits the allocation into disjoint aligned regions of the same total
    // size. Wide volatile stores avoid one volatile instruction per byte while
    // retaining full-allocation erasure, including uninitialized spare capacity.
    let (head, blocks, tail) = unsafe { bytes.align_to_mut::<core::mem::MaybeUninit<[u64; 8]>>() };
    for byte in head.iter_mut().chain(tail.iter_mut()) {
        // SAFETY: each byte is exclusively writable; the write initializes it.
        unsafe { core::ptr::write_volatile(byte, core::mem::MaybeUninit::new(0)) };
    }
    for block in blocks {
        // SAFETY: each padding-free block is exclusively writable and aligned.
        unsafe { core::ptr::write_volatile(block, core::mem::MaybeUninit::new([0; 8])) };
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    #[cfg(all(test, feature = "std"))]
    observe_release(pointer, capacity);
}

/// Unlike a mutable Vec facade, this type cannot grow without first retaining
/// and wiping the old allocation. Cleared/truncated bytes remain within the
/// owned allocation until its full-capacity wipe; they are not freed by reset.
pub(crate) struct WipingBytes {
    inner: ReusableVec<u8>,
}

impl WipingBytes {
    pub(crate) const fn new() -> Self {
        Self {
            inner: ReusableVec::new(),
        }
    }

    pub(crate) fn from_static(inner: ReusableVec<u8>) -> Self {
        assert!(!inner.is_owned());
        Self { inner }
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        let needed = self
            .inner
            .len()
            .checked_add(additional)
            .expect("byte capacity overflow");
        if needed <= self.inner.capacity() {
            return;
        }
        assert!(
            self.inner.is_owned(),
            "prepared static decode workspace capacity was exceeded"
        );
        let capacity = needed.max(self.inner.capacity().saturating_mul(2)).max(8);
        let mut next = ReusableVec::from_owned(Vec::with_capacity(capacity));
        next.extend_from_slice(&self.inner);
        let mut old = core::mem::replace(&mut self.inner, next);
        #[cfg(all(test, feature = "std"))]
        note_initialized(&old);
        // SAFETY: the old owned Vec remains exclusively live until after wiping.
        unsafe { wipe_allocation(old.as_mut_ptr(), old.capacity()) };
    }

    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.inner
    }
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    pub(crate) fn resize(&mut self, length: usize, byte: u8) {
        self.reserve(length.saturating_sub(self.len()));
        self.inner.resize(length, byte);
    }

    pub(crate) fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.reserve(bytes.len());
        self.inner.extend_from_slice(bytes);
    }

    pub(crate) fn push(&mut self, byte: u8) {
        self.reserve(1);
        self.inner.push(byte);
    }
}

impl Deref for WipingBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl DerefMut for WipingBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
impl Drop for WipingBytes {
    fn drop(&mut self) {
        if self.inner.is_owned() {
            #[cfg(all(test, feature = "std"))]
            note_initialized(&self.inner);
            // SAFETY: the Vec is still owned and exclusively borrowed; it drops
            // only after this destructor has wiped its entire allocation.
            unsafe { wipe_allocation(self.inner.as_mut_ptr(), self.inner.capacity()) };
        }
    }
}

#[cfg(all(test, feature = "std"))]
std::thread_local! {
    static RELEASES: std::cell::RefCell<Option<(bool, Vec<Release>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(all(test, feature = "std"))]
#[derive(Debug)]
struct Release {
    capacity: usize,
    had_nonzero_initialized_bytes: bool,
}

#[cfg(all(test, feature = "std"))]
pub(super) fn note_initialized(bytes: &[u8]) {
    RELEASES.with(|events| {
        if let Some((nonzero, _)) = events.borrow_mut().as_mut() {
            *nonzero |= bytes.iter().any(|byte| *byte != 0);
        }
    });
}

#[cfg(all(test, feature = "std"))]
fn observe_release(pointer: *mut u8, capacity: usize) {
    RELEASES.with(|releases| {
        if let Some((nonzero, releases)) = releases.borrow_mut().as_mut() {
            // SAFETY: wipe_allocation has initialized every byte, and observes
            // the allocation before its owner frees it. Never read freed memory.
            let bytes = unsafe { core::slice::from_raw_parts(pointer, capacity) };
            assert!(
                bytes.iter().all(|byte| *byte == 0),
                "decoder allocation was not wiped"
            );
            if capacity != 0 {
                releases.push(Release {
                    capacity,
                    had_nonzero_initialized_bytes: *nonzero,
                });
            }
            *nonzero = false;
        }
    });
}

#[cfg(all(test, feature = "std"))]
fn audit(operation: impl FnOnce()) -> Vec<Release> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            RELEASES.with(|events| *events.borrow_mut() = None);
        }
    }
    let _reset = Reset;
    RELEASES.with(|events| *events.borrow_mut() = Some((false, Vec::new())));
    operation();
    RELEASES.with(|events| events.borrow_mut().take().unwrap().1)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn allocation_wipe_covers_unaligned_and_uninitialized_regions() {
        use core::mem::MaybeUninit;
        for offset in 0..64 {
            for length in 0..193 {
                let mut bytes = [MaybeUninit::new(0xa5u8); 320];
                bytes[offset..offset + length].fill(MaybeUninit::uninit());
                // SAFETY: the selected range is live, exclusively writable and
                // inside this allocation. Its contents need not be initialized.
                unsafe { wipe_allocation(bytes.as_mut_ptr().add(offset).cast(), length) };
                for (index, byte) in bytes.iter().enumerate() {
                    // SAFETY: guards started initialized; wiping initialized
                    // every byte in the selected range. Read only afterward.
                    let value = unsafe { byte.assume_init() };
                    assert_eq!(
                        value,
                        if (offset..offset + length).contains(&index) {
                            0
                        } else {
                            0xa5
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn owned_bytes_wipe_growth_and_truncated_spare_capacity() {
        let events = audit(|| {
            let mut bytes = WipingBytes::new();
            bytes.resize(257, 0xab);
            bytes.resize(17, 0);
            bytes.reserve(4096);
            assert_eq!(&*bytes, &[0xab; 17]);
            bytes.clear();
        });
        assert_eq!(events.len(), 2);
        assert!(events[0].capacity >= 257 && events[1].capacity >= 4113);
        assert!(events[0].had_nonzero_initialized_bytes);
    }

    #[test]
    fn owned_history_wipes_growth_reset_and_drop() {
        let events = audit(|| {
            let mut history = super::super::ringbuffer::RingBuffer::new();
            history.extend(&[0xcd; 257]);
            history.reserve(4096);
            assert_eq!(history.len(), 257);
            history.clear();
        });
        assert_eq!(events.len(), 2);
        assert!(events[0].capacity >= 257 && events[1].capacity > events[0].capacity);
        assert!(events[0].had_nonzero_initialized_bytes);
    }

    #[test]
    fn decoder_owned_dictionary_replacement_and_drop_wipe_spare_capacity() {
        use super::super::scratch::{FSEScratch, HuffmanScratch};
        use crate::decoding::{Dictionary, FrameDecoder};
        let dictionary = |length| {
            let mut content = alloc::vec![0xab; length];
            content.truncate(17);
            Dictionary {
                id: 7,
                fse: FSEScratch::new(),
                huf: HuffmanScratch::new(),
                dict_content: content,
                offset_hist: [1, 4, 8],
            }
        };
        let events = audit(|| {
            let mut decoder = FrameDecoder::new();
            decoder.add_dict(dictionary(257)).unwrap();
            decoder.add_dict(dictionary(1024)).unwrap();
        });
        assert_eq!(
            events
                .iter()
                .map(|event| event.capacity)
                .collect::<Vec<_>>(),
            [257, 1024]
        );
        assert!(events
            .iter()
            .all(|event| event.had_nonzero_initialized_bytes));
    }

    #[cfg(feature = "hash")]
    #[test]
    fn decoder_release_wipes_success_failure_and_concatenated_growth() {
        use crate::decoding::FrameDecoder;
        use std::io::Write;
        let mut expected = Vec::new();
        let mut frames = Vec::new();
        for length in [8192, 262144] {
            let input: Vec<_> = (0..length)
                .map(|n| ((n / 71 + n / 251) % 19) as u8)
                .collect();
            let mut encoder = zstd::stream::Encoder::new(Vec::new(), 3).unwrap();
            encoder
                .window_log(if length == 8192 { 13 } else { 18 })
                .unwrap();
            encoder.include_checksum(true).unwrap();
            encoder.write_all(&input).unwrap();
            frames.push(encoder.finish().unwrap());
            expected.extend_from_slice(&input);
        }
        let valid = frames.concat();
        for failure in 0..3 {
            let mut input = valid.clone();
            if failure == 1 {
                *input.last_mut().unwrap() ^= 1;
            }
            if failure == 2 {
                input.truncate(input.len() - 8);
            }
            let events = audit(|| {
                let mut decoder = FrameDecoder::new();
                let mut output = alloc::vec![0; expected.len()];
                let result = decoder.decode_all(&input, &mut output);
                if failure == 0 {
                    assert_eq!(result.unwrap(), expected.len());
                    assert_eq!(output, expected);
                } else {
                    assert!(result.is_err());
                }
            });
            // History plus compressed-block and literal allocations, including
            // at least one growth between the differently sized frames.
            assert!(
                events.len() >= 4,
                "missing owned-buffer releases: {:?}",
                events
            );
            assert!(events.iter().any(|event| event.capacity >= 262144));
            assert!(
                events
                    .iter()
                    .filter(|event| event.had_nonzero_initialized_bytes)
                    .count()
                    >= 2
            );
        }
    }
}
