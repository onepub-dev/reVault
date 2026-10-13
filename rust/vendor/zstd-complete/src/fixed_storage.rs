use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull, slice};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityError {
    Overflow,
    AllocationFailed,
    Exhausted { required: usize, capacity: usize },
}

pub(crate) struct FixedStorage<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    marker: PhantomData<T>,
}

// SAFETY: construction grants exclusive backing access for the entire storage
// lifetime. Moving the owner transfers that obligation; shared access exposes
// only initialized shared references. The element bounds match Vec semantics.
unsafe impl<T: Send> Send for FixedStorage<T> {}
unsafe impl<T: Sync> Sync for FixedStorage<T> {}

impl<T> FixedStorage<T> {
    /// The caller retains an exclusive, aligned allocation of `capacity` T
    /// slots through destruction of this value. No slot is initially live.
    pub(crate) unsafe fn new(ptr: NonNull<T>, capacity: usize) -> Self {
        Self {
            ptr,
            len: 0,
            capacity,
            marker: PhantomData,
        }
    }

    /// Transfers the original full-allocation pointer and initialized prefix.
    /// The recipient must retain both destruction and backing-lifetime duties.
    pub(crate) fn into_parts(self) -> (NonNull<T>, usize, usize) {
        let value = core::mem::ManuallyDrop::new(self);
        (value.ptr, value.len, value.capacity)
    }
    pub(crate) fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }
    pub(crate) fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        // SAFETY: precisely this prefix is initialized; self is borrowed.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: exclusive self access covers precisely the live prefix.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub(crate) fn reserve(&self, additional: usize) -> Result<(), CapacityError> {
        let required = self
            .len
            .checked_add(additional)
            .ok_or(CapacityError::Overflow)?;
        if required > self.capacity {
            return Err(CapacityError::Exhausted {
                required,
                capacity: self.capacity,
            });
        }
        Ok(())
    }

    pub(crate) fn push(&mut self, value: T) -> Result<(), CapacityError> {
        self.reserve(1)?;
        // SAFETY: admission precedes the write into the first unused slot.
        unsafe { self.ptr.as_ptr().add(self.len).write(value) };
        self.len += 1;
        Ok(())
    }

    pub(crate) fn extend_from_slice(&mut self, values: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        self.reserve(values.len())?;
        for value in values {
            // A clone panic leaves only previous successful clones recorded.
            let copy = value.clone();
            unsafe { self.ptr.as_ptr().add(self.len).write(copy) };
            self.len += 1;
        }
        Ok(())
    }

    pub(crate) fn resize(&mut self, length: usize, value: T) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        if length <= self.len {
            self.truncate(length);
            return Ok(());
        }
        self.reserve(length - self.len)?;
        while self.len < length {
            let copy = value.clone();
            unsafe { self.ptr.as_ptr().add(self.len).write(copy) };
            self.len += 1;
        }
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(unsafe { self.ptr.as_ptr().add(self.len).read() })
    }

    pub(crate) fn truncate(&mut self, length: usize) {
        while self.len > length {
            // Remove ownership from the live prefix before calling Drop.
            self.len -= 1;
            unsafe { self.ptr.as_ptr().add(self.len).drop_in_place() };
        }
    }

    pub(crate) fn clear(&mut self) {
        self.truncate(0);
    }
    pub(crate) fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast(),
                self.capacity - self.len,
            )
        }
    }
    /// Every newly exposed slot must already hold a valid initialized T.
    pub(crate) unsafe fn set_len(&mut self, length: usize) -> Result<(), CapacityError> {
        if length > self.capacity {
            return Err(CapacityError::Exhausted {
                required: length,
                capacity: self.capacity,
            });
        }
        self.len = length;
        Ok(())
    }
}

impl<T> Drop for FixedStorage<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
