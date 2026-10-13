cwd=/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
timeout --kill-after=30s 300s cargo +1.88.0 test --locked --offline -p revault_cli --test cli_flow vault_beget_supports_custom_output_contact_and_no_contact -- --exact --nocapture --test-threads=1
