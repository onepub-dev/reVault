cwd=/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/rust
TMPDIR=/home/bsutton/.cache/rv4-security-20261011
timeout --kill-after=30s 900s cargo +1.88.0 test --locked --offline -p revault_cli --test cli_flow vault_lockbox_list_reports_owner_size_and_path -- --exact --nocapture --test-threads=1
