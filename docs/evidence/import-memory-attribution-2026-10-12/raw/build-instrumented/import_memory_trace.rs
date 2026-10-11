//! Temporary opt-in Linux capacity ledger, not a heap census or product API.
//! Allocations occur outside this mutex. A snapshot is coherent for the ledger,
//! not atomic with allocator/RSS updates; handoff gaps and sampling perturbation
//! are deliberate limitations. Cache weights and encode-only PAGE are gauges.
use std::io::Write;
use std::sync::{Mutex, OnceLock};

pub(crate) const QUEUED: usize = 0;
pub(crate) const INPUT: usize = 1;
pub(crate) const PREPARED: usize = 2;
pub(crate) const ORDERED: usize = 3;
pub(crate) const WRITING: usize = 4;
pub(crate) const SCRATCH: usize = 5;
pub(crate) const ENCODER: usize = 6;
pub(crate) const POOLED: usize = 7;
pub(crate) const OBJECT_STREAM: usize = 8;
pub(crate) const BODY_TEMP: usize = 9;
pub(crate) const BODY: usize = 10;
pub(crate) const STORED_BODY: usize = 11;
pub(crate) const PAGE: usize = 12;
const N: usize = 13;
const MAX_ROWS: u64 = 256;
const MAX_BOUNDARIES: u64 = 4;

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        cfg!(target_os = "linux")
            && std::env::var_os("REVAULT_IMPORT_MEMORY_TRACE").as_deref()
                == Some(std::ffi::OsStr::new("1"))
    })
}
#[derive(Clone, Copy)]
struct Snapshot {
    event: &'static str,
    sequence: u64,
    rss: u64,
    hwm: u64,
    live: usize,
    bytes: [usize; N],
    counts: [usize; N],
    added: [u64; N],
    removed: [u64; N],
    cache: [u64; 4],
    balanced: bool,
}
#[derive(Default)]
struct State {
    bytes: [usize; N],
    counts: [usize; N],
    added: [u64; N],
    removed: [u64; N],
    events: u64,
    emitted: u64,
    boundaries: u64,
    suppressed: u64,
    peaks: [Option<Snapshot>; 3],
    cache: [u64; 4],
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::default()))
}
fn rss() -> (u64, u64) {
    let text = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |name: &str| {
        text.lines()
            .find_map(|line| {
                line.strip_prefix(name)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(0)
            * 1024
    };
    (field("VmRSS:"), field("VmHWM:"))
}
fn emit(p: Snapshot, label: &'static str, suppressed: u64) {
    // Only fixed labels and numeric metadata. Never reenter the ledger here.
    let _ = writeln!(std::io::stderr(), "RV_IMPORT_MEMORY {{\"record\":\"{}\",\"event\":\"{}\",\"sequence\":{},\"rss\":{},\"hwm\":{},\"live\":{},\"bytes\":{:?},\"counts\":{:?},\"added\":{:?},\"removed\":{:?},\"cache_used_limit_entries_sequence\":{:?},\"balanced\":{},\"suppressed\":{}}}", label, p.event, p.sequence, p.rss, p.hwm, p.live, p.bytes, p.counts, p.added, p.removed, p.cache, p.balanced, suppressed);
}
fn sample(s: &mut State, event: &'static str, force: bool) -> Snapshot {
    s.events += 1;
    let (rss, hwm) = rss();
    let p = Snapshot {
        event,
        sequence: s.events,
        rss,
        hwm,
        live: s.bytes.iter().sum(),
        bytes: s.bytes,
        counts: s.counts,
        added: s.added,
        removed: s.removed,
        cache: s.cache,
        balanced: (0..N).all(|i| s.added[i].checked_sub(s.removed[i]) == Some(s.bytes[i] as u64)),
    };
    let values = [rss, hwm, p.live as u64];
    let mut changed = false;
    for (i, value) in values.into_iter().enumerate() {
        let prior = s.peaks[i].map(|p| [p.rss, p.hwm, p.live as u64][i]);
        if prior.is_none_or(|prior| value > prior) {
            s.peaks[i] = Some(p);
            changed = true;
        }
    }
    // Peak snapshots always update, even after bounded ordinary output stops.
    if changed || force {
        if s.emitted < MAX_ROWS {
            emit(p, "sample", s.suppressed);
            s.emitted += 1;
        } else {
            s.suppressed += 1;
        }
    }
    p
}
#[derive(Default)]
pub(crate) struct Lease {
    category: usize,
    bytes: usize,
    active: bool,
}
impl Lease {
    pub(crate) fn new(category: usize, bytes: usize) -> Self {
        let mut value = Self::default();
        value.set(category, bytes);
        value
    }
    pub(crate) fn set(&mut self, category: usize, bytes: usize) {
        if !enabled() {
            return;
        }
        let Ok(mut s) = state().lock() else {
            return;
        };
        if self.active {
            s.bytes[self.category] -= self.bytes;
            s.counts[self.category] -= 1;
            s.removed[self.category] += self.bytes as u64;
        }
        self.category = category;
        self.bytes = bytes;
        self.active = true;
        s.bytes[category] += bytes;
        s.counts[category] += 1;
        s.added[category] += bytes as u64;
        sample(&mut s, "ownership", false);
    }
    pub(crate) fn stage(&mut self, category: usize) {
        self.set(category, self.bytes);
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut s) = state().lock() {
            s.bytes[self.category] -= self.bytes;
            s.counts[self.category] -= 1;
            s.removed[self.category] += self.bytes as u64;
            sample(&mut s, "release", false);
        }
    }
}
pub(crate) fn cache(event: &'static str, used: u64, limit: u64, entries: usize) {
    if !enabled() {
        return;
    }
    if let Ok(mut s) = state().lock() {
        s.cache = [used, limit, entries as u64, s.events + 1];
        sample(&mut s, event, true);
    }
}
pub(crate) fn boundary(event: &'static str, used: u64, limit: u64, entries: usize) {
    if !enabled() {
        return;
    }
    if let Ok(mut s) = state().lock() {
        s.cache = [used, limit, entries as u64, s.events + 1];
        let p = sample(&mut s, event, false);
        if s.boundaries < MAX_BOUNDARIES {
            emit(p, "boundary", s.suppressed);
            for (peak, label) in s.peaks.iter().zip(["peak_rss", "peak_hwm", "peak_live"]) {
                if let Some(peak) = peak {
                    emit(*peak, label, s.suppressed);
                }
            }
            s.boundaries += 1;
        } else {
            s.suppressed += 4;
        }
    }
}
