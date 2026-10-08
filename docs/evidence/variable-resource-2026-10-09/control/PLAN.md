# Guarded variable source-copy experiment

Frozen hypothesis, 2026-10-09: a borrowed SecretString range-copy encoder can
remove the full-value SecureVec clone before segmentation without changing
wire bytes, selected ownership, validation, secret access or allocator policy.
The control is the qualified typed-variable implementation at `9800aab2` plus
this identical test-only probe. Candidate source changes follow only after the
probe/control binary and their identities are frozen.

## Fixed protocol

- Linux FileStore; fresh owned archive and process for each run; synthetic
  1 MiB ASCII `a` then `b` values, with source hashes retained. All 16 encryption,
  signing, compression and padding combinations. Small filesystem seed creation
  and initial file synchronization precede measurement.
- Lifecycle order: create, independently reopen/verify, no-change, exact stored
  hash comparison, drop old caller input, construct replacement input, replace,
  fresh open and read. The first caller input is dropped before the replacement
  input is constructed; no caller-held duplicate is silently introduced.
- Write handles are acquired outside write timers and dropped before independent
  readers; FileStore exclusive ownership is never bypassed. Each write timer
  ends at the operation's return. Read begins before fresh open
  (including signed-plaintext eager verification) and ends at fully assembled
  callback entry; value byte verification follows that endpoint without cloning.
- Capture process CPU user/system time and wall time, phase-endpoint VmLck and
  VmRSS before out-of-timing verification, and whole-process VmHWM/getrusage peak
  RSS. Endpoint VmRSS is not an incremental peak. Secure arenas remain retained;
  the next phase baseline intentionally includes arenas allocated by prior
  operations and declared verification. No memory is forcibly reclaimed.
- A separate fresh verifier process reopens every completed output, compares
  all value bytes and unchanged neighbor file bytes, and checks revision, length
  and complete stored image hash. Only public owner verification material and
  explicitly synthetic fixture keys are used.
- One declared warmup per variant/mode, then exactly **30 paired fresh-process
  runs per mode**. Alternate control/candidate order by pair; retain every
  attempt and failure, with no retry, sequential stopping or additional samples.
  A failed pair is reported and excluded from numerical ratios explicitly.
- No owned builds, tests or other benchmarks overlap sampling. Record binary,
  source, compiler, features, corpus and runner identities. Keep one completed
  artifact per variant/mode (last measured sample); other task-owned sample
  files may be removed only after separate verification and hash recording.
- Keep the host 8,192 KiB locked-memory policy and secure allocator unchanged.
  A baseline pilot establishes harness correctness across all modes before
  comparison; pilot results/failures are retained separately and are not samples.

Report paired CPU/wall ratios by mode and phase (geometric mean and seeded
bootstrap 95% intervals), absolute VmLck endpoints/differences and whole-process
RSS. No multiplication with other experiments, ZIP comparison, A3/A4 pass,
public activation or whole-format resource qualification follows. Existing
parallel-test allocation failures and region-loss trade-offs remain evidence.
