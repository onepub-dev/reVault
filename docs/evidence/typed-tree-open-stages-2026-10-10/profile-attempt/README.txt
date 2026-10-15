Diagnostic profiling fallback — 2026-10-10

Frozen source: issue-310 worktree HEAD a47f686f18b1cfd2bfa18db814f610976a591e60. No source edits or rebuilds. Runs pinned to CPU 2. Fixture symlink view in ./fixture points to the frozen raw64m-random tree.lbox/tree.public, parent case.json, and source directory; original fixture files were not modified.

Executable SHA-256 before and after (identical):
- revault_lockbox_api_tests: 0fda7dd38313e0d35a58f0acc02210fe00b976c615cf6dc25efd357a73ad7012
- adapter_tree: 58dfe180cac797170680b7758c2c3f5b16e0bf2fb32b61a8bc51bf1730e3a77b
- adapter_packed: 17d21e7d847c58b4d5e485f5714f7b13632d9baef5649c00072f65d5582e10db
- archive_evaluation: fc8c419adfc158f44512416b2bd6acf06cf374640f45d90e54a420315a2a879e
Fixture hashes: tree.lbox 0c098194eb7cfe027762af2a933f96a814854366773cd3bf6d67f9efeacf6775; tree.public d49f86bc45d4d0d7d389b764144a7a2ada0dad328cd97b9ab173cc860b93f008; source/file-000000.bin 34bef3bb1ce80841f85d406982294557ae9221fb0f6e63ca9a28b6a5e6d8cf3a (matches inventory.json).

CPU profiling unavailable without changing host policy: /proc/sys/kernel/perf_event_paranoid=4; `perf stat -e cpu-clock:u -- true` reported “No supported events found” and denied performance monitoring. `strace -f -c ...` failed with PTRACE_TRACEME/PTRACE_SEIZE Operation not permitted. `bpftrace` failed reading /sys/kernel/tracing/available_events (Permission denied). No policy changes or privilege escalation were attempted. There are no perf samples/call-graph reports.

Non-invasive alternative: frozen candidate_file_resource_probe itself, 1000 passes, verified result, CPU2. Important scope: sample() opens the image once before the 1000-pass visit loop. Its open_seconds/open_resources cover that single open; the passes repeat visits only. Range verification then reopens and reads/compares the full source archive outside the timed interval. These are probe stage timings, not whole-process profiling samples or a repeated-open measurement. Exact commands:

Stream:
env REVAULT_CANDIDATE_ROOT=/tmp/revault-tree-profile-20261010/fixture REVAULT_CANDIDATE_PHASE=tree-sample REVAULT_CANDIDATE_UNIT=65536 REVAULT_CANDIDATE_ACCESS=stream REVAULT_CANDIDATE_PASSES=1000 taskset -c 2 /tmp/revault-tree-read-20261010-final/bin/revault_lockbox_api_tests --exact file_format::candidate_files::resource_probe::candidate_file_resource_probe --ignored --nocapture

Midpoint range:
env REVAULT_CANDIDATE_ROOT=/tmp/revault-tree-profile-20261010/fixture REVAULT_CANDIDATE_PHASE=tree-sample REVAULT_CANDIDATE_UNIT=65536 REVAULT_CANDIDATE_ACCESS=range REVAULT_CANDIDATE_PASSES=1000 taskset -c 2 /tmp/revault-tree-read-20261010-final/bin/revault_lockbox_api_tests --exact file_format::candidate_files::resource_probe::candidate_file_resource_probe --ignored --nocapture

Both tests passed and emitted verified:true with the frozen test executable hash. Raw stdout/stderr are raw64m-stream.{stdout,stderr} and raw64m-range.{stdout,stderr}.

Observed stage timings (single diagnostic run; not benchmark qualification):
- Stream: 67,108,864,000 bytes across 1000 passes; open 0.002182 s, visit/read 42.000915 s, total 42.003099 s; read-stage CPU 41.995125 s.
- Midpoint range: 4,096,000 bytes across 1000 passes; open 0.002462 s, visit/read 0.037026 s, total 0.039490 s; read-stage CPU 0.037018 s.

Interpretation limit: in the timed loop, stream cost is overwhelmingly in payload visits. In the midpoint-range case, the single timed open took about 6.2% of total; repeated 4 KiB visits took the rest. This says nothing about the cost of repeatedly opening the image. Verification reopens once and scans the full archive after the timers stop. The probe does not decompose visit/read into function-level CPU hotspots, so it cannot identify exact open-path functions. No comparison reruns or qualification claims were made.
