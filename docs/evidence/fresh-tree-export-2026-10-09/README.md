# Direct packed-C to typed-tree fresh export

Date: 2026-10-09 (Australia/Melbourne). This is an experimental private adapter,
not public format activation, migration, protocol selection or release readiness.
It follows the [October 2 large-file evidence](../typed-tree-64m-2026-10-02/README.md),
whose raw 64 MiB fresh exporter failed dense-catalogue admission and therefore
used a different authenticated update construction route.

## Implementation and retained contracts

The cost model and fresh dense exporter now share an audited repacking stage
with the direct tree exporter. That stage fully audits source logical contents,
reconstructs each fragment's complete descriptor binding, verifies each stored
fragment digest, and emits bounded physical packs. Its in-memory descriptor DTO
can become typed records directly. The new tree path never constructs the
64 KiB dense body or the cost model's 1 MiB serialized intermediate. Existing
cost reporting/dense export keep their prior body, slot and admission rules.

The caller must retain a stable source snapshot/read lock and own an empty
destination. The exporter authenticates the selected source commitment before
writing, checks it again before destination publication, then again after
independently reopening and fully verifying destination bytes and ownership.
It validates placement against the actual destination length, padding, exact
source file IDs/paths/lengths and typed metadata. Caller metadata may add directory
or symlink semantics already supported by this adapter, but cannot add, omit,
rename or change the length of an authenticated source file. Access-slot
translation remains refused. Nonempty destination bytes are untouched.

Returned failures erase, synchronize, truncate and synchronize fresh output.
A failed cleanup returns both the original failure and cleanup error instead
of claiming empty output. This is returned-error cleanup, not process-death
installation/recovery qualification. No in-place source mutation occurs.

Payload copy buffers remain bounded by existing fragment/pack sizes. Metadata
is held as bounded records rather than one serialized body. Existing experimental
limits remain: 1,024 source files, 4,096 fragments, typed-node/path budgets and
the shared reader's ownership/page limits. These are not accepted public
capacity limits, and this change does not establish the 100,000-file product gate.

## Validation before formatting

- Seven routine focused tests pass, including **all 16 modes** with 512 files
  of 4 KiB and long canonical paths. Each independently reopened tree must exceed
  dense metadata capacity; exact metadata, IDs, lengths, digests and every byte
  are checked, and source archive bytes remain unchanged.
- **93** one-shot destination mutation failures across four representative
  modes preserve the source and leave empty fresh output. These use 256 files
  with valid 220-character suffixes and 17-byte contents, beyond dense capacity.
- Refusals cover file metadata omission/addition/rename/length mismatch, wrong
  owner, corrupted source, nonempty destination and 1,025 empty source files.
- A real synthetic password-slot directory is stored through the authenticated
  allocation transaction's key-record route and independently read through the
  authenticated index before access-translation refusal is tested. There is no
  public candidate access writer; this internal fixture exception is explicit.
- Authenticated generation-only source successors are injected after payload
  copy and during destination publication. Both late commitment checks reject
  the changed source and discard output while preserving the successor.
- A synthetic truncate failure proves compound cleanup errors are reported;
  retained output remains zero-filled and is not claimed empty.
- Release format suite: **248 passed, 8 ignored**. Strict library/tests/benches
  Clippy: **passed**. Rust is pinned to 1.88.0, external-source enabled.

These tests deliberately use private APIs because the public CLI does not
create this experimental layout. They are not public CLI E2E coverage.

## Separate file-backed 64 MiB functional probe

The explicit ignored probe streams source generation, uses file-backed source
and destination, drops/reopens the destination with a separate file handle,
compares every logical byte incrementally, then audits the retained source.
No 64 MiB plaintext Vec is created. Each case has exactly 1,024 fragments.

| Mode | Source bytes | Tree bytes | Dense catalogue fits |
| --- | ---: | ---: | --- |
| Raw plaintext, unsigned, padded | 68,419,584 | 68,026,368 | No |
| Compressed plaintext, signed, padded | 68,419,584 | 1,572,864 | Yes |
| Compressed protected, signed, padded | 68,419,584 | 1,572,864 | Yes |

The raw case directly reproduces the earlier capacity shape without its dense
intermediate. Compressed fragments share fewer physical packs, so their metadata
can fit dense capacity despite the same logical length/fragment count; the
all-mode long-path fixture separately exceeds dense capacity in every mode.

The test binary ran once successfully under `/usr/bin/time -v`: **8.79 seconds**
wall time and **15,972 KiB** whole-process peak RSS, including source creation,
exports, reopens and all assertions. Compilation is retained separately. This
is a descriptive functional probe, not isolated export CPU, incremental memory,
a statistical performance comparison, A3/A4/A5 qualification or a ZIP claim.

## Retained development failures and next checks

The first test build lacked two required Storage methods and referenced an
unavailable temporary-directory crate; the fixture now uses the repository's
nonce/PID directory pattern. The next run rejected invalid overlong path
components; fixtures were corrected without relaxing canonical-path validation.
The first large raw fixture incorrectly supplied a signer for an unsigned mode;
the final probe uses mode-aware signing. A later probe completed the raw case
then rejected an incorrect assumption that compressed metadata must also exceed
dense capacity; the final assertions distinguish these physical pack shapes.
Failure logs are retained alongside successful logs; none is a production
decoder/exporter failure or hidden comparative performance trial.

The source manifests retain both pre-format and post-format identities. Commit
`14d1d195` passed the tracked hook (four Rust files formatted), then the full
release format suite again passed **248 tests, 8 ignored**, the explicit three-mode
streaming probe passed again (8.81 s functional verification), and strict Clippy
passed. These are affected post-format checks, not a repeated statistical trial. Native recovery,
variables/forms and secure segmented values, public APIs, compatibility,
dependency publication, architecture choice and full resource qualification
remain outstanding. Earlier timing evidence is not relabeled as this exporter.


The two preserved native failures were reproduced independently after this
checkpoint with `external-source,native-block-layout` in release mode. Truncated
selected publication still recovers 0 rather than the required 3 intact files
(`api_tests.rs:3364`). The multiframe writer/recovery test still reports 2 rather
than 1 incomplete files in signed plaintext/raw/default-padding mode
(`block_frame_tests.rs:317`). No assertions or owner/membership checks were
weakened; these remain separate architecture/recovery blockers, not exporter
regressions. Exact native logs are retained here.
