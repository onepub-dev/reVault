use crate::checked::{read_u32_le, read_u64_le};
use crate::free_slot::FreeSlot;
use crate::{Error, Result};

const VERSION: u8 = 1;
const HEADER_LEN: usize = 32;
const ENTRY_LEN: usize = 16;
pub(crate) const MAX_REDACTION_RANGES: usize = 65_536;
pub(crate) const MAX_REDACTION_OBJECT_IDS: usize = 262_144;
pub(crate) const RANGES_PER_PAGE: usize = 4_096;
pub(crate) const MAX_REDACTION_PAGES: usize = MAX_REDACTION_RANGES.div_ceil(RANGES_PER_PAGE);
pub(crate) const MAX_REDACTION_MANIFEST_PAYLOAD_BYTES: usize =
    MAX_REDACTION_PAGES * (HEADER_LEN + RANGES_PER_PAGE * ENTRY_LEN);
pub(crate) const MAX_REDACTION_TOTAL_BYTES: u64 = 1 << 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RedactionManifestPage {
    pub(crate) transaction_sequence: u64,
    pub(crate) next_page_offset: u64,
    pub(crate) total_range_count: u32,
    pub(crate) ranges: Vec<FreeSlot>,
}

pub(crate) fn encode_page(page: &RedactionManifestPage) -> Result<Vec<u8>> {
    if page.ranges.is_empty()
        || page.ranges.len() > RANGES_PER_PAGE
        || page.total_range_count as usize > MAX_REDACTION_RANGES
        || page.ranges.len() > page.total_range_count as usize
    {
        return Err(Error::SecurityLimitExceeded(
            "redaction manifest range count exceeds its bounded format".to_string(),
        ));
    }
    validate_ranges(&page.ranges)?;
    let mut out = Vec::with_capacity(HEADER_LEN + page.ranges.len() * ENTRY_LEN);
    out.push(VERSION);
    out.extend_from_slice(&[0; 3]);
    out.extend_from_slice(&(page.ranges.len() as u32).to_le_bytes());
    out.extend_from_slice(&page.total_range_count.to_le_bytes());
    out.extend_from_slice(&page.transaction_sequence.to_le_bytes());
    out.extend_from_slice(&page.next_page_offset.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    for range in &page.ranges {
        out.extend_from_slice(&range.offset.to_le_bytes());
        out.extend_from_slice(&range.len.to_le_bytes());
    }
    Ok(out)
}

pub(crate) fn decode_page(payload: &[u8]) -> Result<RedactionManifestPage> {
    if payload.len() < HEADER_LEN || payload[0] != VERSION || payload[1..4] != [0; 3] {
        return Err(Error::CorruptRecord);
    }
    let count = read_u32_le(&payload[4..8])? as usize;
    let total_range_count = read_u32_le(&payload[8..12])?;
    let expected_len = HEADER_LEN
        .checked_add(count.checked_mul(ENTRY_LEN).ok_or(Error::CorruptRecord)?)
        .ok_or(Error::CorruptRecord)?;
    if payload.len() != expected_len
        || count == 0
        || count > RANGES_PER_PAGE
        || total_range_count as usize > MAX_REDACTION_RANGES
        || count > total_range_count as usize
        || payload[28..32].iter().any(|byte| *byte != 0)
    {
        return Err(Error::CorruptRecord);
    }
    let transaction_sequence = read_u64_le(&payload[12..20])?;
    let next_page_offset = read_u64_le(&payload[20..28])?;
    let mut ranges = Vec::with_capacity(count);
    let mut cursor = HEADER_LEN;
    for _ in 0..count {
        ranges.push(FreeSlot {
            offset: read_u64_le(&payload[cursor..cursor + 8])?,
            len: read_u64_le(&payload[cursor + 8..cursor + 16])?,
        });
        cursor += ENTRY_LEN;
    }
    validate_ranges(&ranges)?;
    Ok(RedactionManifestPage {
        transaction_sequence,
        next_page_offset,
        total_range_count,
        ranges,
    })
}

pub(crate) fn bounded_ranges(ranges: &[FreeSlot]) -> Result<Vec<FreeSlot>> {
    if ranges.len() > MAX_REDACTION_RANGES {
        return Err(Error::SecurityLimitExceeded(format!(
            "redaction transaction has {} ranges; maximum is {MAX_REDACTION_RANGES}",
            ranges.len()
        )));
    }
    let mut ordered = ranges.to_vec();
    ordered.sort_by_key(|range| range.offset);
    let mut coalesced: Vec<FreeSlot> = Vec::with_capacity(ordered.len());
    for range in ordered {
        if range.len == 0 || range.offset.checked_add(range.len).is_none() {
            return Err(Error::CorruptRecord);
        }
        if let Some(previous) = coalesced.last_mut() {
            let previous_end = previous.offset + previous.len;
            if range.offset <= previous_end {
                let end = previous_end.max(range.offset + range.len);
                previous.len = end - previous.offset;
                continue;
            }
        }
        coalesced.push(range);
    }
    if coalesced.len() > MAX_REDACTION_RANGES {
        return Err(Error::SecurityLimitExceeded(
            "coalesced redaction manifest exceeds its range limit".to_string(),
        ));
    }
    Ok(coalesced)
}

fn validate_ranges(ranges: &[FreeSlot]) -> Result<()> {
    let mut previous_end = None;
    for range in ranges {
        let end = range
            .offset
            .checked_add(range.len)
            .filter(|_| range.len != 0)
            .ok_or(Error::CorruptRecord)?;
        if previous_end.is_some_and(|previous| range.offset < previous) {
            return Err(Error::CorruptRecord);
        }
        previous_end = Some(end);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_page_round_trips() {
        let page = RedactionManifestPage {
            transaction_sequence: 7,
            next_page_offset: 900,
            total_range_count: 2,
            ranges: vec![
                FreeSlot {
                    offset: 300,
                    len: 100,
                },
                FreeSlot {
                    offset: 800,
                    len: 200,
                },
            ],
        };
        assert_eq!(decode_page(&encode_page(&page).unwrap()).unwrap(), page);
    }

    #[test]
    fn adjacent_ranges_are_coalesced() {
        let ranges = bounded_ranges(&[
            FreeSlot {
                offset: 200,
                len: 10,
            },
            FreeSlot {
                offset: 100,
                len: 100,
            },
        ])
        .unwrap();
        assert_eq!(
            ranges,
            vec![FreeSlot {
                offset: 100,
                len: 110
            }]
        );
    }

    #[test]
    fn malformed_overlap_is_rejected_when_decoding() {
        let mut payload = encode_page(&RedactionManifestPage {
            transaction_sequence: 1,
            next_page_offset: 0,
            total_range_count: 2,
            ranges: vec![
                FreeSlot { offset: 1, len: 2 },
                FreeSlot { offset: 3, len: 2 },
            ],
        })
        .unwrap();
        payload[HEADER_LEN + ENTRY_LEN..HEADER_LEN + ENTRY_LEN + 8]
            .copy_from_slice(&2u64.to_le_bytes());
        assert!(matches!(decode_page(&payload), Err(Error::CorruptRecord)));
    }
}
