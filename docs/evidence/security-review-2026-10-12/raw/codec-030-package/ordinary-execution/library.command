binary=/tmp/revault-zstd-030-security-package/target-verify/debug/deps/zstd_complete-92d15c5cfa447775
filters=777
args=<selected names> --exact --test-threads=1 --nocapture
timeout --kill-after=30s 1800s env -i HOME=/tmp/rv-zstd-030-ordinary-home TMPDIR=/home/bsutton/.cache/rv4-security-20261011 PATH=/usr/bin:/bin LANG=C.UTF-8 /tmp/revault-zstd-030-security-package/target-verify/debug/deps/zstd_complete-92d15c5cfa447775 <777 selected exact filters> --exact --test-threads=1 --nocapture
