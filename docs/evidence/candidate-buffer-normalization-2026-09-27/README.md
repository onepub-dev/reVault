# Candidate bulk buffers use the existing secure memory abstraction

Source `a6b67546` reuses production `ZeroizingBytes` for C's bulk data and index
buffers, including spare capacity, input staging and bounded decoder scratch.
The existing volatile wiping implementation and key handling are unchanged. There
is no new unsafe code, crypto/codec change, skipped check or stored-byte change.
Small index keys/values retain their existing generic zeroizing wrappers.

This corrects the mismatch identified in the [CPU profiles](../ordered-file-reads-2026-09-27/README.md).
Both before and after formatting, all 113 format tests and all-targets
native/external-source Clippy pass, including wiping, fixed wire vectors,
corruption, publication and recovery tests. Logs are retained.

## Predeclared paired comparison

Repeat the same eight cases exactly once. Primary is frozen ordered C (`4c236915`),
other is normalized C (`a6b67546`), with ZIP as the third participant. Thirty paired
observations follow three warm-ups; ordering rotates; warm OS cache, fresh child
and handle, one worker on CPU 2, default padding. No builds or tests overlap timing.
All observations independently reopen and verify bytes against retained source
inventories. The source was clean and unchanged throughout the batch.

Ratios divide normalized C by previous ordered C except the last column. Elapsed
intervals are paired bootstrap 95%; CPU, RSS and other full summaries are retained.
Signed/encrypted versus unprotected ZIP ratios are cost context only. All existing
file-only prototype limitations remain: no public access/bootstrap integration,
small-file packing, mutation API, complete recovery or compaction.

| Case | Elapsed normalized / prior [95%] | CPU normalized / prior | Elapsed normalized / ZIP |
| --- | --- | --- | --- |
| compressed64 | 0.605 [0.604, 0.607] | 0.605 | 7.841 |
| compressed256 | 0.538 [0.536, 0.541] | 0.538 | 3.844 |
| raw64 | 0.590 [0.588, 0.592] | 0.590 | 3.754 |
| small | 0.639 [0.638, 0.640] | 0.639 | 28.967 |
| compressed-signed | 0.611 [0.611, 0.612] | 0.611 | 10.050 |
| compressed-encrypted-signed | 0.586 [0.584, 0.588] | 0.586 | 5.507 |
| raw-range | 0.688 [0.682, 0.696] | 0.689 | 7.347 |
| raw-encrypted-range | 0.739 [0.736, 0.743] | 0.739 | 11.061 |

The 8 MiB raw and 256 KiB compressed cases now take about 3.75× and 3.84× ZIP's
duration. This remains a failed A3 target. It is not a contemporaneous paired
C/A comparison, so do not infer an A4 pass from historical A numbers. Small-file
space is unchanged. Buffer normalization does not select or qualify a format.

Build the C test binary at the source above and follow the previous checkpoint's
paired-run commands. Compile the frozen primary adapter against
`/tmp/revault-candidate-ordered-resource` (or the rebuilt equivalent); use the
normalized binary for the runtime-selected other adapter. The batch, executable
hashes, compressed case/inventory/sample records and summaries are retained here.
The unchanged runner binary was built at `4c236915`; it records its own embedded
source hash and the actual executable hashes for both candidates.

## Next architecture work

Return to shared small-file packs and their mutation/recovery contract. Each
logical slice needs authenticated membership separate from physical pack identity.
Open must reject overlaps, gaps and incompatible descriptors for a shared physical
allocation. Removing/replacing one member must rewrite surviving members, publish
all references atomically, and reclaim the complete old pack; leaving deleted
logical bytes inside a retained neighbour's pack is unacceptable. Test faults and
recovery with exact physical accounting before interpreting packing speed/space.

Keep fixed control-space overhead separate from payload packing. No checksum,
padding, signing or durability guarantee has been relaxed to meet a timing target.
The proposed ZIP space/read gates still require either a qualifying implementation
or an explicit reviewed trade-off; the current evidence does not close them.
