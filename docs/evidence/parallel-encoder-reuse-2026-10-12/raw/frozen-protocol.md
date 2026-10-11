# Parallel encoder reuse — frozen public add trial

Fresh Auto/serial public diagnostics establish current1GiB write RSS pressure.
Serial lowers working-set RSS but roughly doubles compressed-source elapsed;
default serial is not the proposed remedy. This trial reuses one existing scoped
encoder per parallel worker, leaving worker count, queues, ordering, page caches,
transaction preparation, authentication and fallback policy unchanged. It adapts
an unmeasured historical prototype at docs/evidence/issue310-parallel-encoder-prototype.patch.
The current explicit optimisation goal and fresh diagnosis justify this trial;
no historical performance failure is overwritten or assumed resolved.

Freeze112 fresh Auto-worker public add processes: two1GiB pinned source types ×
two workflows (single-file Interactive, one-file-directory BulkImport) × old/new
variants ×14 rounds. Exclude first2 rounds per variant/cohort (16preparations),
leaving96 measured. Alternate variant order by round+case+workflow parity. Each
sample creates a fresh empty plaintext unsigned Zstd3/defaultpadding archive
outside timing; public add+commit+drop is the whole timed process. Original85cadd
CLI creates/verifies both variants, isolating the new5b831735 add executable.

Use six adjacent two-round paired log-ratio blocks for each cohort,10,000 seeded
bootstrap draws and separate round-parity views. Seeds derive from0x6b2175d9 by
UTF8 label XOR/multiply16777619 modulo2^32 (zero becomes1). Freeze analysis/helper
hashes before timing. Retain only if ALL cohort elapsed upper95% bounds<=1.05 and
ALL cohort maximum-new-RSS/maximum-old-RSS<=1.05, AND compressed BulkImport satisfies
(median-new-RSS/median-old-RSS<=.95 OR elapsed upper95%<=.95). These gates are fixed
before results. The100MB whole-CLI write target remains open; this intermediate
mechanism gate does not redefine it. No outcome-driven reruns/extra samples.

Use same binary/source provenance and output gates as completed public-add epochs:
clean isolated contexts, short owned-session roundtrip, no credentials, CPU0–4,
zero-resident source admission, finite GNUtime elapsed/CPU/RSS, public doctor exact
format-option checks, recursive listing of sole intended file, independent public
extraction/cmp/mode verification, archive+output drain outside timing and bounded
owned-session cleanup. Helper-agent RSS excluded. Both sources remain unchanged;
no corpus copies.24GiB accounting includes prior epochs and newbinary, reserves
3×source+256MiB before each add and rechecks actual usage before extraction.

30minute work cap with60second cleanup reserve and600second percommand caps,
clamped to remaining budget; bounded cleanup kill grace may extend totalwall.
No builds/tests/runtime edits during timing. Stop on failure and preserve evidence.
If retained, focused workspace wipe, mixed/short parallel reuse, source error,
abort/reopen and CLI lifecycle checks follow; no fullsuite for this individual
experiment. Neither ZIP/PGP parity nor architecture/release qualification follows.
