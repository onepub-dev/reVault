#!/usr/bin/env bash
set -euo pipefail

# Keep this list in sync with production crates. New Rust crates should be
# added here once they are intended to be part of the supported build.
cargo fmt -p revault_page_api -p revault_lockbox_api -p revault_cli -p revault_vault_api -p revault_publish_protocol -p revault_key_server --check
cargo clippy -p revault_page_api -p revault_lockbox_api -p revault_cli -p revault_vault_api -p revault_publish_protocol -p revault_key_server --all-targets -- -D warnings
cargo test -p revault_page_api
cargo test -p revault_lockbox_api
if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  # The exhaustive CLI flow suite exercises Unix-style process and terminal
  # behavior. Windows agent behavior has a dedicated workflow; retain the
  # portable CLI integration targets here without hanging the native matrix.
  cargo test -p revault_cli \
    --test contact_receive_alias \
    --test help_open_key \
    --test publish_integration
else
  cargo test -p revault_cli
fi
cargo test -p revault_vault_api
cargo test -p revault_publish_protocol
cargo test -p revault_key_server
