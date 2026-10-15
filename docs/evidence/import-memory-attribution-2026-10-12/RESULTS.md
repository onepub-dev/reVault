# Import memory attribution results

Four fixed diagnostic imports completed successfully in 44.828 seconds. All
independent public-CLI extraction, byte, permission, cold-source, identity and
owned-agent cleanup checks passed. Every ledger snapshot balanced; all thirteen
transient byte and allocation-count categories were zero after each parallel
join. Successful commit boundaries and all three complete peak snapshots were
present. The temporary instrumentation was then removed: Rust and binding
sources exactly match checkpoint `80709b0634a1c1dc188970363f3b62843069e987`.

This screen identifies a possible allocation improvement. It does not establish
a speedup, a memory reduction, or completion of the 100 MB target. Instrumentation
serializes bookkeeping and changes scheduling and allocation layout. There was
one observation per workload, with no comparative timing inference.

| One-GiB input / workflow | GNU time peak RSS (MB) | Maximum observed /proc RSS (MB) | Peak tracked capacity (MB) | Trace rows / suppressed |
| --- | ---: | ---: | ---: | ---: |
| Compressible / Interactive | 92.115 | 92.434 | 61.999 | 179 / 0 |
| Compressible / BulkImport | 88.449 | 88.887 | 66.352 | 145 / 0 |
| Random / Interactive | 71.324 | 71.418 | 35.653 | 264 / 134 |
| Random / BulkImport | 58.216 | 58.384 | 35.653 | 264 / 124 |

MB means 1,000,000 bytes. GNU time and /proc are separate accounting/sampling
sources; their small differences are retained rather than treated as agreement.
The raw traces retain complete peak snapshots even after ordinary-row suppression.

## Allocation observations

At the maximum tracked-capacity snapshot for compressed Interactive, object-stream
capacity was 13,024,160 bytes while the temporary body and resulting body were
8,118,434 and 8,118,450 bytes. BulkImport showed the same pattern: 13,096,768 bytes
versus 8,325,443 and 8,325,459 bytes. Both snapshots included 21,843,565 bytes of
active scoped encoder backing across five workers. The raw cases had about
8.389 MB of object-stream capacity alongside two approximately 6.292 MB body
buffers, with no scoped encoder backing recorded.

The source constructs the object stream with an initially empty Vec and appends
headers and payloads. Its existing checked `encoded_object_stream_len` helper
can compute the exact final length before allocation. Reserving that size is a
specific next hypothesis for avoiding geometric excess capacity and reallocation;
it requires a separate uninstrumented matched experiment before retention.
No encoder, worker, cache or format-policy change is justified by this screen.

At the observed RSS maxima the page-encoding leases had already cleared; remaining
tracked capacity was approximately 27 MB for compressed data and 14.68 MB for raw
data. These observations do not prove what caused the kernel peak or assign the
untracked remainder. Page-cache gauges are potentially stale and overlap other
owners. Allocator arenas, packed payloads, fallback codec internals and other
untracked allocations prevent treating ledger totals as total heap or subtracting
them from RSS. Separately sampled maxima must not be added together.

## Reproduction and disposition

The unchanged [frozen protocol](README.md),
[four accepted rows](raw/diagnostic-samples.jsonl),
[completion record](raw/complete.json), and
[frozen identities](raw/baseline-identities.json) retain exact admission checks.
`raw/c0-w0-trace.json` through `raw/c1-w1-trace.json` contain the full ledger output;
`raw/commands` and `raw/phase-runs` preserve public invocations and terminal logs.

The instrumented release-mode development binary hash was
`84bf1bf67210a9f62d2b2b4d0c2a6591b3330ee236da9b3e898cfe286584227b`.
Normal verification used preserved CLI
`08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2`.
The original 48.56-second build log, six-file temporary patch, seventh new module
and matching before/after source manifests are in `raw/build-instrumented`.
The new module was compared byte-for-byte with its evidence copy before removal.
No diagnostic runtime code, generated WASM change, or optimization is retained.
No qualification tests or broad suite were run for this measurement-only screen.

The commit hook formatted the initial `.rs` evidence copy. Before publication,
the original hash-verified bytes were preserved as `import_memory_trace.rs.txt`;
`raw/build-instrumented/evidence-relocation.json` maps the original frozen path
to that copy. Frozen identities and source manifests remain unchanged and refer
to the measured epoch, not the restored runtime. No measurement was repeated.
