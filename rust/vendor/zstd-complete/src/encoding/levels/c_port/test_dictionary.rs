use alloc::vec::Vec;

use crate::decoding::dictionary::MAGIC_NUM;

pub(crate) const DICT_ID: u32 = 0x4723_2101;

pub(crate) fn full_dictionary_fixture() -> Vec<u8> {
    let mut raw = Vec::new();
    raw.extend_from_slice(&MAGIC_NUM);
    raw.extend_from_slice(&DICT_ID.to_le_bytes());
    raw.extend_from_slice(dictionary_tables());
    for offset in [3_u32, 10, 25] {
        raw.extend_from_slice(&offset.to_le_bytes());
    }
    raw.extend_from_slice(dictionary_content());
    raw
}

fn dictionary_tables() -> &'static [u8] {
    &[
        54, 16, 192, 155, 4, 0, 207, 59, 239, 121, 158, 116, 220, 93, 114, 229, 110, 41, 249, 95,
        165, 255, 83, 202, 254, 68, 74, 159, 63, 161, 100, 151, 137, 21, 184, 183, 189, 100, 235,
        209, 251, 174, 91, 75, 91, 185, 19, 39, 75, 146, 98, 177, 249, 14, 4, 35, 0, 0, 0, 40, 40,
        20, 10, 12, 204, 37, 196, 1, 173, 122, 0, 4, 0, 128, 1, 2, 2, 25, 32, 27, 27, 22, 24, 26,
        18, 12, 12, 15, 16, 11, 69, 37, 225, 48, 20, 12, 6, 2, 161, 80, 40, 20, 44, 137, 145, 204,
        46, 0, 0, 0, 0, 0, 116, 253, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

pub(crate) fn dictionary_content() -> &'static [u8] {
    b"method=GET path=/v1/projects/beta status=200 bytes=1847\n\
      method=POST path=/v1/projects/beta status=202 bytes=932\n"
}
