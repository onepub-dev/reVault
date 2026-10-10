//! Current typed-tree adapter for the shared fresh-process read protocol.
//! Fixture construction is not a comparable public write benchmark.
use super::*;
use crate::file_format::candidate_files::dense_catalogue::Metadata;
use crate::file_format::candidate_files::tree_image::{self, TreeImage};

impl<S: Storage> ReadImage for TreeImage<S> {
    fn stored_name(index: usize) -> String {
        format!("/file-{index:06}.bin")
    }
    fn info(&self, path: &[u8]) -> Result<Option<FileInfo>> {
        self.image.info(path)
    }
    fn visit(
        &mut self,
        path: &[u8],
        offset: u64,
        len: u64,
        visitor: impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        self.image.read_range(path, offset, len, visitor)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    root: &Path,
    count: usize,
    bytes: u64,
    unit: usize,
    mode: FormatMode,
    key: Option<&[u8]>,
    phase: &str,
) {
    let target = root.join("tree.lbox");
    let public_path = root.join("tree.public");
    if phase == "tree-create" {
        let staging_path = root.join("tree-staging.lbox");
        assert!(!target.exists() && !public_path.exists() && !staging_path.exists());
        let owner = OwnerSigningKeyPair::generate().unwrap();
        let public = owner.public_key();
        let authority = if mode.signed() {
            Authority::Owner(&public)
        } else if mode.plaintext() {
            Authority::Checksum
        } else {
            Authority::Symmetric(KEY)
        };
        let signer = mode.signed().then_some(&owner);
        let inputs = (0..count).map(|index| Input {
            path: TreeImage::<StorageBackend>::stored_name(index).into_bytes(),
            reader: File::open(root.join("source").join(name(index))).unwrap(),
        });
        let staging = Files::create(
            StorageBackend::create_file(&staging_path, &[]).unwrap(),
            archive(),
            mode,
            &authority,
            signer,
            key,
            unit,
            inputs,
        )
        .unwrap();
        let mut source = Files::open(staging, archive(), mode, &authority, key).unwrap();
        let metadata: Vec<_> = (0..count)
            .map(|index| Metadata {
                entry: crate::LockboxEntry {
                    path: crate::LockboxPath::new(TreeImage::<StorageBackend>::stored_name(index))
                        .unwrap(),
                    kind: crate::LockboxEntryKind::File,
                    len: bytes,
                    permissions: 0o600,
                },
                target: None,
            })
            .collect();
        let tree = tree_image::from_candidate(
            &mut source,
            StorageBackend::create_file(&target, &[]).unwrap(),
            &authority,
            signer,
            key,
            &metadata,
        )
        .unwrap();
        drop(tree);
        let mut opened = TreeImage::open(
            StorageBackend::file(&target).unwrap(),
            archive(),
            mode,
            &authority,
            key,
        )
        .unwrap();
        verify(&mut opened, root, count, bytes);
        assert_eq!(opened.image.filesystem_metadata().unwrap(), metadata);
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&public_path)
            .unwrap()
            .write_all(&public.to_bytes())
            .unwrap();
        drop(opened);
        drop(source);
        std::fs::remove_file(staging_path).unwrap();
        println!(
            "CANDIDATE_SAMPLE {}",
            json!({"kind":"typed_tree_fixture","layout":"shared-control-typed-tree","archive_bytes":std::fs::metadata(&target).unwrap().len(),"archive_sha256":digest_file(&target),"verified":true,"scope":"fresh direct typed export; construction is not a write-performance sample"})
        );
        return;
    }
    assert_eq!(phase, "tree-sample");
    let public = OwnerSigningPublicKey::from_bytes(&std::fs::read(&public_path).unwrap()).unwrap();
    let authority = if mode.signed() {
        Authority::Owner(&public)
    } else if mode.plaintext() {
        Authority::Checksum
    } else {
        Authority::Symmetric(KEY)
    };
    let open = || {
        TreeImage::open(
            StorageBackend::file(&target).unwrap(),
            archive(),
            mode,
            &authority,
            key,
        )
        .unwrap()
    };
    let mut result = sample(root, count, bytes, open);
    result["layout"] = json!("shared-control-typed-tree");
    result["candidate_scope"] = json!(
        "file-read component with explicit keys; no full-format, write or CLI RSS qualification"
    );
    result["candidate_test_executable_sha256"] =
        json!(digest_file(&std::env::current_exe().unwrap()));
    result["extent_unit"] = json!(unit);
    println!("CANDIDATE_SAMPLE {result}");
}
