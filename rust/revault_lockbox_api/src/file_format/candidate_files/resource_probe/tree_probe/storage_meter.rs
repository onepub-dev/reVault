//! Read-only diagnostic wrapper. No ordinary reader uses this instrumentation.
use crate::storage::Storage;
use crate::Result;
use std::{cell::Cell, rc::Rc, time::Instant};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Counts {
    lengths: u64,
    reads: u64,
    into_reads: u64,
    requested_bytes: u64,
    failures: u64,
    length_seconds: f64,
    read_seconds: f64,
}
impl Counts {
    pub(super) fn json(self) -> serde_json::Value {
        serde_json::json!({"length_calls":self.lengths,"read_calls":self.reads,
            "read_into_calls":self.into_reads,"requested_bytes":self.requested_bytes,
            "failed_calls":self.failures,"length_seconds":self.length_seconds,
            "read_seconds":self.read_seconds})
    }
}
#[derive(Clone, Debug)]
pub(super) struct Meter<S> {
    storage: S,
    counts: Rc<Cell<Counts>>,
}
impl<S> Meter<S> {
    pub(super) fn new(storage: S) -> Self {
        Self {
            storage,
            counts: Rc::new(Cell::new(Counts::default())),
        }
    }
    pub(super) fn take(&self) -> Counts {
        self.counts.take()
    }
    fn measure<T>(&self, kind: u8, bytes: usize, f: impl FnOnce() -> Result<T>) -> Result<T> {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_secs_f64();
        let mut counts = self.counts.get();
        match kind {
            0 => {
                counts.lengths += 1;
                counts.length_seconds += elapsed;
            }
            1 => {
                counts.reads += 1;
                counts.read_seconds += elapsed;
            }
            2 => {
                counts.into_reads += 1;
                counts.read_seconds += elapsed;
            }
            _ => unreachable!(),
        }
        counts.requested_bytes += bytes as u64;
        counts.failures += u64::from(result.is_err());
        self.counts.set(counts);
        result
    }
}
impl<S: Storage> Storage for Meter<S> {
    fn len(&self) -> Result<u64> {
        self.measure(0, 0, || self.storage.len())
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.measure(1, len, || self.storage.read_at(offset, len))
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        self.measure(2, out.len(), || self.storage.read_at_into(offset, out))
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("read-only diagnostic")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("read-only diagnostic")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("read-only diagnostic")
    }
    fn sync(&self) -> Result<()> {
        panic!("read-only diagnostic")
    }
}
