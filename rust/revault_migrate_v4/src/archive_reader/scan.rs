use crate::archive_reader::record::DecodedRecord;

#[derive(Debug)]
pub(crate) struct Scan {
    #[cfg(any(test, feature = "native-block-layout"))]
    pub(crate) native_pages:
        Vec<crate::archive_reader::file_format::indexed_frame::block_page::ScannedBlockPage>,
    pub(crate) records: Vec<DecodedRecord>,
    pub(crate) corrupt_records: usize,
}
