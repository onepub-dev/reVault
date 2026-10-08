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

Post-format candidate checks/freeze and the fixed comparison are next. The
retained runner requires all four phases, independent persisted verification,
stable binary/runner identities and inherited CPU-affinity reporting. Analysis
requires exactly 992 distinct declared attempts (32 warmups and 960 measured
processes) plus stable completion. Reciprocal frozen-version readers follow
sampling; none of this is released CLI/binding compatibility qualification.
