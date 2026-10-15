# Owned FileData page correction — 2026-10-14

Status: focused correctness and lint pass; resource qualification pending. No
release-ready claim. [Sanitized results](results.json) retain both the failed
screen and the single allocation diagnostic without credential or payload output.

The prior extraction-worker correction at `d0d7b35c2ce4` reduced unnecessary
metadata ownership, but its uninstrumented plaintext 10,000-file screen still
observed 100,524,032 bytes of CLI RSS, above the unchanged 100,000,000-byte limit.
The command succeeded; the resource guard stopped before independent verification.
No protected lifecycle ran. Cleanup and exact input identities passed. The earlier
c17 comparison failure remains preserved separately.

One isolated diagnostic then completed the same plaintext corpus, including bytes,
permissions and cleanup. Each worker observed a stored-page allocation of
7,190,584 bytes alongside 7,190,644 bytes of copied object storage at the same
decode boundary. These per-thread allocation observations are not simultaneous
process-wide peaks. Cache retirement completed normally. This evidence motivated
removing duplicate page ownership, rather than changing cache limits or workers.

The correction consumes the storage read buffer. Owned raw-body pages containing
only ordinary FileData objects retain one immutable, fully wiped backing owner and
validated payload ranges. Encrypted input releases ciphertext after authentication.
Ordinary object clones copy only that object's range, avoiding detached retention
of an entire page. Borrowed decoding, metadata, control, mixed, secure and whole-page
compression paths retain independent payload ownership. Shared validation preserves
the existing legacy envelope's accepted trailing-body behavior. No format, public
API, workload, cache-cap or worker-count change is made.

All 51 focused tests pass: page decoding 18, buffer wiping 3, page cache 20 and
parallel extraction 10. New coverage checks borrowed/owned equivalence, valid-envelope
inner corruption, fallback modes, detached clones and final-owner/error wiping.
The wipe observer exists only in tests and examines still-live storage after the
existing wipe operation. All-target Clippy with warnings denied passes. The test
source manifest was unchanged; tracked-hook formatting is applied only at commit.

The next gate is a fresh uninstrumented checkpoint build and bounded plaintext and
encrypted owner-signed resource qualification. A passing result would still not
qualify the incomplete statistical campaign or an immutable release candidate.
