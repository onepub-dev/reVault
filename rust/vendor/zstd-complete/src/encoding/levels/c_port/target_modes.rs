use super::superblock::{EntropyTableMode, SequenceEntropyModes};

pub(super) fn basic_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Basic,
        ml: EntropyTableMode::Basic,
        of: EntropyTableMode::Basic,
    }
}

pub(super) fn rle_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Rle,
        ml: EntropyTableMode::Rle,
        of: EntropyTableMode::Rle,
    }
}

pub(super) fn repeat_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Repeat,
        ml: EntropyTableMode::Repeat,
        of: EntropyTableMode::Repeat,
    }
}

pub(super) fn compressed_sequence_modes() -> SequenceEntropyModes {
    SequenceEntropyModes {
        ll: EntropyTableMode::Compressed,
        ml: EntropyTableMode::Compressed,
        of: EntropyTableMode::Compressed,
    }
}

pub(super) fn sequence_modes_clear_previous(modes: SequenceEntropyModes) -> bool {
    matches!(modes.ll, EntropyTableMode::Basic | EntropyTableMode::Rle)
        && matches!(modes.ml, EntropyTableMode::Basic | EntropyTableMode::Rle)
        && matches!(modes.of, EntropyTableMode::Basic | EntropyTableMode::Rle)
}

pub(super) fn sequence_modes_are_mixed(modes: SequenceEntropyModes) -> bool {
    !sequence_modes_are(modes, EntropyTableMode::Basic)
        && !sequence_modes_are(modes, EntropyTableMode::Rle)
        && !sequence_modes_are(modes, EntropyTableMode::Repeat)
        && !sequence_modes_are(modes, EntropyTableMode::Compressed)
}

fn sequence_modes_are(modes: SequenceEntropyModes, mode: EntropyTableMode) -> bool {
    modes.ll == mode && modes.ml == mode && modes.of == mode
}
