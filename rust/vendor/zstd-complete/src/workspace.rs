//! Allocation-free storage primitives used by prepared contexts.

use alloc::vec::Vec;
use core::{
    fmt,
    marker::PhantomData,
    mem::{align_of, size_of, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ArenaError {
    InsufficientStorage { required: usize, provided: usize },
    CapacityExceeded { capacity: usize },
    SizeOverflow,
}

impl fmt::Display for ArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientStorage { required, provided } => write!(
                formatter,
                "workspace needs {required} bytes but only {provided} were provided"
            ),
            Self::CapacityExceeded { capacity } => {
                write!(
                    formatter,
                    "workspace vector exceeded its {capacity}-item capacity"
                )
            }
            Self::SizeOverflow => formatter.write_str("workspace size calculation overflowed"),
        }
    }
}

pub(crate) struct Arena<'a> {
    start: NonNull<u8>,
    len: usize,
    cursor: usize,
    marker: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ArenaSize {
    bytes: usize,
}

impl ArenaSize {
    pub(crate) const fn new() -> Self {
        Self { bytes: 0 }
    }

    pub(crate) fn add<T>(&mut self, count: usize) -> Result<(), ArenaError> {
        self.bytes = self
            .bytes
            .checked_add(align_of::<T>() - 1)
            .and_then(|bytes| bytes.checked_add(size_of::<T>().checked_mul(count)?))
            .ok_or(ArenaError::SizeOverflow)?;
        Ok(())
    }

    pub(crate) const fn finish(self) -> usize {
        self.bytes
    }
}

impl<'a> Arena<'a> {
    pub(crate) fn new(storage: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            start: NonNull::new(storage.as_mut_ptr().cast()).unwrap_or_else(NonNull::dangling),
            len: storage.len(),
            cursor: 0,
            marker: PhantomData,
        }
    }

    pub(crate) fn allocate_vec<T>(
        &mut self,
        capacity: usize,
    ) -> Result<ArenaVec<'a, T>, ArenaError> {
        if size_of::<T>() == 0 {
            return Err(ArenaError::SizeOverflow);
        }
        let address = self.start.as_ptr() as usize;
        let current = address
            .checked_add(self.cursor)
            .ok_or(ArenaError::SizeOverflow)?;
        let padding = current.wrapping_neg() & (align_of::<T>() - 1);
        let bytes = size_of::<T>()
            .checked_mul(capacity)
            .ok_or(ArenaError::SizeOverflow)?;
        let end = self
            .cursor
            .checked_add(padding)
            .and_then(|offset| offset.checked_add(bytes))
            .ok_or(ArenaError::SizeOverflow)?;
        if end > self.len {
            return Err(ArenaError::InsufficientStorage {
                required: end,
                provided: self.len,
            });
        }
        // SAFETY: `end <= self.len`; padding aligns the pointer for `T`. The
        // arena cursor advances past this exclusive region before another
        // mutable slice can be created.
        let ptr = unsafe { self.start.as_ptr().add(self.cursor + padding).cast::<T>() };
        self.cursor = end;
        Ok(ArenaVec {
            ptr: NonNull::new(ptr).unwrap_or_else(NonNull::dangling),
            len: 0,
            capacity,
            marker: PhantomData,
        })
    }

    pub(crate) fn allocate_reusable_vec<T: 'a>(
        &mut self,
        capacity: usize,
    ) -> Result<ReusableVec<T>, ArenaError> {
        let vector = self.allocate_vec::<T>(capacity)?;
        let ptr = vector.ptr.as_ptr();
        let capacity = vector.capacity;
        core::mem::forget(vector);
        // SAFETY: the arena owns this exclusive aligned region for its full
        // borrowed lifetime, and callers use the fixed capacity calculated by
        // the workspace layout.
        Ok(unsafe { ReusableVec::from_static_parts(ptr, capacity) })
    }

    pub(crate) fn allocate_uninit_slice<T: 'a>(
        &mut self,
        capacity: usize,
    ) -> Result<&'a mut [MaybeUninit<T>], ArenaError> {
        let vector = self.allocate_vec::<T>(capacity)?;
        let ptr = vector.ptr.as_ptr().cast::<MaybeUninit<T>>();
        core::mem::forget(vector);
        // SAFETY: `allocate_vec` reserved this exclusive aligned region.
        Ok(unsafe { slice::from_raw_parts_mut(ptr, capacity) })
    }
}

