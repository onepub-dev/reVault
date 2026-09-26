//! Opt-in bounded parallel compression of independent Zstandard frames.

use alloc::{collections::BTreeMap, format, vec, vec::Vec};
use core::{convert::TryFrom, fmt, num::NonZeroUsize};
use std::{
    io::{self, Read, Write},
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread::{self, JoinHandle},
};

use super::{
    streaming_encoder::{encode_frame_for_options, EncodeError, Encoder, EncoderOptions},
    CompressionLevel, EncoderDictionary, EncoderWorkspace,
};

/// A bounded streaming encoder that compresses independent frames in parallel.
///
/// A worker count of one delegates directly to [`Encoder`]. Two or more
/// workers use a bounded set of frame jobs and write completed frames in input
/// order. The existing single-threaded compressor remains unchanged.
pub struct ParallelEncoder<W: Write> {
    inner: ParallelEncoderInner<W>,
}

enum ParallelEncoderInner<W: Write> {
    Single(Encoder<W>),
    Multi(MultiEncoder<W>),
}

impl<W: Write> ParallelEncoder<W> {
    /// Creates an encoder with the requested non-zero worker count.
    ///
    /// The configured memory limit applies to the aggregate conservative
    /// estimate for all workers and the caller's current input frame.
    pub fn new(
        inner: W,
        options: EncoderOptions,
        workers: NonZeroUsize,
    ) -> Result<Self, EncodeError> {
        let inner = if workers.get() == 1 {
            ParallelEncoderInner::Single(Encoder::new(inner, options)?)
        } else {
            ParallelEncoderInner::Multi(MultiEncoder::new(inner, options, workers)?)
        };
        Ok(Self { inner })
    }

    /// Returns the conservative aggregate memory estimate for this mode.
    pub fn estimated_memory_usage(options: &EncoderOptions, workers: NonZeroUsize) -> usize {
        if workers.get() == 1 {
            options.estimated_memory_usage()
        } else {
            let worker_memory = options
                .estimated_memory_usage()
                .saturating_mul(workers.get())
                .saturating_add(options.frame_chunk_size());
            options.dictionary().map_or(worker_memory, |dictionary| {
                worker_memory.saturating_add(dictionary.raw_size().saturating_mul(2))
            })
        }
    }

    /// Borrows the target without waiting for pending frames.
    pub fn get_ref(&self) -> &W {
        match &self.inner {
            ParallelEncoderInner::Single(encoder) => encoder.get_ref(),
            ParallelEncoderInner::Multi(encoder) => encoder.get_ref(),
        }
    }

    /// Mutably borrows the target without waiting for pending frames.
    ///
    /// Writing archive bytes directly can corrupt ordering. Call [`Self::flush`]
    /// first when the target must observe all previously supplied input.
    pub fn get_mut(&mut self) -> &mut W {
        match &mut self.inner {
            ParallelEncoderInner::Single(encoder) => encoder.get_mut(),
            ParallelEncoderInner::Multi(encoder) => encoder.get_mut(),
        }
    }

    /// Emits all pending frames, joins every worker, and returns the writer.
    pub fn finish(self) -> Result<W, EncodeError> {
        match self.inner {
            ParallelEncoderInner::Single(encoder) => encoder.finish(),
            ParallelEncoderInner::Multi(encoder) => encoder.finish(),
        }
    }
}

impl<W: Write> Write for ParallelEncoder<W> {
    fn write(&mut self, source: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            ParallelEncoderInner::Single(encoder) => encoder.write(source),
            ParallelEncoderInner::Multi(encoder) => encoder.write(source),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            ParallelEncoderInner::Single(encoder) => encoder.flush(),
            ParallelEncoderInner::Multi(encoder) => encoder.flush(),
        }
    }
}

impl<W: Write> fmt::Debug for ParallelEncoder<W> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParallelEncoder")
            .field("workers", &self.worker_count())
            .finish_non_exhaustive()
    }
}

impl<W: Write> ParallelEncoder<W> {
    pub fn worker_count(&self) -> NonZeroUsize {
        match &self.inner {
            ParallelEncoderInner::Single(_) => NonZeroUsize::MIN,
            ParallelEncoderInner::Multi(encoder) => encoder.worker_count(),
        }
    }
}

