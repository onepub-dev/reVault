//! Code for creating a separate content dictionary.
//!
//! Effective dictionaries are up to 1% the size of the complete training body,
//! and are trained on many examples of the original data.
//!
//! Implemented following the paper "Effective construction of
//! Relative Lempel-Ziv Dictionaries", by Kewen Liao, Matthias Petri,
//! Alistair Moffat, and Anthony Wirth

// The algorithm is summarized here
// 1. The text is split into "epochs", or chunks from the original source
// 2. From within each epoch, we select the "segment", or 1 KiB contiguous section
//    that's predicted to be the best option to include in the dictionary. Concatenated,
//    these segments form the dictionary.
//
// This segment scoring algorithm operates as follows:
// For a given epoch:
//  - Run a reservoir sampler over the entire epoch, creating a
//    reservoir of n/t, where `t` is the desired number of occurances
//    we want the most common k-mers to have
//  - Have the ability to estimate
//    the frequency of a given k-mer: `f(w: k-mer)` calculates
//    the frequency of w in the reservoir using a rolling karp-rabin hash
//  - The score of a segment is the sum of `f(w)` called on every kmer within the segment
mod cover;
mod frequency;
mod reservoir;

use crate::dictionary::reservoir::create_sample;
use crate::{
    bit_io::BitWriter,
    decoding::dictionary::MAGIC_NUM,
    fse::fse_encoder::{default_ll_table, default_ml_table, default_of_table},
    huff0::huff0_encoder::HuffmanTable,
};
use core::cmp::Reverse;
use core::fmt;
use cover::*;
use std::{
    boxed::Box,
    collections::{BinaryHeap, HashMap},
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    vec::Vec,
};

/// Options for producing a standard formatted Zstandard dictionary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DictionaryTrainerOptions {
    dictionary_size: usize,
    dictionary_id: u32,
    max_training_bytes: usize,
}

impl DictionaryTrainerOptions {
    /// Creates options for a dictionary no larger than `dictionary_size`.
    pub const fn new(dictionary_size: usize, dictionary_id: u32) -> Self {
        Self {
            dictionary_size,
            dictionary_id,
            max_training_bytes: 64 * 1024 * 1024,
        }
    }

    /// Bounds the amount of sample data retained while training.
    pub const fn with_max_training_bytes(mut self, bytes: usize) -> Self {
        self.max_training_bytes = bytes;
        self
    }

    pub const fn dictionary_size(&self) -> usize {
        self.dictionary_size
    }

    pub const fn dictionary_id(&self) -> u32 {
        self.dictionary_id
    }

    pub const fn max_training_bytes(&self) -> usize {
        self.max_training_bytes
    }
}

/// Failures reported before or during formatted dictionary training.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DictionaryTrainingError {
    EmptySamples,
    ZeroDictionaryId,
    DictionaryTooSmall,
    TrainingLimitTooSmall,
}

impl fmt::Display for DictionaryTrainingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptySamples => "dictionary training requires at least one non-empty sample",
            Self::ZeroDictionaryId => "a formatted dictionary ID must be non-zero",
            Self::DictionaryTooSmall => "dictionary size is too small for formatted metadata",
            Self::TrainingLimitTooSmall => "training byte limit must be greater than zero",
        })
    }
}

impl std::error::Error for DictionaryTrainingError {}

