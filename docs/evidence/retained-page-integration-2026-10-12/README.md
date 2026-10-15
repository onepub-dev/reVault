# Retained page optimization integration checkpoint

This checkpoint validates retained runtime revision
`e08a1000a54b912e7050054469f7ce899ef979d7`. It follows the passing
[focused qualification](../direct-clear-page-2026-10-12/RESULTS.md) and includes
the normally enabled candidate tests. It is not a timed performance campaign.

Earlier full default and native API suites used a 1,200-second cap and stopped
without summaries, with 283 and 287 observed passing tests. Their logs remain in
`../credential-open-2026-10-12/raw/integration`. A subsequent isolated default
field recovery campaign passed in 578.56 seconds; that establishes completion
of that case, not the cause of the earlier whole-suite timeouts. Additional
long compaction, definition, deletion, move and selected-source campaigns justify
a larger bounded integration attempt.

## Fixed execution scope

Build the complete release-mode API library tests separately for default features,
then for `native-block-layout`, using locked offline dependencies and Rust 1.88.0.
Each build has a 900-second bound. Record the exact Cargo-selected test binary
and its hash, then capture its complete `--list` inventory. Run the executable
without a test-name filter, with `--test-threads=2`, a 3,600-second runtime bound
and a 30-second termination grace. Existing ignored tests remain ignored; no
additional skips or ignores are permitted.

Configurations run sequentially. Stop at the first failed or timed-out build or
suite for diagnosis; native runs only after default succeeds. Do not extend the
bound or rerun automatically. No benchmarks, runtime edits or competing builds
run during this checkpoint. Capture source identities before and after, commands,
exit statuses, separate build/test elapsed times, inventory and terminal counts.

The default configuration listed 713 tests. Its build completed separately in
1m08s. The suite reported a failure in
`typed_tree_variables_guarded_transaction_and_recovery_faults` after about
51 minutes. The owned test process was then stopped; the wrapper exited 143.
There are 439 passing lines and seven total failure markings in the retained
log, with no terminal suite summary or captured panic details. The first failure
was observed before termination. The other six cannot be attributed to shutdown:
their individual timestamps and causes were not captured. Native was not run.

The same pinned executable then ran the exact variable test with `--nocapture`
and one thread. It passed normally: one test, 712 filtered, 196.82 seconds,
3,432 mutation cases and 2,724 interrupted recoveries. Binary and source hashes
matched before and after. This does not clear the whole-suite failure.

Read-only source and ordering review identified a concrete concurrent test:
`selected_source_replacement_guarded_atomic_recovery_faults` was still running
when the variable failure appeared. Secure heap arenas retain locked backing
after slots are freed, and the current process memlock limit is 8 MiB. Resource
accumulation or overlap is a hypothesis, not an established diagnosis.

The second bounded diagnostic selected exactly those two tests from the same
binary, with immediate panic capture and two threads. Both passed normally in
889.57 seconds (889.597 seconds observed by the runner): two passed, 711 filtered.
The variable test reported 3,432 mutation and 2,724 interrupted recovery cases;
selected-source replacement reported 3,288 and 1,020. Source and executable
identities matched before and after.

The original observer's raw regular expression did not interpolate field names,
so its VmLck/VmRSS fields are null. Those rows remain unchanged. A separate
corrected observer attached to the same live test PID without restarting it and
captured 157 numeric five-second samples. Maximum sampled VmLck was 1,456 KiB
and VmRSS 86,516 KiB; the first approximately 105 seconds are missing. These
sampled values are not continuous peaks. This pair did not approach its 8 MiB
memlock limit and does not reproduce or explain the full-suite failure.

The bounded public SecureVec profile completed successfully: 97 rows across
16 distinct capacities from 8 through 68 KiB, allocated and dropped one at a
time, then repeated in the same process. VmLck started at zero and accumulated
to exactly 1,060,864 bytes, remaining allocated after drops. The second pass
reused that backing without further locked growth. Limits stayed at 8 MiB;
the profile remained below its 2 MiB bound. Capacity fields are inferred from
the source's size classes, not a public capacity getter. This establishes
retained backing and reuse, not the full-suite failure's cause.

The initial profile launcher failed before compilation with exit 127 because
Cargo was absent from its restricted PATH. `raw/arena-retention-profile/result.txt`
describes that preparation failure only. The corrected absolute-Cargo build and
successful profile are recorded in `corrected-build.json`, `profile-run.json`,
`profile-run.stdout` and `validation.json` in that directory. Temporary source
and runner were removed; tracked runtime hashes matched. Exact source is saved
as `.rs.txt` so the commit hook cannot reformat its frozen bytes.

The full-default capture diagnostic repeated the original pinned binary and
713-test inventory with immediate timestamped output, corrected resource
sampling, two threads and the same one-hour bound. After 3,395.911 seconds,
`form_segments_empty_utf8_boundary_and_one_mib_all_modes` failed at its encoder
unwrap with `SecurityLimitExceeded("weakened secure memory allocation is disabled")`.
The sampled locked memory was 8,108 KiB against the unchanged 8,192 KiB limit.
On this platform that refusal follows failed `mlock`; the original log does not
identify the exact requested allocation size. The earlier variable recovery
campaign passed in this run, 26.76 seconds before the form-segment failure.

The runner preserved the panic and stopped the owned process on the actual
failed result. It recorded 380 passes, 15 ignored tests and one failure before
termination; this remains incomplete integration evidence. There was no rebuild,
native continuation, protection change or runtime edit during the diagnostic.
The subsequent [allocator correction and pressure comparison](../secure-arena-reclamation-2026-10-12/README.md)
address retained empty backing without raising the limit or weakening allocation.