pub(crate) struct ArenaVec<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    marker: PhantomData<&'a mut [MaybeUninit<T>]>,
}

#[allow(dead_code)]
impl<T> ArenaVec<'_, T> {
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) const fn capacity(&self) -> usize {
        self.capacity
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn clear(&mut self) {
        // SAFETY: exactly `0..len` is initialized by the type invariant.
        unsafe {
            core::ptr::drop_in_place(core::ptr::slice_from_raw_parts_mut(
                self.ptr.as_ptr(),
                self.len,
            ))
        };
        self.len = 0;
    }

    pub(crate) fn push(&mut self, value: T) -> Result<(), ArenaError> {
        if self.len == self.capacity {
            return Err(ArenaError::CapacityExceeded {
                capacity: self.capacity,
            });
        }
        // SAFETY: `len < capacity`, and the destination slot is uninitialized.
        unsafe { self.ptr.as_ptr().add(self.len).write(value) };
        self.len += 1;
        Ok(())
    }

    pub(crate) fn extend_from_slice(&mut self, source: &[T]) -> Result<(), ArenaError>
    where
        T: Copy,
    {
        let new_len = self
            .len
            .checked_add(source.len())
            .ok_or(ArenaError::SizeOverflow)?;
        if new_len > self.capacity {
            return Err(ArenaError::CapacityExceeded {
                capacity: self.capacity,
            });
        }
        // SAFETY: the source is initialized and non-overlapping, and the
        // capacity check proves the destination range is in bounds.
        unsafe {
            self.ptr
                .as_ptr()
                .add(self.len)
                .copy_from_nonoverlapping(source.as_ptr(), source.len())
        };
        self.len = new_len;
        Ok(())
    }

    pub(crate) fn resize(&mut self, new_len: usize, value: T) -> Result<(), ArenaError>
    where
        T: Clone,
    {
        if new_len > self.capacity {
            return Err(ArenaError::CapacityExceeded {
                capacity: self.capacity,
            });
        }
        while self.len > new_len {
            self.len -= 1;
            // SAFETY: the old final element was initialized.
            unsafe { self.ptr.as_ptr().add(self.len).drop_in_place() };
        }
        while self.len < new_len {
            // Capacity was checked once above.
            let result = self.push(value.clone());
            debug_assert!(result.is_ok());
        }
        Ok(())
    }

    pub(crate) fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        // SAFETY: `len <= capacity`; this is the uninitialized suffix and the
        // mutable borrow prevents any simultaneous access through `self`.
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast::<MaybeUninit<T>>(),
                self.capacity - self.len,
            )
        }
    }

    /// Runs existing `Vec`-based code against this fixed arena allocation.
    ///
    /// The closure must not require more than `capacity()` elements. Workspace
    /// layout calculations establish that bound before entering codec code.
    pub(crate) fn with_vec<R>(
        &mut self,
        operation: impl FnOnce(&mut alloc::vec::Vec<T>) -> R,
    ) -> R {
        // SAFETY: the arena allocation has `capacity` properly aligned slots,
        // and `0..len` is initialized. `ManuallyDrop` prevents `Vec` from
        // attempting to release caller-owned storage.
        let mut view = ManuallyDrop::new(unsafe {
            alloc::vec::Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.capacity)
        });
        let result = operation(&mut view);
        self.len = view.len();
        debug_assert_eq!(view.capacity(), self.capacity);
        result
    }

    /// # Safety
    /// Every item in the newly exposed `old_len..new_len` range must already
    /// be initialized, and `new_len` must not exceed capacity.
    pub(crate) unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity);
        self.len = new_len;
    }
}

