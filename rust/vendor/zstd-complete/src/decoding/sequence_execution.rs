use super::scratch::DecoderScratch;
use crate::decoding::errors::ExecuteSequencesError;

fn block_output_length(
    literals: usize,
    mut matches: impl Iterator<Item = usize>,
) -> Result<usize, ExecuteSequencesError> {
    let length = matches
        .try_fold(literals, |total, length| total.checked_add(length))
        .ok_or(ExecuteSequencesError::BlockOutputTooLarge)?;
    if length > crate::common::MAX_BLOCK_SIZE as usize {
        return Err(ExecuteSequencesError::BlockOutputTooLarge);
    }
    Ok(length)
}

/// Take the provided decoder and execute the sequences stored within
pub fn execute_sequences(scratch: &mut DecoderScratch) -> Result<(), ExecuteSequencesError> {
    // Reject impossible decoded blocks before history writes or offset updates.
    // The bound includes all literal bytes (also the unsequenced final suffix).
    block_output_length(
        scratch.literals_buffer.len(),
        scratch.sequences.iter().map(|seq| seq.ml as usize),
    )?;
    let mut literals_copy_counter = 0;
    let old_buffer_size = scratch.buffer.len();
    let mut seq_sum = 0;

    for idx in 0..scratch.sequences.len() {
        let seq = scratch.sequences[idx];

        if seq.ll > 0 {
            let high = literals_copy_counter + seq.ll as usize;
            if high > scratch.literals_buffer.len() {
                return Err(ExecuteSequencesError::NotEnoughBytesForSequence {
                    wanted: high,
                    have: scratch.literals_buffer.len(),
                });
            }
            let literals = &scratch.literals_buffer[literals_copy_counter..high];
            literals_copy_counter += seq.ll as usize;

            scratch.buffer.push(literals);
        }

        let actual_offset = do_offset_history(seq.of, seq.ll, &mut scratch.offset_hist);
        if actual_offset == 0 {
            return Err(ExecuteSequencesError::ZeroOffset);
        }
        if seq.ml > 0 {
            scratch
                .buffer
                .repeat(actual_offset as usize, seq.ml as usize)?;
        }

        seq_sum += seq.ml;
        seq_sum += seq.ll;
    }
    if literals_copy_counter < scratch.literals_buffer.len() {
        let rest_literals = &scratch.literals_buffer[literals_copy_counter..];
        scratch.buffer.push(rest_literals);
        seq_sum += rest_literals.len() as u32;
    }

    let diff = scratch.buffer.len() - old_buffer_size;
    assert!(
        seq_sum as usize == diff,
        "Seq_sum: {} is different from the difference in buffersize: {}",
        seq_sum,
        diff
    );
    Ok(())
}

/// Update the most recently used offsets to reflect the provided offset value, and return the
/// "actual" offset needed because offsets are not stored in a raw way, some transformations are needed
/// before you get a functional number.
fn do_offset_history(offset_value: u32, lit_len: u32, scratch: &mut [u32; 3]) -> u32 {
    let actual_offset = if lit_len > 0 {
        match offset_value {
            1..=3 => scratch[offset_value as usize - 1],
            _ => {
                //new offset
                offset_value - 3
            }
        }
    } else {
        match offset_value {
            1..=2 => scratch[offset_value as usize],
            3 => scratch[0] - 1,
            _ => {
                //new offset
                offset_value - 3
            }
        }
    };

    //update history
    if lit_len > 0 {
        match offset_value {
            1 => {
                //nothing
            }
            2 => {
                scratch[1] = scratch[0];
                scratch[0] = actual_offset;
            }
            _ => {
                scratch[2] = scratch[1];
                scratch[1] = scratch[0];
                scratch[0] = actual_offset;
            }
        }
    } else {
        match offset_value {
            1 => {
                scratch[1] = scratch[0];
                scratch[0] = actual_offset;
            }
            2 => {
                scratch[2] = scratch[1];
                scratch[1] = scratch[0];
                scratch[0] = actual_offset;
            }
            _ => {
                scratch[2] = scratch[1];
                scratch[1] = scratch[0];
                scratch[0] = actual_offset;
            }
        }
    }

    actual_offset
}

#[cfg(test)]
mod bound_tests {
    use super::*;
    use crate::blocks::sequence_section::Sequence;
    use crate::workspace::Arena;
    use alloc::vec;
    use core::mem::MaybeUninit;

    fn exercise(scratch: &mut DecoderScratch) {
        let maximum = crate::common::MAX_BLOCK_SIZE;
        for matches in [maximum - 1, maximum, u32::MAX] {
            scratch.reset(0);
            scratch.literals_buffer.extend_from_slice(b"x");
            scratch.sequences.push(Sequence {
                ll: 1,
                ml: matches,
                of: 1,
            });
            let before_offsets = scratch.offset_hist;
            let result = execute_sequences(scratch);
            if matches == maximum - 1 {
                result.unwrap();
                let decoded = scratch.buffer.drain();
                assert_eq!(decoded.len(), maximum as usize);
                assert!(decoded.iter().all(|byte| *byte == b'x'));
            } else {
                assert!(matches!(
                    result,
                    Err(ExecuteSequencesError::BlockOutputTooLarge)
                ));
                assert_eq!(scratch.buffer.len(), 0);
                assert_eq!(scratch.offset_hist, before_offsets);
            }
        }
    }

    #[test]
    fn sequence_block_limits_precede_owned_and_static_history_mutation() {
        exercise(&mut DecoderScratch::new(0));
        let required = DecoderScratch::workspace_size(1, 0).unwrap();
        let mut memory = vec![MaybeUninit::new(0x6d); required + 32];
        {
            let mut arena = Arena::new(&mut memory[16..required + 16]);
            let mut scratch = DecoderScratch::new_in(&mut arena, 1, 0).unwrap();
            exercise(&mut scratch);
        }
        for byte in memory[..16].iter().chain(&memory[required + 16..]) {
            // SAFETY: canaries were initialized and are outside the arena.
            assert_eq!(unsafe { byte.assume_init() }, 0x6d);
        }
    }

    #[test]
    fn sequence_block_length_rejects_arithmetic_overflow() {
        assert!(matches!(
            block_output_length(usize::MAX, [1].iter().copied()),
            Err(ExecuteSequencesError::BlockOutputTooLarge)
        ));
        assert!(matches!(
            block_output_length(0, [usize::MAX, 1].iter().copied()),
            Err(ExecuteSequencesError::BlockOutputTooLarge)
        ));
        assert_eq!(
            block_output_length(1, [131071].iter().copied()).unwrap(),
            131072
        );
    }
}
