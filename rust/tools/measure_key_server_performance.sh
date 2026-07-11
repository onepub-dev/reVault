#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p target/perf
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
log="target/perf/key-server-heavy-failover-${stamp}.log"

export LOCKBOX_SHARE_E2E_FLOWS="${LOCKBOX_SHARE_E2E_FLOWS:-50000}"
export LOCKBOX_SHARE_E2E_WORKERS="${LOCKBOX_SHARE_E2E_WORKERS:-128}"

{
    echo "timestamp_utc=${stamp}"
    echo "flows=${LOCKBOX_SHARE_E2E_FLOWS}"
    echo "workers=${LOCKBOX_SHARE_E2E_WORKERS}"
    rustc --version
    cargo --version
    cargo test -p lockbox_key_server --test e2e_failover heavy_failover_recovery_under_load -- --ignored --nocapture
} 2>&1 | tee "${log}"

echo "performance_log=${log}"
