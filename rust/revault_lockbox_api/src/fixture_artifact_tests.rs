use std::path::{Path, PathBuf};

use crate::commit_auth::decode_commit_auth;
use crate::compression::{decode_compression_frame, COMPRESSION_NONE};
use crate::compression_frame_manifest::CompressionFrameManifest;
use crate::constants::DEFAULT_FILE_PERMISSIONS;
use crate::file_chunk::{CompressionFrameSegment, FileChunk};
use crate::file_format::{
    decode_compression_frame_segment_payload_view, encode_compression_frame_segment_payload,
};
use crate::lockbox_path::LockboxPath;
use crate::node_kind::NodeKind;
use crate::page::{PAGE_HEADER_LEN, PAGE_MAGIC};
use crate::page_cache::{PageCache, PageReadKey, PageSecurity};
use crate::storage::StorageBackend;
use crate::toc_codec::{decode_toc_entries, encode_toc_entries};
use crate::toc_entry::TocEntry;
use crate::{CacheLimit, Error, ListOptions, Lockbox, LockboxId, VariableName};

const FIXTURE_KEY: &[u8] = b"lockbox fixture content key";

#[test]
fn golden_content_key_basic_lockbox_opens() {
    let bytes = read_fixture("golden/v1/content_key_basic.lbox.hex");

    let reopened = Lockbox::open_bytes_with_key(bytes, FIXTURE_KEY).unwrap();

    assert_eq!(
        reopened
            .get_file(&LockboxPath::new("/docs/readme.txt").unwrap())
            .unwrap(),
        b"golden fixture readme\n"
    );
    assert_eq!(
        reopened
            .get_file(&LockboxPath::new("/data/repeated.bin").unwrap())
            .unwrap(),
        vec![b'x'; 4096]
    );
    assert_eq!(
        reopened
            .get_variable(&VariableName::new("FEATURE_FLAG").unwrap())
            .unwrap()
            .as_deref(),
        Some("enabled")
    );
    assert_eq!(
        reopened
            .get_symlink_target(&LockboxPath::new("/docs/latest.txt").unwrap())
            .unwrap(),
        "/docs/readme.txt"
    );

    let entries = reopened
        .list(ListOptions {
            path: LockboxPath::new("/").unwrap(),
            glob: None,
            recursive: true,
            include_files: true,
            include_symlinks: true,
            include_directories: true,
            limit: None,
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry.path == "/docs/readme.txt" && entry.permissions == 0o640));
}

#[test]
fn adversarial_page_oversized_body_header_is_rejected_before_body_read() {
    let bytes = read_fixture("adversarial/v1/page/oversized_body_header.page.hex");
    let storage = StorageBackend::memory(bytes);
    let mut cache = PageCache::new(CacheLimit::Bytes(1024 * 1024));

    assert!(matches!(
        cache.read_page(
            &storage,
            0,
            LockboxId::from_bytes([1; 16]),
            PageSecurity::Normal,
            PageReadKey::Normal(FIXTURE_KEY),
        ),
        Err(Error::SecurityLimitExceeded(_))
    ));
}

#[test]
fn adversarial_commit_auth_oversized_key_count_is_rejected_before_allocating() {
    let bytes = read_fixture("adversarial/v1/commit_auth/oversized_key_count.bin.hex");

    assert!(matches!(
        decode_commit_auth(&bytes),
        Err(Error::SecurityLimitExceeded(_))
    ));
}

#[test]
fn adversarial_compression_frame_stored_len_gt_frame_len_is_rejected() {
    let bytes = read_fixture("adversarial/v1/compression/stored_len_gt_frame.bin.hex");

    assert!(matches!(
        decode_compression_frame(COMPRESSION_NONE, &bytes, 1),
        Err(Error::CorruptRecord)
    ));
}

#[test]
fn adversarial_toc_compressed_len_gt_frame_len_is_rejected() {
    let bytes = read_fixture("adversarial/v1/toc/compressed_len_gt_frame.toc.hex");

    assert!(matches!(
        decode_toc_entries(&bytes),
        Err(Error::CorruptRecord)
    ));
}

#[test]
fn adversarial_segment_compressed_len_gt_frame_len_is_rejected() {
    let bytes = read_fixture("adversarial/v1/payload/segment_compressed_len_gt_frame.payload.hex");

    assert!(matches!(
        decode_compression_frame_segment_payload_view(&bytes),
        Err(Error::CorruptRecord)
    ));
}

#[test]
#[ignore = "set LOCKBOX_UPDATE_FIXTURES=1 to rewrite checked-in fixture artifacts"]
fn write_fixture_artifacts() {
    if std::env::var("LOCKBOX_UPDATE_FIXTURES").as_deref() != Ok("1") {
        panic!("set LOCKBOX_UPDATE_FIXTURES=1 before rewriting fixture artifacts");
    }

    write_fixture(
        "golden/v1/content_key_basic.lbox.hex",
        &golden_content_key_basic_lockbox(),
    );
    write_fixture(
        "adversarial/v1/page/oversized_body_header.page.hex",
        &adversarial_oversized_body_header_page(),
    );
    write_fixture(
        "adversarial/v1/commit_auth/oversized_key_count.bin.hex",
        &adversarial_commit_auth_oversized_key_count(),
    );
    write_fixture(
        "adversarial/v1/compression/stored_len_gt_frame.bin.hex",
        b"xx",
    );
    write_fixture(
        "adversarial/v1/toc/compressed_len_gt_frame.toc.hex",
        &adversarial_toc_compressed_len_gt_frame(),
    );
    write_fixture(
        "adversarial/v1/payload/segment_compressed_len_gt_frame.payload.hex",
        &adversarial_segment_compressed_len_gt_frame(),
    );
}

