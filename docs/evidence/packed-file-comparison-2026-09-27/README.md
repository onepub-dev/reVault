# Independent physical packing: performance and resource results

Sources: packed candidate C `e23b4d33`, previous unpacked C `a6b67546`, unchanged
common runner `4c236915`. Both use the corrected encoder and the same production
bulk-buffer wiping abstraction. Executable hashes, source inventories, every
observation and complete summaries are retained. This is a file-only prototype
comparison, not selection or activation of a new format.

## Fixed paired batch

The predeclared eight cases repeat exactly once: 30 observations after three
warm-ups, rotating ZIP/old C/new C ordering, one worker on CPU 2, warm OS cache,
fresh processes and handles, default padding. No owned builds or other tests
overlap measurements. Each observation independently verifies persisted content
against source bytes outside timing. Source and protocol were unchanged during
the batch. Intervals are paired-bootstrap 95% with the existing fixed seed and
10,000 resamples. Wider intervals are retained rather than resampled away.

| Case | Packed / unpacked elapsed [95%] | CPU ratio | Packed / ZIP elapsed |
| --- | --- | --- | --- |
| compressed64 | 1.068 [1.022, 1.115] | 1.067 | 8.004 |
| compressed256 | 1.072 [1.032, 1.114] | 1.072 | 4.020 |
| raw64 | 1.037 [1.002, 1.075] | 1.037 | 3.767 |
| small | 0.767 [0.715, 0.820] | 0.780 | 23.399 |
| compressed-signed | 0.908 [0.881, 0.934] | 0.909 | 9.019 |
| compressed-encrypted-signed | 0.969 [0.936, 1.000] | 0.969 | 5.286 |
| raw-range | 1.060 [1.043, 1.078] | 1.059 | 7.645 |
| raw-encrypted-range | 2.616 [2.557, 2.669] | 2.613 | 25.392 |

Large files are 8 MiB; `small` is 512 × 4 KiB. `mixed` at this small size uses the
compressible segment of the frozen generator, not an incompressible-small-file
claim. Signed/encrypted versus unsigned ZIP ratios describe protection cost, not
equivalent security. ZIP range reads omit full-entry CRC; full-entry reads include
it. No A3 pass or contemporaneous same-protection C/A A4 result is established.

The small archive falls from **34,734,080 to 1,703,936 bytes**, compared with
237,078 bytes for ZIP: 95.1% smaller than unpacked C, but still **7.19× ZIP** and
well above the proposed 1.50× target. Fixed publication/journal/map space, mirrored
metadata and padding all remain part of the result. Do not subtract them to claim
a passing archive size.

Encrypted raw-range read time itself is unchanged (0.998×), but open increases
3.552×, making the total 2.616×. The new packed semantic audit reads and decrypts
separate padding across packs at open. This introduces work proportional to
unrelated padding. The implementation retains those checks; the result identifies
an unresolved layout/open-contract cost, not permission to skip validation.

## Descriptive scale probes

These are one creation and one fresh-process whole-tree read per case, without
repeated observations or controlled cold-cache treatment. They use the same host,
CPU 2 and one worker, with no owned builds/tests overlapping. Each verifies all
persisted bytes independently after the timed operation. They are useful bounded
resource observations, not statistical qualification. Peak RSS is worker-lifetime
RSS captured before independent verification; the recorded baseline is the process
before the operation, not a separately measured empty-archive-open baseline.

| Corpus / mode | Create elapsed / CPU | Create peak RSS | Open + read elapsed / CPU | Read peak RSS | Archive bytes |
| --- | --- | --- | --- | --- | --- |
| 100k × 128 B, raw plaintext | 2.259 / 2.163 s | 66.25 MiB | 41.185 / 41.179 s | 35.16 MiB | 82,837,504 |
| 100k × 128 B, compressed encrypted-signed | 4.915 / 3.249 s | 67.37 MiB | 60.719 / 60.708 s | 35.37 MiB | 85,655,552 |
| 1 GiB raw plaintext | 11.773 / 3.058 s | 14.47 MiB | 0.836 / 0.836 s | 10.11 MiB | 1,082,392,576 |

The 100k cases contain only 12.8 MB of logical data. Open takes 0.247/0.300 s;
the remaining 40.938/60.419 s is selected-path file reading. Together with the
[previous metadata profiles](../ordered-file-reads-2026-09-27/README.md), this points
to repeated authenticated index lookup/decoding as a major many-file cost. The
current adapter retains no decoded metadata cache and exposes per-path reads;
a bulk extraction API and a bounded cache are separate architecture experiments,
not substituted results for this workload. RSS is promising on these exact cases,
but does not qualify every parser length/count combination or the full A5 matrix.

## Reproduction

Build and freeze the test executable and protocol adapters at the named sources.
The driver source is `rust/revault_lockbox_api/benches/evaluation/candidate_driver.rs`.
Compile old/new drivers with `REVAULT_CANDIDATE_FROZEN_TEST_BINARY` pointing to
their respective test executables. For each entry in `batch.json`, run:

```sh
REVAULT_CANDIDATE_UNIT=262144 taskset -c 2 RUNNER run NEW_ROOT FILES BYTES \
  CORPUS CODEC MODE 30 ACCESS 1 default PACKED_DRIVER UNPACKED_DRIVER
RUNNER summarize NEW_ROOT/samples.jsonl
```

Use each case's unit and arguments from the retained batch. The scale corpus
formula, file naming and protocol are in `resources/protocol.json`; compare generated
files against the retained inventories before running `PACKED_DRIVER create ROOT
lockbox` and `PACKED_DRIVER sample ROOT lockbox stream 1`. The GB corpus is the same
retained common-runner random corpus used in the earlier creation probe.

## Decision implications

Keep the bounded production-A writer and mirror fixes. Do not activate C or claim
ZIP parity based on packing's substantial space reduction. C's independent recovery
and whole-pack retirement are useful correctness results, while the proposed read
and small-file space gates remain failed. Normal-open padding work, metadata access,
fixed control overhead and full public record/access semantics must be considered
together before selection. Any budget/guarantee change needs explicit review.

The next work is the remaining record/access and compaction comparison plus
expanded lifecycle qualification, with these failures kept visible. Do not restart
an unlimited sequence of buffer/encoder changes to improve one timing column.
