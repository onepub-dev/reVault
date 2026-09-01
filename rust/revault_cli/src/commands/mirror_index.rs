use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const INITIAL_BUCKETS: u64 = 1024;
const NODE_BYTES: u64 = 24;
const RECORD_HEADER_BYTES: u64 = 8;
const MAX_RECORD_PART_BYTES: usize = 64 * 1024 * 1024;

/// An append-only binary key/value table with a growable, on-disk hash index.
///
/// Heap use is bounded by one key and value. Buckets and collision nodes remain
/// on disk, and the bucket array doubles at a 75% load factor. The format is
/// deliberately command-local: it is not a persistent compatibility surface.
pub(super) struct BinaryTable {
    data_path: PathBuf,
    state: Mutex<TableState>,
}

struct TableState {
    data: File,
    buckets: File,
    nodes: File,
    bucket_count: u64,
    entries: u64,
}

impl BinaryTable {
    pub(super) fn create(directory: &Path, name: &str) -> io::Result<Self> {
        let data_path = directory.join(format!("{name}.records"));
        let data = create_file(&data_path)?;
        let mut buckets = create_file(&directory.join(format!("{name}.buckets")))?;
        initialize_buckets(&mut buckets, INITIAL_BUCKETS)?;
        let nodes = create_file(&directory.join(format!("{name}.nodes")))?;
        Ok(Self {
            data_path,
            state: Mutex::new(TableState {
                data,
                buckets,
                nodes,
                bucket_count: INITIAL_BUCKETS,
                entries: 0,
            }),
        })
    }

    pub(super) fn clear(&self) -> io::Result<()> {
        let mut state = self.lock()?;
        state.data.set_len(0)?;
        state.data.seek(SeekFrom::Start(0))?;
        state.nodes.set_len(0)?;
        state.nodes.seek(SeekFrom::Start(0))?;
        state.bucket_count = INITIAL_BUCKETS;
        state.entries = 0;
        initialize_buckets(&mut state.buckets, INITIAL_BUCKETS)
    }

    pub(super) fn insert_if_absent(&self, key: &[u8], value: &[u8]) -> io::Result<bool> {
        validate_part(key)?;
        validate_part(value)?;
        let mut state = self.lock()?;
        if state.entries.saturating_mul(4) >= state.bucket_count.saturating_mul(3) {
            rehash(&mut state)?;
        }
        let hash = stable_hash(key);
        let bucket = hash % state.bucket_count;
        let mut node = read_bucket(&mut state.buckets, bucket)?;
        while let Some(offset) = decode_offset(node) {
            let stored = read_node(&mut state.nodes, offset)?;
            if stored.hash == hash && read_key(&mut state.data, stored.record_offset)? == key {
                return Ok(false);
            }
            node = stored.next;
        }

        let record_offset = append_record(&mut state.data, key, value)?;
        let previous = read_bucket(&mut state.buckets, bucket)?;
        let node_offset = state.nodes.seek(SeekFrom::End(0))?;
        write_node(
            &mut state.nodes,
            node_offset,
            Node {
                hash,
                record_offset,
                next: previous,
            },
        )?;
        write_bucket(&mut state.buckets, bucket, encode_offset(node_offset))?;
        state.entries += 1;
        Ok(true)
    }

    pub(super) fn contains(&self, key: &[u8]) -> io::Result<bool> {
        Ok(self.get(key)?.is_some())
    }

    pub(super) fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        let mut state = self.lock()?;
        let hash = stable_hash(key);
        let bucket = hash % state.bucket_count;
        let mut node = read_bucket(&mut state.buckets, bucket)?;
        while let Some(offset) = decode_offset(node) {
            let stored = read_node(&mut state.nodes, offset)?;
            if stored.hash == hash {
                let (stored_key, value) = read_record(&mut state.data, stored.record_offset)?;
                if stored_key == key {
                    return Ok(Some(value));
                }
            }
            node = stored.next;
        }
        Ok(None)
    }

    pub(super) fn iter(&self) -> io::Result<BinaryTableIter> {
        Ok(BinaryTableIter {
            file: File::open(&self.data_path)?,
            offset: 0,
            length: std::fs::metadata(&self.data_path)?.len(),
        })
    }

    fn lock(&self) -> io::Result<std::sync::MutexGuard<'_, TableState>> {
        self.state
            .lock()
            .map_err(|_| io::Error::other("temporary binary index lock was poisoned"))
    }
}

pub(super) struct BinaryTableIter {
    file: File,
    offset: u64,
    length: u64,
}

impl Iterator for BinaryTableIter {
    type Item = io::Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.length {
            return None;
        }
        if self.offset > self.length {
            return Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "temporary binary index extends beyond its recorded length",
            )));
        }
        let offset = self.offset;
        match read_record(&mut self.file, offset) {
            Ok((key, value)) => {
                self.offset = offset + RECORD_HEADER_BYTES + key.len() as u64 + value.len() as u64;
                Some(Ok((key, value)))
            }
            Err(error) => {
                self.offset = self.length.saturating_add(1);
                Some(Err(error))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Node {
    hash: u64,
    record_offset: u64,
    next: u64,
}

fn create_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(path)
}

fn initialize_buckets(file: &mut File, count: u64) -> io::Result<()> {
    file.set_len(0)?;
    file.set_len(count.checked_mul(8).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "binary index bucket size overflow",
        )
    })?)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(())
}

