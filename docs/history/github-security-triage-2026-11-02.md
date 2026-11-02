# GitHub security alert triage — November 2, 2026

[Issue #347](https://github.com/onepub-dev/reVault/issues/347) reviewed all 339
open GitHub security alerts against source `88a55a428bc35e040a4be7840ae043c5897ce823`.
The inventory comprised 319 CodeQL findings and 20 Dependabot findings, rather
than 339 independent defects plus another 20 vulnerabilities.

## Results

| Finding | Reviewed | Disposition |
| --- | ---: | --- |
| Hard-coded cryptographic value | 172 | 72 synthetic test/benchmark fixtures; 100 false positives |
| Cleartext logging | 125 | Test diagnostics, non-output collection operations, intended CLI metadata or explicit credential retrieval |
| Invalid pointer access | 18 | Null checks or asserted constructor success prevent the reported null path |
| Path injection | 4 | Configured store root plus validated hexadecimal identifiers |
| Historical dependency advisories | 20 | Dependencies not used by current workspace/release inputs |

No new implementation defect was established in these reported flows. Each
dismissal was individually applied and read back from GitHub; a final query
returned zero open CodeQL and zero open Dependabot alerts. This is an assessment
of these findings, not a claim that the entire product is free of vulnerabilities.
No query, workflow, alert category or repository-wide path exclusion was added.

The [per-alert evidence](../evidence/security-triage-2026-11-02/alerts.json)
records rule, location, source commit/blob, dismissal reason and explanation.
CodeQL analysis `2073343758` used version `2.27.3` at the reviewed source.
Its 336 total SARIF results included findings already dismissed before this
review; 319 were open. Older alert creation dates did not imply a stale scan.

## Findings requiring contextual review

Cryptographic-value alerts included permission masks, filenames, enum
discriminants, allocation lengths and zero-initialized read buffers. Encrypted
page branches generate random nonces; zero nonce headers belong to explicitly
cleartext pages. The reviewed nonce initializer is overwritten by successful
random generation before encryption. Synthetic corruption tests deliberately
use fixed values. None of these dismissals permits fixed production keys or
encrypted-payload nonce reuse.

Pointer findings flowed from null-returning constructors into guarded uses.
Lazy `bool::then` closures execute only after the non-null condition succeeds.
The file-opening API rejects missing contact/Vault handles for those credential
modes; password-mode tests intentionally pass null and never enter those
branches. Test handle constructors assert success before dereferencing. The
exchange-session path converts null to an error before reaching safe vector
iteration. These conclusions retain the documented FFI requirement that
non-null handles are live and of the expected type.

Exchange-store paths use a server-configured root and IDs restricted to exactly
64 lowercase hexadecimal characters. Request and replica entry points validate
IDs; startup validates persisted IDs against filenames; expiry cleanup uses
only that validated index. The reported remote-input flow through Axum state
does not establish remote control of the configured storage root.

Logging findings included `Vec::remove`, `Vec::insert` and cache collection
operations, which are not logging sinks. Other reported outputs contain counts,
paths, names, descriptions, public IDs or revisions. Secret form inspection
redacts values. Secret field/variable retrieval requires explicit `--secret`;
its stdout output is the requested operation. Setup deliberately displays a
newly generated Vault passphrase and requires confirmation that it was stored;
existing, environment-supplied and manually entered passphrases are not printed
by that branch. Exchange output includes a shareable invitation and public
verification fingerprint, not private management capabilities.

## Historical lockfiles

The 20 dependency findings represent ten advisories repeated in two immutable
October 10 performance-evidence lockfiles:

- `docs/evidence/journal-buffer-2026-10-10/raw/source/Cargo.lock`
- `docs/evidence/wiped-read-buffer-2026-10-10/raw/Cargo.lock`

These preserve the exact graphs used for historical measurements and are outside
current workspace/release inputs. They retain zip 8.6.0 and rustls 0.23.44;
the reviewed active `rust/Cargo.lock` uses zip 9.0.5 and rustls 0.23.46. Each alert
was dismissed as `not_used` with this scope, without rewriting historical evidence.
Reusing those snapshots as executable/build inputs requires a fresh dependency
security review; this disposition does not make their old dependencies safe.

Current ZIP usage is distribution/package tooling and Lockbox development
benchmarks, not the current Lockbox archive runtime format.

## Validation and follow-up

This change records source/data-flow review and GitHub state verification; it
does not alter runtime code. No new runtime tests or broad reruns were necessary.
Existing CLI tests cover secret retrieval refusal without `--secret`, and FFI
tests assert successful construction before handle use. The independent release
candidate continues its required tests.

Keep scanning enabled and review new or reopened findings individually. If
source changes invalidate a recorded guard or data-flow assumption, reopen the
affected alert. The immutable per-alert source blob makes that comparison
possible. Do not copy these dismissals to another source revision without review.
