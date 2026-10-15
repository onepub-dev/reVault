//! Disposable unsigned/raw layout experiment, NOT a lockbox format or API.
//! Numeric file IDs replace the production path tree; timings are a lower bound.
//! No signing, encryption, mutation, migration, or production compatibility.
//! Each header, descriptor and block has a SHA-256 checksum. Block checksums
//! bind the file ID and block ordinal. Like unsigned lockboxes these checksums
//! detect corruption, not malicious rewriting of content and its checksum.
//! REVAULT_BLOCK_WORKERS=1..8 selects the lazy verification pool (default 1).
//! REVAULT_BLOCK_MMAP=1 enables Unix-only immutable-fixture mapping (default off).
//! REVAULT_BLOCK_READV=1 enables Unix vectored reads (default off, no mmap).
//! REVAULT_BLOCK_LARGE_SIZE changes the four-large-file case (default 8 MiB).
use rayon::prelude::*;
use revault_lockbox_api::{
    Compression, Encryption, Lockbox, LockboxCreateOptions, LockboxOpen, LockboxPath, Signing,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    time::Instant,
};
use zeroize::Zeroize;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const HEADER: usize = 64;
const ENTRY: usize = 64;
const BLOCK: usize = 16 * 1024;
const RECORD: usize = BLOCK + 32;
const MAGIC: &[u8; 8] = b"RVEXP001";

#[derive(Clone, Copy)]
struct Timings {
    open: f64,
    first_file: f64,
    total: f64,
    first_byte: f64,
    read_calls: f64,
    output_wipe: f64,
    drop: f64,
}

// Benchmark fixtures are exclusively owned by this process and never modified
// while readers exist. This is NOT a general-purpose safe mapping API.
#[cfg(unix)]
struct FixtureMapping {
    address: *mut libc::c_void,
    len: usize,
}
#[cfg(unix)]
impl FixtureMapping {
    unsafe fn new(file: &File, len: usize) -> io::Result<Self> {
        use std::os::fd::AsRawFd;
        if len == 0 || len > isize::MAX as usize {
            return Err(invalid());
        }
        // SAFETY: caller guarantees the file stays unmodified/untruncated for
        // the mapping lifetime. Descriptor is live; no executable/writable map.
        let address = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if address == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { address, len })
    }
    fn bytes(&self) -> &[u8] {
        // SAFETY: successful read-only mmap covers len initialized file bytes;
        // the borrowed slice cannot outlive self and no mutable alias is exposed.
        unsafe { std::slice::from_raw_parts(self.address.cast(), self.len) }
    }
}
#[cfg(unix)]
impl Drop for FixtureMapping {
    fn drop(&mut self) {
        // SAFETY: this object uniquely owns this still-live mmap region.
        unsafe {
            libc::munmap(self.address, self.len);
        }
    }
}

