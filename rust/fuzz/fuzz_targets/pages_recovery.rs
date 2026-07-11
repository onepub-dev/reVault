#![no_main]

use libfuzzer_sys::fuzz_target;
use revault_lockbox_api::{OwnerSigningKeyPair, RecoveryScanner};

fuzz_target!(|data: &[u8]| {
    let _ = RecoveryScanner::scan_bytes(data.to_vec(), b"fuzz key");
    let signing_key = OwnerSigningKeyPair::generate().unwrap();
    let _ = RecoveryScanner::salvage_bytes(data.to_vec(), b"fuzz key", &signing_key);
});
