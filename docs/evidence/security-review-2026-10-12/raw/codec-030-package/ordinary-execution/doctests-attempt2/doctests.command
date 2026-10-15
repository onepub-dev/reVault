cwd=/tmp/revault-zstd-030-security-package/unpacked/zstd-complete-0.3.0
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
PATH=/home/bsutton/.cargo/bin:/usr/bin:/bin
CARGO_HOME=/home/bsutton/.cargo
RUSTUP_HOME=/home/bsutton/.rustup
CARGO_TARGET_DIR=/tmp/revault-zstd-030-security-package/target-verify
timeout --kill-after=30s 300s env -i HOME=/tmp/rv-zstd-030-doctests-home PATH=/home/bsutton/.cargo/bin:/usr/bin:/bin CARGO_HOME=/home/bsutton/.cargo RUSTUP_HOME=/home/bsutton/.rustup CARGO_TARGET_DIR=/tmp/revault-zstd-030-security-package/target-verify TMPDIR=/home/bsutton/.cache/rv4-security-20261011 LANG=C.UTF-8 cargo +1.88.0 test --locked --offline --doc -- --nocapture
