# Bounded typed payload mutation

Date: 2026-10-02. Uncommitted experimental checkpoint after
[typed inline/tree transitions](../typed-transitions-2026-10-02/README.md).
The [seven-file source overlay](source.sha256) supersedes overlapping hashes
from prior checkpoints. No native-format activation or public API is implied.

## Connected behavior

Tree-only file addition, replacement and removal now stage changed packs under
the authenticated preparation journal. The writer validates expected base,
retired extents and rebound record shape before persistence; staged bytes are
read back and checked against their digests before publication. Retired whole
packs are erased only after selected publication is mirrored. Live neighbours
in affected packs are verified and copied as independently encoded fragments,
without recompression or changed protection/compression descriptors.

Sources are seekable and bounded, hashed twice before persistence. Replacements
retain identities and permissions; new experimental files use mode 0600.
Unchanged content is a byte-identical no-op. Changed sources and encoded staging
have explicit 64 MiB bounds and wipeable buffers. This checkpoint appends changed
payload packs; it is not an unrestricted streaming writer or a payload-aging pass.

## Validation

- [Focused tests](tests.log): 5 passed, including 900 removal interruption cases
  and 924 addition/replacement interruption cases in bounded four-mode matrices.
- All 16 modes exercise successful mutation, neighbour bytes, erasure, no-change,
  invalid-input and copy-loss paths. Stale/duplicate retirement claims refuse
  unchanged; changed sources refuse before persistence.
- A staged-byte readback check was added after the focused run; the
  [final format suite](format-tests.log) includes it: 233 passed, 6 manual probes
  ignored. It includes the earlier recovery and transition regressions.
- [Strict Clippy](clippy.log): passed.

The economical runner used release external-source library tests, then strict
external-source library/test/benchmark Clippy. No gates/assertions were relaxed.

## Descriptive resource observation

The [serial existing-binary run](resource.log) selected
`file_format::candidate_files::tests::tree_tests::mutation::typed_tree_payload_add_replace_no_change_and_invalid_plans_all_modes`.
Initial size was 327,680 bytes and changed logical content was 140,008 bytes.
Completed sizes were 786,432 bytes in modes 0–3, 655,360 in modes 4–7 and
851,968 in modes 8–15. These are completed sizes, not transient allocation peaks.

Process user CPU was 1.93 seconds, system CPU 0.53 seconds, elapsed 2.47 seconds
and peak RSS 81,356 KB, including setup/verification. Load was 1.78/2.12/1.70;
HMB collectors and a lockbox agent were present, with no other Cargo process
reported. This is not an isolated paired speedup, write-peak memory or ZIP gate.

Vacant payload reuse, mixed-update aging, public streaming, variables/forms,
access mutation and public/native activation remain outside this checkpoint.
Historical native recovery failures remain separate; no commit or deployment
occurred. Prior rejected trials and source-specific evidence are preserved.
