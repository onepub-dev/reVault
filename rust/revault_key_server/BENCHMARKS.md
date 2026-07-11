# reVault Key Server Benchmarks

Benchmarks were run on the local development host with the release binary.
HTTP benchmarks use one binary `POST /v1/publish` per TCP connection because the
expected client model is a single CLI request rather than a long-lived
keep-alive session.

## Store Benchmark

Command:

```bash
/usr/bin/time -v target/release/revault_key_server \
  bench-store \
  --state-dir /tmp/revault-publish-store-bench-200k-compact-001 \
  --requests 200000 \
  --payload-bytes 512
```

Result:

```text
store_create_rps=216805
store_receive_rps=302815
live=200000
max_rss_kb=52436
```

This keeps 200k live published payloads with 512-byte payloads below the 100 MB memory
target. Payloads are persisted in append-only segment files; the live in-memory
index stores hashes, offsets, lengths, expiry, and receive state.

## HTTP Benchmark

Command:

```bash
/usr/bin/time -v target/release/revault_key_server \
  bench-http \
  --state-dir /tmp/revault-publish-http-bench-compact-001 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Result:

```text
http_single_request_publish_rps=54179
requests=50000
concurrency=128
live=50000
max_rss_kb=18292
```

The HTTP path handles tens of thousands of single-request published payloads per second
while staying well under the 100 MB target.

## HTTP Receive Benchmark

Command:

```bash
/usr/bin/time -v target/release/revault_key_server \
  bench-http-receive \
  --state-dir /tmp/revault-publish-http-receive-bench-001 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Result:

```text
http_single_request_receive_rps=58846
requests=50000
concurrency=128
live=0
max_rss_kb=20616
```

This preloads 50k single-use published payloads, then receives them over HTTP using one TCP
connection per request. `live=0` verifies that successful single-use receives
consume and tombstone the pending published payloads.

## HTTP End-to-End Flow Benchmark

Command:

```bash
/usr/bin/time -v target/release/revault_key_server \
  bench-http-flow \
  --state-dir /tmp/revault-publish-http-flow-bench-001 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Result:

```text
http_single_request_flow_rps=25782
http_single_request_total_rps=51564
flows=50000
concurrency=128
preloaded=0
live=0
max_rss_kb=13284
```

Each flow performs two separate TCP connections: one `SHARE` request followed
by one `RECEIVE` request. This is the closest benchmark to the expected CLI
usage pattern. `live=0` confirms single-use published payloads are consumed after receipt.

## HTTP End-to-End Flow With 1M Preloaded Published Payloads

Command:

```bash
/usr/bin/time -v target/release/revault_key_server \
  bench-http-flow \
  --state-dir target/revault-publish-http-flow-preload-1m-bucket-002 \
  --preload-published-payloads 1000000 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Result:

```text
http_single_request_flow_rps=25254
http_single_request_total_rps=50508
flows=50000
concurrency=128
preloaded=1000000
live=1000000
max_rss_kb=82376
```

This preloads one million pending published payloads before timing the end-to-end flow.
The result stays below the 100 MB memory target by using compact disk bucket
index files plus a bounded in-memory recent-published-payload cache. This benchmark uses
12-digit publish codes because 8-digit codes have too much collision pressure at
one million live published payloads.

Benchmarks disable the per-IP rate limiter so they measure server throughput.
Production defaults enable a per-IP token bucket.

## Compaction

Single-use published payloads are tombstoned immediately after successful receive. Background
compaction rewrites shard segment files when they contain enough dead bytes,
and explicit `compact()` tests prove tombstoned single-use backlogs can shrink
back to zero segment bytes when no live published payloads remain.

## Indexing

Payloads are stored in append-only segment files. Live lookup metadata is
stored in compact fixed-size disk bucket records keyed by the published payload-code hash.
The process keeps only a bounded recent-published-payload cache in memory. This avoids
retaining every pending published payload in RAM while preserving single-key lookup without
scanning the full store.

## Publish Code Space