impl<T> Deref for ArenaVec<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: exactly `0..len` is initialized by the type invariant.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for ArenaVec<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: exactly `0..len` is initialized and `&mut self` is exclusive.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Drop for ArenaVec<'_, T> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// A `Vec`-compatible allocation which may be owned or leased from a static
/// arena. Borrowed instances must never grow beyond their established capacity.
/// Vector storage that can either own a heap allocation or borrow a fixed
/// region from a codec workspace.
///
/// This type is public only because legacy decoder table fields expose their
/// backing vectors. Codec users should construct typed workspaces instead.
#[doc(hidden)]
pub struct ReusableVec<T> {
    inner: ManuallyDrop<Vec<T>>,
    owned: bool,
}

#[derive(Debug)]
pub(crate) struct VecLease<T> {
    owned: bool,
    ptr: *mut T,
    capacity: usize,
}

impl<T> ReusableVec<T> {
    pub(crate) const fn new() -> Self {
        Self {
            inner: ManuallyDrop::new(Vec::new()),
            owned: true,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: ManuallyDrop::new(Vec::with_capacity(capacity)),
            owned: true,
        }
    }

    pub(crate) fn from_owned(value: Vec<T>) -> Self {
        Self {
            inner: ManuallyDrop::new(value),
            owned: true,
        }
    }

    /// Returns the underlying allocation when this vector owns it.
    pub(crate) fn into_owned_vec(self) -> Vec<T> {
        assert!(self.owned, "caller-owned workspace storage cannot escape");
        let (ptr, len, capacity, owned) = self.into_raw_parts();
        debug_assert!(owned);
        // SAFETY: the owned reusable vector was originally constructed from
        // these exact `Vec` parts and ownership transfers to the result.
        unsafe { Vec::from_raw_parts(ptr, len, capacity) }
    }

    pub(crate) const fn is_owned(&self) -> bool {
        self.owned
    }

    /// # Safety
    /// The region must remain exclusively borrowed for this value's lifetime,
    /// contain `capacity` aligned `T` slots, and the vector must never grow.
    pub(crate) unsafe fn from_static_parts(ptr: *mut T, capacity: usize) -> Self {
        Self {
            // SAFETY: guaranteed by the caller's workspace partition.
            inner: ManuallyDrop::new(unsafe { Vec::from_raw_parts(ptr, 0, capacity) }),
            owned: false,
        }
    }

    /// Temporarily transfers the Vec facade to code whose ownership-shaped
    /// API predates reusable workspaces. The returned lease must be used to
    /// recover the vector before it can be dropped.
    pub(crate) fn lease_vec(self) -> (Vec<T>, VecLease<T>) {
        let (ptr, len, capacity, owned) = self.into_raw_parts();
        // SAFETY: these are the exact parts extracted from the vector above.
        let value = unsafe { Vec::from_raw_parts(ptr, len, capacity) };
        (
            value,
            VecLease {
                owned,
                ptr,
                capacity,
            },
        )
    }

    /// Recovers a vector transferred by [`Self::lease_vec`].
    pub(crate) fn recover_vec(value: Vec<T>, lease: VecLease<T>) -> Self {
        if !lease.owned {
            assert_eq!(value.as_ptr(), lease.ptr, "static workspace vector moved");
            assert_eq!(
                value.capacity(),
                lease.capacity,
                "static workspace vector grew"
            );
        }
        Self {
            inner: ManuallyDrop::new(value),
            owned: lease.owned,
        }
    }

    pub(crate) fn into_uninit(self) -> ReusableVec<MaybeUninit<T>> {
        let (ptr, len, capacity, owned) = self.into_raw_parts();
        ReusableVec {
            // SAFETY: `MaybeUninit<T>` has the same layout as `T`; all old
            // initialized elements remain valid initialized MaybeUninit values.
            inner: ManuallyDrop::new(unsafe {
                Vec::from_raw_parts(ptr.cast::<MaybeUninit<T>>(), len, capacity)
            }),
            owned,
        }
    }

