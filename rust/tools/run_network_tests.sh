#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo test -p revault_key_server --test e2e_failover -- --ignored --nocapture
cargo test -p revault_key_server --test protocol_store -- --ignored --nocapture
cargo test -p revault_cli --test publish_integration -- --ignored --nocapture
