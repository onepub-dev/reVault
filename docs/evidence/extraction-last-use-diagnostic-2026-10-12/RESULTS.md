# Last-use retirement: system-overhead diagnostic

All eight fixed diagnostic extractions completed in 131.741 seconds. Exact output bytes, tree and modes, cold archive admission, draining, input identities and private-agent cleanup passed. No runtime changes, builds, added samples or retention decision were made. The rejected last-use implementation remains reverted.

The [frozen protocol](PROTOCOL.txt) defines four untraced counter observations and four separate child-only syscall summaries. These are single observations, not a performance comparison or confidence interval. Tracing perturbs execution; its elapsed time and GNUtime tracer/descendant accounting must not be substituted for untraced CLI measurements.

| Untraced 1 GiB extraction | User CPU s | System CPU s | Minor faults | Major faults | Voluntary / involuntary switches |
| --- | --- | --- | --- | --- | --- |
| Raw retained | 3.12 | 2.16 | 63,332 | 0 | 664 / 169 |
| Raw rejected last-use | 3.03 | 1.88 | 17,776 | 0 | 730 / 174 |
| Compressed retained | 3.29 | 1.18 | 85,076 | 0 | 290 / 41 |
| Compressed rejected last-use | 3.09 | 2.42 | 763,034 | 0 | 317 / 49 |

For compressed extraction, the separate traced runs recorded 50 → 520 `madvise` calls. Read/write call counts were unchanged: 1,066 `pread64`, 1,029 `write` and 28 `read` calls. Mapping counts were close (45 → 43 `mmap`, 23 → 21 `munmap`); `brk` was 50 in each. These aggregate summaries do not identify the advice values, allocation call sites or bytes transferred.

The combined observations support a discard/refault hypothesis for the compressed regression rather than an increase in read syscall count. They do not prove its precise cause or rule out other effects. In particular, trace summaries cannot attribute faults directly to a buffer. The next source review will distinguish decoded-frame buffers from page payloads before proposing bounded, fully wiped buffer reuse scoped to active extraction. No global allocator or environment tuning, security-policy relaxation, cohort filtering or format change follows from this screen.

Full counters, commands and summaries are retained in [raw](raw/), with [completion](raw/diagnostic-complete.json) and [samples](raw/diagnostic-samples.jsonl). GNUtime filesystem input/output counters report kernel operations, not bytes. Agent HWM snapshots are separate; this diagnostic computes no CLI-plus-helper bound. Persistent child checks span all agent tasks at boundaries, not a transient process census.

Both executables are preserved original artifacts: retained `4142dd043db7ccd608f5eaccdf3d9673b76b5879fb0a729e2527207c0924d4d6` and rejected `2f56a9986cae3c50b36a2c71e3308fe96e39a018b9e60a454d9682e02cbf783e`. Current restored source, their original build evidence, helpers and diagnostic scripts were frozen and checked after execution.
