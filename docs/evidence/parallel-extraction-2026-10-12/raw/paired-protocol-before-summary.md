# Public many-file extraction cache budget — 2026-10-12

This is the next bounded experiment after the single-file ExtractMany and public
password-API improvements. Complete their batched integration checkpoint first.
Use the current default A CLI (`issue303-protocol`); no candidate, ZIP, alternate
native-block layout or encrypted-performance claim is made here.

Create two plaintext unsigned, default-padded, uncompressed archives through the
public CLI: 256 files of 256 KiB (64 MiB total), and 1,024 files of 1 MiB (1 GiB).
Split the existing pinned random corpus sequentially into owned files; retain the
original source identity and every source-file hash. Use public directory import,
then a separate public extraction and exact recursive byte/tree comparison before
timing. Verify all persisted files, not a sentinel. Source/fixture construction is
untimed. Preserve prior evidence and partial failed setup.

The public `extract --to` path selects ExtractMany. At this file count it uses up
to four workers, each cloning the handle and its cache; the present 32+8 MiB targets
are per handle, not a combined parallel budget. Establish actual whole-CLI RSS and
elapsed time before changing this behavior. Do not treat the code-level maximum
as measured simultaneous cache residency.

First run one excluded preparation plus three measured rounds per case: eight
fresh public CLI processes. Use CPUs 0–4, normal default permissions and normal
secret-activity registration. Short isolated agent contexts must pass a real
public session roundtrip and socket check before timing; only owned agents are
cleaned up afterward. Helper-agent RSS is excluded from the CLI process maximum
and must remain an explicit qualification limit. No ambient user configuration,
Vault or platform credential store is used.

Before each run, sync/evict only the owned archive and require zero fincore
resident bytes (at most three attempts). GNU time reports whole-process elapsed,
user/system CPU and maximum RSS. There is no reader-TID/physical-read admission
claim. New output directories must not exist. Outside timing, verify the exact
file tree, default permissions, and every persisted byte with a native recursive
comparison against pinned sources; drain output and directories before removal.
Keep command logs and failures. No tests/builds overlap measurements.

If this baseline demonstrates cache multiplication, propose a bounded Auto-only
worker budget without changing explicit cache policies or authentication. Freeze
both release executables and a fixed old/new comparison before timing: two excluded
preparations plus twelve measured rounds per case/variant (56 processes), alternating
order by round+case. Use the established seeded 10,000-replicate paired bootstrap
of six adjacent two-round log-ratio blocks, plus even/odd round views. Retain only
if every new measured CLI stays below 100,000,000 RSS bytes and both aggregate
elapsed upper 95% bounds are at most 1.05; report stratum uncertainty. No posthoc
sample increase, threshold adjustment or architecture promotion follows.

Bound each process to 180 seconds plus 10-second kill grace and each batch to 30 minutes,
shortening commands to remaining work budget and reserving 60 seconds for cleanup.
Owned cleanup remains independently bounded even after the work deadline; kill
grace can extend total process lifetime beyond 30 minutes. Enforce the existing combined 24 GiB owned
artifact cap, including previous fixtures, new source/archive and next verification
output. Freeze runner/protocol/binary/source/archive identities before timing and
verify afterward. On failure, stop and retain partial evidence. After measured
retention, focused parallel extraction, adjacent shared-frame reuse, cache policy
and authenticated refusal checks precede a later batched integration checkpoint.

Raw frames may use the decoded frame cache; this experiment makes no compressed-data performance claim. The runner sets and restores Linux umask 0022, requires regular output files with mode 0644 and directories with mode 0755, and rejects missing/extra entries or file-type changes. Native streaming concatenation comparison proves every generated source slice matches the pinned original bytes.

## Baseline and frozen trial

The eight baseline processes completed with exact tree/bytes/modes, cold archive,
identity and owned-session cleanup gates passing. The three measured runs per
case had median elapsed 0.64/4.34 seconds and median RSS 210.82/257.88 MB for
256 files/64 MiB and 1,024 files/1 GiB respectively. This is descriptive evidence
of high many-file memory; the paired intervention tests the cache explanation.

The trial preserves the parent Auto ExtractMany targets of 32 MiB pages plus 8 MiB
decoded frames. Actual nonempty worker jobs divide an additional 8 MiB page/native
target and 8 MiB decoded target, retaining at least one ordinary 2 MiB frame each
with at most four jobs. The maximum sum of clean retention targets is 56 MiB;
shared Arc ownership can make that sum conservative, while dirty pages, live
references, metadata and transient buffers mean it is not a hard memory/RSS cap.
Explicit policies, WASM, other profiles, job grouping and authentication stay
unchanged. The parent is untouched on both success and error.

The fixed 56-process paired protocol above is unchanged and uses fresh old and
new processes, not the earlier descriptive rows as a comparison arm. The old
CLI remains pinned; the new optimized build, three-file runtime diff and source
hashes are in `raw/build-new/`. The exact baseline runner and protocol are
preserved separately before the paired extension. No tests or builds overlap
timing, and focused validation follows only if the frozen retention gates pass.
