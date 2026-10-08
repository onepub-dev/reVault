# Dense payload dispatch and authenticated recovery

Date: 2026-10-02. Uncommitted extension of the
[vacant-payload reuse checkpoint](../typed-payload-reuse-2026-10-02/README.md).
The [seven-file source overlay](source.sha256) supersedes earlier hashes for
those paths. All seven matched the worktree when this evidence was copied.

Bounded file addition, replacement and removal now dispatch from either dense
or tree metadata. Small results return to dense metadata after authenticated
publication; oversized results retain the tree. A read-only admission check
validates both dense node count and private-envelope capacity before relocation.
Post-publication cleanup errors remain errors. Relocation places the private
root pair beyond the payload end so the existing retirement proof remains
strict. Dense-base overflow aborts use their explicit recovery route.

## Validation

- [Focused lifecycle and interruption tests](tests.log): 2 passed; 1,620
  operation/prefix/persistence fault cases across four modes. Recovery selects a
  complete old or new file set and repeated recovery preserves that selection.
- [Capacity refusal test](capacity-tests.log): 1 passed; node-count and envelope
  refusal both preserve bytes without relocation.
- [Overflow preparation test](overflow-tests.log): 1 passed; 192 sampled faults
  across dense/tree starting layouts and two modes with 12 MiB payloads, with
  104 linked-arena observations. This large case is sampled, not exhaustive.
- [Final format regression](format-tests.log): 240 passed, 6 manual probes ignored.
- [Strict Clippy](clippy.log): passed.

The economical runner used local release external-source tests and strict
external-source library/test/benchmark Clippy. No production release occurred.

## Bounded aging observation

The lifecycle test completes 32 mixed payload cycles in all 16 modes, preserves
unchanged-repeat bytes and verifies reopen after loss of either control bank.
Completed archive sizes after cycle 16 stay within the maximum completed size
observed during the first 16 cycles.

| Modes | First-16 completed maximum bytes | Final bytes |
| --- | ---: | ---: |
| 0, 2 | 393,216 | 205,096 |
| 1, 3 | 395,280 | 395,280 |
| 4, 6 | 262,193 | 262,171 |
| 5, 7 | 262,277 | 262,227 |
| 8–15 | 720,896 | 524,288 |

These are fixture observations, not transient peaks, universal size bounds or
the full 1,000-cycle whole-format aging gate. Modes cross encryption, signing,
compression and padding (bits 1, 2, 4 and 8). This completes the scoped candidate
dense/tree payload dispatch and recovery tranche. Public streaming, variables,
forms, access semantics, migration and native activation remain unqualified.
The two retained native recovery failures are not resolved by this test-only
candidate. Controlled read performance is the next experiment.
