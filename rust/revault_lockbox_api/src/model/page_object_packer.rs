use crate::page::{encoded_object_len, uncompressed_objects_fit, PageObject};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub(crate) struct PackedPageObject<T> {
    pub(crate) object: PageObject,
    pub(crate) context: T,
}

#[derive(Debug, Clone)]
pub(crate) struct PageObjectPacker<T> {
    page_size: usize,
    pending: Vec<PackedPageObject<T>>,
    pending_object_stream_len: usize,
}

impl<T> PageObjectPacker<T> {
    pub(crate) fn new(page_size: usize) -> Self {
        Self {
            page_size,
            pending: Vec::new(),
            pending_object_stream_len: 4,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Transfer payload ownership while retaining the configured page size.
    pub(crate) fn take_pending(&mut self) -> Vec<PackedPageObject<T>> {
        self.pending_object_stream_len = 4;
        std::mem::take(&mut self.pending)
    }

    pub(crate) fn encoded_object_len(&self, object: &PageObject) -> Result<usize> {
        encoded_object_len(object)
    }

    pub(crate) fn fits_encoded_len(&self, encoded_len: usize) -> Result<bool> {
        let stream_len = self
            .pending_object_stream_len
            .checked_add(encoded_len)
            .ok_or_else(|| Error::SecurityLimitExceeded("page is too large".to_string()))?;
        Ok(uncompressed_objects_fit(self.page_size, stream_len))
    }

    pub(crate) fn push_encoded(
        &mut self,
        object: PageObject,
        context: T,
        encoded_len: usize,
    ) -> Result<()> {
        if !self.fits_encoded_len(encoded_len)? {
            return Err(Error::SecurityLimitExceeded(
                "page object does not fit in a page".to_string(),
            ));
        }
        self.pending_object_stream_len = self
            .pending_object_stream_len
            .checked_add(encoded_len)
            .ok_or_else(|| Error::SecurityLimitExceeded("page is too large".to_string()))?;
        self.pending.push(PackedPageObject { object, context });
        Ok(())
    }
}

impl<T> Default for PageObjectPacker<T> {
    fn default() -> Self {
        Self::new(crate::page::DEFAULT_PAGE_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::PageObjectKind;

    #[test]
    fn taking_pending_preserves_payload_ownership_and_reuses_empty_page_capacity() {
        // A context need not implement Clone: taking a page transfers it too.
        struct Context(u64);
        let mut payload = Vec::with_capacity(3500);
        payload.resize(3000, 73);
        let pointer = payload.as_ptr();
        let object = PageObject::new(PageObjectKind::FileData, 41, payload);
        let mut packer = PageObjectPacker::new(4096);
        let encoded_len = packer.encoded_object_len(&object).unwrap();
        assert!(packer.fits_encoded_len(encoded_len).unwrap());
        packer
            .push_encoded(object, Context(17), encoded_len)
            .unwrap();
        assert!(!packer.fits_encoded_len(encoded_len).unwrap());

        let taken = packer.take_pending();
        assert!(packer.is_empty());
        assert!(packer.fits_encoded_len(encoded_len).unwrap());
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].object.id, 41);
        assert_eq!(taken[0].context.0, 17);
        taken[0]
            .object
            .with_payload(|bytes| {
                assert_eq!(bytes.as_ptr(), pointer);
                assert_eq!(bytes, &[73; 3000]);
            })
            .unwrap();

        let next = PageObject::new(PageObjectKind::FileData, 42, vec![29; 3000]);
        packer.push_encoded(next, Context(18), encoded_len).unwrap();
        assert!(!packer.fits_encoded_len(encoded_len).unwrap());
        let next = packer.take_pending();
        assert_eq!(next[0].context.0, 18);
        next[0]
            .object
            .with_payload(|bytes| assert_eq!(bytes, &[29; 3000]))
            .unwrap();
        taken[0]
            .object
            .with_payload(|bytes| assert_eq!(bytes, &[73; 3000]))
            .unwrap();
        assert!(packer.take_pending().is_empty());
    }
}
