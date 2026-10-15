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
    Capacity(CapacityError),
    InsufficientStorage { required: usize, provided: usize },
    CapacityExceeded { capacity: usize },
    SizeOverflow,
}

impl fmt::Display for ArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capacity(error) => {
                write!(formatter, "workspace storage admission failed: {error:?}")
            }
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

impl From<CapacityError> for ArenaError {
    fn from(error: CapacityError) -> Self {
        Self::Capacity(error)
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
        while self.len != 0 {
            self.len -= 1;
            unsafe { self.ptr.as_ptr().add(self.len).drop_in_place() };
        }
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

enum VectorStorage<T> {
    Owned(Vec<T>),
    Borrowed(crate::fixed_storage::FixedStorage<T>),
}

/// Owned dynamic or caller-backed fixed storage. Mutable access is a slice;
/// fallible growth never reallocates caller-backed storage.
#[doc(hidden)]
pub struct ReusableVec<T> {
    storage: VectorStorage<T>,
}

pub use crate::fixed_storage::CapacityError;

#[derive(Debug)]
pub(crate) struct VecLease<T> {
    owned: bool,
    ptr: *mut T,
    capacity: usize,
}

impl<T> ReusableVec<T> {
    pub(crate) const fn new() -> Self {
        Self {
            storage: VectorStorage::Owned(Vec::new()),
        }
    }
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::from_owned(Vec::with_capacity(capacity))
    }
    pub(crate) fn from_owned(value: Vec<T>) -> Self {
        Self {
            storage: VectorStorage::Owned(value),
        }
    }
    #[cfg(test)]
    pub(crate) fn into_owned_vec(self) -> Vec<T> {
        match self.storage {
            VectorStorage::Owned(value) => value,
            VectorStorage::Borrowed(_) => panic!("caller-owned workspace storage cannot escape"),
        }
    }
    pub(crate) const fn is_owned(&self) -> bool {
        matches!(self.storage, VectorStorage::Owned(_))
    }
    /// Caller retains exclusive, aligned backing through destruction. No slot
    /// is initialized. The enclosing workspace carries the backing lifetime.
    pub(crate) unsafe fn from_static_parts(ptr: *mut T, capacity: usize) -> Self {
        Self {
            storage: VectorStorage::Borrowed(unsafe {
                crate::fixed_storage::FixedStorage::new(
                    NonNull::new(ptr).expect("non-null workspace"),
                    capacity,
                )
            }),
        }
    }
    pub fn capacity(&self) -> usize {
        match &self.storage {
            VectorStorage::Owned(v) => v.capacity(),
            VectorStorage::Borrowed(v) => v.capacity(),
        }
    }
    /// Pointer to the original allocation, including its spare capacity.
    /// Dereferencing requires the caller to uphold initialization and aliasing.
    pub fn as_ptr(&self) -> *const T {
        match &self.storage {
            VectorStorage::Owned(v) => v.as_ptr(),
            VectorStorage::Borrowed(v) => v.as_ptr(),
        }
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        match &mut self.storage {
            VectorStorage::Owned(v) => v.as_mut_ptr(),
            VectorStorage::Borrowed(v) => v.as_mut_ptr(),
        }
    }
    pub fn as_slice(&self) -> &[T] {
        match &self.storage {
            VectorStorage::Owned(v) => v.as_slice(),
            VectorStorage::Borrowed(v) => v.as_slice(),
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.storage {
            VectorStorage::Owned(v) => v.as_mut_slice(),
            VectorStorage::Borrowed(v) => v.as_mut_slice(),
        }
    }
    pub fn reserve(&mut self, additional: usize) -> Result<(), CapacityError> {
        self.len()
            .checked_add(additional)
            .ok_or(CapacityError::Overflow)?;
        match &mut self.storage {
            VectorStorage::Owned(v) => v
                .try_reserve(additional)
                .map_err(|_| CapacityError::AllocationFailed),
            VectorStorage::Borrowed(v) => v.reserve(additional),
        }
    }
    pub fn reserve_exact(&mut self, additional: usize) -> Result<(), CapacityError> {
        self.len()
            .checked_add(additional)
            .ok_or(CapacityError::Overflow)?;
        match &mut self.storage {
            VectorStorage::Owned(v) => v
                .try_reserve_exact(additional)
                .map_err(|_| CapacityError::AllocationFailed),
            VectorStorage::Borrowed(v) => v.reserve(additional),
        }
    }
    pub fn push(&mut self, value: T) -> Result<(), CapacityError> {
        match &mut self.storage {
            VectorStorage::Owned(v) => {
                v.try_reserve(1)
                    .map_err(|_| CapacityError::AllocationFailed)?;
                v.push(value);
                Ok(())
            }
            VectorStorage::Borrowed(v) => v.push(value),
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        match &mut self.storage {
            VectorStorage::Owned(v) => v.pop(),
            VectorStorage::Borrowed(v) => v.pop(),
        }
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len(), "swap_remove index out of bounds");
        let last = self.pop().expect("checked nonempty storage");
        if index == self.len() {
            last
        } else {
            core::mem::replace(&mut self[index], last)
        }
    }
    pub fn truncate(&mut self, length: usize) {
        match &mut self.storage {
            VectorStorage::Owned(v) => v.truncate(length),
            VectorStorage::Borrowed(v) => v.truncate(length),
        }
    }
    pub fn clear(&mut self) {
        self.truncate(0);
    }
    pub fn resize(&mut self, length: usize, value: T) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        match &mut self.storage {
            VectorStorage::Owned(v) => {
                v.try_reserve(length.saturating_sub(v.len()))
                    .map_err(|_| CapacityError::AllocationFailed)?;
                v.resize(length, value);
                Ok(())
            }
            VectorStorage::Borrowed(v) => v.resize(length, value),
        }
    }
    pub fn resize_with(
        &mut self,
        length: usize,
        mut create: impl FnMut() -> T,
    ) -> Result<(), CapacityError> {
        if length <= self.len() {
            self.truncate(length);
            return Ok(());
        }
        self.reserve(length - self.len())?;
        while self.len() < length {
            self.push(create())?;
        }
        Ok(())
    }
    pub fn extend_from_slice(&mut self, values: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        match &mut self.storage {
            VectorStorage::Owned(v) => {
                v.try_reserve(values.len())
                    .map_err(|_| CapacityError::AllocationFailed)?;
                v.extend_from_slice(values);
                Ok(())
            }
            VectorStorage::Borrowed(v) => v.extend_from_slice(values),
        }
    }
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        match &mut self.storage {
            VectorStorage::Owned(v) => v.spare_capacity_mut(),
            VectorStorage::Borrowed(v) => v.spare_capacity_mut(),
        }
    }
    /// Newly exposed elements must already be initialized. Shrinking transfers
    /// responsibility for removed elements to the caller, as with Vec::set_len.
    pub unsafe fn set_len(&mut self, length: usize) -> Result<(), CapacityError> {
        if length > self.capacity() {
            return Err(CapacityError::Exhausted {
                required: length,
                capacity: self.capacity(),
            });
        }
        match &mut self.storage {
            VectorStorage::Owned(v) => {
                unsafe { v.set_len(length) };
                Ok(())
            }
            VectorStorage::Borrowed(v) => unsafe { v.set_len(length) },
        }
    }
    pub(crate) fn lease_vec(self) -> (Self, VecLease<T>) {
        let lease = VecLease {
            owned: self.is_owned(),
            ptr: self.as_ptr() as *mut T,
            capacity: self.capacity(),
        };
        (self, lease)
    }
    pub(crate) fn recover_vec(value: Self, lease: VecLease<T>) -> Self {
        assert_eq!(value.is_owned(), lease.owned, "workspace ownership changed");
        if !lease.owned {
            assert_eq!(value.as_ptr(), lease.ptr);
            assert_eq!(value.capacity(), lease.capacity);
        }
        value
    }
    /// T/U have equal layout and compatible validity/destruction semantics.
    pub(crate) unsafe fn cast<U>(self) -> ReusableVec<U> {
        assert_eq!(size_of::<T>(), size_of::<U>());
        assert_eq!(align_of::<T>(), align_of::<U>());
        match self.storage {
            VectorStorage::Owned(value) => {
                let mut value = ManuallyDrop::new(value);
                ReusableVec::from_owned(unsafe {
                    Vec::from_raw_parts(value.as_mut_ptr().cast(), value.len(), value.capacity())
                })
            }
            VectorStorage::Borrowed(value) => {
                let (ptr, len, capacity) = value.into_parts();
                let ptr = ptr.cast::<U>().as_ptr();
                let mut result = unsafe { ReusableVec::from_static_parts(ptr, capacity) };
                // Equal layout preserves capacity, initialized prefix and ownership.
                unsafe { result.set_len(len) }.expect("layout-preserving length");
                result
            }
        }
    }
    pub(crate) fn into_uninit(self) -> ReusableVec<MaybeUninit<T>> {
        unsafe { self.cast() }
    }
}