/// Streams all input through a bounded parallel encoder.
pub fn encode_parallel<R: Read, W: Write>(
    mut source: R,
    target: W,
    options: EncoderOptions,
    workers: NonZeroUsize,
) -> Result<(), EncodeError> {
    if workers.get() == 1 {
        return super::streaming_encoder::encode(source, target, options);
    }
    let mut encoder = ParallelEncoder::new(target, options, workers)?;
    io::copy(&mut source, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Compresses all input in parallel and returns the ordered frame archive.
pub fn encode_all_parallel<R: Read>(
    source: R,
    options: EncoderOptions,
    workers: NonZeroUsize,
) -> Result<Vec<u8>, EncodeError> {
    if workers.get() == 1 {
        return super::streaming_encoder::encode_all(source, options);
    }
    let mut output = Vec::new();
    encode_parallel(source, &mut output, options, workers)?;
    Ok(output)
}

struct MultiEncoder<W: Write> {
    inner: Option<W>,
    options: EncoderOptions,
    input: Vec<u8>,
    workers: Vec<Worker>,
    results: Receiver<WorkerResult>,
    completed: BTreeMap<usize, Vec<u8>>,
    next_job: usize,
    next_output: usize,
    next_worker: usize,
    in_flight: usize,
    emitted_frame: bool,
    accepted_input: u64,
}

impl<W: Write> MultiEncoder<W> {
    fn new(
        inner: W,
        options: EncoderOptions,
        worker_count: NonZeroUsize,
    ) -> Result<Self, EncodeError> {
        options.validate()?;
        let required = ParallelEncoder::<W>::estimated_memory_usage(&options, worker_count);
        if required > options.memory_limit() {
            return Err(EncodeError::MemoryLimitExceeded {
                limit: options.memory_limit(),
                required,
            });
        }

        let (result_sender, results) = mpsc::channel();
        let mut workers = Vec::with_capacity(worker_count.get());
        for index in 0..worker_count.get() {
            workers.push(Worker::spawn(
                index,
                WorkerOptions::from_options(&options),
                result_sender.clone(),
            )?);
        }
        drop(result_sender);

        let input = Vec::with_capacity(options.frame_chunk_size());
        Ok(Self {
            inner: Some(inner),
            options,
            input,
            workers,
            results,
            completed: BTreeMap::new(),
            next_job: 0,
            next_output: 0,
            next_worker: 0,
            in_flight: 0,
            emitted_frame: false,
            accepted_input: 0,
        })
    }

    fn get_ref(&self) -> &W {
        self.inner
            .as_ref()
            .expect("writer is present before finish")
    }

    fn get_mut(&mut self) -> &mut W {
        self.inner
            .as_mut()
            .expect("writer is present before finish")
    }

    fn worker_count(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.workers.len()).expect("multi encoder has workers")
    }

    fn finish(mut self) -> Result<W, EncodeError> {
        let result: Result<(), EncodeError> = (|| {
            self.verify_pledged_size()?;
            if !self.input.is_empty() || !self.emitted_frame {
                self.dispatch_frame()?;
            }
            self.drain_all()?;
            self.get_mut().flush()?;
            Ok(())
        })();
        let worker_result = self.stop_workers();
        result?;
        worker_result?;
        Ok(self.inner.take().expect("writer is present before finish"))
    }

    fn verify_pledged_size(&self) -> Result<(), EncodeError> {
        if let Some(pledged) = self.options.pledged_source_size() {
            if pledged != self.accepted_input {
                return Err(EncodeError::PledgedSourceSizeMismatch {
                    pledged,
                    actual: self.accepted_input,
                });
            }
        }
        Ok(())
    }

    fn dispatch_frame(&mut self) -> Result<(), EncodeError> {
        while self.in_flight >= self.workers.len() {
            self.receive_one()?;
        }

        let input = core::mem::replace(
            &mut self.input,
            Vec::with_capacity(self.options.frame_chunk_size()),
        );
        let job = FrameJob {
            sequence: self.next_job,
            input,
            #[cfg(test)]
            panic_before_compression: false,
        };
        self.workers[self.next_worker]
            .sender
            .send(WorkerMessage::Frame(job))
            .map_err(|_| EncodeError::WorkerFailed)?;
        self.next_worker = (self.next_worker + 1) % self.workers.len();
        self.next_job += 1;
        self.in_flight += 1;
        self.emitted_frame = true;
        self.drain_available()
    }

    fn drain_available(&mut self) -> Result<(), EncodeError> {
        loop {
            match self.results.try_recv() {
                Ok(result) => self.accept_result(result)?,
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    return if self.in_flight == 0 {
                        Ok(())
                    } else {
                        Err(EncodeError::WorkerFailed)
                    };
                }
            }
        }
    }

    fn receive_one(&mut self) -> Result<(), EncodeError> {
        let result = self.results.recv().map_err(|_| EncodeError::WorkerFailed)?;
        self.accept_result(result)
    }

    fn accept_result(&mut self, result: WorkerResult) -> Result<(), EncodeError> {
        self.in_flight -= 1;
        let frame = result.frame?;
        self.completed.insert(result.sequence, frame);
        while let Some(frame) = self.completed.remove(&self.next_output) {
            self.get_mut().write_all(&frame)?;
            self.next_output += 1;
        }
        Ok(())
    }

    fn drain_all(&mut self) -> Result<(), EncodeError> {
        while self.in_flight != 0 {
            self.receive_one()?;
        }
        debug_assert!(self.completed.is_empty());
        Ok(())
    }

    fn stop_workers(&mut self) -> Result<(), EncodeError> {
        for worker in &self.workers {
            let _ = worker.sender.send(WorkerMessage::Stop);
        }
        let mut failed = false;
        for worker in &mut self.workers {
            if worker
                .handle
                .take()
                .is_some_and(|handle| handle.join().is_err())
            {
                failed = true;
            }
        }
        if failed {
            Err(EncodeError::WorkerFailed)
        } else {
            Ok(())
        }
    }
}

