//! Internal prepared-writer fixtures; not public CLI archive operations.
use super::*;
use crate::file_format::authenticated_index::Entry;
#[cfg(target_os = "linux")]
use crate::file_format::page::secure_storage::PreparedSecurePages;
use crate::file_format::secure_segments;

fn replacement<'a>(
    original: &StorageBackend,
    mode: FormatMode,
    authority: &Authority<'_>,
    source: &'a SecretString,
    failure: &'static str,
    at: usize,
) -> (Vec<Entry>, shared::tree::PreparedSecurePayloadPlan<'a>) {
    let opened = TreeImage::open(original.clone(), archive(), mode, authority, key(mode)).unwrap();
    let base = shared::commitment(&opened.tree.anchor).unwrap();
    let old = &opened.image.catalogue.variables[0];
    let prepared = secure_segments::prepare_source(
        secure_segments::Source::Secret(source),
        archive(),
        mode,
        opened.image.value_key.as_ref().unwrap(),
        old.layout.id,
        old.layout.revision + 1,
        crate::VariableSensitivity::Secret,
    )
    .unwrap();
    let mut catalogue = opened.image.catalogue;
    let retired = catalogue.variables[0].layout.extents.clone();
    catalogue.variables[0].layout = prepared.layout;
    let records = catalogue.tree_records().unwrap();
    let mut payload = prepared.payload;
    (
        records,
        shared::tree::PreparedSecurePayloadPlan {
            base,
            retired,
            pages: prepared.pages,
            payload: Box::new(move |index| {
                if failure == "callback" && index == at {
                    return Err(Error::Io("injected payload callback failure".into()));
                }
                let mut raw = payload(index)?;
                if failure == "source" && index == at {
                    raw.with_mut_bytes(|bytes| {
                        let last = bytes.len() - 1;
                        bytes[last] ^= 1;
                    })?;
                }
                Ok(raw)
            }),
            rebind: Box::new(move |extents| {
                if failure == "rebind" {
                    return Err(Error::InvalidInput(
                        "injected preflight rebind failure".into(),
                    ));
                }
                catalogue.variables[0].layout.rebind(extents)?;
                let mut records = catalogue.tree_records()?;
                if failure == "shape" {
                    records.pop();
                }
                Ok(records)
            }),
        },
    )
}

