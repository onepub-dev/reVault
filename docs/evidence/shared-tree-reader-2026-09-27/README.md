# Authenticated shared-control overflow reader

Date: 2026-09-27. Based on `d6f895fd`; the containing implementation commit
connects a private root manifest to the existing authenticated index and shared
physical ownership graph. It does not select a production archive format.

## Connected authority and ownership

A distinct 64-byte `RV4TRE01` private manifest contains a mirrored index reference
(two offsets, stored length and digest). The existing private envelope, owner/MAC
publication selection and idle preparation checks authenticate that manifest before
any tree records are exposed. The index retains its existing bounded-page encoding,
key domain, archive/mode binding, ordered traversal and stored-byte verification.
This is an alternative private body; existing dense inline images are unchanged.

Opening walks every reachable index page and its authenticated ownership records.
Namespace zero is reserved for free, pending and payload allocation records; raw
application records use other namespaces. The complete graph checks both copies
of each descendant against control slots, payload, other descendants and vacancies.
Each node copy starts on a separate 64 KiB region. For short unpadded nodes, the
rest of that allocation is derived free space and must be zero; it cannot hide
another payload or metadata record. Missing or duplicate ownership is refused.

The reader exposes random lookup and streaming ordered records only after the
complete ownership walk succeeds. Callers retain the same stable snapshot/read
lock and stage visitor output until success. A missing selected child pair is
fatal; the reader does not scan for an older tree or expose partial membership.
Payload claims establish allocation ownership, not typed payload correctness.
Record-specific validation and payload verification remain the adapter's job.

## Scope and remaining integration

The persisted fixtures contain 2,048 records with 128-byte values, exceeding the
64 KiB decoded inline catalogue. They are raw index records, not a claim that
2,048 public filesystem entries or full variable/form semantics are integrated.
The public variable limit remains 1 MiB; the index's 49,152-byte per-value limit
still requires authenticated segmented values with appropriate secure memory.

Current limits are 4,096 reachable page pairs, 4,096 allocation records and the
existing 8,192 graph claims. The existing index supplies its entry, node and height
bounds. These are experimental admission limits, not accepted product-capacity
reductions. Records are visited without collecting the complete record set, but
no CPU/RSS acceptance claim follows without measurements.

Only the reader is connected. Fresh persisted test fixtures use the existing
index builder and shared initializer with a test placement backend. Public image
creation, inline/overflow transitions, mutation, journal overflow, interrupted
retirement, typed records, credentials and public API installation still require
integration. Compression-mode tests exercise the private manifest; index pages
retain their existing encoding. No codec or integrity policy changes are made.

## Validation scope

Internal storage tests are necessary because no public CLI constructs this layout.
The mode matrix covers encryption, owner signing, compression and padding on/off.
It checks exact record values and ordered traversal, random lookups and absence,
loss of either complete control bank, and separate loss of every page copy.
Losing both copies of a selected child fails opening. Wrong keys/owner pins,
nonzero unpadded allocation tails, malformed allocation records, control/payload
aliases and unowned appended bytes are refused. Valid free, pending and payload
records are accepted; dirty free or pending bytes fail normal opening. Test
sources remain byte-for-byte unchanged by reads.

The [three focused tests](tests.log), [202-test format regression suite](format-tests.log)
(five explicit probes ignored), and [strict Clippy](clippy.log) pass.