fn golden_content_key_basic_lockbox() -> Vec<u8> {
    let mut lockbox = Lockbox::create(FIXTURE_KEY);
    lockbox
        .add_file_with_permissions(
            &LockboxPath::new("/docs/readme.txt").unwrap(),
            b"golden fixture readme\n",
            0o640,
            false,
        )
        .unwrap();
    lockbox
        .add_file(
            &LockboxPath::new("/data/repeated.bin").unwrap(),
            &vec![b'x'; 4096],
            false,
        )
        .unwrap();
    lockbox
        .add_symlink(
            &LockboxPath::new("/docs/latest.txt").unwrap(),
            &LockboxPath::new("/docs/readme.txt").unwrap(),
            false,
        )
        .unwrap();
    lockbox
        .set_variable(&VariableName::new("FEATURE_FLAG").unwrap(), "enabled")
        .unwrap();
    lockbox.commit().unwrap();
    lockbox.to_bytes()
}

fn adversarial_oversized_body_header_page() -> Vec<u8> {
    let mut page = vec![0_u8; PAGE_HEADER_LEN];
    page[0..8].copy_from_slice(PAGE_MAGIC);
    page[12..16].copy_from_slice(&(PAGE_HEADER_LEN as u32).to_le_bytes());
    page[44..48].copy_from_slice(&u32::MAX.to_le_bytes());
    page
}

fn adversarial_commit_auth_oversized_key_count() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"LBX1AUTH");
    payload.push(1);
    payload.extend_from_slice(&[0; 7]);
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&7_u64.to_le_bytes());
    payload.extend_from_slice(&1024_u64.to_le_bytes());
    payload.extend_from_slice(&[2; 32]);
    payload.extend_from_slice(&512_u64.to_le_bytes());
    payload.extend_from_slice(&[3; 32]);
    payload.extend_from_slice(&9_u64.to_le_bytes());
    payload.extend_from_slice(&u32::MAX.to_le_bytes());
    payload
}

fn adversarial_toc_compressed_len_gt_frame() -> Vec<u8> {
    let mut entry = TocEntry {
        path: LockboxPath::new("/tree/file.bin").unwrap(),
        len: 128,
        record_offset: 4096,
        record_len: 4096,
        record_object_id: 10,
        deleted: false,
        node_kind: NodeKind::File,
        permissions: DEFAULT_FILE_PERMISSIONS,
        chunks: vec![shared_frame_chunk()],
    };
    entry.chunks[0].compression_frame_len = 4;
    entry.chunks[0].compressed_len = 5;
    encode_toc_entries([&entry])
}

fn adversarial_segment_compressed_len_gt_frame() -> Vec<u8> {
    let manifest = CompressionFrameManifest {
        compression_frame_id: 7,
        compression: COMPRESSION_NONE,
        compression_frame_len: 4,
        compressed_len: 5,
        compression_frame_digest: [3; 32],
        slices: Vec::new(),
    };
    encode_compression_frame_segment_payload(&manifest, 0, b"abcde").unwrap()
}

fn shared_frame_chunk() -> FileChunk {
    FileChunk {
        stored_path: LockboxPath::new("/tree/file.bin").unwrap(),
        file_offset: 0,
        len: 128,
        compression_frame_offset: 0,
        compression_frame_len: 512,
        compressed_len: 42,
        compression: 1,
        compression_frame_id: 7,
        compression_frame_digest: [9; 32],
        segments: vec![CompressionFrameSegment {
            page_offset: 4096,
            page_len: 4096,
            object_id: 10,
            segment_offset: 0,
            segment_len: 42,
        }],
    }
}

fn read_fixture(relative: &str) -> Vec<u8> {
    let path = fixture_path(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    hex_to_bytes(&text).unwrap_or_else(|err| panic!("decode fixture {}: {err}", path.display()))
}

fn write_fixture(relative: &str, bytes: &[u8]) {
    let path = fixture_path(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("create fixture dir {}: {err}", parent.display()));
    }
    std::fs::write(&path, bytes_to_hex(bytes))
        .unwrap_or_else(|err| panic!("write fixture {}: {err}", path.display()));
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

fn hex_to_bytes(text: &str) -> Result<Vec<u8>, String> {
    let mut nibbles = Vec::new();
    for ch in text.chars().filter(|ch| !ch.is_ascii_whitespace()) {
        let Some(value) = ch.to_digit(16) else {
            return Err(format!("invalid hex character {ch:?}"));
        };
        nibbles.push(value as u8);
    }
    if nibbles.len() % 2 != 0 {
        return Err("odd number of hex digits".to_string());
    }
    Ok(nibbles
        .chunks_exact(2)
        .map(|chunk| (chunk[0] << 4) | chunk[1])
        .collect())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2 + bytes.len() / 32);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && index % 32 == 0 {
            out.push('\n');
        }
        out.push_str(&format!("{byte:02x}"));
    }
    out.push('\n');
    out
}