#[test]
fn prepared_staging_preflight_and_second_pass_failures_preserve_selected_values_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let source = SecretString::try_from_slice(&vec![b'x'; 65543]).unwrap();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let mut original = super::super::mutation::seed(mode, &authority, &owner);
        set_secret_variable(
            &mut original,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &VariableName::new("target").unwrap(),
            &SecretString::try_from_slice(b"old").unwrap(),
        )
        .unwrap();
        let bytes = original.read_all().unwrap();
        let old_extents = extents(&original, mode, &authority);
        let ranges: Vec<_> = old_extents
            .iter()
            .map(|e| (e.start, e.start + e.len))
            .collect();
        for failure in ["rebind", "shape"] {
            let (records, plan) = replacement(&original, mode, &authority, &source, failure, 0);
            let mut storage = Guarded::new(
                CrashStore::new(bytes.clone(), None, 0, false),
                ranges.clone(),
            );
            assert!(shared::tree::rewrite_prepared_secure_payload_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key(mode),
                records,
                plan
            )
            .is_err());
            assert_eq!(storage.inner.operations(), 0);
            assert_eq!(storage.inner.durable(), bytes);
        }
        for failure in ["callback", "source"] {
            for at in 0..2 {
                let (records, plan) =
                    replacement(&original, mode, &authority, &source, failure, at);
                let mut storage =
                    Guarded::new(StorageBackend::memory(bytes.clone()), ranges.clone());
                assert!(shared::tree::rewrite_prepared_secure_payload_records(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    signer,
                    key(mode),
                    records,
                    plan
                )
                .is_err());
                tree_image::recover(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
                tree_image::recover(&mut storage, archive(), mode, &authority, key(mode)).unwrap();
                let mut opened =
                    TreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
                        .unwrap();
                assert_eq!(
                    opened.image.catalogue.variables[0].layout.extents,
                    old_extents
                );
                assert_eq!(
                    opened
                        .with_secret_variable(&VariableName::new("target").unwrap(), |v| v
                            .with_str(|s| assert_eq!(s, "old"))
                            .unwrap())
                        .unwrap(),
                    Some(())
                );
                let mut neighbor = Vec::new();
                opened
                    .image
                    .read_range(b"/docs/neighbor", 0, 8, |bytes| {
                        neighbor.extend_from_slice(bytes);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(neighbor, b"neighbor");
                let recovered = storage.inner.read_all().unwrap();
                for (start, end) in &ranges {
                    assert_eq!(
                        &recovered[*start as usize..*end as usize],
                        &bytes[*start as usize..*end as usize]
                    );
                }
                for (start, end) in storage
                    .spans
                    .borrow()
                    .iter()
                    .filter(|span| !ranges.contains(span))
                {
                    if *start < recovered.len() as u64 {
                        assert!(
                            recovered[*start as usize..(*end as usize).min(recovered.len())]
                                .iter()
                                .all(|b| *b == 0)
                        );
                    }
                }
            }
        }
        let invalid = SecretString::try_from_slice(&[0xff]).unwrap();
        assert!(secure_segments::prepare_source(
            secure_segments::Source::Secret(&invalid),
            archive(),
            mode,
            &[17; 32],
            [81; 16],
            1,
            crate::VariableSensitivity::Secret
        )
        .is_err());
        assert_eq!(original.read_all().unwrap(), bytes);
    }
}

#[cfg(target_os = "linux")]
fn memory() -> serde_json::Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    serde_json::json!({"vm_lck_kib":field("VmLck:"),"vm_rss_kib":field("VmRSS:"),"vm_hwm_kib":field("VmHWM:")})
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "fresh-process guarded aggregate staging probe"]
fn prepared_staging_aggregate_guarded_source_probe() {
    let bits: usize = std::env::var("REVAULT_PREPARED_MODE")
        .unwrap()
        .parse()
        .unwrap();
    let count: usize = std::env::var("REVAULT_PREPARED_COUNT")
        .unwrap()
        .parse()
        .unwrap();
    assert!(bits < 16 && [1, 12].contains(&count));
    let path = std::path::PathBuf::from(std::env::var_os("REVAULT_PREPARED_IMAGE").unwrap());
    assert!(!path.exists());
    let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = mode.signed().then_some(&owner);
    {
        let seed = super::super::mutation::seed(mode, &authority, &owner);
        std::fs::write(&path, seed.read_all().unwrap()).unwrap();
    }
    let before_source = memory();
    let source = SecretString::try_from_slice(&vec![b'x'; 1024 * 1024]).unwrap();
    let after_source = memory();
    let mut storage = Guarded::new(StorageBackend::file_for_write(&path).unwrap(), Vec::new());
    let opened = TreeImage::open(
        crate::file_format::allocation_map::compaction::View(&storage),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    let base = shared::commitment(&opened.tree.anchor).unwrap();
    let content_key =
        crate::file_format::candidate_files::dense_image::value_key(mode, key(mode)).unwrap();
    let mut catalogue = opened.image.catalogue;
    let mut pages = PreparedSecurePages::new(count * 16).unwrap();
    let mut producers = Vec::new();
    for index in 0..count {
        let mut id = [82; 16];
        id[0] = index as u8 + 1;
        let prepared = secure_segments::prepare_source(
            secure_segments::Source::Secret(&source),
            archive(),
            mode,
            &content_key,
            id,
            1,
            crate::VariableSensitivity::Secret,
        )
        .unwrap();
        pages.append(prepared.pages).unwrap();
        producers.push(prepared.payload);
        catalogue.variables.push(
            crate::file_format::candidate_files::dense_catalogue::Variable {
                name: VariableName::new(format!("value_{index:02}")).unwrap(),
                layout: prepared.layout,
            },
        );
    }
    let stored_payload_bytes: u64 = (0..pages.len())
        .map(|i| pages.size(i).unwrap() as u64)
        .sum();
    if count == 12 {
        assert!(stored_payload_bytes > 8 * 1024 * 1024);
    }
    let after_prepare = memory();
    let records = catalogue.tree_records().unwrap();
    let plan = shared::tree::PreparedSecurePayloadPlan {
        base,
        retired: Vec::new(),
        pages,
        payload: Box::new(move |index| producers[index / 16](index % 16)),
        rebind: Box::new(move |extents| {
            if extents.len() != count * 16 {
                return Err(Error::CorruptRecord);
            }
            for (index, variable) in catalogue.variables.iter_mut().enumerate() {
                variable
                    .layout
                    .rebind(&extents[index * 16..(index + 1) * 16])?;
            }
            catalogue.tree_records()
        }),
    };
    assert!(shared::tree::rewrite_prepared_secure_payload_records(
        &mut storage,
        archive(),
        mode,
        &authority,
        signer,
        key(mode),
        records,
        plan
    )
    .unwrap());
    let after_write = memory();
    let ranges = storage.spans.borrow().clone();
    drop(storage);
    drop(source);
    let opened = TreeImage::open(
        Guarded::new(StorageBackend::file(&path).unwrap(), ranges),
        archive(),
        mode,
        &authority,
        key(mode),
    )
    .unwrap();
    assert_eq!(opened.image.catalogue.variables.len(), count);
    for index in 0..count {
        assert_eq!(
            opened
                .with_secret_variable(
                    &VariableName::new(format!("value_{index:02}")).unwrap(),
                    |v| v
                        .with_str(|s| {
                            assert_eq!(s.len(), 1024 * 1024);
                            assert!(s.bytes().all(|b| b == b'x'));
                        })
                        .unwrap()
                )
                .unwrap(),
            Some(())
        );
    }
    let after_read = memory();
    let archive_bytes = opened.image.storage.len().unwrap();
    for metric in [
        &before_source,
        &after_source,
        &after_prepare,
        &after_write,
        &after_read,
    ] {
        assert!(metric["vm_lck_kib"].as_u64().unwrap() <= 8192);
    }
    println!(
        "PREPARED_STAGING {}",
        serde_json::json!({"mode":bits,"values":count,"source_bytes":1024*1024,"logical_bytes":count*1024*1024,
        "stored_payload_bytes":stored_payload_bytes,"archive_bytes":archive_bytes,"before_source":before_source,"after_source":after_source,
        "after_prepare":after_prepare,"after_write":after_write,"after_read":after_read})
    );
}
