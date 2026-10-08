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

Plan/harness/control identities will be frozen before candidate changes, and
the exact fixed paired batch and candidate correctness results retained next.