impl<T> ReusableVec<MaybeUninit<T>> {
    /// Every live element must contain an initialized T.
    pub(crate) unsafe fn assume_init(self) -> ReusableVec<T> {
        unsafe { self.cast() }
    }
}

impl<T> Default for ReusableVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Deref for ReusableVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T> DerefMut for ReusableVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<T: Clone> Clone for ReusableVec<T> {
    fn clone(&self) -> Self {
        // Borrowed copies need an explicit caller-backed destination; silently
        // allocating an owned copy would break sensitive scratch containment.
        assert!(
            self.is_owned(),
            "borrowed workspace cloning requires explicit storage"
        );
        Self::from_owned(self.as_slice().to_vec())
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
        values
            .push(DropTracker { _value: 1 })
            .expect("test storage admission");

        let (mut facade, lease) = values.lease_vec();
        facade
            .push(DropTracker { _value: 2 })
            .expect("test storage admission");
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
        facade
            .extend_from_slice(&[2_u32, 3, 5])
            .expect("test storage admission");
        let reusable = ReusableVec::recover_vec(facade, lease);
        assert!(reusable.is_owned());
        assert_eq!(reusable.into_owned_vec(), [2, 3, 5]);
    }
    #[test]
    fn transferred_workspace_facade_preserves_borrowed_identity_on_early_return() {
        fn visit(values: ReusableVec<u32>) -> Result<(), ()> {
            let pointer = values.as_ptr();
            let capacity = values.capacity();
            let (facade, _normal_return_identity) = values.lease_vec();
            assert!(!facade.is_owned());
            assert_eq!(facade.as_ptr(), pointer);
            assert_eq!(facade.capacity(), capacity);
            assert_eq!(facade.as_slice(), &[11, 17]);
            Err(())
        }
        let mut storage = [MaybeUninit::uninit(); 128];
        {
            let mut arena = Arena::new(&mut storage);
            let mut values = arena.allocate_reusable_vec::<u32>(4).unwrap();
            values
                .extend_from_slice(&[11, 17])
                .expect("test storage admission");
            assert_eq!(visit(values), Err(()));
        }
        // Caller storage remains available after the scoped visitor releases it.
        let mut arena = Arena::new(&mut storage);
        let mut values = arena.allocate_reusable_vec::<u32>(4).unwrap();
        values
            .extend_from_slice(&[19, 23])
            .expect("test storage admission");
        assert_eq!(values.as_slice(), &[19, 23]);
    }

