cwd=/tmp/revault-codec-fixed-storage-compile28/crate
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
CARGO_TARGET_DIR=/tmp/revault-codec-fixed-storage-compile27/target
cargo +1.88.0 check --locked --offline -p zstd-complete@0.2.0 --lib --features dict_builder,c-port-validation,force-scalar,multithreading,fuzz_exports