The production default is one server routing digit plus a 12 digit random body.
Six random body digits are still supported for smaller deployments, but they
are not appropriate for sustained high-rate pending-published-payload populations because
the live code space is capped at one million per server id. The server clamps
configurable random body length to 6..12 digits.

## Abuse Controls

Production defaults are intentionally bounded:

```text
payload_cap=8 KiB
verification_ttl=30 minutes
default_receive_ttl=2 hours
max_receive_ttl=2 hours
max_receives_per_publish=8
rate_limit_per_ip=120 requests/minute
rate_limit_burst=40
verification_email_rate_limit=5/hour/email/cluster
verification_email_ip_rate_limit=30/hour/source-ip
```

The server validates typed, versioned reVault publish payloads before storing
them, so arbitrary blobs are rejected. The remaining controls reduce usefulness
as a store-and-forward relay by limiting payload size, lifetime, fan-out, and
request rate for syntactically valid but still untrusted publish messages.

The CLI keeps topology-based publish selection sticky for 24 hours and treats
`RateLimited` as terminal for the current operation. A rate-limited client
therefore waits instead of retrying the same operation against another cluster
member. Key servers also replicate 24 hour anonymous-client blocks after a
rate-limit violation, so the limit is enforced at cluster scope rather than by
one server process only.

Verification email sending is also cluster-scoped. The topology assigns each
normalized publisher email address to one primary server plus one backup
server, and only those two servers queue verification email for the address.
Requests sent directly to other servers are rejected as rate limited, so a
publisher or attacker cannot multiply the per-email SMTP limit by trying every
server in the topology. Clients fail over verification-email publishes only to
the one deterministic backup server, and only when the primary is unavailable.

## CPU Profile

Command:

```bash
perf stat \
  -e cycles,instructions,context-switches,cpu-migrations,page-faults \
  target/release/revault_key_server \
  bench-http \
  --state-dir /tmp/revault-publish-http-perf-002 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Result:

```text
http_single_request_publish_rps=70819
cycles=39657318266
instructions=20756037342
context_switches=87850
cpu_migrations=23410
page_faults=4964
elapsed_seconds=0.759955423
user_seconds=0.555298
sys_seconds=9.250256
```

Receive-path counter command:

```bash
perf stat \
  -e cycles,instructions,context-switches,cpu-migrations,page-faults \
  target/release/revault_key_server \
  bench-http-receive \
  --state-dir /tmp/revault-publish-http-receive-perf-001 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

Receive-path result:

```text
http_single_request_receive_rps=70551
cycles=40298295480
instructions=21396337648
context_switches=86477
cpu_migrations=22371
page_faults=5472
elapsed_seconds=0.978261193
user_seconds=0.622768
sys_seconds=9.316255
```

End-to-end flow counter command:

```bash
perf stat \
  -e cycles,instructions,context-switches,cpu-migrations,page-faults \
  target/release/revault_key_server \
  bench-http-flow \
  --state-dir /tmp/revault-publish-http-flow-perf-001 \
  --requests 50000 \
  --payload-bytes 512 \
  --concurrency 128
```

End-to-end flow result:

```text
http_single_request_flow_rps=30149
http_single_request_total_rps=60298
cycles=95370493307
instructions=49265979596
context_switches=169530
cpu_migrations=39103
page_faults=3092
elapsed_seconds=1.711987716
user_seconds=1.126379
sys_seconds=22.379968
```

Sampled `perf record` data showed most samples in kernel space. Kernel symbol
resolution was restricted on the benchmark host, but the high system time and
context-switch count are consistent with the intentional benchmark shape:
single HTTP request per TCP connection.

## Persistence Tests

The test suite covers:

```text
live publish replay after store reopen
receive count replay after store reopen
exhausted published payload tombstone replay
single-use publish removal on successful receive
20k-record persistent store replay
compaction removes tombstoned single-use backlog
compaction preserves live records
```

Run:

```bash
cargo test -p revault_key_server
```

Current result:

```text
12 passed
```