    #[test]
    fn scalar_layout_transfer_preserves_storage_ownership_and_contents() {
        let mut storage = [MaybeUninit::uninit(); 128];
        let mut arena = Arena::new(&mut storage);
        let mut values = arena.allocate_reusable_vec::<[u32; 4]>(2).unwrap();
        values.push([1, 3, 5, 7]).expect("test storage admission");
        let pointer = values.as_ptr();
        // SAFETY: identical scalar layout and drop behavior in both directions.
        let moved = unsafe { values.cast::<[u32; 4]>() };
        assert!(!moved.is_owned());
        assert_eq!(moved.as_ptr(), pointer);
        assert_eq!(moved.as_slice(), &[[1, 3, 5, 7]]);
        let owned = ReusableVec::from_owned(alloc::vec![[2_u32, 4, 6, 8]]);
        let pointer = owned.as_ptr();
        // SAFETY: identical scalar layout and drop behavior.
        let moved = unsafe { owned.cast::<[u32; 4]>() };
        assert!(moved.is_owned());
        assert_eq!(moved.as_ptr(), pointer);
        assert_eq!(moved.into_owned_vec(), [[2, 4, 6, 8]]);
    }
}

/// Fallible vector operations shared by owned and fixed-capacity storage.
/// Mutating access remains a slice; no implementation exposes a borrowed Vec.
#[doc(hidden)]
pub trait Growable<T>: core::ops::DerefMut<Target = [T]> {
    fn capacity(&self) -> usize;
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>];
    /// Newly exposed elements must already be initialized; removed elements
    /// become the caller's responsibility. Admission still checks capacity.
    unsafe fn set_len(&mut self, length: usize) -> Result<(), CapacityError>;
    /// Successful prefix elements remain owned when an iterator or admission fails.
    fn extend(&mut self, values: impl IntoIterator<Item = T>) -> Result<(), CapacityError> {
        for value in values {
            self.push(value)?;
        }
        Ok(())
    }
    fn reserve(&mut self, additional: usize) -> Result<(), CapacityError>;
    fn reserve_exact(&mut self, additional: usize) -> Result<(), CapacityError>;
    fn push(&mut self, value: T) -> Result<(), CapacityError>;
    fn pop(&mut self) -> Option<T>;
    fn truncate(&mut self, length: usize);
    fn clear(&mut self) {
        self.truncate(0);
    }
    fn resize(&mut self, length: usize, value: T) -> Result<(), CapacityError>
    where
        T: Clone;
    fn resize_with(
        &mut self,
        length: usize,
        create: impl FnMut() -> T,
    ) -> Result<(), CapacityError>;
    fn extend_from_slice(&mut self, values: &[T]) -> Result<(), CapacityError>
    where
        T: Clone;
}

