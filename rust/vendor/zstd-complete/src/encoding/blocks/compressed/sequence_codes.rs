#[derive(Clone, Copy)]
struct SmallLengthCode {
    add_bits: u32,
    code: u8,
    num_bits: u8,
}

const _: () = assert!(core::mem::size_of::<SmallLengthCode>() == 8);

const LITERAL_LENGTH_SMALL_CODES: [SmallLengthCode; 64] = small_literal_length_codes();
const MATCH_LENGTH_SMALL_CODES: [SmallLengthCode; 128] = small_match_length_codes();

#[inline(always)]
pub(crate) fn literal_length_code(len: u32) -> u8 {
    encode_literal_length(len).0
}

#[inline(always)]
pub(crate) fn match_length_code(len: u32) -> u8 {
    encode_match_len(len).0
}

#[inline(always)]
pub(crate) fn offset_code(offset_value: u32) -> u8 {
    encode_offset(offset_value).0
}

#[inline(always)]
pub(super) fn encode_literal_length(len: u32) -> (u8, u32, usize) {
    if len < LITERAL_LENGTH_SMALL_CODES.len() as u32 {
        let entry = LITERAL_LENGTH_SMALL_CODES[len as usize];
        return (entry.code, entry.add_bits, usize::from(entry.num_bits));
    }

    match len {
        0..=63 => unreachable!(),
        64..=127 => (25, len - 64, 6),
        128..=255 => (26, len - 128, 7),
        256..=511 => (27, len - 256, 8),
        512..=1023 => (28, len - 512, 9),
        1024..=2047 => (29, len - 1024, 10),
        2048..=4095 => (30, len - 2048, 11),
        4096..=8191 => (31, len - 4096, 12),
        8192..=16383 => (32, len - 8192, 13),
        16384..=32767 => (33, len - 16384, 14),
        32768..=65535 => (34, len - 32768, 15),
        65536..=131071 => (35, len - 65536, 16),
        131072.. => unreachable!(),
    }
}

#[inline(always)]
pub(super) fn encode_match_len(len: u32) -> (u8, u32, usize) {
    if (3..=130).contains(&len) {
        let entry = MATCH_LENGTH_SMALL_CODES[(len - 3) as usize];
        return (entry.code, entry.add_bits, usize::from(entry.num_bits));
    }

    match len {
        0..=2 => unreachable!(),
        3..=130 => unreachable!(),
        131..=258 => (43, len - 131, 7),
        259..=514 => (44, len - 259, 8),
        515..=1026 => (45, len - 515, 9),
        1027..=2050 => (46, len - 1027, 10),
        2051..=4098 => (47, len - 2051, 11),
        4099..=8194 => (48, len - 4099, 12),
        8195..=16386 => (49, len - 8195, 13),
        16387..=32770 => (50, len - 16387, 14),
        32771..=65538 => (51, len - 32771, 15),
        65539..=131074 => (52, len - 65539, 16),
        131075.. => unreachable!(),
    }
}

const fn small_literal_length_codes() -> [SmallLengthCode; 64] {
    let mut codes = [SmallLengthCode {
        add_bits: 0,
        code: 0,
        num_bits: 0,
    }; 64];
    let mut len = 0usize;
    while len < codes.len() {
        let (code, add_bits, num_bits) = match len {
            0..=15 => (len as u8, 0, 0),
            16..=17 => (16, len as u32 - 16, 1),
            18..=19 => (17, len as u32 - 18, 1),
            20..=21 => (18, len as u32 - 20, 1),
            22..=23 => (19, len as u32 - 22, 1),
            24..=27 => (20, len as u32 - 24, 2),
            28..=31 => (21, len as u32 - 28, 2),
            32..=39 => (22, len as u32 - 32, 3),
            40..=47 => (23, len as u32 - 40, 3),
            48..=63 => (24, len as u32 - 48, 4),
            _ => unreachable!(),
        };
        codes[len] = SmallLengthCode {
            add_bits,
            code,
            num_bits: num_bits as u8,
        };
        len += 1;
    }
    codes
}

const fn small_match_length_codes() -> [SmallLengthCode; 128] {
    let mut codes = [SmallLengthCode {
        add_bits: 0,
        code: 0,
        num_bits: 0,
    }; 128];
    let mut idx = 0usize;
    while idx < codes.len() {
        let len = idx + 3;
        let (code, add_bits, num_bits) = match len {
            3..=34 => (len as u8 - 3, 0, 0),
            35..=36 => (32, len as u32 - 35, 1),
            37..=38 => (33, len as u32 - 37, 1),
            39..=40 => (34, len as u32 - 39, 1),
            41..=42 => (35, len as u32 - 41, 1),
            43..=46 => (36, len as u32 - 43, 2),
            47..=50 => (37, len as u32 - 47, 2),
            51..=58 => (38, len as u32 - 51, 3),
            59..=66 => (39, len as u32 - 59, 3),
            67..=82 => (40, len as u32 - 67, 4),
            83..=98 => (41, len as u32 - 83, 4),
            99..=130 => (42, len as u32 - 99, 5),
            _ => unreachable!(),
        };
        codes[idx] = SmallLengthCode {
            add_bits,
            code,
            num_bits: num_bits as u8,
        };
        idx += 1;
    }
    codes
}

#[inline(always)]
pub(super) fn encode_offset(len: u32) -> (u8, u32, usize) {
    let log = len.ilog2();
    let lower = len & ((1 << log) - 1);
    (log as u8, lower, log as usize)
}