fn validate_part(value: &[u8]) -> io::Result<()> {
    if value.len() > MAX_RECORD_PART_BYTES || value.len() > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "temporary binary index record is too large",
        ));
    }
    Ok(())
}

fn append_record(file: &mut File, key: &[u8], value: &[u8]) -> io::Result<u64> {
    let offset = file.seek(SeekFrom::End(0))?;
    file.write_all(&(key.len() as u32).to_le_bytes())?;
    file.write_all(&(value.len() as u32).to_le_bytes())?;
    file.write_all(key)?;
    file.write_all(value)?;
    Ok(offset)
}

fn read_key(file: &mut File, offset: u64) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(offset))?;
    let key_len = read_u32(file)? as usize;
    let value_len = read_u32(file)? as usize;
    validate_lengths(key_len, value_len)?;
    let mut key = vec![0; key_len];
    file.read_exact(&mut key)?;
    Ok(key)
}

fn read_record(file: &mut File, offset: u64) -> io::Result<(Vec<u8>, Vec<u8>)> {
    file.seek(SeekFrom::Start(offset))?;
    let key_len = read_u32(file)? as usize;
    let value_len = read_u32(file)? as usize;
    validate_lengths(key_len, value_len)?;
    let mut key = vec![0; key_len];
    let mut value = vec![0; value_len];
    file.read_exact(&mut key)?;
    file.read_exact(&mut value)?;
    Ok((key, value))
}

fn validate_lengths(key_len: usize, value_len: usize) -> io::Result<()> {
    if key_len > MAX_RECORD_PART_BYTES || value_len > MAX_RECORD_PART_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "temporary binary index record length is invalid",
        ));
    }
    Ok(())
}

fn read_u32(file: &mut File) -> io::Result<u32> {
    let mut bytes = [0; 4];
    file.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_bucket(file: &mut File, bucket: u64) -> io::Result<u64> {
    file.seek(SeekFrom::Start(bucket * 8))?;
    let mut bytes = [0; 8];
    file.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn write_bucket(file: &mut File, bucket: u64, value: u64) -> io::Result<()> {
    file.seek(SeekFrom::Start(bucket * 8))?;
    file.write_all(&value.to_le_bytes())
}

fn read_node(file: &mut File, offset: u64) -> io::Result<Node> {
    if offset % NODE_BYTES != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "temporary binary index node offset is invalid",
        ));
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = [0; NODE_BYTES as usize];
    file.read_exact(&mut bytes)?;
    Ok(Node {
        hash: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        record_offset: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        next: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
    })
}

fn write_node(file: &mut File, offset: u64, node: Node) -> io::Result<()> {
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(&node.hash.to_le_bytes())?;
    file.write_all(&node.record_offset.to_le_bytes())?;
    file.write_all(&node.next.to_le_bytes())
}

fn rehash(state: &mut TableState) -> io::Result<()> {
    let new_count = state
        .bucket_count
        .checked_mul(2)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "binary index is too large"))?;
    initialize_buckets(&mut state.buckets, new_count)?;
    let node_count = state.nodes.metadata()?.len() / NODE_BYTES;
    for index in 0..node_count {
        let offset = index * NODE_BYTES;
        let mut node = read_node(&mut state.nodes, offset)?;
        let bucket = node.hash % new_count;
        node.next = read_bucket(&mut state.buckets, bucket)?;
        write_node(&mut state.nodes, offset, node)?;
        write_bucket(&mut state.buckets, bucket, encode_offset(offset))?;
    }
    state.bucket_count = new_count;
    Ok(())
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn encode_offset(offset: u64) -> u64 {
    offset + 1
}

fn decode_offset(encoded: u64) -> Option<u64> {
    encoded.checked_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_and_resolves_binary_keys_without_loading_the_table() {
        let directory = tempfile::tempdir().unwrap();
        let table = BinaryTable::create(directory.path(), "test").unwrap();
        for value in 0..5000_u32 {
            assert!(table
                .insert_if_absent(&value.to_le_bytes(), &(value * 2).to_le_bytes())
                .unwrap());
        }
        assert!(!table
            .insert_if_absent(&123_u32.to_le_bytes(), b"duplicate")
            .unwrap());
        for value in [0_u32, 123, 1024, 4999] {
            assert_eq!(
                table.get(&value.to_le_bytes()).unwrap().unwrap(),
                (value * 2).to_le_bytes()
            );
        }
        assert_eq!(table.iter().unwrap().count(), 5000);
        table.clear().unwrap();
        assert!(!table.contains(&123_u32.to_le_bytes()).unwrap());
    }
}
