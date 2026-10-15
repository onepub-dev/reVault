//! Explicit CPU-only experiment. Storage, authenticated lookup and sinks remain
//! on the caller thread, including for non-Send backends. No background work
//! survives a batch, error, or callback unwind.
use super::*;
use crate::page_buffer::ZeroizingBytes;
use rayon::prelude::*;

const CHUNKS_PER_WORKER: usize = 8;

pub(super) struct Job {
    pub selected: Selected,
    pub stored: ZeroizingBytes,
    pub decoded: Option<Result<ZeroizingBytes>>,
}

pub(super) struct Parallel {
    pool: rayon::ThreadPool,
    codecs: Vec<Codec>,
}
impl Parallel {
    pub(super) fn new(
        archive: LockboxId,
        mode: FormatMode,
        key: Option<&[u8]>,
        workers: usize,
    ) -> Result<Self> {
        if !matches!(workers, 2 | 4) {
            return Err(Error::InvalidInput(
                "parallel read workers must be 2 or 4".into(),
            ));
        }
        let codecs = (0..workers)
            .map(|_| Codec::shared_packed(archive, mode, key))
            .collect::<Result<Vec<_>>>()?;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .thread_name(|index| format!("revault-read-{index}"))
            .build()
            .map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self { pool, codecs })
    }
    pub(super) fn capacity(&self) -> usize {
        self.codecs.len() * CHUNKS_PER_WORKER
    }
    pub(super) fn decode(&mut self, jobs: &mut [Job], sealed: u64) {
        // Contiguous lanes preserve the indexed output order. A lane's bounded
        // decoder workspace is reused across batches and files in this session.
        let chunk = jobs.len().div_ceil(self.codecs.len());
        let codecs = &mut self.codecs;
        self.pool.install(|| {
            jobs.par_chunks_mut(chunk)
                .zip(codecs.par_iter_mut())
                .for_each(|(jobs, codec)| {
                    for job in jobs {
                        let stored =
                            std::mem::replace(&mut job.stored, ZeroizingBytes::new(Vec::new()));
                        job.decoded = Some(codec.load_stored(
                            stored,
                            job.selected.extent,
                            sealed,
                            &job.selected.descriptor,
                        ));
                    }
                });
        });
    }
}