/// Trains a standard formatted dictionary from independent representative samples.
///
/// The content dictionary is selected with the native COVER-style trainer. The
/// literal table is learned from the sample distribution; portable predefined
/// sequence distributions provide complete initial FSE tables. The result has a
/// dictionary ID and can be passed directly to [`crate::encoding::EncoderDictionary`]
/// or [`crate::decoding::Dictionary::decode_dict`].
pub fn train_dictionary<S: AsRef<[u8]>>(
    samples: &[S],
    options: DictionaryTrainerOptions,
) -> Result<Vec<u8>, DictionaryTrainingError> {
    if options.dictionary_id == 0 {
        return Err(DictionaryTrainingError::ZeroDictionaryId);
    }
    if options.max_training_bytes == 0 {
        return Err(DictionaryTrainingError::TrainingLimitTooSmall);
    }

    let mut training = Vec::new();
    let mut literal_counts = [1_usize; 256];
    for sample in samples {
        let sample = sample.as_ref();
        let remaining = options.max_training_bytes.saturating_sub(training.len());
        let retained = &sample[..sample.len().min(remaining)];
        for &byte in retained {
            literal_counts[usize::from(byte)] = literal_counts[usize::from(byte)].saturating_add(1);
        }
        training.extend_from_slice(retained);
        if training.len() == options.max_training_bytes {
            break;
        }
    }
    if training.is_empty() {
        return Err(DictionaryTrainingError::EmptySamples);
    }

    let huffman = HuffmanTable::build_from_counts(&literal_counts);
    let mut metadata = Vec::new();
    metadata.extend_from_slice(&MAGIC_NUM);
    metadata.extend_from_slice(&options.dictionary_id.to_le_bytes());
    metadata.extend_from_slice(huffman.table_description());
    append_fse_description(&mut metadata, default_of_table());
    append_fse_description(&mut metadata, default_ml_table());
    append_fse_description(&mut metadata, default_ll_table());

    let minimum_content = 8;
    let Some(content_budget) = options
        .dictionary_size
        .checked_sub(metadata.len().saturating_add(12))
        .filter(|budget| *budget >= minimum_content)
    else {
        return Err(DictionaryTrainingError::DictionaryTooSmall);
    };

    let mut content = Vec::new();
    create_raw_dict_from_source(
        training.as_slice(),
        training.len(),
        &mut content,
        content_budget,
    );
    if content.len() < minimum_content {
        let missing = minimum_content - content.len();
        content.extend(training.iter().copied().cycle().take(missing));
    }
    content.truncate(content_budget);

    metadata.extend_from_slice(&1_u32.to_le_bytes());
    metadata.extend_from_slice(&(content.len().min(4) as u32).to_le_bytes());
    metadata.extend_from_slice(&(content.len().min(8) as u32).to_le_bytes());
    metadata.extend_from_slice(&content);
    Ok(metadata)
}

fn append_fse_description(output: &mut Vec<u8>, table: crate::fse::fse_encoder::FSETable) {
    let mut writer = BitWriter::new();
    table.write_table(&mut writer);
    output.extend_from_slice(&writer.dump());
}

/// A set of values that are used during dictionary construction.
///
/// Changing these values can improve the resulting dictionary size for certain datasets.
// TODO: move `k` here.
pub(super) struct DictParams {
    /// Segment size.
    ///
    /// As found under "4. Experiments - Varying Segment Size" in the original paper, a
    /// segment size of 2 kiB was effective.
    ///
    /// "We explored a range of \[`segment_size`\] values and found the performance of LMC is insensitive
    /// to \[`segment_size`\]. We fix \[`segment_size`\] to 2kiB
    ///
    /// Reasonable range: [16, 2048+]
    pub segment_size: u32,
}

/// Creates a "raw content" dictionary, training off of every file in this directory and all
/// sub-directories.
///
/// The resulting dictionary will be approxamitely `dict_size` or less, and written to `output`.
///
/// # Errors
/// This function returns `Ok(())` if the dictionary was created successfully, and an
/// `Err(io::Error)` if an error was encountered reading the input directory.
///
/// # Examples
/// ```no_run
/// use std::fs::File;
/// // Create a roughly 1mb dictionary, training off of file in `sample_files`
/// let input_folder = "sample_files/";
/// let mut output = File::create("output.dict").unwrap();
/// zstd_complete::dictionary::create_raw_dict_from_dir(input_folder, &mut output, 1_000_000);
/// ```
pub fn create_raw_dict_from_dir<P: AsRef<Path>, W: io::Write>(
    path: P,
    output: &mut W,
    dict_size: usize,
) -> Result<(), io::Error> {
    // Collect a list of a path to every file in the directory into `file_paths`
    let mut file_paths: Vec<PathBuf> = Vec::new();
    let dir: fs::ReadDir = fs::read_dir(path)?;
    fn recurse_read(dir: fs::ReadDir, file_paths: &mut Vec<PathBuf>) -> Result<(), io::Error> {
        for entry in dir {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                recurse_read(fs::read_dir(entry.path())?, file_paths)?;
            } else {
                file_paths.push(entry.path());
            }
        }
        Ok(())
    }
    recurse_read(dir, &mut file_paths)?;

    // Open each file and chain the readers together
    let mut total_file_len: u64 = 0;
    let mut file_handles: Vec<fs::File> = Vec::new();
    for path in file_paths {
        let handle = File::open(path)?;
        total_file_len += handle.metadata()?.len();
        file_handles.push(handle);
    }
    let empty_reader: Box<dyn Read> = Box::new(io::empty());
    let chained_files = file_handles
        .iter()
        .fold(empty_reader, |acc, reader| Box::new(acc.chain(reader)));

    // Create a dict using the new reader
    create_raw_dict_from_source(chained_files, total_file_len as usize, output, dict_size);
    Ok(())
}

