//! Disposable unsigned/raw layout experiment, NOT a lockbox format or API.
//! Numeric file IDs replace the production path tree; timings are a lower bound.
//! No signing, encryption, mutation, migration, or production compatibility.
//! Each header, descriptor and block has a SHA-256 checksum. Block checksums
//! bind the file ID and block ordinal. Like unsigned lockboxes these checksums
//! detect corruption, not malicious rewriting of content and its checksum.
//! REVAULT_BLOCK_WORKERS=1..8 selects the lazy verification pool (default 1).
//! REVAULT_BLOCK_MMAP=1 enables Unix-only immutable-fixture mapping (default off).
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
        #[cfg(not(unix))]
        if std::env::var("REVAULT_BLOCK_MMAP").is_ok_and(|v| v == "1") {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "fixture mapping requires Unix",
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
            pool: None,
            #[cfg(unix)]
            mapping,
            workers: std::env::var("REVAULT_BLOCK_WORKERS")
                .map(|value| value.parse::<usize>().unwrap().clamp(1, 8))
                .unwrap_or(1),
        })
    }
    fn read(&mut self, id: usize, start: usize, len: usize) -> io::Result<Vec<u8>> {
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
    fn visit(
        &mut self,
        id: usize,
        start: usize,
        len: usize,
        mut consume: impl FnMut(&[u8]),
    ) -> io::Result<()> {
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
        if len == 0 {
            return Ok(());
        }
        let first = start / BLOCK;
        let last = (end - 1) / BLOCK;
        if len >= 256 * 1024 && self.workers > 1 && self.pool.is_none() {
            self.pool = Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(self.workers)
                    .build()
                    .map_err(io::Error::other)?,
            );
        }
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
    println!("files,size,access,format,open_us,first_byte_us,first_file_us,total_us,archive_bytes");
    for (count, size) in [(1, 128), (512, 4096), (4, 8 * 1024 * 1024)] {
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
            let mut measurements = vec![Vec::new(); 4];
            // Rotate order every sample to reduce systematic ordering bias.
            for sample in 0..samples {
                for turn in 0..4 {
                    let variant = (sample + turn) % 4;
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
                    for (position, id) in (0..count).rev().enumerate() {
                        let start = if access == "range" { size / 2 } else { 0 };
                        let len = if access == "range" {
                            8192.min(size - start)
                        } else {
                            size
                        };
                        let expected = &payloads[id][start..start + len];
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
                        if first_byte == 0.0 {
                            first_byte = started.elapsed().as_secs_f64() * 1e6;
                        }
                        assert_eq!(bytes, expected);
                        if position == 0 {
                            first = started.elapsed().as_secs_f64() * 1e6;
                        }
                        // Apply the same output-buffer wiping to every contender.
                        wipe(&mut bytes);
                    }
                    drop(current);
                    drop(prototype);
                    drop(zip);
                    measurements[variant].push((
                        open,
                        first,
                        started.elapsed().as_secs_f64() * 1e6,
                        first_byte,
                    ));
                }
            }
            for (variant, values) in measurements.iter().enumerate() {
                let median = |field: fn(&(f64, f64, f64, f64)) -> f64| {
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
                    "{count},{size},{access},{name},{:.3},{:.3},{:.3},{:.3},{}",
                    median(|v| v.0),
                    median(|v| v.3),
                    median(|v| v.1),
                    median(|v| v.2),
                    fs::metadata(path).unwrap().len()
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
