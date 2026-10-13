use std::{
    ops::Range,
    ptr,
    sync::{Mutex, MutexGuard, OnceLock},
};

use crate::{
    allocation::Allocation,
    arena::Arena,
    canary::{canary_offset, expected_canary, CanarySide, CANARY_LEN},
    config::{allocation_chunk_bytes, SIZE_CLASSES},
    error::{Error, Result},
    memory_region,
    page_permission::PagePermission,
    secure_access::read_access_is_active,
};

static GLOBAL_HEAP: OnceLock<Mutex<SecureHeap>> = OnceLock::new();

pub(crate) struct SecureHeap {
    pub(crate) arenas: Vec<Arena>,
}

impl SecureHeap {
    fn new() -> Self {
        Self { arenas: Vec::new() }
    }

    pub(crate) fn allocate(&mut self, capacity: usize) -> Result<Allocation> {
        self.allocate_with(capacity, &mut memory_region::MemoryRegion::new)
    }

    fn allocate_with(
        &mut self,
        capacity: usize,
        create: &mut impl FnMut(usize) -> Result<memory_region::MemoryRegion>,
    ) -> Result<Allocation> {
        match self.allocate_once(capacity, create) {
            Err(
                error @ (Error::AllocationFailed
                | Error::LockFailed
                | Error::WeakAllocationDisabled),
            ) => {
                let mut released = false;
                for arena in &mut self.arenas {
                    released |= arena.release_empty_backing();
                }
                if released {
                    // One retry with the same allocator and security policy;
                    // live arenas are never released and no weak fallback is added.
                    self.allocate_once(capacity, create)
                } else {
                    Err(error)
                }
            }
            result => result,
        }
    }

    fn allocate_once(
        &mut self,
        capacity: usize,
        create: &mut impl FnMut(usize) -> Result<memory_region::MemoryRegion>,
    ) -> Result<Allocation> {
        let slot_size = slot_size_for(capacity)?;
        for (arena_index, arena) in self.arenas.iter_mut().enumerate() {
            if arena.slot_size == slot_size {
                if let Some(allocation) = arena.allocate(arena_index) {
                    self.write_canaries(allocation)?;
                    return Ok(allocation);
                }
            }
        }

        // Prefer existing backed free slots above before remapping an empty
        // arena of the same class. Its stable slot generations are retained.
        for (arena_index, arena) in self.arenas.iter_mut().enumerate() {
            if arena.slot_size == slot_size && !arena.has_backing() {
                arena.restore_backing(create)?;
                let allocation = arena.allocate(arena_index).ok_or(Error::AllocationFailed)?;
                self.write_canaries(allocation)?;
                return Ok(allocation);
            }
        }

        let data_size = allocation_chunk_bytes().max(Arena::required_data_size(slot_size));
        let mut arena = Arena::new_with(slot_size, data_size, create)?;
        let arena_index = self.arenas.len();
        let allocation = arena.allocate(arena_index).ok_or(Error::AllocationFailed)?;
        self.arenas.push(arena);
        self.write_canaries(allocation)?;
        Ok(allocation)
    }

    pub(crate) fn clone_allocation(
        &mut self,
        allocation: Allocation,
        len: usize,
    ) -> Result<Allocation> {
        self.validate_allocation(allocation)?;
        self.check_canaries(allocation)?;
        if len > allocation.capacity {
            return Err(Error::CapacityOverflow);
        }
        let cloned = self.allocate(len)?;
        if let Err(err) = self.copy_range(allocation, 0, cloned, 0, len) {
            let _ = self.free(cloned);
            return Err(err);
        }
        Ok(cloned)
    }

    pub(crate) fn free(&mut self, allocation: Allocation) -> Result<()> {
        self.validate_allocation(allocation)?;
        self.with_unprotected_allocation(allocation, PagePermission::Write, |ptr| {
            check_canaries_at(allocation, ptr)?;
            // SAFETY: the whole allocation, including canaries, is writable
            // for the duration of this closure.
            unsafe {
                ptr::write_bytes(ptr, 0, allocation.capacity);
                ptr::write_bytes(ptr.sub(CANARY_LEN), 0, CANARY_LEN);
                ptr::write_bytes(ptr.add(allocation.capacity), 0, CANARY_LEN);
            }
            Ok(())
        })??;
        self.arenas[allocation.arena].free(allocation.slot);
        Ok(())
    }

