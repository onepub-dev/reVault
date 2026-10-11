# Single-file add cache — retained result

The fixed 56-process trial completed all 48 measured and 8 preparation processes,
with persisted-byte/mode checks, final identity verification and owned-agent
cleanup passing. Runtime was 583.775 seconds. Both source cohorts pass the frozen
combined rule: aggregate elapsed upper 95% bound ≤1.05, maximum RSS ratio ≤1.05,
and median RSS ratio ≤0.80. No samples were repeated.

| 1 GiB source | Elapsed new/old (95% interval) | Maximum RSS old → new, MB | Median RSS reduction |
| --- | --- | --- | --- |
| Source Zstd | 0.99924 [0.97862, 1.01721] | 242.43 → 150.52 | 38.0% |
| Random raw fallback | 0.99374 [0.98124, 1.00340] | 252.54 → 149.72 | 44.4% |

MB means 1,000,000 bytes. Neither aggregate interval establishes a speedup. Raw
even rounds show a small slowdown: 1.01490 [1.00442, 1.02648]; odd rounds improve.
Both maxima remain above 100 MB, so the whole-CLI write target stays open.

Retain the 32 MiB page/native-frame target only for an existing archive's single
non-directory source add. Workload profile, compression, parallelism, encoder
reuse, authentication and durable preparation are unchanged. BulkImport branches
are untouched; they were not timed again. Dirty pages and active owners may exceed
the retention target. No small-file timing, total-memory cap or release claim.

## Validation and API consistency

All 51 focused test executions passed: 46 default/native API executions across
overlapping filters, two public CLI lifecycle filters, two additional decoded
cache transition checks, and one native ABI setter lifecycle. They cover pending
commit/abort/reopen, held clean owners and dirty payload preservation, authenticated
cache reads, Bytes-to-Auto decoded retention, and public refusal/overwrite behavior.
No full suite was rerun for this experiment; earlier broad API timeout limitations
remain unchanged.

The additive public Rust setter has matching native ABI, WASM and facade routes.
Native and WASM release builds passed, and the four tracked WASM artifacts were
regenerated. Dart analysis and actual Dart/JavaScript setter lifecycles passed,
including an observed 8 KiB limit, invalid-mode refusal without policy mutation,
and persisted bytes after commit/reopen. Portable WASM setter and password
lifecycles passed; portable WASM has no cache-stat accessor, so its setter test
establishes content/lifecycle behavior rather than an observed retention limit.
C11/C++20 syntax checks, Python syntax compilation and direct Java 22 compilation
with lint/Werror passed. Go, C#, PHP, Ruby, Lua and Swift runtimes were unavailable;
their updated routes and conformance assertions were statically reviewed.

The repository binding checker stops on 39 unreviewed names. An isolated HEAD
snapshot reports the identical set: this is pre-existing review/scanner debt,
not a passing complete binding check. The new symbol's mapping, inventories and
routes were reviewed separately. System Gradle 7.6 failed before compilation due
to a plugin API mismatch; direct javac passed. Harness corrections are preserved
in the binding evidence: Dart carrier/import setup and WASM Number-versus-BigInt
attempts retain separate logs. The first JavaScript harness wrongly expected a
false return instead of the facade's exception; its full stderr was accidentally
overwritten, and only the recovered error/context is preserved. The corrected
exception/state-preservation lifecycle passed. No product assertion was weakened.

See [raw summary](raw/metrics-summary.json), [completion](raw/paired-complete.json),
[identities](raw/paired-identities.json), [focused checks](raw/focused/),
[binding audit](raw/bindings/compile-check-audit.txt), and
[frozen protocol](raw/frozen-protocol.md).

## Evidence preservation

All 1,878 completed benchmark command files are losslessly packaged in
[command-evidence.tar.gz](raw/command-evidence.tar.gz); every extracted file was
verified against [the manifest](raw/command-evidence.sha256). Originals remain in
ignored local build storage. Samples, summaries, build and validation logs remain
readable. The measured source/build identities precede test and binding additions.
[Pre-format checkpoint sources](raw/preformat-checkpoint-sources.tar.gz) and
[their hashes](raw/preformat-checkpoint-sources.sha256) preserve the validated
checkpoint before the tracked commit hook formats Rust; they do not claim byte
identity with post-hook formatted source. No archive/corpus payloads are included.

## Frozen protocol


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
