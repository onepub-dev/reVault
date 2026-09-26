# Bounded file-page staging: CPU and memory evidence

This change addresses the default layout's whole-file dirty-page retention found
in the [architecture baseline](../archive-evaluation-2026-09-26/README.md). It does
not select an archive layout or satisfy the complete v4 evaluation contract.

## Implementation and safety

Newly allocated file pages become eligible for an early flush when the decoded
cache exceeds its configured limit. Before writing them, the archive publishes
the existing preparation journal into both header slots. That journal protects
appended allocations and reused free space during interruption and rollback.
Only those file pages are flushed: unrelated dirty metadata and pending erasure
stay on their existing publication paths. After a successful flush, file pages
can be evicted even when ordinary cache eviction stops at dirty metadata.

Small writes below the cache threshold retain their previous staging behaviour.
There is no wire-format change. This does not bound pending small-file inputs,
all metadata/index memory, or every parallel-worker path. It is a fix for the
measured single-worker large-file writer, not proof of the whole A5 budget.

## Validation

* Default library with external-source support: 358 passed, 5 ignored.
* Native-layout library with external-source support: 364 passed, 2 known
  recovery failures, 5 ignored. The failures remain truncated-tail recovery and
  signed-plaintext intact-neighbour recovery; this change does not resolve A2.
* Public mirror CLI: 8 passed, 1 ignored large-tree stress case, including the
  deterministic source-change refusal after append/reuse preparation.
* New cache-pressure regression crosses encryption/signing on/off and raw/Zstd:
  streamed replacement, read-before-commit, abort, commit, independent reopen,
  exact bytes and storage verification.
* Every storage-operation failure in the pressured streaming fixture recovers
  the old committed neighbour, removes the abandoned path, restores sealed length
  and leaves reusable ranges zero. A zero cache limit forces pressure writes.
* Cache regression proves unrelated metadata/queued erasure remain untouched,
  cache snapshots preserve flush eligibility, and restaging revokes eligibility.
  All 15 cache unit tests and both evaluation-runner tests pass.
* Targeted library/tests/benchmark Clippy passes with warnings denied.

The default/native full suites preceded a final adjustment that moves native
staging bookkeeping after input validation. The complete cache test module and
Clippy were rerun after that adjustment. No formatting preceded these checks.

## Measurement method

The baseline executable is the preserved default-layout binary used for the
original architecture measurements. The candidate starts at `c3f6d5b0` plus
[the exact measured source patch](measured-source.patch). The pre-formatting
[runner source](measured-runner.rs.txt) is retained to resolve its embedded hash.
Both use one worker, the default Interactive cache and default size padding.
Timing uses CPU 2 affinity, warm OS caches, fresh child processes, three warm-up
pairs, and alternating/rotating ZIP/baseline/candidate order. No owned build or
test overlaps measurement. Host load remains uncontrolled and is recorded.

The new `create` access mode repeatedly invokes the existing creation-worker
protocol, including in the frozen baseline executable. Each trial creates a
fresh archive, commits and synchronizes it, snapshots CPU/wall/RSS, then reopens
and verifies every byte before removing that trial. Source generation, signing
key generation, verification, hashing and cleanup are outside the timed region.
The raw observations retain archive hashes, lengths and validation status.

Creation summaries compare elapsed time, CPU and peak RSS. Read summaries retain
open/read/first-byte metrics. Peak RSS is a process lifetime high-water mark before
verification, not an exact allocation count or an empty-open-subtracted value.
Read and write samples share the fixed paired-log bootstrap protocol documented
in the baseline. A 30-pair result only supports its recorded case.

```sh
cargo bench -p revault_lockbox_api --bench archive_evaluation --no-run
# Retain the new executable separately; keep the old baseline executable unchanged.
taskset -c 2 /path/to/new run /new/1g-probe 1 1073741824 pattern raw plain 1 stream 1 default /path/to/baseline
taskset -c 2 /path/to/new run /new/256m-create 1 268435456 pattern raw plain 30 create 1 default /path/to/baseline
taskset -c 2 /path/to/new run /new/8m-read 1 8388608 pattern compressed plain 30 stream 1 default /path/to/baseline
/path/to/new summarize /new/256m-create/samples.jsonl
```

## Results

The JSONL labels `primary` as the bounded writer and `other` as the preserved
baseline. Tables below invert the stored baseline/candidate comparisons to show
new/old ratios, with the interval endpoints inverted in the opposite order.
Toolchain: [Rust 1.88.0 on x86_64 Linux](toolchain.txt). All three inventory hashes,
runner hashes, baseline executable hashes and deterministic summaries were checked.

### 1 GiB raw plaintext creation probe

One creation observation per backend; three warm-up and one measured read pair.
[Raw evidence](1g-pattern-raw-plain.jsonl), [read summary](1g-pattern-raw-plain-summary.jsonl),
[corpus inventory](1g-pattern-raw-plain-inventory.json).

| Writer | Peak RSS | Elapsed | CPU |
| --- | --- | --- | --- |
| Baseline default | 1,067.14 MiB | 5.934 s | 5.860 s |
| Bounded default | 190.43 MiB | 5.562 s | 5.489 s |
| ZIP stored | 3.63 MiB | 1.929 s | 1.845 s |

The writer no longer retains the whole GiB in its decoded-page cache. This probe
falls below the proposed 256 MiB budget even using total RSS, but does not establish
a universal bound for all workloads. Its timing sample count cannot establish a
performance gate.

### 256 MiB raw plaintext creation, 30 measured pairs

[Raw evidence](256m-pattern-raw-create.jsonl), [summary](256m-pattern-raw-create-summary.jsonl),
[corpus inventory](256m-pattern-raw-create-inventory.json).

| Metric | Baseline median | Bounded median | Paired new/old ratio (95% interval) |
| --- | --- | --- | --- |
| Elapsed | 1.558 s | 1.294 s | 0.823 (0.808–0.835) |
| CPU | 1.521 s | 1.268 s | 0.829 (0.816–0.838) |
| Peak RSS | 298.98 MiB | 180.64 MiB | 0.604 (0.604–0.604) |

The paired estimates show 17.7% less elapsed time, 17.1% less CPU and 39.6% less
peak RSS for this case. It passes the proposed +5% write nonregression threshold
for this case only. It remains slower than ZIP stored creation: 2.148× elapsed
(95% interval 2.113–2.187). ZIP's operation is not the complete Lockbox transaction
and format workload.

### 8 MiB compressed plaintext read, 30 measured pairs

[Raw evidence](8m-pattern-compressed-read.jsonl), [summary](8m-pattern-compressed-read-summary.jsonl),
[corpus inventory](8m-pattern-compressed-read-inventory.json).

| Metric | Baseline median | Bounded median | Paired new/old ratio (95% interval) |
| --- | --- | --- | --- |
| Open plus read | 7.482 ms | 7.499 ms | 1.000 (0.995–1.004) |
| CPU | 7.488 ms | 7.504 ms | 0.999 (0.995–1.004) |
| Peak RSS | 10.65 MiB | 10.54 MiB | 0.991 (0.987–0.995) |

There is no meaningful measured read regression here. This change does not solve
ZIP read parity; the independent-access-unit and owner-authorization work remain
necessary. Protected-mode performance, small-file staging, large-index memory,
aged mutations and complete release qualification remain unmeasured by this batch.