impl<W: Write> Write for MultiEncoder<W> {
    fn write(&mut self, source: &[u8]) -> io::Result<usize> {
        if source.is_empty() {
            return Ok(0);
        }
        if self.input.len() == self.options.frame_chunk_size() {
            self.dispatch_frame().map_err(encode_error_as_io)?;
        }
        let available = self.options.frame_chunk_size() - self.input.len();
        let consumed = available.min(source.len());
        if let Some(pledged) = self.options.pledged_source_size() {
            let remaining = pledged.saturating_sub(self.accepted_input);
            if remaining == 0 {
                return Err(io::Error::other(EncodeError::PledgedSourceSizeMismatch {
                    pledged,
                    actual: self.accepted_input.saturating_add(source.len() as u64),
                }));
            }
            let consumed = consumed.min(usize::try_from(remaining).unwrap_or(usize::MAX));
            self.input.extend_from_slice(&source[..consumed]);
            self.accepted_input += consumed as u64;
            return Ok(consumed);
        }
        self.input.extend_from_slice(&source[..consumed]);
        self.accepted_input += consumed as u64;
        Ok(consumed)
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.input.is_empty() {
            self.dispatch_frame().map_err(encode_error_as_io)?;
        }
        self.drain_all().map_err(encode_error_as_io)?;
        self.get_mut().flush()
    }
}

impl<W: Write> Drop for MultiEncoder<W> {
    fn drop(&mut self) {
        let _ = self.stop_workers();
    }
}

struct Worker {
    sender: SyncSender<WorkerMessage>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn spawn(
        index: usize,
        options: WorkerOptions,
        results: mpsc::Sender<WorkerResult>,
    ) -> Result<Self, EncodeError> {
        let (sender, jobs) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name(format!("zstd-complete-{index}"))
            .spawn(move || worker_loop(options, jobs, results))?;
        Ok(Self {
            sender,
            handle: Some(handle),
        })
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.sender.send(WorkerMessage::Stop);
            let _ = handle.join();
        }
    }
}

struct WorkerOptions {
    level: CompressionLevel,
    frame_chunk_size: usize,
    dictionary: Option<Vec<u8>>,
    checksum: bool,
    tuning: super::CompressionTuning,
    content_size_policy: super::ContentSizePolicy,
}

impl WorkerOptions {
    fn from_options(options: &EncoderOptions) -> Self {
        Self {
            level: options.level(),
            frame_chunk_size: options.frame_chunk_size(),
            dictionary: options
                .dictionary()
                .map(|dictionary| dictionary.raw().to_vec()),
            checksum: options.checksum(),
            tuning: options.tuning(),
            content_size_policy: options.content_size_policy(),
        }
    }

