# Guarded variable source-copy resource evidence

This is the bounded test-only experiment declared in [PLAN.md](PLAN.md), not
public activation or a full-format resource gate. The control is the typed
variable implementation at `9800aab2` with an identical FileStore probe.

The initial pilot exposed a harness lock-protocol error: modes 0–5 created
archives but independent reopen timed out while the writer handle was held.
Mode 6 was interrupted; 7–15 were not run. No output was independently verified,
so none of those attempts establishes resource performance. Its logs, stopped
outcomes and artifact hashes are retained. FileStore ownership was not bypassed.

The corrected harness closes writers before independent readers and opens the
next writer outside write timers. All **16** corrected pilot producers and
separate fresh verifier processes pass. Each final value contains exactly
1 MiB of synthetic `b`, at revision 2 with 16 segments; neighbor file bytes and
whole stored image hashes verify. Outputs are 2,949,120 bytes in unpadded modes
and 4,915,200 bytes in padded modes. These are pilot functional observations,
not paired timing samples or a size-gate pass. The 8,192 KiB memlock policy is
unchanged. Build and strict Clippy pass.

The probe records four lifecycle phases: create, no-change, replacement, and
fresh open through assembled read-callback entry. CPU/wall endpoints and memory
endpoints precede independent verification; later phase baselines include the
retained arenas created by declared prior verification. VmRSS is a snapshot;
VmHWM is a whole-process peak including setup. Read verification occurs after
its timer without cloning the returned secret.

The control was frozen at `9d00053c`; its binary SHA-256 is
`2dbd1a1502fe3f356eafbd009d991d6b4f3cdb12bb5d76fc69d21ca77fdc5796`.
Its manifest, probe/plan snapshots and post-format mode 0/15 independent-reader
smoke logs are retained under `control/`. Binaries and synthetic archives remain
under `/tmp/revault-variable-resource-2026-10-09/`; they are not committed.

## Borrowed source correctness checkpoint

The candidate adds checked secure-string byte-range copying and borrows the
existing guarded source while constructing authenticated segments. It removes
the full-value guarded clone in the test-only secret setter; normal-value
allocation, validation, read assembly and allocator policy are unchanged.
Empty/invalid/overflow ranges, split UTF-8 bytes and active read-scope refusal
are covered. All eight plaintext mode combinations preserve exact segment wire
bytes for empty, short and cross-segment UTF-8 inputs. Existing all-mode typed
lifecycles cover protected values and the 1 MiB limit.

Pre-format release checks pass: page API **20**, segment component **4**, and
serialized typed-variable **5** tests (272.96 s). The serial result is retained
as the agent's command-session summary rather than a full transcript. Exactly
one declared default-parallel typed-variable control also passes **5** tests
(283.56 s); it does not establish arbitrary concurrent maximum-value capacity
or erase the two earlier allocation failures. Strict Clippy passes both crates.
An initial archive Clippy failure identified a safety comment moved away from
its nested unsafe block by formatting. Relocating only that comment fixes the
lint; `control/probe-comment-only.diff` preserves the exact source difference.
Control and candidate execute the same probe, but their probe source bytes and
hashes differ. No production behavior or allocator protection was changed.

Post-format checks at `b6af42eb` pass: page API **20**, segments **4**, typed
variables **5** serialized and strict Clippy for both crates. Full transcripts
are retained under `candidate/correctness/`; the parallel control was not
repeated. The candidate manifest/snapshots bind its executable SHA-256
`377372f7ac9efd75384dcf62c1804b1e00b9b58ecd78c40754bbfdf6cf3a1a54`.

## Fixed resource comparison

All **992** declared producer attempts pass: 32 warmups plus 960 measured
processes, providing exactly **30 complete pairs in each of 16 modes**. Every
output passed a separate fresh reader and full stored digest comparison; no
attempts were retried or excluded. Binary/runner identities stayed stable.
No owned builds/tests overlapped sampling. After sampling, all **32** reciprocal
frozen-version reader checks pass against retained last-sample outputs. This
is bounded experimental wire compatibility, not released CLI/binding qualification.

[All 64 mode/phase results](fixed-batch/summary.md) include fixed seeded paired
bootstrap 95% intervals. Across-mode geometric means of candidate/control CPU
ratios are **0.939 create**, **0.959 no-change**, **0.980 replace**, and **1.004
fresh open/read**; wall ratios round to the same values. Every create/no-change
mode interval is below 1; replacement has 13 below and 3 overlapping 1. Three
read modes regress with intervals above 1: mode 8 **1.021** [1.008, 1.033], mode
13 **1.017** [1.003, 1.030], mode 15 **1.016** [1.003, 1.029]. The other 13 read
intervals overlap 1. The read algorithm was unchanged, but these observations
remain measured regressions, not a claim of identical read cost or grounds for
resampling. Aggregate ratios do not have an aggregate confidence interval.

Every phase's median locked-memory endpoint drops **1,028 KiB**: control endpoints
range 3,992–6,620 KiB, candidate 2,964–5,592 KiB. Paired median RSS endpoint changes
range **−1,242 to −1,122 KiB**. Whole-process VmHWM paired median changes range
**−68 to +58 KiB**, offering no useful incremental-peak evidence. Retained arenas
and earlier verification affect later phase baselines. The 8 MiB locked-memory
policy is unchanged; a single passing parallel regression and this bounded
fresh-process lifecycle do not establish arbitrary concurrent capacity.

Raw per-attempt metrics, metadata, stable-completion record, analysis and
cross-reader results are retained under `fixed-batch/`. The artifact manifest
hashes all retained `/tmp` producer/verifier logs and final synthetic images;
large binaries/images are deliberately not committed. Nonfinal successful
sample images were removed only after separate verification and digest recording,
according to the frozen protocol. Source hashes and exact commands are in both
variant manifests. No ZIP, A3/A4/A5, full-format resource or public-activation
claim follows. Next implement bounded test-only variable moves, preserving
selected ownership, existing public validation and secret access semantics.
