# Compiler-matched typed read and aging qualification

Date: 2026-10-02. This is the authoritative October 2 retained-control read
batch, superseding the cross-compiler qualification limits of the
[initial batch](../typed-tree-read-2026-10-02/README.md). It keeps the validated
[single-traversal implementation](../typed-tree-single-pass-2026-10-02/README.md).

## Source and qualification

The exact repository [toolchain pin](rust-toolchain.toml) is now included in the
isolated snapshot. Candidate resource/driver and retained C/ZIP binaries all use
Rust 1.88.0, confirmed by compiler-version files and ELF comments. The
[core manifest](aging-core-source.sha256), [harness manifest](protected-harness-source.sha256)
and [binary manifest](pinned88-binaries.sha256) identify the measured artifacts.
Harness sources are preserved under [harness/](harness/) and their hashes match.

The pinned build passes [241 format tests](protected-pinned88-format-tests.log)
and [strict Clippy](protected-pinned88-clippy.log). Seven manual probes are
ignored in the normal suite: the prior six plus the new 1,000-cycle qualification
entry point, which was separately executed successfully below. No earlier test
was disabled. The preserved `protected-clippy.log` records seven existing-code
lint failures from the earlier mistaken Rust 1.94.1 isolated-library invocation;
unrelated code was not changed to satisfy the wrong compiler.

## Controlled reads

Each of six cases uses 30 paired observations after three warmups, alternating
ZIP/C/tree and tree/C/ZIP. One worker/pass, CPU 2, fresh process/handle and warm
OS cache are retained. Independent stream/range byte smokes pass. All source
inventories and archive/public-key hashes are checked before and after timing.
No task-owned tests/builds overlap timing; host context is retained. Raw samples,
summaries and source inventories are under [measurements/](measurements/).

`primary` means retained packed C; `other` means the pinned optimized typed
reader. Protected C roots lacked source files, so new control directories hold
exact copied C archives/public keys and the corresponding existing ZIP source
corpus. Independent C byte verification passed before timing. Original retained
controls were not modified or replaced with newly generated archives.

Protected cases use encryption plus owner signing. Synthetic already-unwrapped
key and supplied public-key setup are outside both readers' clocks; normal
mode-dependent integrity/owner verification remains inside open/read. They do
not measure password derivation or access-slot migration. ZIP has weaker
protection, so its protected-case ratios are descriptive only. ZIP range reads
omit full-entry CRC.

Ratios are tree/control elapsed time including normal open; lower is better.

| Case | Tree / ZIP, paired 95% interval | Tree / C, paired 95% interval | Median tree total | Median process peak RSS |
| --- | --- | --- | ---: | ---: |
| Small plaintext | 0.546 [0.537, 0.560] | 0.0247 [0.0244, 0.0253] | 4.768 ms | 7,744 KiB |
| Raw 8 MiB plaintext | 3.567 [3.515, 3.618] | 0.973 [0.968, 0.979] | 6.030 ms | 7,208 KiB |
| Compressed 8 MiB plaintext | 2.524 [2.480, 2.565] | 0.642 [0.635, 0.648] | 4.139 ms | 8,000 KiB |
| Raw 4 KiB midpoint range | 5.896 [5.738, 6.058] | 0.754 [0.739, 0.767] | 0.609 ms | 7,252 KiB |
| Small encrypted/signed | 0.831 [0.813, 0.845] | 0.0232 [0.0229, 0.0235] | 6.991 ms | 8,248 KiB |
| Compressed 8 MiB encrypted/signed | 3.192 [3.145, 3.256] | 0.587 [0.580, 0.598] | 5.102 ms | 8,452 KiB |

Small means 512 × 4 KiB mixed compressed bytes. Only the small plaintext subcase
passes the proposed comparable-protection ZIP target; the larger/range failures
remain. Whole-worker RSS is 1.51–1.58× C, higher than the earlier cross-compiler
observation. It is not incremental memory above empty-open. Raw-open tree/C is
1.037 [1.024, 1.051]; this is not a full A4 pass against production candidate A.

## 1,000-cycle typed-candidate aging

Four separately bounded groups cover all 16 modes, 1,000 mixed payload cycles
each: **16,000 cycles passed**. The unchanged 32-cycle test body is reused with
the existing first-16-cycle completed-size ceiling, exact-byte independent
reopens after updates/removals, byte-identical no-op checks and final recovery
after loss of either control bank. Logs are `typed-aging-group-0.log` through
`typed-aging-group-3.log`; all exit zero.

| Modes | First-16 completed maximum bytes | Final bytes |
| --- | ---: | ---: |
| 0, 2 | 393,216 | 332,152 |
| 1, 3 | 395,280 | 332,236 |
| 4, 6 | 262,193 | 262,193 |
| 5, 7 | 262,277 | 262,277 |
| 8–15 | 720,896 | 589,824 |

Group peak RSS is 78,492 / 78,160 / 78,376 / 78,712 KiB, including fixture setup
and repeated verification. These deterministic bounded-file observations pass
the typed-candidate workload; they do not qualify public whole-format aging,
streaming, variables/forms, access mutation, migration or native activation.
The two known native recovery failures remain outside this candidate work.

## Compiler-matched payload attribution

The frozen pinned reader also completed bounded verified raw/compressed profiles,
retained under [profiles/](profiles/) with commands, exit statuses, host/compiler
identity and binary hashes. Raw has 891 samples and compressed 606, with zero
lost samples and both exits zero. Raw sampled user CPU is 91.69% hardware
SHA-256 in the verified payload read path. Compressed includes copying (19.97%,
including decoder history/output), sequence decoding (15.51%), XXHash (15.02%)
and FSE-table construction (9.74%). This confirms the earlier diagnostic under
the correct compiler; it is not another fresh-handle timing experiment.

No further high-value duplicate application work was identified. Do not bypass
checksums, wiping, padding or verification-before-exposure to manufacture parity.
Further decoder work requires its own scoped implementation and evidence;
changing persisted integrity/codec contracts requires an architecture decision.
