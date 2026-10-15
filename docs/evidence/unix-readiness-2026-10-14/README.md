# Unix listener readiness correction

The nonblocking Session Agent listener previously slept for 100 ms after an
empty accept queue. A client connecting during that sleep waited for the next
loop, adding fixed latency to the public open/extract control requests. Archive
payload bytes do not cross this IPC channel.

The correction waits for listener readability until the same 100 ms maintenance
deadline. EINTR recomputes the remaining deadline; readiness errors remain errors.
Peer admission, authentication, compatibility and activity requests are unchanged.

Validation: 15 Unix tests and four active-secret tests passed; all-target Vault
Clippy with warnings denied passed. A separate exact-source release build has
1,747 matching pre/post inputs (manifest `01eb4fb26afdb5cb49e92269414171d3f08c6fe2c810d489169bc4b23a64451d`)
and binary SHA-256 `4c3dd27fd35bf71b93f7b1720491aa371ad8dcb492f8f0c0ae16103f97c1f579`.

One plaintext 64 MiB diagnostic before and after the change verifies persisted
bytes/modes and cleanup. Baseline: client connect 1791944807.440808 waited while
the helper slept until .540599. Corrected: helper poll starts 1791945561.828126;
client connects at .919095, poll returns ready around .919143 and accept succeeds
at .919203. This establishes the causal wakeup change, not a statistical release
pass. Traces capture only connection/readiness/sleep syscalls, never payload IO.

Raw local evidence is retained at `/tmp/revault-ipc-readiness-diagnostic-d337`,
`/tmp/revault-ipc-readiness-diagnostic-after` and
`/tmp/revault-listener-readiness-cli-d337`. Earlier compile/setup failures remain
retained separately. The pre-commit hook may change formatting; its exact delta
must be reviewed against the tested source before candidate applicability.

The completed d337 campaign remains immutable and unqualified. Its two small-read
failures, three inconclusive combined RSS bounds, missing CI seal, and actual
distribution-artifact qualification remain release gates.
