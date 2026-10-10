//! Read-only comparison of retained file-only candidate fixtures. No creation or
//! mutation is allowed here; all source inventories and archive hashes are checked.
use super::*;

fn inventory(root: &Path, case: &Case) -> Value {
    json!((0..case.files)
        .map(|index| {
            let name = Case::name(index);
            let path = root.join("source").join(&name);
            assert_eq!(fs::metadata(&path).unwrap().len(), case.bytes);
            json!({"path":name,"bytes":case.bytes,"sha256":hash_file(&path)})
        })
        .collect::<Vec<_>>())
}
fn comparable(case: &Case, other: &Case, zip: bool) {
    let mut expected = case.json();
    if zip {
        expected["mode"] = json!(&other.mode);
    }
    assert_eq!(expected, other.json(), "fixture settings differ");
}
pub(super) fn compare(args: &[String]) {
    assert_eq!(args.len(), 10, "compare-existing NEW_EVIDENCE ZIP_ROOT PRIMARY_EXE PRIMARY_ROOT OTHER_EXE OTHER_ROOT stream|range PASSES SAMPLES");
    let output = Path::new(&args[1]);
    let zip_root = Path::new(&args[2]);
    let primary = Path::new(&args[3]);
    let primary_root = Path::new(&args[4]);
    let other = Path::new(&args[5]);
    let other_root = Path::new(&args[6]);
    assert!(matches!(args[7].as_str(), "stream" | "range"));
    let passes: usize = args[8].parse().unwrap();
    let samples: usize = args[9].parse().unwrap();
    assert!((1..=1000).contains(&passes) && (1..=1000).contains(&samples));
    let case = Case::read(primary_root);
    comparable(&case, &Case::read(other_root), false);
    comparable(&case, &Case::read(zip_root), true);
    let source = inventory(primary_root, &case);
    assert_eq!(source, inventory(other_root, &case));
    assert_eq!(source, inventory(zip_root, &case));
    // Accept exactly one complete shared-image fixture pair. Ambiguous or
    // incomplete sets must not silently select a different comparison input.
    let layouts: Vec<_> = ["dense", "tree"]
        .into_iter()
        .filter(|layout| {
            other_root.join(format!("{layout}.lbox")).exists()
                || other_root.join(format!("{layout}.public")).exists()
        })
        .collect();
    assert_eq!(layouts.len(), 1, "select exactly one shared-image fixture");
    let shared_layout = layouts[0];
    let artifacts = [
        zip_root.join("archive.zip"),
        primary_root.join("candidate.lbox"),
        primary_root.join("candidate.public"),
        other_root.join(format!("{shared_layout}.lbox")),
        other_root.join(format!("{shared_layout}.public")),
    ];
    let hashes: Vec<_> = artifacts.iter().map(|path| hash_file(path)).collect();
    fs::create_dir(output).expect("use a new evidence directory");
    fs::write(
        output.join("inventory.json"),
        serde_json::to_vec_pretty(&source).unwrap(),
    )
    .unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut evidence = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("samples.jsonl"))
        .unwrap();
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let toolchain = Command::new("rustc").arg("--version").output().unwrap();
    write_record(
        &mut evidence,
        &json!({"kind":"environment","case":case.json(),
        "layout":"retained-candidate-versus-shared-control","executable_sha256":hash_file(&executable),
        "primary_executable_sha256":hash_file(primary),"other_executable_sha256":hash_file(other),
        "source_revision_at_run":String::from_utf8_lossy(&revision.stdout).trim(),
        "runner_source_sha256":hex(&Sha256::digest(include_bytes!("runner.rs"))),
        "existing_comparison_source_sha256":hex(&Sha256::digest(include_bytes!("existing.rs"))),
        "cargo_lock_sha256":hash_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.lock")),
        "rustc":String::from_utf8_lossy(&toolchain.stdout).trim(),
        "cpuinfo":fs::read_to_string("/proc/cpuinfo").unwrap(),
        "meminfo":fs::read_to_string("/proc/meminfo").unwrap(),
        "kernel":fs::read_to_string("/proc/sys/kernel/osrelease").unwrap().trim(),
        "process_status":fs::read_to_string("/proc/self/status").unwrap(),
        "loadavg":fs::read_to_string("/proc/loadavg").unwrap().trim(),
        "cache":"warm OS / fresh process and handle per sample","warmup_pairs":3,
        "samples":samples,"passes":passes,"access":args[7],"workers":1,
        "rss_scope":"worker lifetime before separate content verification; baseline process high water reported; Linux KiB",
        "zip_protection":"unsigned and unencrypted; ranges omit full-entry CRC",
        "inventory_sha256":hash_file(&output.join("inventory.json")),
        "artifacts":artifacts.iter().zip(&hashes).map(|(path,hash)|json!({"path":path,"sha256":hash})).collect::<Vec<_>>() }),
    );
    for pair in 0..samples + 3 {
        let mut order = [0, 1, 2];
        if pair % 2 == 1 {
            order.reverse();
        }
        for which in order {
            let (worker, root, backend) = match which {
                0 => (executable.as_path(), zip_root, "zip"),
                1 => (primary, primary_root, "lockbox"),
                _ => (other, other_root, "lockbox"),
            };
            let mut record = child(
                worker,
                &[
                    "sample".into(),
                    root.display().to_string(),
                    backend.into(),
                    args[7].clone(),
                    args[8].clone(),
                ],
            );
            assert_eq!(record["verified"], true);
            record["pair"] = json!(pair);
            record["warmup"] = json!(pair < 3);
            record["other"] = json!(which == 2);
            write_record(&mut evidence, &record);
        }
    }
    for (path, expected) in artifacts.iter().zip(&hashes) {
        assert_eq!(&hash_file(path), expected);
    }
    for root in [zip_root, primary_root, other_root] {
        assert_eq!(source, inventory(root, &case));
    }
    write_record(
        &mut evidence,
        &json!({"kind":"retained_inputs_verified","archives_and_sources_unchanged":true}),
    );
    evidence.sync_all().unwrap();
    println!(
        "{}",
        json!({"evidence":output.join("samples.jsonl"),"completed_pairs":samples,"warmup_pairs":3,"retained_inputs_unchanged":true})
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_zip_may_have_reduced_protection() {
        use super::*;
        fn case(mode: &str) -> Case {
            Case {
                files: 512,
                bytes: 4096,
                corpus: "mixed".into(),
                compressed: true,
                mode: mode.into(),
                unpadded: false,
            }
        }
        comparable(&case("encrypted-signed"), &case("plain"), true);
        comparable(&case("encrypted-signed"), &case("encrypted-signed"), false);
        assert!(std::panic::catch_unwind(|| comparable(
            &case("encrypted-signed"),
            &case("plain"),
            false
        ))
        .is_err());
        let mut changed = case("plain");
        changed.bytes += 1;
        assert!(std::panic::catch_unwind(|| comparable(&case("plain"), &changed, true)).is_err());
    }
}
