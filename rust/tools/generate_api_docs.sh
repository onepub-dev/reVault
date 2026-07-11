#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo doc -p revault_lockbox_api --no-deps

cat <<'MSG'
Generated revault_lockbox_api API docs:
  target/doc/revault_lockbox_api/index.html
MSG
