# Parallel encoder reuse — retained public write improvement

The fixed 112 processes completed (16 preparation, 96 measured), all output,
source/binary identity and cleanup gates passed, and the frozen summary retained
the change. Each parallel worker now reuses the existing scoped encoder lease;
ordering, worker count, cache policy, transaction preparation and authentication
are unchanged. This adapts the preserved unmeasured historical prototype after
fresh current-source public Auto/serial diagnostics, not a previously failed run.

|1 GiB workload/profile|New/old elapsed ratio [95% interval]|Maximum old → new RSS bytes|
|---|---|---|
|Source Zstd / Interactive|0.9672 [0.9473,0.9912]|279,539,712 → 246,951,936|
|Source Zstd / BulkImport|0.9926 [0.9790,1.0068]|153,882,624 → 106,979,328|
|Random raw / Interactive|0.9922 [0.9729,1.0065]|242,429,952 → 242,237,440|
|Random raw / BulkImport|1.0060 [0.9884,1.0300]|116,690,944 → 114,110,464|

All aggregate elapsed upper bounds are <=1.05 and all maximum-RSS ratios <=1.05.
The useful compressed-BulkImport gate passes through median RSS ratio 0.7267
(27.3% reduction), with a 30.5% peak reduction. Its elapsed interval includes one;
a speedup is not established there. Raw BulkImport median RSS is 13.3% higher,
despite its lower maximum, and its odd-round elapsed upper bound 1.0571 does not
establish the 5% bound for every stratum. Preserve these qualifications rather
than treating the aggregate gate as universal nonregression.

Every new cohort maximum still exceeds 100 MB: the whole-CLI write-memory target
remains open. This is an intermediate retained mechanism improvement, not ZIP/PGP
parity, architecture selection or release qualification. All 112 samples and
[the seeded audit](raw/metrics-summary.json) remain; no outcome-driven rerun.
Focused correctness and wiping validation passed after the retained result.
## Frozen experiment description


Fresh Auto/serial public diagnostics establish current1 GiB write RSS pressure.
Serial lowers working-set RSS but roughly doubles compressed-source elapsed;
default serial is not the proposed remedy. This trial reuses one existing scoped
encoder per parallel worker, leaving worker count, queues, ordering, page caches,
transaction preparation, authentication and fallback policy unchanged. It adapts
an unmeasured historical prototype at docs/evidence/issue310-parallel-encoder-prototype.patch.
The current explicit optimisation goal and fresh diagnosis justify this trial;
no historical performance failure is overwritten or assumed resolved.

Freeze112 fresh Auto-worker public add processes: two1 GiB pinned source types ×
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

## Evidence preservation

Command logs are losslessly stored in [command-evidence.tar.gz](raw/command-evidence.tar.gz).
Every extracted byte was checked against [the SHA-256 manifest](raw/command-evidence.sha256);
originals remain in ignored local build storage. Sample rows, summaries and
identities remain directly readable. [frozen-protocol.md](raw/frozen-protocol.md)
preserves the pre-result protocol. No corpus or archive payload bytes are committed.
## Focused validation

All 15 release-mode checks passed: 6 API tests in the default configuration,
8 with native-block-layout/external-source, and the public CLI large-file add
and extraction lifecycle test. The new parallel regression exercises more frames
than workers, mixed compressed/raw input, a short tail, fragmented reads and
midstream errors followed by abort and independent reopen, across encrypted/plain
and signed/unsigned modes. Existing scoped-encoder tests verify numeric output,
size changes, reentry, wiping and unwind cleanup. Their injected panic is expected
and caught by a passing test. Native pipeline tests preserve identity/content
refusal and serial/parallel output guarantees. [Focused logs](raw/focused/commands.txt)
record exact commands; no full-suite rerun was performed for this experiment.
Earlier broad integration timeout limitations remain open.

[Pre-format source snapshot](raw/validation/pre-format-source.tar.gz), hashes and
diff preserve the validated epoch. The tracked commit hook may subsequently apply
formatting-only changes, so these hashes are not claimed as final commit hashes.
