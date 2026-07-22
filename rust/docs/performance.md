# Performance tracking

The key-server performance baseline is captured with the ignored heavy failover
test and the workspace task:

```bash
cargo xtask measure-key-server-performance
```

The task writes logs under `target/perf/` and preserves the benchmark output
line emitted by `heavy_failover_recovery_under_load`.

Current post-Axum reference from local validation:

- Workload: 50,000 publish/receive flows
- Workers: 128
- Publish creation: about 11.2k creates/sec
- Late standby catch-up: about 63-66 seconds
- Receive phase: about 1.4k-1.5k receives/sec

Interpretation:

- The Axum server path passes the same heavy failover workload.
- The original 60 second catch-up timeout was too tight for this stack; the ignored test now allows 180 seconds.
- Treat the numbers as local reference points, not universal pass/fail thresholds. Hardware, filesystem, and scheduler behavior materially affect these tests.

When changing server, replication, topology, protocol, or storage behavior:

- Run `cargo xtask run-network-tests` for correctness.
- Run `cargo xtask measure-key-server-performance` for a captured performance log.
- Compare the final `heavy_failover flows=...` line against recent local or CI artifacts.
