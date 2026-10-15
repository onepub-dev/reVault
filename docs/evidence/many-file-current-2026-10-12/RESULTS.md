# Current many-file extraction

All 16 fixed extractions completed in 157.046 seconds: four excluded preparation
and 12 measured operations. Four existing public-CLI-created plaintext unsigned
archives were pinned and independently verified. Current default extraction
matched every file's bytes, the exact tree and the modes observed through a
separate explicit-restoration setup. Cold-archive, identity, drain and owned-agent
cleanup checks passed. The retained `4142dd04` CLI and original build provenance
were reused; no runtime, cache or worker policy changed.

| Data / shape | Median elapsed (range) | Maximum measured CLI RSS | Maximum measured CLI + helper bound |
| --- | ---: | ---: | ---: |
| Raw, 256 × 256 KiB | 0.67 s (0.51–0.68) | 211,038,208 B | 219,348,992 B |
| Raw, 1,024 × 1 MiB | 4.21 s (4.02–4.41) | 254,296,064 B | 262,627,328 B |
| Zstd source, 256 × 256 KiB | 0.49 s (0.41–0.51) | 119,627,776 B | 127,959,040 B |
| Zstd source, 1,024 × 1 MiB | 1.45 s (1.44–1.54) | 234,790,912 B | 243,331,072 B |

All four workloads fail the 100,000,000-byte requirement in the CLI process alone.
The largest preparation helper-inclusive bound was 262,275,072 bytes. These
three-repetition results are descriptive; historical timings were not reused as
controls and no speedup claim is made. The previously rejected worker-cache
budget policy remains absent.

The helper-inclusive value is command peak RSS plus the fresh owned agent's
post-command VmHWM, a conservative sum of peaks rather than an observed concurrent
peak. Shared pages may count twice. Agent identity used socket inode, executable
hash and PID start-time; every agent task's child list was checked at observation
boundaries. This is still not a complete transient process census. Only numeric
memory and process identity were recorded, without command lines or environments.

These file counts select the parallel ExtractMany path, eligible for up to four
cloned handles. Actual worker activation was not instrumented. Page-cache clones
share existing immutable payload ownership, but subsequent per-worker retention
can differ. This baseline establishes the process-level gap, not precise causal
allocation ownership. Next work targets useful data lifetime across workers while
preserving packed-frame reuse and authenticated reads; merely removing raw-frame
copies cannot by itself establish the complete many-file memory requirement.

Two premature summary-analyzer invocations failed before setup/sample completion
(one omitted the argument, another had no completed ledger). Their logs remain
as preparation incidents. Static analysis passed and the single fixed batch then
completed; no failed timing samples were replaced. No tests or builds were run
for this measurement-only checkpoint. Frozen protocol and complete raw records
remain in `PROTOCOL.txt` and `raw/`.