/// Read from `source` to create a "raw content" dictionary of `dict_size`.
/// The completed dictionary is written to `output`.
///
/// - `source` will be used as training data for the entire dictionary.
/// - `source_size` influences how the data is divided and sampled and is measured
///   in bytes. While this does not need to be exact, estimates should attempt to be
///   larger than the actual collection size.
/// - `output` is where the completed dictionary will be written.
/// - `dict_size` determines how large the complete dictionary should be. The completed
///   dictionary will be this size or smaller.
///
/// This function buffers the training source internally so sampling and epoch scoring
/// can both inspect the same training data.
pub fn create_raw_dict_from_source<R: io::Read, W: io::Write>(
    mut source: R,
    source_size: usize,
    output: &mut W,
    dict_size: usize,
) {
    if source_size < 16 {
        let mut source = source;
        let mut buf = Vec::new();
        source
            .read_to_end(&mut buf)
            .expect("Could not read from source");
        output.write_all(&buf).expect("Could not write to output");
        return;
    }
    vprintln!("create_dict: creating {dict_size} byte dict from {source_size} byte source");
    if dict_size == 0 {
        return;
    }

    let mut source_data = Vec::with_capacity(source_size);
    source
        .read_to_end(&mut source_data)
        .expect("can read input");
    if source_data.is_empty() {
        return;
    }

    if source_data.len() < K {
        output
            .write_all(&source_data[..usize::min(dict_size, source_data.len())])
            .expect("can write to output");
        return;
    }

    let params = DictParams { segment_size: 2048 };
    let segment_size = params.segment_size as usize;
    let num_segments = usize::max(1, source_data.len().div_ceil(segment_size));
    // According to 4. Experiments - Varying Reservoir Sampler Thresholds,
    // setting reservoir size to collection size / min{collection size / (2 * number of segments),
    // 256} was effective
    let sample_divisor = usize::min(usize::max(1, source_data.len() / (2 * num_segments)), 256);
    let sample_size = usize::max(K, source_data.len() / sample_divisor);
    vprintln!("create_dict: creating {sample_size} byte sample of collection");
    let collection_sample = create_sample(&mut source_data.as_slice(), sample_size);

    // A collection of segments to be used in the final dictionary.
    //
    // Contains the best segment from every epoch.
    // Reverse is used because we want a min heap, where
    // the lowest scoring items come first
    let mut pool: BinaryHeap<Reverse<Segment>> = BinaryHeap::new();
    let (_, epoch_kmers) = compute_epoch_info(&params, dict_size, source_data.len() / K);
    let epoch_size = usize::min(source_data.len(), usize::max(K, epoch_kmers * K));
    let num_epochs = source_data.len().div_ceil(epoch_size);
    vprintln!("create_dict: computed epoch info, using {num_epochs} epochs of {epoch_size} bytes");
    let mut ctx = Context {
        frequencies: HashMap::with_capacity(epoch_size / K),
    };
    // Score each segment in the epoch and select the highest scoring segment
    // for the pool
    for (epoch_index, current_epoch) in source_data.chunks(epoch_size).enumerate() {
        let Some(best_segment) =
            pick_best_segment(&params, &mut ctx, current_epoch, &collection_sample)
        else {
            continue;
        };
        vprintln!(
            "\tcreate_dict: epoch {}/{} has best segment score {}",
            epoch_index + 1,
            num_epochs,
            best_segment.score
        );
        pool.push(Reverse(best_segment));
        // Wipe frequency list for next epoch
        ctx.frequencies.clear();
    }
    vprintln!(
        "create_dict: {num_epochs} epochs written, writing {} segments",
        pool.len()
    );
    // Write the dictionary with the highest scoring segment last because
    // closer items can be represented with a smaller offset
    let mut remaining = dict_size;
    while let Some(segment) = pool.pop() {
        if remaining == 0 {
            break;
        }
        let bytes_to_write = usize::min(remaining, segment.0.raw.len());
        output
            .write_all(&segment.0.raw[..bytes_to_write])
            .expect("can write to output");
        remaining -= bytes_to_write;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_raw_dict_from_source, train_dictionary, DictionaryTrainerOptions,
        DictionaryTrainingError,
    };
    use std::{io::Read, vec::Vec};

    #[test]
    fn raw_dict_builder_scores_epochs_after_sampling() {
        let mut source = Vec::new();
        for _ in 0..512 {
            source.extend_from_slice(b"aaaaaaaaaaaaaaaabbbbbbbbbbbbbbbb");
            source.extend_from_slice(b"ccccccccccccccccdddddddddddddddd");
        }
        let mut dictionary = Vec::new();

        create_raw_dict_from_source(source.as_slice(), source.len(), &mut dictionary, 1024);

        assert!(!dictionary.is_empty());
        assert!(dictionary.len() <= 1024);
    }

    #[test]
    fn raw_dict_builder_handles_tiny_sources() {
        let source = b"tiny";
        let mut dictionary = Vec::new();

        create_raw_dict_from_source(source.as_slice(), source.len(), &mut dictionary, 1024);

        assert_eq!(dictionary, source);
    }

    #[test]
    fn formatted_dictionary_is_usable_by_encoder_and_decoder() {
        let samples = [
            b"customer=alice action=login status=success".repeat(100),
            b"customer=bob action=logout status=success".repeat(100),
        ];
        let dictionary =
            train_dictionary(&samples, DictionaryTrainerOptions::new(2048, 0xC0DE)).unwrap();
        let decoded = crate::decoding::Dictionary::decode_dict(&dictionary).unwrap();
        assert_eq!(decoded.id, 0xC0DE);
        assert!(dictionary.len() <= 2048);

        let input = b"customer=alice action=logout status=success".repeat(200);
        let encoder_dictionary = crate::encoding::EncoderDictionary::copy(&dictionary).unwrap();
        let compressed = crate::encoding::encode_all(
            input.as_slice(),
            crate::encoding::EncoderOptions::default().with_dictionary(encoder_dictionary),
        )
        .unwrap();
        let mut c_decoder =
            zstd::stream::read::Decoder::with_dictionary(compressed.as_slice(), &dictionary)
                .unwrap();
        let mut output = Vec::new();
        c_decoder.read_to_end(&mut output).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn formatted_dictionary_options_are_validated() {
        assert_eq!(
            train_dictionary(&[b"sample"], DictionaryTrainerOptions::new(1024, 0)),
            Err(DictionaryTrainingError::ZeroDictionaryId)
        );
        assert_eq!(
            train_dictionary(&[b"sample"], DictionaryTrainerOptions::new(8, 1)),
            Err(DictionaryTrainingError::DictionaryTooSmall)
        );
    }
}

#[test]
fn create_raw_dict_from_source_no_panics_on_small_input() {
    use std::io::Cursor;

    for size in 0..1024 {
        let input = alloc::vec![b'A'; size];
        let mut output = Vec::new();

        create_raw_dict_from_source(Cursor::new(input.clone()), input.len(), &mut output, 64);
    }
}
