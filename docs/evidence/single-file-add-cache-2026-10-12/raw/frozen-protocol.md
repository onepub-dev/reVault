# Single-file add cache — frozen public write trial

Current public single-file imports into existing archives retain the Interactive
profile and exceed the100MB write RSS target. The serial diagnostic also exceeds
that target, while the rejected in-flight window did not meet its combined gates.
This trial changes only that CLI branch to a32MiB page/native-frame cache target,
using a cache-limit setter. Compression, workload profile, worker count, scoped
encoder reuse, authentication and durable preparation remain unchanged.

The setter preserves dirty pages and active shared owners, performs no I/O, and
reconciles separate decoded-frame retention after releasing its page-cache borrow.
The target is soft clean-cache retention, not total RSS. BulkImport/new archives,
directory imports and multiple sources keep their existing CLI policy.

Freeze56 fresh processes: two pinned1GiB source types × old/new ×14rounds.
The first2rounds per case/variant are preparation (8 excluded),48measured remain.
Alternate variant order by round+case parity. Preserved optimized baseline5b831735
and the new build are pinned before timing. The old CLI creates empty plaintext,
unsigned Zstd3 archives outside timing and verifies both variants. Full single-file
add/commit/drop is timed; no credentials or phase diagnostics are used.

Use six adjacent two-round paired log-ratio blocks,10,000 seeded bootstrap draws
and separate parity views. Seed starts0x6b2175d9 then UTF8 label XOR/multiply16777619
modulo2^32 (zero becomes1). Retain only if BOTH aggregate elapsed upper95%bounds
are<=1.05, BOTH maximum-new-RSS/maximum-old-RSS are<=1.05, AND BOTH median-new-RSS/
median-old-RSS are<=.80. No additional samples or alternative post-result gate.
The100MB write target remains open unless the measured process actually meets it.

Freeze runner, protocol, analysis/helper, executable and build/source identities.
Same verified public lifecycle as prior add trials: short isolated agent contexts,
CPU0–4 affinity, Auto workers, zero-resident source admission, finite GNUtime
CPU/time/RSS, public doctor/list checks, separate public extraction, independent
cmp/regular-file/size/0644 checks, archive/output drain, cleanup and final hashes.
Helper-agent RSS is excluded.24GiB owned budget includes prior contexts and the
new binary; reserve3×source+256MiB and recheck usage before extraction.30minute
work cap reserves60seconds for cleanup; percommand600seconds or remaining budget,
with cleanup kill grace allowed beyond the work cap. No builds/tests during timing.

If retained, focused tests cover cache policy/profile transitions, dirty and active
owners, commit/abort/reopen, public replacement/refusal and unchanged BulkImport
scope. No full suite per experiment, small-file performance claim, ZIP/PGP parity,
architecture selection or release qualification is implied.
