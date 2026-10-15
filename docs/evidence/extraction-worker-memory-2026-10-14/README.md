# Many-file extraction resource correction

Status: release blocked pending correction and qualification. This is a required
release resource fix, not a restart of general performance optimization.

## Failed frozen comparison

Candidate `c17c21128730f66e5b8a4bed4ddd6c3b101f0555` used CLI SHA-256
`83da86388d06ff50f8bb9d102f7cba8c09b7fa298f6c875caf884ebcc43dd1a7`
and 1,747-input source manifest
`388d35b4a0ff6f8a986de3ba0d24f58160d49dffa3890664f42215fe8cd745a1`.
The fixed comparison stopped once at schedule sequence 528, during the candidate
plaintext 10,000-file warm-up write lifecycle. It retained 528 completed
observations and an incomplete terminal record. These observations do not qualify
the planned comparison; no statistical pass is claimed.

The subsequent independent extraction used to verify the write exited successfully
but reached **148,746,240 bytes of CLI peak RSS**, above the frozen decimal
100,000,000-byte hard limit. GNU time recorded 145,260 KiB and 1.63 seconds.
This command was outside the write timing endpoint but inside the all-command
resource gate. The resource guard stopped verification before its byte comparison;
`operation_verified:false` therefore does not establish a content mismatch.
The owned helper high-water mark was 8,634,368 bytes. The CLI alone establishes
the failure, without relying on a summed or sampled process-tree estimate.

Public archive close and agent stop completed, service cleanup was verified, and
the candidate binary and source manifests remained unchanged. The original
campaign directory and failed command records are preserved. No sample was retried
or extended and no full campaign restart was authorized by this correction.

The earlier six-case screen remains a scoped positive for its smaller entry-count
workloads. Its maximum conditional bound of 97,619,968 bytes does not override the
observed failure on this additional required workload.

## Source correction

Parallel extraction previously called general `Lockbox::try_clone` for each of
up to four workers. That deep-copied the complete staged archive state, including
the path map, TOC tree and leaf entries, record-reference counts, and unrelated
variable/form/free-space metadata. The supplied job already borrows its entry
references from the parent's authenticated plan.

A private extraction-only wrapper now constructs its data reader without those
metadata collections. It retains protected content-key ownership, storage identity,
format and archive identity, relevant control/poison state, and the existing page
cache clone, including shared cached pages and uncommitted dirty data. It starts
with an independent empty decoded-frame cache and does not retain an owner signing
key. No general Lockbox or Deref API exposes the incomplete metadata view.

Worker count, corpus, extraction policy, frame digests, page validation, cache
retirement and stored wire encodings remain unchanged. The constructor rejects
pending small-file state, matching the existing parallel admission condition.
The removal of unnecessary copies is established by source review; the resulting
peak RSS still requires measurement and is not inferred from this change alone.

## Qualification

Focused tests cover empty worker metadata, uncommitted large-file data, unchanged
parent state, pending-small-file refusal, a changed frame digest with valid lengths,
existing cache retirement, and persisted plaintext and encrypted owner-signed
parallel extraction with exact bytes. Existing permission and symlink tests remain
in the selected module.

The first focused compile attempt stopped before running tests because two new
assertions used a nonexistent full-file API. Both test-only calls were corrected
to the existing `read_file_range` API; the failed attempt is retained. All 10 focused extraction tests pass (724 unrelated tests filtered out), and
all-target Clippy passes with warnings denied. Both source manifests are unchanged
through each successful run. [Curated command/results and artifact hashes](results.json)
retain the failed compile attempt without copying raw output.

A separate bounded resource screen will execute one retained
10,000-file lifecycle for each of plaintext and encrypted owner-signed mode, with
fresh source/binary identities, setup and helper accounting, independent bytes and
modes, and verified cleanup. It will make no timing comparison claim. The full
release comparison, immutable replacement candidate and platform gates remain
separate requirements.

The earlier c17 six-case raw evidence was relocated with a same-filesystem rename
into ignored release-preparation storage. The [relocation inventory](raw-evidence-relocation.json)
records every unchanged file hash; this does not replace or modify the failed comparison.
