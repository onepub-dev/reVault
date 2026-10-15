# Unix framed transport correction

Candidate bf491e0e831c4d8b5ee08982b3eb8d9fffb67607 failed three macOS
installed-package jobs in prepare run 38502741129. Kotlin on x86_64 and Lua
reported ENOTCONN at the client write shutdown. Go, Kotlin and C# also reported
Session Agent startup failures. Failures include default Go and agent-mode Kotlin, C# and Lua. Raw failure
logs remain in `/tmp/revault-ci-bf491e0e-failures`; the failed candidate is not
eligible for publication. Its remaining long API jobs were sent cancellation
after all package failure logs were retained.

The server consumes a length-framed request and sends its reply without waiting
for EOF. Both client request paths now omit the redundant write half-close,
which can race a fast server reply and close on macOS. Deadline handling,
credential checks, frame limits and reply parsing remain unchanged. Compatibility
probes previously interpreted any Info exchange error as incompatibility; this
could explain the startup failures, but that inference still requires macOS
package validation.

Focused Linux validation: 17 Unix tests and four active-secret tests passed,
with zero failures or ignored tests. Both new exchange tests assert that the
server sees WouldBlock rather than EOF after consuming the request, then return
and parse a framed reply. Existing buffered-close and absolute-deadline checks
remain covered. All-target Vault Clippy with warnings denied passed.

Raw commands, logs and exits are retained
at `/tmp/revault-framed-client-tests-bf491`. Tested unix.rs SHA-256 is
`bfd51b57eceb92bdaff8e5c2b4d627af8111f5cf3ace461263266b158dd52560`.
The initial pre/post inventory commands ran from the wrong directory and both
produced an invalid one-line empty-input digest. Those files are preserved as
setup failures. A complete post-run manifest covers 1,746 Rust files; no valid
full pre-run inventory exists. The test owner made no source edits, and the
reviewed diff and exact tested unix.rs hash support the scoped results without
claiming a complete pre/post source proof.

A subsequent cache-path explanation is comment-only; the tracked hook formats
the staged source. Both pre-hook snapshots are retained for equivalence review.

The actual bf491 Linux distribution artifact was cryptographically verified and
passed an untimed two-profile process admission: persisted bytes and modes,
24 thread clones, no non-thread creation, and owned cleanup. That does not
qualify its resource use. The new six-case plan remains disarmed. No resource
or 450-observation qualification was run on this failed candidate.

The next harness uses per-operation filesystem space admission plus a 2 GiB
reserve, replacing the artificial 24 GiB history cap and repeated history scans.
Read archives are reused with fresh outputs and complete content verification.
All older frozen campaigns and their failed or inconclusive gates are preserved.
