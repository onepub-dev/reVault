# Direct raw cleartext final-page serialization trial

Base: 9229f2fc, after the completed temporary import memory diagnostic and exact
runtime restoration. Latest quota: 49% used, 51% remaining; stop below 10% or
above 99% remaining. No release.

This distinct fast path serializes a raw cleartext page directly into its final
allocation: existing 96-byte header, 32-byte body checksum, 16-byte raw-body header,
object stream and zero padding. It applies only when the existing cleartext
decision is true AND body compression is disabled by the original predicate.
Encrypted and compressed page paths keep existing codec/AEAD behavior. Cleartext
KeyDirectory classification and mixed-object refusal remain in the shared check.
Checked complete size/u32 conversion and exact unpadded extent are enforced before
allocation. Shared object serializer and page-header writer preserve the wire
definition. ZeroizingBytes owns partial plaintext through every error/unwind;
only successful completion transfers the final Vec. Checksums cover the same
body/header bytes and padding remains zero. No cache/worker/format policy changes.

Prior direct-body trial was rejected by raw maximum-RSS guards despite speed
benefits. This candidate additionally removes body and stored-body intermediates
on the eligible path; it is not a selective promotion of the rejected body path.
Source ownership supports reduced overlap, but does not prove the cause of prior
raw peaks. Fresh fixed measurements must establish useful memory and timing.

## Fixed comparison

Reuse the established public-add harness and pinned compressible/random 1 GiB
sources. Four cohorts: each source with existing-archive single-file Interactive
add (8 MiB command-local cache) and containing-directory BulkImport with exact
include filtering. Plaintext unsigned Zstd level 3, default padding, Auto write
workers, CPU 0–4 affinity. Baseline is preserved normal CLI 08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2,
whose runtime matches this checkpoint. Preserve new release-mode development
binary, original build output and exact source patch/manifests separately.

Exactly 112 timed public add processes: four cohorts × two variants × fourteen
rounds. First two rounds per variant/cohort are excluded preparations; twelve
measured rounds remain, alternating variant order. Fresh archive creation is
outside timing. Both variants use identical public create/add/doctor/list/extract
workflow. Independently compare every persisted output byte, assert exact file
membership and stored output mode 0600, record archive size/hash, and drain output
outside the timed add. Reject failed commands, incomplete measurements, changed
identities or verification failures. No outcome-dependent extra samples.

Use the existing six adjacent two-round paired blocks and 10,000 deterministic
seeded bootstrap draws for mean log(new/old) elapsed ratios; report exp(mean) and
95% percentile interval. Freeze the exact existing stats helper/analysis source
and seed derivation before timing. Report even/odd round views separately.
RSS statistics are per-cohort medians and maxima in bytes from GNU time.

Retain only if ALL four aggregate elapsed upper 95% bounds are <= 1.05 AND ALL
four new/old maximum-RSS ratios are <= 1.05 AND the named compressible Interactive
new/old median-RSS ratio is <= 0.95. The absolute 100 MB peak target remains a
separate open requirement. Other gains cannot replace the named usefulness gate;
stratum uncertainty must be reported even if aggregate gates pass.

Use the existing clean isolated environment, short owned agent directories,
cold-source admission, 24 GiB aggregate artifact/headroom budget including all
retained prior contexts and new binary, bounded subprocesses/whole campaign and
owned-agent cleanup. No test/build/runtime changes overlap timing. Freeze runner,
protocol, analyzer, source, helper and binary hashes before the one campaign.

If rejected restore only this final-page serialization change, preserve all evidence and run
no qualification suite/WASM for it. If retained run focused exact serialization,
page/protection arithmetic/roundtrip, secure payload and public lifecycle checks;
then refresh the tracked portable WASM artifact/lifecycle as appropriate. Formatting
belongs to the final tracked commit hook, after validation. Broad qualification
remains separately bounded; this trial establishes no release compatibility claim.

Seed labels are exactly case/workflow/aggregate and case/workflow/stratum0 or
stratum1, passed to unchanged seedFor and Seeded in the hash-pinned
`summarize-adaptive-workers.dart`. The frozen analyzer records each resulting
seed and uses percentile .025/.975 of 10,000 six-block resamples.
