#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo test -p lockbox_key_server --test e2e_failover -- --ignored --nocapture
cargo test -p lockbox_key_server --test protocol_store -- --ignored --nocapture
cargo test -p lockbox_cli --test publish_integration -- --ignored --nocapture
