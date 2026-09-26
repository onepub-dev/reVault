use alloc::vec::Vec;

use super::{
    cctx_params::{LdmParameters, ParamSwitch},
    ldm::{
        sequence::{
            fill_prefix_hash_table, generate_sequences_no_dict, generate_sequences_with_prefix,
        },
        LdmHashTable,
    },
};

#[test]
fn ldm_no_dict_rejects_matches_beyond_window_like_c() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 10,
        hash_log: 10,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 0,
    };
    let max_distance = 1_u32 << params.window_log;
    let mut data = Vec::with_capacity(4096);
    let mut value = 0x9e37_79b9_u32;
    for _ in 0..4000 {
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        data.push(value as u8);
    }

    let old_repeat = data[128..192].to_vec();
    data[3800..3864].copy_from_slice(&old_repeat);
    let near_repeat = data[3200..3264].to_vec();
    data[3900..3964].copy_from_slice(&near_repeat);
    let mut table = LdmHashTable::new(params);

    let result = generate_sequences_no_dict(&data, params, &mut table);

    assert!(!result.sequences.is_empty());
    assert!(
        result
            .sequences
            .iter()
            .all(|sequence| sequence.offset <= max_distance),
        "{:?}",
        result.sequences
    );
}

#[test]
fn ldm_dictionary_match_can_exceed_window_until_dictionary_expires_like_c() {
    let params = LdmParameters {
        enable_ldm: ParamSwitch::Enable,
        window_log: 10,
        hash_log: 16,
        min_match_length: 16,
        bucket_size_log: 3,
        hash_rate_log: 0,
    };
    let max_distance = 1_u32 << params.window_log;
    let mut dictionary = Vec::with_capacity(2048);
    let mut value = 0xa5a5_1234_u32;
    for _ in 0..2048 {
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        dictionary.push(value as u8);
    }
    let repeat = dictionary[128..192].to_vec();
    let mut combined = dictionary;
    let source_start = combined.len();
    combined.extend_from_slice(&repeat);
    let mut table = LdmHashTable::new(params);

    fill_prefix_hash_table(&combined, 0..source_start, params, &mut table);
    let result =
        generate_sequences_with_prefix(&combined, source_start..combined.len(), params, &mut table);

    assert!(
        result
            .sequences
            .iter()
            .any(|sequence| sequence.offset > max_distance),
        "{:?}",
        result.sequences
    );
}
