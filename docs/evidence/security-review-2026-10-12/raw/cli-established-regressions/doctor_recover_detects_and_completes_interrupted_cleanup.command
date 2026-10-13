cwd=/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
timeout --kill-after=30s 300s cargo +1.88.0 test --locked --offline -p revault_cli --test cli_flow doctor_recover_detects_and_completes_interrupted_cleanup -- --exact --nocapture --test-threads=1