    fn prepare(self) -> Result<EncoderOptions, EncodeError> {
        let mut options = EncoderOptions::new(self.level)
            .with_frame_chunk_size(self.frame_chunk_size)
            .with_checksum(self.checksum)
            .with_tuning(self.tuning)
            .with_content_size_policy(self.content_size_policy);
        if let Some(dictionary) = self.dictionary {
            let dictionary = EncoderDictionary::copy(&dictionary)
                .map_err(|_| EncodeError::InvalidOptions("worker dictionary preparation failed"))?;
            options = options.with_dictionary(dictionary);
        }
        Ok(options)
    }
}

enum WorkerMessage {
    Frame(FrameJob),
    Stop,
}

struct FrameJob {
    sequence: usize,
    input: Vec<u8>,
    #[cfg(test)]
    panic_before_compression: bool,
}

struct WorkerResult {
    sequence: usize,
    frame: Result<Vec<u8>, EncodeError>,
}

fn worker_loop(
    options: WorkerOptions,
    jobs: Receiver<WorkerMessage>,
    results: mpsc::Sender<WorkerResult>,
) {
    let options = options.prepare();
    let mut workspace = options.as_ref().ok().and_then(|options| {
        (options.dictionary().is_none()
            && !options.checksum()
            && options.tuning() == super::CompressionTuning::new()
            && options.content_size_policy() == super::ContentSizePolicy::Include)
            .then(|| EncoderWorkspace::new(options.level(), options.frame_chunk_size()))
            .transpose()
            .ok()
            .flatten()
    });
    while let Ok(message) = jobs.recv() {
        let WorkerMessage::Frame(job) = message else {
            break;
        };
        let frame = match options.as_ref() {
            Ok(options) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                #[cfg(test)]
                if job.panic_before_compression {
                    panic!("injected worker panic");
                }
                if let Some(workspace) = workspace.as_mut() {
                    let output_size = EncoderWorkspace::required_output_size(job.input.len())
                        .map_err(|_| EncodeError::WorkerFailed)?;
                    let mut frame = vec![0_u8; output_size];
                    let length = workspace
                        .encode_into(&job.input, &mut frame)
                        .map_err(|_| EncodeError::WorkerFailed)?
                        .len();
                    frame.truncate(length);
                    Ok(frame)
                } else {
                    Ok(encode_frame_for_options(&job.input, options))
                }
            }))
            .map_err(|_| EncodeError::WorkerFailed)
            .and_then(|frame| frame),
            Err(_) => Err(EncodeError::WorkerFailed),
        };
        if results
            .send(WorkerResult {
                sequence: job.sequence,
                frame,
            })
            .is_err()
        {
            break;
        }
    }
}