// Match production's full-capacity wiping rather than Vec's redundant wipe.
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
struct WipeBlock([u64; 8]);
impl zeroize::DefaultIsZeroes for WipeBlock {}
fn wipe(bytes: &mut Vec<u8>) {
    wipe_slice(bytes.as_mut_slice());
    bytes.spare_capacity_mut().zeroize();
}
fn wipe_slice(bytes: &mut [u8]) {
    // SAFETY: initialized bytes, all bit patterns valid, no padding; the three
    // disjoint aligned partitions cover the entire original slice.
    let (head, words, tail) = unsafe { bytes.align_to_mut::<WipeBlock>() };
    head.zeroize();
    words.zeroize();
    tail.zeroize();
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid experimental layout")
}
fn number(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
fn checksum(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg_attr(not(unix), allow(dead_code))]
fn initialize_output(source: &[u8], destination: &mut [std::mem::MaybeUninit<u8>]) {
    assert_eq!(source.len(), destination.len());
    // SAFETY: equal-length disjoint slices. Initialize destination without
    // reading it; the caller exposes it only after the full operation succeeds.
    unsafe {
        std::ptr::copy_nonoverlapping(
            source.as_ptr(),
            destination.as_mut_ptr().cast::<u8>(),
            source.len(),
        );
    }
}

#[cfg(unix)]
fn finish_vectored_read(
    vectors: &mut [libc::iovec],
    mut physical: libc::off_t,
    mut read_once: impl FnMut(&[libc::iovec], libc::off_t) -> io::Result<usize>,
) -> io::Result<()> {
    let mut at = 0;
    while at < vectors.len() {
        if vectors[at].iov_len == 0 {
            at += 1;
            continue;
        }
        let read = match read_once(&vectors[at..], physical) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if read == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        physical = physical
            .checked_add(libc::off_t::try_from(read).map_err(|_| invalid())?)
            .ok_or_else(invalid)?;
        let mut remaining = read;
        while at < vectors.len() && remaining >= vectors[at].iov_len {
            remaining -= vectors[at].iov_len;
            at += 1;
        }
        if remaining != 0 {
            if at == vectors.len() {
                return Err(invalid());
            }
            vectors[at].iov_base = vectors[at]
                .iov_base
                .cast::<u8>()
                .wrapping_add(remaining)
                .cast();
            vectors[at].iov_len -= remaining;
        }
    }
    Ok(())
}

fn block_hash(id: u64, ordinal: u64, data: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(id.to_le_bytes());
    hash.update(ordinal.to_le_bytes());
    hash.update(data);
    hash.finalize().into()
}

fn verify_batch(bytes: &[u8], id: u64, ordinal: usize) -> bool {
    bytes.chunks(RECORD).enumerate().all(|(index, record)| {
        record.len() >= 32
            && block_hash(id, (ordinal + index) as u64, &record[32..]) == record[..32]
    })
}

fn verify_parallel(
    bytes: &[u8],
    id: u64,
    ordinal: usize,
    pool: Option<&rayon::ThreadPool>,
) -> bool {
    if bytes.len() < 256 * 1024 || pool.is_none() {
        return verify_batch(bytes, id, ordinal);
    }
    pool.unwrap().install(|| {
        bytes
            .par_chunks(RECORD * 8)
            .enumerate()
            .all(|(index, part)| verify_batch(part, id, ordinal + index * 8))
    })
}

struct Prototype {
    file: File,
    count: u64,
    padded: bool,
    disk_len: u64,
    directory: Option<(u64, Vec<u8>)>,
    scratch: Vec<u8>,
    workers: usize,
    #[cfg(unix)]
    vectored_reads: bool,
    pool: Option<rayon::ThreadPool>,
    #[cfg(unix)]
    mapping: Option<FixtureMapping>,
}
impl Prototype {
    fn write(path: &Path, payloads: &[Vec<u8>], padded: bool) -> io::Result<()> {
        let mut file = File::create(path)?;
        let mut header = [0u8; HEADER];
        header[..8].copy_from_slice(MAGIC);
        header[8..16].copy_from_slice(&(payloads.len() as u64).to_le_bytes());
        header[16..24].copy_from_slice(&(BLOCK as u64).to_le_bytes());
        header[24..32].copy_from_slice(&u64::from(padded).to_le_bytes());
        let hash = checksum(&header[..32]);
        header[32..].copy_from_slice(&hash);
        file.write_all(&header)?;
        let align = |n: u64| if padded { n.div_ceil(1024) * 1024 } else { n };
        let mut offset = align((HEADER + ENTRY * payloads.len()) as u64);
        let mut offsets = Vec::new();
        for (id, payload) in payloads.iter().enumerate() {
            let mut entry = [0u8; ENTRY];
            entry[..8].copy_from_slice(&(id as u64).to_le_bytes());
            entry[8..16].copy_from_slice(&(payload.len() as u64).to_le_bytes());
            entry[16..24].copy_from_slice(&offset.to_le_bytes());
            let hash = checksum(&entry[..32]);
            entry[32..].copy_from_slice(&hash);
            file.write_all(&entry)?;
            offsets.push(offset);
            offset =
                align(offset + payload.len() as u64 + 32 * payload.len().div_ceil(BLOCK) as u64);
        }
        for (id, (payload, offset)) in payloads.iter().zip(offsets).enumerate() {
            file.seek(SeekFrom::Start(offset))?;
            for (ordinal, block) in payload.chunks(BLOCK).enumerate() {
                file.write_all(&block_hash(id as u64, ordinal as u64, block))?;
                file.write_all(block)?;
            }
        }
        file.set_len(offset)?;
        Ok(())
    }
    fn open(path: &Path) -> io::Result<Self> {
        if std::env::var("REVAULT_BLOCK_MMAP").is_ok_and(|v| v == "1")
            && std::env::var("REVAULT_BLOCK_READV").is_ok_and(|v| v == "1")
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "select mapping or vectored reads, not both",
            ));
        }
        #[cfg(not(unix))]
        if std::env::var("REVAULT_BLOCK_MMAP").is_ok_and(|v| v == "1")
            || std::env::var("REVAULT_BLOCK_READV").is_ok_and(|v| v == "1")
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "selected I/O experiment requires Unix",
            ));
        }
        let mut file = File::open(path)?;
        let disk_len = file.metadata()?.len();
        let mut header = [0u8; HEADER];
        file.read_exact(&mut header)?;
        if &header[..8] != MAGIC
            || checksum(&header[..32]) != header[32..]
            || number(&header[16..24]) != BLOCK as u64
            || number(&header[24..32]) > 1
        {
            return Err(invalid());
        }
        let count = number(&header[8..16]);
        if count > disk_len.saturating_sub(HEADER as u64) / ENTRY as u64 {
            return Err(invalid());
        }
        #[cfg(unix)]
        let mapping = if std::env::var("REVAULT_BLOCK_MMAP").is_ok_and(|v| v == "1") {
            // SAFETY: all callers belong to this benchmark/test. Fixture writers
            // are closed before open; corruption tests drop readers before edits.
            Some(unsafe {
                FixtureMapping::new(&file, usize::try_from(disk_len).map_err(|_| invalid())?)?
            })
        } else {
            None
        };
        Ok(Self {
            file,
            count,
            padded: number(&header[24..32]) == 1,
            disk_len,
            directory: None,
            scratch: Vec::new(),
            #[cfg(unix)]
            vectored_reads: std::env::var("REVAULT_BLOCK_READV").is_ok_and(|v| v == "1"),
            pool: None,
            #[cfg(unix)]
            mapping,
            workers: std::env::var("REVAULT_BLOCK_WORKERS")
                .map(|value| value.parse::<usize>().unwrap().clamp(1, 8))
                .unwrap_or(1),
        })
    }
    fn read(&mut self, id: usize, start: usize, len: usize) -> io::Result<Vec<u8>> {
        #[cfg(unix)]
        if self.vectored_reads {
            if let Some(output) = self.read_vectored(id, start, len)? {
                return Ok(output);
            }
        }
        #[cfg(unix)]
        if self.mapping.is_some() {
            let (size, offset) = self.descriptor(id, start, len)?;
            self.prepare_pool(len)?;
            let mapped = self.mapping.as_ref().unwrap().bytes();
            let offset = usize::try_from(offset).map_err(|_| invalid())?;
            let mut output = Vec::<u8>::with_capacity(len);
            let copy = |(index, destination): (usize, &mut [std::mem::MaybeUninit<u8>])| -> io::Result<()> {
                let mut logical = start + index * BLOCK * 8;
                let mut done = 0;
                while done < destination.len() {
                    let ordinal = logical / BLOCK;
                    let within = logical % BLOCK;
                    let n = BLOCK.min(size - ordinal * BLOCK);
                    let begin = offset + ordinal * RECORD;
                    let record = mapped.get(begin..begin + 32 + n).ok_or_else(invalid)?;
                    if block_hash(id as u64, ordinal as u64, &record[32..]) != record[..32] {
                        return Err(invalid());
                    }
                    let take = (n - within).min(destination.len() - done);
                    initialize_output(&record[32 + within..32 + within + take], &mut destination[done..done + take]);
                    logical += take;
                    done += take;
                }
                Ok(())
            };
            let destination = &mut output.spare_capacity_mut()[..len];
            let result = if let Some(pool) = self.pool.as_ref().filter(|_| len >= 256 * 1024) {
                pool.install(|| {
                    destination
                        .par_chunks_mut(BLOCK * 8)
                        .enumerate()
                        .try_for_each(copy)
                })
            } else {
                destination
                    .chunks_mut(BLOCK * 8)
                    .enumerate()
                    .try_for_each(copy)
            };
            if let Err(error) = result {
                wipe(&mut output);
                return Err(error);
            }
            // SAFETY: every disjoint destination chunk completed successfully,
            // initializing its full length. No bytes are exposed on failure;
            // wipe covers full spare capacity even while Vec length is zero.
            unsafe {
                output.set_len(len);
            }
            return Ok(output);
        }
        let mut output = Vec::new();
        if let Err(error) = self.visit(id, start, len, |data| {
            // Bounds have been validated before the visitor is called.
            if output.capacity() == 0 {
                output.reserve_exact(len);
            }
            output.extend_from_slice(data);
        }) {
            wipe(&mut output);
            return Err(error);
        }
        Ok(output)
    }
    #[cfg(unix)]
    fn read_vectored(
        &mut self,
        id: usize,
        start: usize,
        len: usize,
    ) -> io::Result<Option<Vec<u8>>> {
        use std::os::fd::AsRawFd;
        let (size, offset) = self.descriptor(id, start, len)?;
        let end = start + len;
        if start % BLOCK != 0 || (end % BLOCK != 0 && end != size) {
            return Ok(None);
        }
        self.prepare_pool(len)?;
        let fd = self.file.as_raw_fd();
        let group_blocks = 8;
        let group_bytes = group_blocks * BLOCK;
        let mut output = Vec::<u8>::with_capacity(len);
        let fill =
            |(index, destination): (usize, &mut [std::mem::MaybeUninit<u8>])| -> io::Result<()> {
                let ordinal = (start + index * group_bytes) / BLOCK;
                let mut digests = [[0u8; 32]; 8];
                let mut vectors = Vec::with_capacity(group_blocks * 2);
                let mut done = 0;
                for (block, digest) in digests
                    .iter_mut()
                    .enumerate()
                    .take(destination.len().div_ceil(BLOCK))
                {
                    let n = BLOCK.min(size - (ordinal + block) * BLOCK);
                    if n > destination.len() - done {
                        return Err(invalid());
                    }
                    vectors.push(libc::iovec {
                        iov_base: digest.as_mut_ptr().cast(),
                        iov_len: 32,
                    });
                    vectors.push(libc::iovec {
                        iov_base: destination[done..done + n].as_mut_ptr().cast(),
                        iov_len: n,
                    });
                    done += n;
                }
                let physical = libc::off_t::try_from(offset + ordinal as u64 * RECORD as u64)
                    .map_err(|_| invalid())?;
                finish_vectored_read(&mut vectors, physical, |vectors, physical| {
                    // SAFETY: live fd, <=16 stable disjoint writable iovecs.
                    // Output and checksum allocations cannot alias other workers.
                    let read = unsafe {
                        libc::preadv(fd, vectors.as_ptr(), vectors.len() as i32, physical)
                    };
                    if read < 0 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(read as usize)
                    }
                })?;
                // SAFETY: the completed vectored read initialized every destination
                // byte. No output is returned until its block checksums also pass.
                let initialized = unsafe {
                    std::slice::from_raw_parts(destination.as_ptr().cast::<u8>(), destination.len())
                };
                for (block, bytes) in initialized.chunks(BLOCK).enumerate() {
                    if block_hash(id as u64, (ordinal + block) as u64, bytes) != digests[block] {
                        return Err(invalid());
                    }
                }
                Ok(())
            };
        let destination = &mut output.spare_capacity_mut()[..len];
        let result = if let Some(pool) = self.pool.as_ref().filter(|_| len >= 256 * 1024) {
            pool.install(|| {
                destination
                    .par_chunks_mut(group_bytes)
                    .enumerate()
                    .try_for_each(fill)
            })
        } else {
            destination
                .chunks_mut(group_bytes)
                .enumerate()
                .try_for_each(fill)
        };
        if let Err(error) = result {
            wipe(&mut output);
            return Err(error);
        }
        // SAFETY: every chunk was fully initialized and verified; failures wipe
        // the allocation while its public length remains zero.
        unsafe {
            output.set_len(len);
        }
        Ok(Some(output))
    }
    fn descriptor(&mut self, id: usize, start: usize, len: usize) -> io::Result<(usize, u64)> {
        if id as u64 >= self.count {
            return Err(invalid());
        }
        let group = id as u64 / 64 * 64;
        if self
            .directory
            .as_ref()
            .is_none_or(|(cached, _)| *cached != group)
        {
            self.file
                .seek(SeekFrom::Start(HEADER as u64 + group * ENTRY as u64))?;
            let mut entries = vec![0; (self.count - group).min(64) as usize * ENTRY];
            self.file.read_exact(&mut entries)?;
            self.directory = Some((group, entries));
        }
        let mut entry = [0u8; ENTRY];
        let within = (id as u64 - group) as usize * ENTRY;
        entry.copy_from_slice(&self.directory.as_ref().unwrap().1[within..within + ENTRY]);
        if checksum(&entry[..32]) != entry[32..]
            || number(&entry[..8]) != id as u64
            || entry[24..32] != [0; 8]
        {
            return Err(invalid());
        }
        let size = usize::try_from(number(&entry[8..16])).map_err(|_| invalid())?;
        let offset = number(&entry[16..24]);
        let end = start.checked_add(len).ok_or_else(invalid)?;
        let stored = (size as u64)
            .checked_add(
                (size.div_ceil(BLOCK) as u64)
                    .checked_mul(32)
                    .ok_or_else(invalid)?,
            )
            .ok_or_else(invalid)?;
        if end > size
            || offset < HEADER as u64 + self.count * ENTRY as u64
            || offset
                .checked_add(stored)
                .is_none_or(|end| end > self.disk_len)
            || (self.padded && offset % 1024 != 0)
        {
            return Err(invalid());
        }
        Ok((size, offset))
    }
    fn prepare_pool(&mut self, len: usize) -> io::Result<()> {
        if len >= 256 * 1024 && self.workers > 1 && self.pool.is_none() {
            self.pool = Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(self.workers)
                    .build()
                    .map_err(io::Error::other)?,
            );
        }
        Ok(())
    }
    fn visit(
        &mut self,
        id: usize,
        start: usize,
        len: usize,
        mut consume: impl FnMut(&[u8]),
    ) -> io::Result<()> {
        let (size, offset) = self.descriptor(id, start, len)?;
        if len == 0 {
            return Ok(());
        }
        let end = start + len;
        let first = start / BLOCK;
        let last = (end - 1) / BLOCK;
        #[cfg(unix)]
        if self.vectored_reads && start % BLOCK == 0 && (end % BLOCK == 0 || end == size) {
            let mut cursor = start;
            while cursor < end {
                // Deliver one verified block before paying pool startup or
                // filling the throughput-sized buffers used for the remainder.
                let batch = if cursor == start {
                    BLOCK
                } else {
                    4 * 1024 * 1024
                };
                let take = batch.min(end - cursor);
                let mut output = self.read_vectored(id, cursor, take)?.ok_or_else(invalid)?;
                consume(&output);
                wipe(&mut output);
                cursor += take;
            }
            return Ok(());
        }
        self.prepare_pool(len)?;
        #[cfg(unix)]
        if let Some(mapping) = self.mapping.as_ref() {
            let mut ordinal = first;
            while ordinal <= last {
                let batch_end = (ordinal + 255).min(last + 1);
                let bytes_len =
                    (batch_end - ordinal) * 32 + (batch_end * BLOCK).min(size) - ordinal * BLOCK;
                let begin = usize::try_from(offset)
                    .map_err(|_| invalid())?
                    .checked_add(ordinal.checked_mul(RECORD).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                let bytes = mapping
                    .bytes()
                    .get(begin..begin.checked_add(bytes_len).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                if !verify_parallel(bytes, id as u64, ordinal, self.pool.as_ref()) {
                    return Err(invalid());
                }
                for (index, record) in bytes.chunks(RECORD).enumerate() {
                    let logical = (ordinal + index) * BLOCK;
                    let data = &record[32..];
                    consume(&data[start.saturating_sub(logical)..(end - logical).min(data.len())]);
                }
                ordinal = batch_end;
            }
            // Mapping aliases persistent archive bytes: no extra plaintext heap
            // scratch exists to wipe, and wiping here would modify the archive.
            return Ok(());
        }
        self.file
            .seek(SeekFrom::Start(offset + first as u64 * RECORD as u64))?;
        // Batch physical reads up to 1 MiB; never one syscall per small block.
        let mut ordinal = first;
        while ordinal <= last {
            let batch_end = (ordinal + 63).min(last + 1);
            let mut bytes = std::mem::take(&mut self.scratch);
            bytes.resize(
                (batch_end - ordinal) * 32 + (batch_end * BLOCK).min(size) - ordinal * BLOCK,
                0,
            );
            let result = (|| {
                self.file.read_exact(&mut bytes)?;
                if !verify_parallel(&bytes, id as u64, ordinal, self.pool.as_ref()) {
                    return Err(invalid());
                }
                let mut cursor = 0;
                for block in ordinal..batch_end {
                    let n = BLOCK.min(size - block * BLOCK);
                    let data = &bytes[cursor + 32..cursor + 32 + n];
                    let lo = start.saturating_sub(block * BLOCK);
                    let hi = (end - block * BLOCK).min(n);
                    consume(&data[lo..hi]);
                    cursor += 32 + n;
                }
                Ok(())
            })();
            wipe(&mut bytes);
            self.scratch = bytes;
            result?;
            ordinal = batch_end;
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn main() {
    let samples = std::env::var("REVAULT_BLOCK_SAMPLES")
        .map(|n| n.parse::<usize>().unwrap())
        .unwrap_or(9);
    assert!(samples > 0);
    let root =
        std::env::temp_dir().join(format!("revault-block-experiment-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    println!("files,size,access,format,open_us,first_byte_us,first_file_us,total_us,archive_bytes,read_calls_us,output_wipe_us,drop_us");
    let large_size = std::env::var("REVAULT_BLOCK_LARGE_SIZE")
        .map(|v| v.parse::<usize>().unwrap())
        .unwrap_or(8 * 1024 * 1024);
    assert!(large_size > 0 && large_size <= 256 * 1024 * 1024);
    for (count, size) in [(1, 128), (512, 4096), (4, large_size)] {
        let payloads: Vec<Vec<u8>> = (0..count)
            .map(|id| {
                (0..size)
                    .map(|n| ((n * 13 + n / 251 + id * 17) % 251) as u8)
                    .collect()
            })
            .collect();
        let paths: Vec<_> = (0..count)
            .map(|id| LockboxPath::new(format!("/file-{id:06}.bin")).unwrap())
            .collect();
        let zip_path = root.join("baseline.zip");
        let current_path = root.join("baseline.lbox");
        let mut zip = ZipWriter::new(File::create(&zip_path).unwrap());
        // Each scenario recreates its fixture; never overwrite an open archive.
        if current_path.exists() {
            fs::remove_file(&current_path).unwrap();
        }
        let mut current = Lockbox::create_file_with_options(
            &current_path,
            LockboxCreateOptions {
                compression: Compression::None,
                ..LockboxCreateOptions::new(Encryption::None, Signing::None)
            },
        )
        .unwrap();
        for (path, payload) in paths.iter().zip(&payloads) {
            zip.start_file(
                &path.as_str()[1..],
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(payload).unwrap();
            current.add_file(path, payload, false).unwrap();
        }
        zip.finish().unwrap();
        current.commit().unwrap();
        drop(current);
        for padded in [true, false] {
            Prototype::write(
                &root.join(if padded { "padded.exp" } else { "compact.exp" }),
                &payloads,
                padded,
            )
            .unwrap();
        }
        for access in ["stream", "whole", "range"] {
            let variants = 4;
            let mut measurements = vec![Vec::new(); variants];
            // Shuffle predecessors too: cyclic rotation leaves each contender
            // behind the same allocation/cache workload in almost every sample.
            let mut shuffle = 0x1234_5678_9abc_def0u64;
            for _ in 0..samples {
                let mut order: Vec<_> = (0..variants).collect();
                for index in (1..variants).rev() {
                    shuffle ^= shuffle << 13;
                    shuffle ^= shuffle >> 7;
                    shuffle ^= shuffle << 17;
                    order.swap(index, shuffle as usize % (index + 1));
                }
                for variant in order {
                    let file_path = match variant {
                        0 => &zip_path,
                        1 => &current_path,
                        2 => &root.join("padded.exp"),
                        _ => &root.join("compact.exp"),
                    };
                    let started = Instant::now();
                    let mut zip = (variant == 0)
                        .then(|| ZipArchive::new(File::open(file_path).unwrap()).unwrap());
                    let current = (variant == 1)
                        .then(|| Lockbox::open(file_path, LockboxOpen::Unencrypted).unwrap());
                    let mut prototype = (variant >= 2).then(|| Prototype::open(file_path).unwrap());
                    let open = started.elapsed().as_secs_f64() * 1e6;
                    let mut first = 0.0;
                    let mut first_byte = 0.0;
                    let mut read_calls = 0.0;
                    let mut output_wipe = 0.0;
                    for (position, id) in (0..count).rev().enumerate() {
                        let start = if access == "range" { size / 2 } else { 0 };
                        let len = if access == "range" {
                            8192.min(size - start)
                        } else {
                            size
                        };
                        let expected = &payloads[id][start..start + len];
                        let call_started = Instant::now();
                        if access == "stream" {
                            let mut checked = 0;
                            let mut consume = |data: &[u8]| {
                                if first_byte == 0.0 {
                                    first_byte = started.elapsed().as_secs_f64() * 1e6;
                                }
                                assert_eq!(data, &expected[checked..checked + data.len()]);
                                checked += data.len();
                            };
                            let mut check_reader = |reader: &mut dyn Read| {
                                let mut buffer = vec![0; 32 * 1024];
                                loop {
                                    let n = reader.read(&mut buffer).unwrap();
                                    if n == 0 {
                                        break;
                                    }
                                    consume(&buffer[..n]);
                                }
                                wipe(&mut buffer);
                            };
                            if let Some(zip) = zip.as_mut() {
                                check_reader(&mut zip.by_name(&paths[id].as_str()[1..]).unwrap());
                            } else if let Some(current) = current.as_ref() {
                                check_reader(&mut current.open_file(&paths[id]).unwrap());
                            } else {
                                prototype
                                    .as_mut()
                                    .unwrap()
                                    .visit(id, 0, size, &mut consume)
                                    .unwrap();
                            }
                            read_calls += call_started.elapsed().as_secs_f64() * 1e6;
                            assert_eq!(checked, size);
                            if position == 0 {
                                first = started.elapsed().as_secs_f64() * 1e6;
                            }
                            continue;
                        }
                        let mut bytes = if let Some(zip) = zip.as_mut() {
                            let mut bytes = Vec::with_capacity(len);
                            if access == "range" {
                                let mut reader =
                                    zip.by_name_seek(&paths[id].as_str()[1..]).unwrap();
                                reader.seek(SeekFrom::Start(start as u64)).unwrap();
                                reader.take(len as u64).read_to_end(&mut bytes).unwrap();
                            } else {
                                zip.by_name(&paths[id].as_str()[1..])
                                    .unwrap()
                                    .read_to_end(&mut bytes)
                                    .unwrap();
                            }
                            bytes
                        } else if let Some(current) = current.as_ref() {
                            current
                                .read_file_range(&paths[id], start as u64, len as u64)
                                .unwrap()
                        } else {
                            prototype.as_mut().unwrap().read(id, start, len).unwrap()
                        };
                        read_calls += call_started.elapsed().as_secs_f64() * 1e6;
                        if first_byte == 0.0 {
                            first_byte = started.elapsed().as_secs_f64() * 1e6;
                        }
                        assert_eq!(bytes, expected);
                        if position == 0 {
                            first = started.elapsed().as_secs_f64() * 1e6;
                        }
                        // Apply the same output-buffer wiping to every contender.
                        let wipe_started = Instant::now();
                        wipe(&mut bytes);
                        output_wipe += wipe_started.elapsed().as_secs_f64() * 1e6;
                    }
                    let drop_started = Instant::now();
                    drop(current);
                    drop(prototype);
                    drop(zip);
                    measurements[variant].push(Timings {
                        open,
                        first_file: first,
                        total: started.elapsed().as_secs_f64() * 1e6,
                        first_byte,
                        read_calls,
                        output_wipe,
                        drop: drop_started.elapsed().as_secs_f64() * 1e6,
                    });
                }
            }
            for (variant, values) in measurements.iter().enumerate() {
                let median = |field: fn(&Timings) -> f64| {
                    let mut v: Vec<_> = values.iter().map(field).collect();
                    v.sort_by(f64::total_cmp);
                    v[v.len() / 2]
                };
                let name = ["zip", "current", "prototype-padded", "prototype-compact"][variant];
                let path = match variant {
                    0 => zip_path.clone(),
                    1 => current_path.clone(),
                    2 => root.join("padded.exp"),
                    _ => root.join("compact.exp"),
                };
                println!(
                    "{count},{size},{access},{name},{:.3},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.3}",
                    median(|v| v.open),
                    median(|v| v.first_byte),
                    median(|v| v.first_file),
                    median(|v| v.total),
                    fs::metadata(path).unwrap().len(),
                    median(|v| v.read_calls),
                    median(|v| v.output_wipe),
                    median(|v| v.drop)
                );
            }
        }
    }
    // Deliberately retain this uniquely named benchmark fixture for inspection.
    eprintln!("Fixtures retained at {}", root.display());
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[cfg(unix)]
    #[test]
    fn streaming_delivers_first_verified_block_before_bulk_buffers() {
        let path =
            std::env::temp_dir().join(format!("revault-first-block-tests-{}", std::process::id()));
        let payload: Vec<u8> = (0..BLOCK * 270 + 13).map(|n| (n % 251) as u8).collect();
        Prototype::write(&path, std::slice::from_ref(&payload), true).unwrap();
        for workers in [1, 4, 8] {
            let mut reader = Prototype::open(&path).unwrap();
            reader.mapping = None;
            reader.vectored_reads = true;
            reader.workers = workers;
            let mut sizes = Vec::new();
            let mut done = 0;
            reader
                .visit(0, 0, payload.len(), |bytes| {
                    assert_eq!(bytes, &payload[done..done + bytes.len()]);
                    done += bytes.len();
                    sizes.push(bytes.len());
                })
                .unwrap();
            assert_eq!(done, payload.len());
            assert_eq!(sizes, [BLOCK, 4 * 1024 * 1024, 13 * BLOCK + 13]);
        }
        // Private codec fault injection, not CLI E2E: the second block must
        // never be delivered if its digest fails, even after a valid prefix.
        let mut bytes = fs::read(&path).unwrap();
        let offset = number(&bytes[HEADER + 16..HEADER + 24]) as usize;
        bytes[offset + RECORD + 32] ^= 1;
        fs::write(&path, bytes).unwrap();
        let mut reader = Prototype::open(&path).unwrap();
        reader.mapping = None;
        reader.vectored_reads = true;
        let mut delivered = Vec::new();
        assert!(reader
            .visit(0, 0, payload.len(), |bytes| delivered
                .extend_from_slice(bytes))
            .is_err());
        assert_eq!(delivered, payload[..BLOCK]);
        drop(reader);
        fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn vectored_short_reads_interrupts_and_eof() {
        for available in [18, 10] {
            let source: Vec<u8> = (0..available).collect();
            let mut first = [0xa5u8; 4];
            let mut second = [0xa5u8; 11];
            let mut vectors = [
                libc::iovec {
                    iov_base: first.as_mut_ptr().cast(),
                    iov_len: first.len(),
                },
                libc::iovec {
                    iov_base: second.as_mut_ptr().cast(),
                    iov_len: second.len(),
                },
            ];
            let mut calls = 0;
            let result = finish_vectored_read(&mut vectors, 3, |vectors, physical| {
                calls += 1;
                if calls == 1 {
                    return Err(io::Error::from(io::ErrorKind::Interrupted));
                }
                let mut done = 0;
                let limit = 3.min(source.len() - physical as usize);
                for vector in vectors {
                    let n = vector.iov_len.min(limit - done);
                    // SAFETY: test-owned disjoint buffers; the helper only
                    // advances/shrinks their original valid ranges.
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            source[physical as usize + done..].as_ptr(),
                            vector.iov_base.cast::<u8>(),
                            n,
                        );
                    }
                    done += n;
                    if done == limit {
                        break;
                    }
                }
                Ok(done)
            });
            if available == 18 {
                result.unwrap();
                assert_eq!([first.as_slice(), second.as_slice()].concat(), source[3..]);
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
                assert_eq!(first, source[3..7]);
                assert_eq!(&second[..3], &source[7..]);
                assert!(second[3..].iter().all(|byte| *byte == 0xa5));
            }
            assert!(calls > 3);
        }
    }
    #[test]
    fn output_store_alignment_and_neighbors() {
        {
            for offset in 0..32 {
                for len in [0, 1, BLOCK - 1, BLOCK, BLOCK + 17] {
                    let source: Vec<_> = (0..len).map(|n| (n % 251) as u8).collect();
                    let mut buffer = vec![std::mem::MaybeUninit::new(0xa5u8); len + 64];
                    initialize_output(&source, &mut buffer[offset..offset + len]);
                    // SAFETY: every element was initialized before the call;
                    // initialize_output may overwrite but never deinitialize it.
                    let actual: Vec<_> = buffer
                        .iter()
                        .map(|byte| unsafe { byte.assume_init() })
                        .collect();
                    assert_eq!(&actual[offset..offset + len], source);
                    assert!(actual[..offset].iter().all(|byte| *byte == 0xa5));
                    assert!(actual[offset + len..].iter().all(|byte| *byte == 0xa5));
                }
            }
        }
    }
    #[test]
    fn lazy_directory_group_boundaries() {
        let path = std::env::temp_dir().join(format!(
            "revault-block-directory-{}.exp",
            std::process::id()
        ));
        let payloads: Vec<_> = (0..130).map(|id| vec![id as u8; 37]).collect();
        Prototype::write(&path, &payloads, false).unwrap();
        let mut reader = Prototype::open(&path).unwrap();
        assert!(reader.directory.is_none());
        assert!(reader.scratch.is_empty());
        for id in [129, 64, 63, 0, 65, 128] {
            assert_eq!(reader.read(id, 1, 35).unwrap(), payloads[id][1..36]);
            assert!(reader.directory.as_ref().unwrap().1.len() <= 64 * ENTRY);
            assert!(reader.scratch.iter().all(|byte| *byte == 0));
        }
        drop(reader);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn output_initialization_at_block_and_worker_boundaries() {
        let path =
            std::env::temp_dir().join(format!("revault-block-output-{}.exp", std::process::id()));
        let payload: Vec<_> = (0..BLOCK * 35 + 17).map(|n| (n % 251) as u8).collect();
        Prototype::write(&path, std::slice::from_ref(&payload), true).unwrap();
        for workers in [1, 4, 8] {
            let mut reader = Prototype::open(&path).unwrap();
            reader.workers = workers;
            for start in [
                0,
                1,
                BLOCK,
                BLOCK * 8,
                BLOCK - 1,
                BLOCK + 1,
                BLOCK * 8 - 1,
                payload.len() - 1,
                payload.len(),
            ] {
                for wanted in [
                    0,
                    1,
                    BLOCK,
                    BLOCK * 8,
                    BLOCK * 16,
                    BLOCK * 32,
                    BLOCK - 1,
                    BLOCK + 1,
                    BLOCK * 8 - 1,
                    BLOCK * 8 + 1,
                    BLOCK * 17 + 3,
                ] {
                    let len = wanted.min(payload.len() - start);
                    assert_eq!(
                        reader.read(0, start, len).unwrap(),
                        payload[start..start + len]
                    );
                }
            }
        }
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn roundtrip_padding_ranges_and_corruption() {
        let root = std::env::temp_dir().join(format!("revault-block-tests-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let payloads = vec![
            vec![],
            vec![7; 127],
            (0..BLOCK * 270 + 13).map(|n| (n % 251) as u8).collect(),
        ];
        for padded in [true, false] {
            let path = root.join(if padded { "padded.exp" } else { "compact.exp" });
            Prototype::write(&path, &payloads, padded).unwrap();
            let mut reader = Prototype::open(&path).unwrap();
            assert_eq!(reader.padded, padded);
            assert!(reader.pool.is_none());
            for (id, payload) in payloads.iter().enumerate() {
                assert_eq!(reader.read(id, 0, payload.len()).unwrap(), *payload);
                assert!(reader.scratch.iter().all(|byte| *byte == 0));
                for offset in [0, payload.len() / 2, payload.len()] {
                    let len = 8192.min(payload.len() - offset);
                    assert_eq!(
                        reader.read(id, offset, len).unwrap(),
                        payload[offset..offset + len]
                    );
                }
                assert!(reader.read(id, payload.len(), 1).is_err());
                assert!(reader.read(id, usize::MAX, 2).is_err());
            }
            assert!(reader.read(3, 0, 0).is_err());
            drop(reader);
            let original = fs::read(&path).unwrap();
            let entry = HEADER + 2 * ENTRY;
            let data_offset = number(&original[entry + 16..entry + 24]) as usize;
            // Direct mutation is intentional: this is a private codec unit test,
            // not CLI E2E. Corrupt a byte outside the requested slice, same block.
            for at in [0, 24, entry + 8, data_offset + 32 + 100] {
                let mut corrupt = original.clone();
                corrupt[at] ^= 1;
                fs::write(&path, corrupt).unwrap();
                assert!(Prototype::open(&path)
                    .and_then(|mut p| p.read(2, 0, 1))
                    .is_err());
            }
            fs::write(&path, &original[..original.len() / 2]).unwrap();
            assert!(Prototype::open(&path)
                .and_then(|mut p| p.read(2, 0, 1))
                .is_err());
            let mut corrupt = original.clone();
            corrupt[data_offset + 20 * RECORD + 32] ^= 1;
            fs::write(&path, corrupt).unwrap();
            for workers in [1, 4] {
                let mut reader = Prototype::open(&path).unwrap();
                reader.workers = workers;
                assert!(reader.read(2, 0, payloads[2].len()).is_err());
                assert!(reader.scratch.iter().all(|byte| *byte == 0));
            }
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(root).unwrap();
    }
}
