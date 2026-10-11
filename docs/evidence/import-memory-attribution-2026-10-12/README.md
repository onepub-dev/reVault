# Temporary import memory attribution

Base checkpoint: 80709b0634a1c1dc188970363f3b62843069e987, clean before
installing the diagnostic. The persistent goal stops below10% or above99%
remaining; the latest authoritative reading is55% remaining. No release.

This is a diagnostic screen, not an optimization or comparative speed trial.
Build a separate release-mode CLI with the reviewed temporary instrumentation,
preserve its full source diff/new module/build output and hashes, and retain the
normal current CLI08bf binary unchanged. Restore the complete temporary diff
after the four runs. Do not commit diagnostic runtime code or replace WASM.

## Fixed workload

Exactly four public add operations: the pinned compressible and random1GiB
sources, each using (a) existing empty plaintext unsigned Zstd3 archive plus
single-file add (Interactive, command-local8MiB cache), and (b) the established
containing-directory import with exact include filtering (BulkImport).
Use the already audited public-add command shapes and source identities.
Create each fresh archive outside the diagnostic using the normal CLI. Enable
REVAULT_IMPORT_MEMORY_TRACE=1 only for its instrumented add, with Auto workers
and CPU0–4 affinity. Keep identical frame/compression/profile/queue choices.
Afterward use the normal CLI to inspect options, list exact membership, extract
all bytes, compare independently, verify expected modes and drain output.
No payload or path appears in the diagnostic records; fixture command metadata
retains its normal public non-secret paths. No user Vault or credential use.

Use existing isolated environment, cold source admission, fresh destinations,
24GiB aggregate artifact cap/headroom and bounded owned-agent cleanup. Each
add has a600second timeout plus kill grace, complete batch30minutes with cleanup
reserve. No tests/builds overlap runs. Four results are descriptive only:
instrumentation serializes counter updates and reads/probes /proc, changes
layout and adds overhead, so elapsed/RSS is not performance acceptance evidence.

## Ledger and output

The one mutex protects these13 capacity/count categories in order:
queued input; active owned input; prepared results; ordered pending results;
frame currently being written; producer scratch; active scoped encoder backing;
pooled scoped encoder backing; object stream; temporary body codec/copy;
body; stored body; encode-only final page.

Input leases transfer through queues into owned payload. Raw fallback reuses
the lease/allocation; compressed output obtains its own lease while the input
still exists, and input lease ends after the existing explicit payload drop.
Borrowed frame views acquire no input lease. Prepared leases survive result
queue and ordering and end with their existing owners. Encoder scratch/output
capacity is active versus pooled, with zeroization/drop behavior unchanged.
No existing vector allocation, size, payload copy or lifetime is modified.

At every tracked capacity transition, read RSS/HWM and snapshot all ledger
categories together. Keep complete snapshots of greatest observed RSS, HWM and
live tracked capacity even after ordinary output reaches256rows. Allow at most
four boundary reports (each current + three peak snapshots), giving a maximum
272rows/process; report suppressed selected snapshots. Peaks from different
times must never be added together. HWM is historical process high-water; the
associated snapshot is when that value was observed, not necessarily when the
kernel peak occurred. A counter snapshot is coherent for the ledger, not atomic
with the actual allocator or kernel RSS. Registration/drop/return handoff gaps
are brief and explicit; sampling can miss peaks or perturb them.

Before/after each page flush, and at commit start/completion, separately report
PageCache used/limit/entries plus last-update sequence. It is an approximate,
potentially stale ownership-weight gauge, not additional unique heap bytes.
After parallel scope returns, all workers have joined, worker TLS and producer
scratch/writer have dropped. Require a `parallel_joined` boundary with every
tracked byte/count zero. Require the successful `commit_done` boundary and
all ledger snapshots balanced (category additions minus removals equals current
bytes); reject the diagnostic admission if any invariant/required row fails.
Zero-byte allocations still have counts and must also clear after join.

## Coverage limits

Final PAGE covers encode_page_with_format only; its lease ends at return even
though returned bytes can remain alive in storage. BODY transfers between
helper/caller leases with short unobserved return gaps; normal default raw file
page framing is the target, not exhaustive metadata codec accounting.
Untracked memory includes pending packed-page payloads, metadata/strings and
index structures, decoded payload/shared Arc ownership beyond the separate
cache gauge, fallback codec internals, allocator retained arenas/fragmentation,
thread stacks, agent memory and the diagnostic's own bookkeeping/I/O buffers.
Do not label instrumented live bytes as total memory or infer untracked bytes
by subtracting overlapping gauges from RSS. Use observed coincident categories
to propose a later independent measured intervention, not to claim causality.

The final report must show peak snapshots, post-join conservation, suppression
counts, whole public-process peak RSS and independent byte verification for all
four cases, and verified restoration to the original runtime after preserving
the instrumented source epoch. No new optimization is retained by this screen.
