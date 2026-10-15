cwd=/tmp/revault-zstd-030-security-package/unpacked/zstd-complete-0.3.0
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
timeout --kill-after=30s 300s env -i HOME=/tmp/rv-zstd-030-doctests-home TMPDIR=/home/bsutton/.cache/rv4-security-20261011 PATH=/usr/bin:/bin LANG=C.UTF-8 cargo +1.88.0 test --locked --offline --doc -- --nocapture