fn encode_error_as_io(error: EncodeError) -> io::Error {
    match error {
        EncodeError::Io(error) => error,
        error => io::Error::other(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::encode_all;

    fn workers(count: usize) -> NonZeroUsize {
        NonZeroUsize::new(count).unwrap()
    }

    #[test]
    fn one_worker_is_the_exact_sequential_path() {
        let input = b"single worker must remain byte exact".repeat(20_000);
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(32 * 1024)
            .with_memory_limit(32 * 1024 * 1024);
        let expected = encode_all(input.as_slice(), options.clone()).unwrap();
        let actual = encode_all_parallel(input.as_slice(), options, workers(1)).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn parallel_frames_are_written_in_input_order() {
        let mut input = Vec::new();
        for value in 0_u8..64 {
            input.extend(core::iter::repeat_n(value, 16 * 1024));
        }
        let options = EncoderOptions::new(CompressionLevel::FASTEST)
            .with_frame_chunk_size(16 * 1024)
            .with_memory_limit(128 * 1024 * 1024);
        let sequential = encode_all(input.as_slice(), options.clone()).unwrap();
        let compressed = encode_all_parallel(input.as_slice(), options, workers(4)).unwrap();
        assert_eq!(compressed, sequential);
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn aggregate_memory_limit_is_checked_before_threads_start() {
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(8 * 1024 * 1024)
            .with_memory_limit(96 * 1024 * 1024);
        assert!(matches!(
            ParallelEncoder::new(io::sink(), options, workers(4)),
            Err(EncodeError::MemoryLimitExceeded { .. })
        ));
    }

    #[test]
    fn parallel_dictionaries_interoperate_with_c() {
        let dictionary_bytes = b"parallel dictionary content and prefixes".repeat(16);
        let input = dictionary_bytes.repeat(2_000);
        let dictionary = EncoderDictionary::copy(&dictionary_bytes).unwrap();
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(32 * 1024)
            .with_memory_limit(128 * 1024 * 1024)
            .with_dictionary(dictionary);
        let compressed = encode_all_parallel(input.as_slice(), options, workers(3)).unwrap();

        let mut decoded = Vec::new();
        zstd::stream::read::Decoder::with_dictionary(compressed.as_slice(), &dictionary_bytes)
            .unwrap()
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded, input);
    }

    #[cfg(feature = "hash")]
    #[test]
    fn parallel_checksums_interoperate_with_c() {
        let input = b"parallel checksummed content".repeat(20_000);
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(32 * 1024)
            .with_memory_limit(128 * 1024 * 1024)
            .with_checksum(true);
        let compressed = encode_all_parallel(input.as_slice(), options, workers(3)).unwrap();
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn parallel_empty_input_emits_one_valid_frame() {
        let options =
            EncoderOptions::new(CompressionLevel::DEFAULT).with_memory_limit(128 * 1024 * 1024);
        let compressed = encode_all_parallel(&[][..], options, workers(2)).unwrap();
        let mut decoded = Vec::new();
        crate::decoding::StreamingDecoder::new(compressed.as_slice())
            .unwrap()
            .read_to_end(&mut decoded)
            .unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn flush_waits_for_ordered_frames() {
        let input = b"flush must wait for every earlier frame".repeat(10_000);
        let options = EncoderOptions::new(CompressionLevel::DEFAULT)
            .with_frame_chunk_size(8 * 1024)
            .with_memory_limit(128 * 1024 * 1024);
        let mut encoder = ParallelEncoder::new(Vec::new(), options, workers(3)).unwrap();
        encoder.write_all(&input).unwrap();
        encoder.flush().unwrap();
        assert_eq!(
            zstd::decode_all(encoder.get_ref().as_slice()).unwrap(),
            input
        );
        encoder.finish().unwrap();
    }

    #[test]
    fn short_writes_preserve_the_archive() {
        #[derive(Default)]
        struct ShortWriter(Vec<u8>);

        impl Write for ShortWriter {
            fn write(&mut self, source: &[u8]) -> io::Result<usize> {
                let count = source.len().min(7);
                self.0.extend_from_slice(&source[..count]);
                Ok(count)
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let input = b"ordered output under backpressure".repeat(20_000);
        let options = EncoderOptions::new(CompressionLevel::FASTEST)
            .with_frame_chunk_size(16 * 1024)
            .with_memory_limit(96 * 1024 * 1024);
        let mut encoder =
            ParallelEncoder::new(ShortWriter::default(), options, workers(2)).unwrap();
        encoder.write_all(&input).unwrap();
        let compressed = encoder.finish().unwrap().0;
        assert_eq!(zstd::decode_all(compressed.as_slice()).unwrap(), input);
    }

    #[test]
    fn target_write_failure_is_returned_without_leaking_workers() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _source: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let options = EncoderOptions::new(CompressionLevel::FASTEST)
            .with_frame_chunk_size(1024)
            .with_memory_limit(64 * 1024 * 1024);
        let error = encode_parallel(
            b"parallel error propagation".repeat(1_000).as_slice(),
            FailingWriter,
            options,
            workers(2),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            EncodeError::Io(ref error) if error.kind() == io::ErrorKind::BrokenPipe
        ));
    }

    #[test]
    fn worker_panics_are_contained_and_reported() {
        let (result_sender, results) = mpsc::channel();
        let worker = Worker::spawn(
            0,
            WorkerOptions::from_options(&EncoderOptions::new(CompressionLevel::FASTEST)),
            result_sender,
        )
        .unwrap();
        worker
            .sender
            .send(WorkerMessage::Frame(FrameJob {
                sequence: 7,
                input: Vec::new(),
                panic_before_compression: true,
            }))
            .unwrap();

        let result = results.recv().unwrap();
        assert_eq!(result.sequence, 7);
        assert!(matches!(result.frame, Err(EncodeError::WorkerFailed)));
        drop(worker);
    }
}
