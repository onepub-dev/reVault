# Authenticated vacant-payload reuse

Date: 2026-10-02. Uncommitted extension of
[bounded typed payload mutation](../typed-payload-2026-10-02/README.md).
The [four-file source overlay](source.sha256) supersedes earlier hashes for its
listed paths; earlier snapshots and rejected trials retain their own scope.

Changed packs use best-fit spans wholly inside one authenticated vacant claim,
excluding metadata and journal arena reservations. Reservations retain original
FREE/PENDING namespace and base, with a combined 2,048-reservation cap. All reused
bytes were verified zero; records and sorted pack indices are rebound before
writes. No fitting span means append. Whole retired packs remain subject to
post-publication erasure. No allocator encoding, integrity or padding rule changed.

## Validation

- [Focused reuse tests](tests.log): 7 passed, including 912 update, 888 removal
  and 792 reused-payload-write fault cases in bounded four-mode matrices.
- [Large catalogue/payload lifecycle](large-catalogue.log): 1 passed across all
  16 modes.
- [12 MiB overflow-payload test](overflow.log): 1 passed, 96 sampled crash cases
  in two modes, with 56 linked-arena observations. This is explicitly sampled,
  not an exhaustive large-write failure matrix.
- [Final format suite](format-tests.log): 237 passed, 6 manual probes ignored.
- [Strict Clippy](clippy.log): passed.

The economical runner used release external-source tests and strict
external-source library/test/benchmark Clippy. Earlier small reused-write tests
exercise every observed mutation boundary; the final suite includes them.

## Bounded aging and resource observation

All 16 modes run 32 mixed update/removal cycles, asserting stable completed size
after cycle 16. The modes are the Cartesian product of encryption, owner signing,
compression and size padding on/off (bits 1, 2, 4 and 8 respectively).

The [serial existing-binary observation](resource.log) began at 327,680 bytes:

| Modes | Observed stable completed bytes | Reused pack count |
| --- | ---: | ---: |
| 0, 2 | 917,504 | 57 |
| 1, 3 | 786,432 | 59 |
| 4–7 | 720,896 | 59 |
| 8–15 | 983,040 | 55 |

Some earlier focused fresh-fixture observations had different final sizes; both
passed stabilization. These numbers are observed fixtures, NOT universal per-mode
bounds, transient peaks or complete 1,000-cycle payload-aging qualification.

User CPU was 7.90 seconds, system CPU 0.56 seconds, elapsed 8.47 seconds and
process peak RSS 80,904 KB, including fixture construction and verification.
Load was 2.42/2.25/1.76, with HMB collectors/lockbox agent and no other Cargo process
reported. No paired speedup or incremental memory gate is claimed.

The adapter remains bounded and Tree-only. Dense payload dispatch, safe return
after reused payload placement, public streaming, variables/forms and access
semantics remain. Native activation and full qualification are not implied.
