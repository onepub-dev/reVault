#![no_main]
use libfuzzer_sys::fuzz_target;
use revault_browser_protocol::{transport, BrowserRequest, Message, MAX_PROTO_BYTES};
fuzz_target!(|data: &[u8]| {
    let _ = transport::read_frame(&mut std::io::Cursor::new(data));
    if let Ok(bytes) = transport::decode_json(data) { let _ = BrowserRequest::decode(bytes.as_slice()); }
    if data.len() <= MAX_PROTO_BYTES { let _ = BrowserRequest::decode(data); }
});
