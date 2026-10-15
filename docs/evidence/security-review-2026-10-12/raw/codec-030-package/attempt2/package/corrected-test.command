/usr/bin/timeout --kill-after=10s 300s cargo +1.88.0 test --manifest-path <attempt2 unpacked Cargo.toml> --locked --offline --lib tests::test_specific_file -- --exact --nocapture --test-threads=1
