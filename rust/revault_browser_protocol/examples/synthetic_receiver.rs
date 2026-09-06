//! Local test receiver only: no HTTP server, monitor or authentication implementation.
use base64ct::{Base64, Encoding};
use ed25519_dalek::{Signer, SigningKey};
use revault_browser_protocol::{
    receiver::{Application, Receiver},
    Message, UnlockEnvelope,
};
use revault_lockbox_api::LockboxPath;
use std::{
    io::{self, BufRead, Read},
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "Usage: synthetic_receiver <synthetic.lbox> <lockbox-id> <https-origin>".into(),
        );
    }
    // Deliberately public fixture key. NEVER use it for a real application.
    let signing = SigningKey::from_bytes(&[17; 32]);
    let app = Application {
        origin: args[2].clone(),
        application_id: "synthetic-receiver".into(),
        lockbox_id: args[1].clone(),
    };
    let mut receiver = Receiver::new()?;
    // Models one authenticated session for a local fixture only.
    let session = [23; 32];
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
    };
    let request = receiver.issue(&app, session, now()?, |b| Ok(signing.sign(b).to_bytes()))?;
    println!(
        "SYNTHETIC ONLY. Public test signing key bytes: {:?}",
        signing.verifying_key().to_bytes()
    );
    println!(
        "SignedUnlockRequest (base64): {}",
        Base64::encode_string(&request.encode_to_vec())
    );
    println!("Paste the encrypted UnlockEnvelope base64, then press Enter:");
    let mut input = String::new();
    io::stdin().lock().take(24 * 1024).read_line(&mut input)?;
    let envelope = UnlockEnvelope::decode(Base64::decode_vec(input.trim())?.as_slice())?;
    let lockbox = receiver
        .accept(&session, &envelope, now()?)?
        .open_lockbox(&args[0])?;
    let mut bytes = Vec::new();
    lockbox
        .open_file(&LockboxPath::new("/synthetic.bin")?)?
        .read_to_end(&mut bytes)?;
    if bytes != b"synthetic diagnostic credential\0synthetic beta credential\xff" {
        return Err("synthetic fixture bytes did not match".into());
    }
    println!("Synthetic bytes match. Exiting clears the handle and recipient key.");
    Ok(())
}