    pub(crate) fn write(
        &mut self,
        allocation: Allocation,
        offset: usize,
        bytes: &[u8],
    ) -> Result<()> {
        self.validate_allocation(allocation)?;
        let end = offset
            .checked_add(bytes.len())
            .ok_or(Error::CapacityOverflow)?;
        if end > allocation.capacity {
            return Err(Error::CapacityOverflow);
        }
        self.with_unprotected_allocation(allocation, PagePermission::Write, |ptr| {
            check_canaries_at(allocation, ptr)?;
            if !bytes.is_empty() {
                // SAFETY: destination is inside a live allocation and was made
                // writable for the duration of this copy.
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(offset), bytes.len());
                }
            }
            Ok(())
        })?
    }

    pub(crate) fn copy(
        &mut self,
        source: Allocation,
        destination: Allocation,
        len: usize,
    ) -> Result<()> {
        self.copy_range(source, 0, destination, 0, len)
    }

    pub(crate) fn copy_range(
        &mut self,
        source: Allocation,
        source_offset: usize,
        destination: Allocation,
        destination_offset: usize,
        len: usize,
    ) -> Result<()> {
        self.validate_allocation(source)?;
        self.validate_allocation(destination)?;
        self.check_canaries(source)?;
        self.check_canaries(destination)?;
        let source_end = source_offset
            .checked_add(len)
            .ok_or(Error::CapacityOverflow)?;
        let destination_end = destination_offset
            .checked_add(len)
            .ok_or(Error::CapacityOverflow)?;
        if source_end > source.capacity || destination_end > destination.capacity {
            return Err(Error::CapacityOverflow);
        }
        if len == 0 {
            return Ok(());
        }
        let source_pages =
            self.unprotect_pages(source, source_offset, len, PagePermission::Read)?;
        let source_guard = PageReprotectGuard::new(self, source.arena, source_pages);
        let destination_pages =
            self.unprotect_pages(destination, destination_offset, len, PagePermission::Write)?;
        let destination_guard = PageReprotectGuard::new(self, destination.arena, destination_pages);
        // SAFETY: source and destination are live allocations, both ranges were
        // unprotected with suitable permissions, and the allocator never
        // returns overlapping live slots.
        unsafe {
            ptr::copy_nonoverlapping(
                self.ptr_for(source).add(source_offset),
                self.ptr_for(destination).add(destination_offset),
                len,
            );
        }
        destination_guard.protect()?;
        source_guard.protect()
    }

    pub(crate) fn read_byte(&mut self, allocation: Allocation, offset: usize) -> Result<u8> {
        self.validate_allocation(allocation)?;
        if offset >= allocation.capacity {
            return Err(Error::CapacityOverflow);
        }
        self.with_unprotected_allocation(allocation, PagePermission::Read, |ptr| {
            check_canaries_at(allocation, ptr)?;
            // SAFETY: the byte is inside a live allocation and was made
            // readable for the duration of this read.
            Ok(unsafe { *ptr.add(offset) })
        })?
    }

    pub(crate) fn zero_range(
        &mut self,
        allocation: Allocation,
        offset: usize,
        len: usize,
    ) -> Result<()> {
        self.validate_allocation(allocation)?;
        let end = offset.checked_add(len).ok_or(Error::CapacityOverflow)?;
        if end > allocation.capacity {
            return Err(Error::CapacityOverflow);
        }
        self.with_unprotected_allocation(allocation, PagePermission::Write, |ptr| {
            check_canaries_at(allocation, ptr)?;
            if len != 0 {
                // SAFETY: the range is inside a live allocation and was made
                // writable for the duration of this zeroization.
                unsafe {
                    ptr::write_bytes(ptr.add(offset), 0, len);
                }
            }
            Ok(())
        })?
    }

    pub(crate) fn with_mut_slice<R>(
        &mut self,
        allocation: Allocation,
        len: usize,
        f: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R> {
        self.validate_allocation(allocation)?;
        if len > allocation.capacity {
            return Err(Error::CapacityOverflow);
        }
        self.with_unprotected_allocation(allocation, PagePermission::Write, |ptr| {
            check_canaries_at(allocation, ptr)?;
            // SAFETY: the slice is inside a live allocation, and the allocation
            // is writable for exactly the duration of this closure.
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
            Ok(f(slice))
        })?
    }

    pub(crate) fn ptr_for(&self, allocation: Allocation) -> *mut u8 {
        self.arenas[allocation.arena].ptr_for_offset(allocation.offset)
    }

    pub(crate) fn validate_allocation(&self, allocation: Allocation) -> Result<()> {
        let arena = self
            .arenas
            .get(allocation.arena)
            .ok_or(Error::CorruptAllocation)?;
        if arena.is_live_generation(allocation.slot, allocation.generation) {
            Ok(())
        } else {
            Err(Error::CorruptAllocation)
        }
    }

    fn unprotect_pages(
        &mut self,
        allocation: Allocation,
        offset: usize,
        len: usize,
        permission: PagePermission,
    ) -> Result<Range<usize>> {
        let page_size = memory_region::page_size();
        let start = (allocation.offset + offset) / page_size;
        let end = (allocation.offset + offset + len).div_ceil(page_size);
        for page in start..end {
            self.arenas[allocation.arena].protect_page(page, permission)?;
        }
        Ok(start..end)
    }

    fn protect_arena_pages_none(&mut self, arena_index: usize, pages: Range<usize>) -> Result<()> {
        for page in pages.rev() {
            self.arenas[arena_index].protect_page_none(page)?;
        }
        Ok(())
    }

    fn write_canaries(&mut self, allocation: Allocation) -> Result<()> {
        self.with_unprotected_allocation(allocation, PagePermission::Write, |ptr| {
            write_canaries_at(allocation, ptr);
        })
    }

    pub(crate) fn check_canaries(&mut self, allocation: Allocation) -> Result<()> {
        self.with_unprotected_allocation(allocation, PagePermission::Read, |ptr| {
            check_canaries_at(allocation, ptr)
        })?
    }

    fn with_unprotected_absolute_range<R>(
        &mut self,
        arena_index: usize,
        offset: usize,
        len: usize,
        permission: PagePermission,
        f: impl FnOnce(*mut u8) -> R,
    ) -> Result<R> {
        let page_size = memory_region::page_size();
        let end_offset = offset.checked_add(len).ok_or(Error::CapacityOverflow)?;
        let start = offset / page_size;
        let end = end_offset.div_ceil(page_size);
        for page in start..end {
            self.arenas[arena_index].protect_page(page, permission)?;
        }
        let guard = PageReprotectGuard::new(self, arena_index, start..end);
        let result = f(self.arenas[arena_index].ptr_for_offset(offset));
        guard.protect()?;
        Ok(result)
    }

    fn with_unprotected_allocation<R>(
        &mut self,
        allocation: Allocation,
        permission: PagePermission,
        f: impl FnOnce(*mut u8) -> R,
    ) -> Result<R> {
        let start = canary_offset(allocation, CanarySide::Before);
        let end = canary_offset(allocation, CanarySide::After)
            .checked_add(CANARY_LEN)
            .ok_or(Error::CapacityOverflow)?;
        let user_offset = allocation
            .offset
            .checked_sub(start)
            .ok_or(Error::CorruptAllocation)?;
        self.with_unprotected_absolute_range(
            allocation.arena,
            start,
            end - start,
            permission,
            |ptr| {
                // SAFETY: `user_offset` points from the beginning of the
                // unprotected slot span to the allocation's user bytes.
                unsafe { f(ptr.add(user_offset)) }
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn canaries_intact_for_test(&mut self, allocation: Allocation) -> bool {
        self.check_canaries(allocation).is_ok()
    }

    #[cfg(test)]
    pub(crate) fn corrupt_after_canary_for_test(&mut self, allocation: Allocation) {
        let offset = canary_offset(allocation, CanarySide::After);
        self.with_unprotected_absolute_range(
            allocation.arena,
            offset,
            1,
            PagePermission::Write,
            |ptr| {
                // SAFETY: the test intentionally modifies the allocation's
                // canary byte after making that canary page writable.
                unsafe {
                    *ptr ^= 0xff;
                }
            },
        )
        .expect("test canary corruption");
    }

    #[cfg(test)]
    pub(crate) fn restore_canaries_for_test(&mut self, allocation: Allocation) {
        self.write_canaries(allocation)
            .expect("test canary restoration");
    }
}

pub(crate) fn lock_secure_heap() -> MutexGuard<'static, SecureHeap> {
    match GLOBAL_HEAP
        .get_or_init(|| Mutex::new(SecureHeap::new()))
        .lock()
    {
        Ok(heap) => heap,
        Err(_) => std::process::abort(),
    }
}

pub(crate) fn lock_secure_heap_for_mutation() -> Result<MutexGuard<'static, SecureHeap>> {
    if read_access_is_active() {
        return Err(Error::ReadAccessActive);
    }
    Ok(lock_secure_heap())
}

fn write_canaries_at(allocation: Allocation, ptr: *mut u8) {
    let before = expected_canary(allocation, CanarySide::Before);
    let after = expected_canary(allocation, CanarySide::After);
    // SAFETY: callers only invoke this while the allocation's full slot span,
    // including both canaries, is writable.
    unsafe {
        ptr::copy_nonoverlapping(before.as_ptr(), ptr.sub(CANARY_LEN), CANARY_LEN);
        ptr::copy_nonoverlapping(after.as_ptr(), ptr.add(allocation.capacity), CANARY_LEN);
    }
}

fn check_canaries_at(allocation: Allocation, ptr: *const u8) -> Result<()> {
    check_canary_at(allocation, ptr, CanarySide::Before)?;
    check_canary_at(allocation, ptr, CanarySide::After)
}

fn check_canary_at(allocation: Allocation, ptr: *const u8, side: CanarySide) -> Result<()> {
    let canary_ptr = match side {
        CanarySide::Before => ptr.wrapping_sub(CANARY_LEN),
        CanarySide::After => ptr.wrapping_add(allocation.capacity),
    };
    let mut actual = [0u8; CANARY_LEN];
    // SAFETY: callers only invoke this while the allocation's full slot span,
    // including both canaries, is readable.
    unsafe {
        ptr::copy_nonoverlapping(canary_ptr, actual.as_mut_ptr(), CANARY_LEN);
    }
    if actual == expected_canary(allocation, side) {
        Ok(())
    } else {
        Err(Error::CorruptAllocation)
    }
}

struct PageReprotectGuard {
    heap: *mut SecureHeap,
    arena_index: usize,
    pages: Range<usize>,
    active: bool,
}

impl PageReprotectGuard {
    fn new(heap: &mut SecureHeap, arena_index: usize, pages: Range<usize>) -> Self {
        Self {
            heap,
            arena_index,
            pages,
            active: true,
        }
    }

    fn protect(mut self) -> Result<()> {
        self.active = false;
        // SAFETY: the guard is created from the active heap borrow, and callers
        // invoke this before resuming normal heap access.
        unsafe { (&mut *self.heap).protect_arena_pages_none(self.arena_index, self.pages.clone()) }
    }
}

impl Drop for PageReprotectGuard {
    fn drop(&mut self) {
        if self.active {
            // SAFETY: this is panic cleanup for pages unprotected through this
            // guard. Errors are ignored because Drop cannot report them.
            let _ = unsafe {
                (&mut *self.heap).protect_arena_pages_none(self.arena_index, self.pages.clone())
            };
        }
    }
}

fn slot_size_for(capacity: usize) -> Result<usize> {
    if capacity == 0 {
        return Ok(64);
    }
    if let Some(size) = SIZE_CLASSES.iter().copied().find(|size| capacity <= *size) {
        return Ok(size);
    }
    let page_size = memory_region::page_size();
    capacity
        .checked_add(page_size - 1)
        .map(|size| size / page_size * page_size)
        .ok_or(Error::CapacityOverflow)
}

#[cfg(test)]
mod reclamation_tests {
    use super::*;
    use crate::memory_region::MemoryRegion;

    #[test]
    fn pressure_reclaims_only_empty_backing_and_preserves_live_and_stale_identity() {
        let mut heap = SecureHeap::new();
        // Distinct classes even on platforms with 16 KiB OS pages.
        let stale_capacity = 8192.max(memory_region::page_size());
        let next_capacity = stale_capacity * 2;
        let live = heap.allocate(64).unwrap();
        heap.write(live, 0, b"retained").unwrap();
        let stale = heap.allocate(stale_capacity).unwrap();
        heap.write(stale, 0, b"old secret").unwrap();
        heap.free(stale).unwrap();
        let mut calls = 0;
        let next = heap
            .allocate_with(next_capacity, &mut |size| {
                calls += 1;
                if calls == 1 {
                    Err(Error::WeakAllocationDisabled)
                } else {
                    MemoryRegion::new(size)
                }
            })
            .unwrap();
        assert_eq!(calls, 2);
        assert!(!heap.arenas[stale.arena].has_backing());
        assert!(heap.arenas[live.arena].has_backing());
        assert_eq!(
            heap.with_mut_slice(live, 8, |bytes| bytes.to_vec())
                .unwrap(),
            b"retained"
        );
        heap.check_canaries(live).unwrap();
        assert_eq!(
            heap.validate_allocation(stale),
            Err(Error::CorruptAllocation)
        );
        let restored = heap.allocate(stale_capacity).unwrap();
        assert_eq!(restored.arena, stale.arena);
        assert_eq!(restored.slot, stale.slot);
        assert_ne!(restored.generation, stale.generation);
        assert_eq!(
            heap.validate_allocation(stale),
            Err(Error::CorruptAllocation)
        );
        assert!(heap
            .with_mut_slice(restored, stale_capacity, |bytes| bytes
                .iter()
                .all(|b| *b == 0))
            .unwrap());
        heap.check_canaries(restored).unwrap();
        heap.free(restored).unwrap();
        heap.free(next).unwrap();
        heap.free(live).unwrap();
    }

    #[test]
    fn pressure_retry_is_bounded_and_failed_rehydration_keeps_metadata_reusable() {
        let mut heap = SecureHeap::new();
        // Distinct classes even on platforms with 16 KiB OS pages.
        let stale_capacity = 8192.max(memory_region::page_size());
        let next_capacity = stale_capacity * 2;
        let live = heap.allocate(64).unwrap();
        heap.write(live, 0, b"alive").unwrap();
        let stale = heap.allocate(stale_capacity).unwrap();
        heap.free(stale).unwrap();
        let mut calls = 0;
        let error = heap
            .allocate_with(next_capacity, &mut |_| {
                calls += 1;
                Err(Error::WeakAllocationDisabled)
            })
            .unwrap_err();
        assert_eq!(error, Error::WeakAllocationDisabled);
        assert_eq!(calls, 2);
        assert!(!heap.arenas[stale.arena].has_backing());
        let mut attempts = 0;
        assert_eq!(
            heap.allocate_with(stale_capacity, &mut |_| {
                attempts += 1;
                Err(Error::WeakAllocationDisabled)
            })
            .unwrap_err(),
            Error::WeakAllocationDisabled
        );
        assert_eq!(attempts, 1); // No empty backing remains to release.
        assert_eq!(
            heap.validate_allocation(stale),
            Err(Error::CorruptAllocation)
        );
        let restored = heap.allocate(stale_capacity).unwrap();
        assert_eq!(restored.arena, stale.arena);
        assert_ne!(restored.generation, stale.generation);
        assert_eq!(
            heap.with_mut_slice(live, 5, |bytes| bytes.to_vec())
                .unwrap(),
            b"alive"
        );
        heap.check_canaries(live).unwrap();
        heap.free(restored).unwrap();
        heap.free(live).unwrap();
    }

    #[test]
    fn backed_reuse_precedes_remapping_and_other_errors_do_not_reclaim() {
        let mut heap = SecureHeap::new();
        let first = heap.allocate(65536).unwrap();
        let second = heap.allocate(65536).unwrap();
        heap.free(first).unwrap();
        assert!(heap.arenas[first.arena].release_empty_backing());
        heap.free(second).unwrap();
        let reused = heap
            .allocate_with(65536, &mut |_| panic!("must reuse backed slot"))
            .unwrap();
        assert_eq!(reused.arena, second.arena);
        heap.free(reused).unwrap();
        let mut attempts = 0;
        assert_eq!(
            heap.allocate_with(131072, &mut |_| {
                attempts += 1;
                Err(Error::ProtectionFailed)
            })
            .unwrap_err(),
            Error::ProtectionFailed
        );
        assert_eq!(attempts, 1);
        assert!(heap.arenas[second.arena].has_backing());
    }
}