    fn into_raw_parts(mut self) -> (*mut T, usize, usize, bool) {
        let parts = (
            self.inner.as_mut_ptr(),
            self.inner.len(),
            self.inner.capacity(),
            self.owned,
        );
        self.owned = false;
        core::mem::forget(self);
        parts
    }
}

impl<T> ReusableVec<MaybeUninit<T>> {
    /// # Safety
    /// Every element in `0..len()` must contain an initialized `T`.
    pub(crate) unsafe fn assume_init(self) -> ReusableVec<T> {
        let (ptr, len, capacity, owned) = self.into_raw_parts();
        ReusableVec {
            // SAFETY: guaranteed by the caller; MaybeUninit preserves layout.
            inner: ManuallyDrop::new(unsafe {
                Vec::from_raw_parts(ptr.cast::<T>(), len, capacity)
            }),
            owned,
        }
    }
}

impl<T> Default for ReusableVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for ReusableVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ReusableVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Clone> Clone for ReusableVec<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ManuallyDrop::new((**self).clone()),
            owned: true,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ReusableVec<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(formatter)
    }
}

impl<T: PartialEq> PartialEq for ReusableVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T, U> PartialEq<Vec<U>> for ReusableVec<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Vec<U>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for ReusableVec<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T, U, const N: usize> PartialEq<&[U; N]> for ReusableVec<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U; N]) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for ReusableVec<T> {}

impl<T> Drop for ReusableVec<T> {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: only owned instances contain a globally allocated Vec.
            unsafe { ManuallyDrop::drop(&mut self.inner) };
        } else {
            self.inner.clear();
        }
    }
}

impl<'a, T> IntoIterator for &'a ReusableVec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ReusableVec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn arena_layout_accepts_every_start_alignment() {
        let mut size = ArenaSize::new();
        size.add::<u8>(1).unwrap();
        size.add::<u64>(3).unwrap();
        let required = size.finish();

        for offset in 0..align_of::<u64>() {
            let mut storage = [MaybeUninit::uninit(); 64];
            let mut arena = Arena::new(&mut storage[offset..offset + required]);
            let mut bytes = arena.allocate_vec::<u8>(1).unwrap();
            bytes.push(7).unwrap();
            let mut words = arena.allocate_vec::<u64>(3).unwrap();
            words.extend_from_slice(&[11, 13, 17]).unwrap();
            assert_eq!(&*bytes, &[7]);
            assert_eq!(&*words, &[11, 13, 17]);
        }
    }

    static DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DropTracker {
        _value: u8,
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn caller_owned_reusable_vec_survives_lease_recovery_and_drops_values() {
        DROPS.store(0, Ordering::Relaxed);
        let mut storage = [MaybeUninit::uninit(); 64];
        let mut arena = Arena::new(&mut storage);
        let mut values = arena.allocate_reusable_vec::<DropTracker>(2).unwrap();
        values.push(DropTracker { _value: 1 });

        let (mut facade, lease) = values.lease_vec();
        facade.push(DropTracker { _value: 2 });
        let values = ReusableVec::recover_vec(facade, lease);
        assert!(!values.is_owned());
        assert_eq!(values.len(), 2);
        drop(values);
        assert_eq!(DROPS.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn owned_reusable_vec_returns_its_allocation_after_lease_recovery() {
        let reusable = ReusableVec::from_owned(Vec::with_capacity(4));
        let (mut facade, lease) = reusable.lease_vec();
        facade.extend_from_slice(&[2_u32, 3, 5]);
        let reusable = ReusableVec::recover_vec(facade, lease);
        assert!(reusable.is_owned());
        assert_eq!(reusable.into_owned_vec(), [2, 3, 5]);
    }
}