impl<T> Growable<T> for Vec<T> {
    fn capacity(&self) -> usize {
        Vec::capacity(self)
    }
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        Vec::spare_capacity_mut(self)
    }
    unsafe fn set_len(&mut self, n: usize) -> Result<(), CapacityError> {
        if n > self.capacity() {
            return Err(CapacityError::Exhausted {
                required: n,
                capacity: self.capacity(),
            });
        }
        unsafe { Vec::set_len(self, n) };
        Ok(())
    }
    fn reserve(&mut self, n: usize) -> Result<(), CapacityError> {
        self.len().checked_add(n).ok_or(CapacityError::Overflow)?;
        self.try_reserve(n)
            .map_err(|_| CapacityError::AllocationFailed)
    }
    fn reserve_exact(&mut self, n: usize) -> Result<(), CapacityError> {
        self.len().checked_add(n).ok_or(CapacityError::Overflow)?;
        self.try_reserve_exact(n)
            .map_err(|_| CapacityError::AllocationFailed)
    }
    fn push(&mut self, v: T) -> Result<(), CapacityError> {
        Growable::reserve(self, 1)?;
        Vec::push(self, v);
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        Vec::pop(self)
    }
    fn truncate(&mut self, n: usize) {
        Vec::truncate(self, n);
    }
    fn resize(&mut self, n: usize, v: T) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        Growable::reserve(self, n.saturating_sub(self.len()))?;
        Vec::resize(self, n, v);
        Ok(())
    }
    fn resize_with(&mut self, n: usize, f: impl FnMut() -> T) -> Result<(), CapacityError> {
        Growable::reserve(self, n.saturating_sub(self.len()))?;
        Vec::resize_with(self, n, f);
        Ok(())
    }
    fn extend_from_slice(&mut self, v: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        Growable::reserve(self, v.len())?;
        Vec::extend_from_slice(self, v);
        Ok(())
    }
}

impl<T> Growable<T> for ReusableVec<T> {
    fn capacity(&self) -> usize {
        ReusableVec::capacity(self)
    }
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        ReusableVec::spare_capacity_mut(self)
    }
    unsafe fn set_len(&mut self, n: usize) -> Result<(), CapacityError> {
        unsafe { ReusableVec::set_len(self, n) }
    }
    fn reserve(&mut self, n: usize) -> Result<(), CapacityError> {
        ReusableVec::reserve(self, n)
    }
    fn reserve_exact(&mut self, n: usize) -> Result<(), CapacityError> {
        ReusableVec::reserve_exact(self, n)
    }
    fn push(&mut self, v: T) -> Result<(), CapacityError> {
        ReusableVec::push(self, v)
    }
    fn pop(&mut self) -> Option<T> {
        ReusableVec::pop(self)
    }
    fn truncate(&mut self, n: usize) {
        ReusableVec::truncate(self, n);
    }
    fn resize(&mut self, n: usize, v: T) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        ReusableVec::resize(self, n, v)
    }
    fn resize_with(&mut self, n: usize, f: impl FnMut() -> T) -> Result<(), CapacityError> {
        ReusableVec::resize_with(self, n, f)
    }
    fn extend_from_slice(&mut self, v: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        ReusableVec::extend_from_slice(self, v)
    }
}

/// Gives a bit writer exclusive access to either a real Vec or a fixed facade.
/// The associated type never grants mutable Vec access to borrowed storage.
pub(crate) trait ByteBufferOwner {
    type Buffer: Growable<u8>;
    fn buffer(&mut self) -> &mut Self::Buffer;
}
impl ByteBufferOwner for Vec<u8> {
    type Buffer = Vec<u8>;
    fn buffer(&mut self) -> &mut Self::Buffer {
        self
    }
}
impl ByteBufferOwner for ReusableVec<u8> {
    type Buffer = ReusableVec<u8>;
    fn buffer(&mut self) -> &mut Self::Buffer {
        self
    }
}
impl<B: Growable<u8>> ByteBufferOwner for &mut B {
    type Buffer = B;
    fn buffer(&mut self) -> &mut Self::Buffer {
        self
    }
}
