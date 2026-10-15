use std::process::Command;
fn main() {
    let binaries: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(binaries.len(), 2);
    let cases = [
        (100, 0, 0, 0),
        (100, 1, 0, 0),
        (100, 0, 1, 0),
        (100, 1, 1, 0),
        (100, 0, 0, 1),
        (100, 1, 0, 1),
        (100, 0, 1, 1),
        (100, 1, 1, 1),
        (1000, 0, 0, 1),
        (1000, 1, 1, 1),
    ];
    for (i, (cycles, encrypted, signed, padded)) in cases.into_iter().enumerate() {
        for design in if i % 2 == 0 { [0, 1] } else { [1, 0] } {
            let label = if design == 0 { "adjacent" } else { "separated" };
            let output = Command::new(&binaries[design])
                .args([
                    "--exact",
                    "file_format::allocation_map::tests::allocation_aging_resource_probe",
                    "--ignored",
                    "--nocapture",
                ])
                .env("REVAULT_ALLOCATION_CYCLES", cycles.to_string())
                .env("REVAULT_ALLOCATION_ENCRYPTED", encrypted.to_string())
                .env("REVAULT_ALLOCATION_SIGNED", signed.to_string())
                .env("REVAULT_ALLOCATION_PADDED", padded.to_string())
                .output()
                .expect("start probe");
            assert!(
                output.status.success(),
                "{label}: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8(output.stdout).unwrap();
            let mut observations = 0;
            let mut totals = 0;
            for line in text.lines() {
                if let Some((_, value)) = line.split_once("ALLOCATION_CYCLE ") {
                    println!("{{\"design\":\"{label}\",\"cycles\":{cycles},\"kind\":\"cycle\",\"sample\":{value}}}");
                    observations += 1;
                }
                if let Some((_, value)) = line.split_once("ALLOCATION_TOTAL ") {
                    println!("{{\"design\":\"{label}\",\"cycles\":{cycles},\"kind\":\"total\",\"sample\":{value}}}");
                    totals += 1;
                }
            }
            assert_eq!(observations, cycles);
            assert_eq!(totals, 1);
            eprintln!("completed {label}: {cycles} cycles, encrypted={encrypted}, signed={signed}, padded={padded}");
        }
    }
}
