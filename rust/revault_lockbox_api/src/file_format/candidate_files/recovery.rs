//! Explicit read-only salvage. Payload checksums never authorize membership.
use super::*;
use crate::file_format::authenticated_index::RecoveryReport;

/// The sink must stage each file, publish it only after `finish(true)`, and
/// discard it after `finish(false)`. Keep the batch staged until this call
/// succeeds; ANY returned error invalidates the batch, including earlier finishes. At most one file is
/// active. Authenticated fragments may precede discovery of a later missing one.
pub(crate) trait Sink {
    fn begin(&mut self, path: &[u8], len: u64) -> Result<()>;
    fn data(&mut self, bytes: &[u8]) -> Result<()>;
    fn finish(&mut self, complete: bool) -> Result<()>;
}
pub(crate) struct Report {
    pub generation: u64,
    pub complete: u64,
    pub incomplete: u64,
    /// Includes unknown file/chunk memberships; not a count of lost files.
    pub membership: RecoveryReport,
    /// A chunk whose file header is unavailable cannot authorize a recovered file.
    pub orphan_chunks: u64,
}
struct State {
    path: Zeroizing<Vec<u8>>,
    info: FileInfo,
    next: u64,
    failed: bool,
    hash: Sha256,
}
fn finish(
    state: State,
    sink: &mut impl Sink,
    complete: &mut u64,
    incomplete: &mut u64,
) -> Result<()> {
    let valid = !state.failed
        && state.next == state.info.count()
        && <[u8; 32]>::from(state.hash.finalize()) == state.info.digest;
    sink.finish(valid)?;
    if valid {
        *complete += 1;
    } else {
        *incomplete += 1;
    }
    Ok(())
}
impl<S: Storage> Files<S> {
    /// Does not require unrelated allocation maps, padding or payload to survive.
    /// The selected publication and membership proof are mandatory. A missing root
    /// fails closed; missing descendants are reported, never sought in older trees.
    /// Storage must remain stable throughout this operation (reader lock/snapshot).
    pub(crate) fn salvage(
        storage: &S,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        sink: &mut impl Sink,
    ) -> Result<Report> {
        let anchor = publication::select(storage, archive, mode, authority)?.anchor;
        let index = Index::new(archive, mode, key)?;
        let mut codec = Codec::packed(archive, mode, key)?;
        let actual_len = storage.len()?;
        let mut states = BTreeMap::<[u8; 16], State>::new();
        let mut active: Option<State> = None;
        let mut headers = 0;
        let mut path_bytes = 0usize;
        let mut complete = 0;
        let mut incomplete = 0;
        let mut orphan_chunks = 0;
        let membership = index.recover(storage, anchor.index, anchor.sealed_len, |entry| {
            let record = OwnedRecord::decode(&entry.value)?;
            match entry.namespace {
                FILE => {
                    headers += 1;
                    path_bytes = path_bytes
                        .checked_add(entry.key.len())
                        .ok_or(Error::CorruptRecord)?;
                    // Explicit metadata budget, separate from bounded payload decode.
                    if headers > MAX_FILES || path_bytes > MAX_PATH_BYTES {
                        return Err(Error::SecurityLimitExceeded(
                            "candidate salvage metadata limit".into(),
                        ));
                    }
                    if !valid_path(&entry.key) || !record.extents.is_empty() {
                        return Err(Error::CorruptRecord);
                    }
                    let info = FileInfo::decode(&record.metadata, &codec)?;
                    let id = info.id;
                    if states
                        .insert(
                            id,
                            State {
                                path: entry.key,
                                info,
                                next: 0,
                                failed: false,
                                hash: Sha256::new(),
                            },
                        )
                        .is_some()
                    {
                        return Err(Error::CorruptRecord);
                    }
                }
                CHUNK => {
                    if entry.key.len() != 24 || record.extents.len() != 1 {
                        return Err(Error::CorruptRecord);
                    }
                    let id: [u8; 16] = entry.key[..16].try_into().unwrap();
                    let ordinal = u64::from_be_bytes(entry.key[16..].try_into().unwrap());
                    if active.as_ref().is_some_and(|state| state.info.id != id) {
                        finish(active.take().unwrap(), sink, &mut complete, &mut incomplete)?;
                    }
                    if active.is_none() {
                        active = states.remove(&id);
                        if let Some(state) = &active {
                            sink.begin(&state.path, state.info.len)?;
                        }
                    }
                    let Some(state) = &mut active else {
                        orphan_chunks += 1;
                        return Ok(());
                    };
                    if state.failed {
                        return Ok(());
                    }
                    let decoded = (|| {
                        if state.next != ordinal {
                            return Err(Error::CorruptRecord);
                        }
                        let slice = Slice::decode(&record.metadata)?;
                        state.info.validate_chunk(ordinal, &slice)?;
                        let fragment = slice.physical(record.extents[0])?;
                        if fragment
                            .start
                            .checked_add(fragment.len)
                            .is_none_or(|end| end > actual_len)
                        {
                            return Err(Error::Truncated);
                        }
                        codec.load(storage, fragment, anchor.sealed_len, &slice.descriptor)
                    })();
                    match decoded {
                        Ok(bytes) => {
                            sink.data(&bytes)?;
                            state.hash.update(&*bytes);
                            state.next += 1;
                        }
                        Err(Error::CorruptRecord | Error::CorruptHeader | Error::Truncated) => {
                            state.failed = true
                        }
                        Err(error) => return Err(error),
                    }
                }
                _ => return Err(Error::CorruptRecord),
            }
            Ok(())
        })?;
        if let Some(state) = active {
            finish(state, sink, &mut complete, &mut incomplete)?;
        }
        for state in states.into_values() {
            sink.begin(&state.path, state.info.len)?;
            finish(state, sink, &mut complete, &mut incomplete)?;
        }
        Ok(Report {
            generation: anchor.generation,
            complete,
            incomplete,
            membership,
            orphan_chunks,
        })
    }
}
