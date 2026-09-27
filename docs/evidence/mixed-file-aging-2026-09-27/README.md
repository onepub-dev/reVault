# Mixed candidate-file aging: retained failed qualification

2026-09-27. Candidate C after the compaction checkpoint `2a83b6d3` remains
test-only. This is a correctness/space qualification, not a timed performance run.

The ten-operation cycle adds files, replaces 128 KiB with 400,000 bytes, removes
entries, replaces a large file with 17 bytes, updates an initially empty file,
repeats unchanged input with an absent removal, adds a 64 KiB file, and performs
mixed add/remove and repeated removal. The persistent neighbour is 8 KiB. Repeat
100 times in each of four protection/codec/padding modes, without compaction.

Every operation reconstructs a fresh storage reader, compares every expected file
and absence, audits complete physical ownership and checks that every reusable or
retired byte is zero. Every unchanged repeat must preserve the entire archive
byte-for-byte. The strict stability hypothesis is that the first 100 operations
establish the high-water size for the remaining 900; failures are retained.

| Mode | First-100 high water | Final bytes after 1,000 | Content, ownership, erasure, 100 unchanged repeats | Stability |
| --- | ---: | ---: | --- | --- |
| Plain unsigned raw, default padding | 1,507,328 | 1,507,328 | Pass | Pass on this workload |
| Encrypted signed compressed, default padding | 1,114,112 | 1,114,112 | Pass | Pass on this workload |
| Plain signed compressed, default padding | 1,114,112 | 1,114,112 | Pass | Pass on this workload |
| Encrypted unsigned raw, no padding | 1,245,184 | 1,376,256 | Pass | **Fail: +131,072 bytes at operation index 421** |

The first run stopped at the growth. The next diagnostic run retained the same
assertion but completed all 4,000 operations before failing, with exactly the same
growth point. It did not weaken the hypothesis or replace the failed workload.
The failure is part of the current scorecard; passing safety invariants does not
make the aging/resource gate green.

The committed allocation map keeps `PENDING` labels for retired ranges after
journal cleanup zeroes them; rewriting the map just to relabel them would require
another publication. The byte audit covers both `FREE` and `PENDING`. An initial
harness assertion that the label count must be zero was corrected to the existing
protocol; erasure verification was retained. `pending` in the records is a map
classification, not proof that nonzero retired payload remains.

Run from the Rust workspace:

```bash
cargo test -p revault_lockbox_api --release --features external-source --lib \
  mixed_file_aging_preserves_contents_and_bounds_physical_growth -- --ignored --nocapture
```

Set `REVAULT_CANDIDATE_TRACE_ARENA=1` for the allocator's diagnostic append records
and operation indices. Those records explain available/aligned space immediately
before an allocation-map extension. The trace only exists in the test-only
candidate; it is not a production logging or environment-variable interface.

Retain this control while reviewing map placement and metadata allocation in the
[whole-format comparison](../../archive_v4_evaluation.md#whole-format-eligibility-review).
Do not add periodic compaction to make the aging assertion pass. Public record,
access, source-handle and full platform qualification still remain.


## Diagnosis and comparison limits

At operation 421, immediately before the map extension, 299,836 bytes remained
reusable in eleven ranges. The largest individually aligned span was 123,672
bytes; even joining adjacent free/pending records gives only 124,077 bytes. Both
are below the 131,072-byte map requirement. The allocator therefore appended a new
map arena. After publication, the prior map becomes reusable; final free space
increases by 131,072 bytes. This is classified alignment/fragmentation headroom,
not an ownership gap or retained deleted payload. A mere free-record coalescing
change would not fix this particular allocation decision.

The strict first-100 stability hypothesis is stronger than the plan's general
requirement to explain all growth. Its retained failure must not be described as
proof of an unbounded leak, nor hidden behind a green ordinary test suite. No
automatic compaction, budget increase or allocator-policy change was introduced.
A future layout must account for metadata reserve placement and transient headroom.

Supplemental whole-format review evidence records the exact compacted small-file
accounting in `small-accounting.json` and rechecks the two known native failures
with `native-block-layout,external-source`. Truncated-tail recovery still reports
zero intact files where the test requires three; signed-plaintext neighbour recovery
still reports two partial files instead of one. No checks or expected counts were
weakened. These are existing architecture blockers, separate from candidate aging.

Ordinary format regression checks pass: 142 tests, with five explicit/manual
qualifications ignored. Strict native/external-source all-target Clippy passes.
The explicitly invoked mixed-aging stability check and the two invoked native
recovery regressions **fail as recorded above**. These results must be reported
separately; the ordinary suite is not a full candidate qualification.

Post-hook validation also passed 142 ordinary format tests (five ignored) and
strict Clippy. The recorded explicit qualification failures remain unchanged.
