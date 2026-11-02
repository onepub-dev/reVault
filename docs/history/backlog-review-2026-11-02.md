# Remaining issue review — November 2, 2026

This is a priority and overlap review of the open issue queue, with selected
source checks at main `c3ee581695259267f26f58205db2eeffdcbce559`. It is not a new
security audit or evidence that all old acceptance criteria have passed.

## Recommended next work

1. **#185: key-server email abuse across a cluster.** Current
   `revault_key_server/src/store.rs::check_email_rate_limit` applies configured
   email and IP buckets through a store-local map. Verify cluster-wide behavior,
   restart persistence and proxy enforcement before selecting any remedy. Do not
   equate a local limiter with proof of protection across every relay.
2. **#228 with #211/#212: signer trust and verification claims.** Reciprocal
   exchange supplies signing keys, but identity trust, author authorization and
   freshness are separate guarantees. Review current behavior against the full
   #228 acceptance criteria; keep broader multi-author policy design distinct
   from verifying and documenting existing owner signatures.
3. **#329 with #197/#96/#4: secret-handling boundaries.** Native credential
   handles address managed-memory copies; scoped agent operations address
   authority and reusable key release. They are complementary projects, not
   interchangeable fixes. Review #20 environment fallback alongside input APIs.
4. **#279 with #207 and #325: headless automation.** Session protection and
   headless initialization now have tests. Agentless operations, managed CI
   lifecycles and public-key-only backup setup remain separate requirements.
5. **#224: shared public session-state API.** Default-lockbox persistence still
   appears in CLI `commands/session.rs`; the proposed public snapshot API was
   not found in the Vault API. This is a bounded API task with explicit tests
   and binding scope in the issue.

## Verify before closing or narrowing

| Issue | Existing evidence and remaining review |
| --- | --- |
| #227 | `creation_options.rs::independent_creation_choices_persist_through_cli_lifecycle` exercises encryption/signing/compression/padding combinations. Compare the entire issue, including API and documentation requirements, before closure. |
| #168 | The published manual already has a Profile rotation and access-refresh guide; check completeness against recovery/rollover expectations. |
| #236 | Secure memory uses Linux `MADV_DONTDUMP`. Review supported-platform coverage and distinguish protected allocations from all process memory. |
| #97/#115 | Independent session-protection controls and headless refusal behavior exist. Real suspend/hibernate lifecycle and permission coverage still need explicit qualification. |
| #113/#124/#125/#216 | A substantial manual, topology guide and Vault material exist. Replace broad historical documentation tickets with specific remaining gaps after comparison. |
| #219 | Dynamic completion has host/archive-path tests; reproduce the requested shell interaction before closure. |
| #217 | Tie closure to verified publication across the intended binding/platform matrix, not merely generated source. |
| #188 | A WASM crate exists; compare supported environments and public-package behavior to the issue, without treating crate presence as completion. |

## Intentionally deferred or design work

- #215 remains open for the agreed dependency policy and future ring replacement;
  frozen historical migrations are excluded.
- #307, #286 and #28 track browser/phone authorization designs and acceptance
  work. Do not merge experimental branches solely to reduce branch count.
- #330, #315, #311 and #304 are performance/capacity investigations; do not restart
  broad experiments during release closeout.
- #332 explicitly defers Android Keystore integration beyond the current release.
- CLI reshaping (#278/#70/#71), glob semantics (#305), interactive-password UX
  (#321), help presentation (#218/#92), and the extension rename (#229) are lower
  priority than the security-boundary and automation work above.

No issues were closed by this review alone. No new vulnerability is asserted
solely from an open issue's wording or this limited source inspection.

## Remaining queue and overlaps

The following completes the grouping of the remaining 56 open issues; it does
not add implementation scope to the current release.

| Issue(s) | Disposition |
| --- | --- |
| #30 | Keep independent cryptographic review distinct from automated tests and the completed alert triage. The latter cannot close this assurance objective. |
| #88, #5 | Review metadata and Vault secret-memory boundaries alongside #236/#329. `LockboxPath` currently wraps a standard `String`, and the TOC retains a `BTreeMap`; protected content buffers do not imply all metadata is protected. Define the intended threat model before proposing a representation change. |
| #299 | Loader policy decision: current Python, Dart and Ruby implementations prefer an explicit library path, then `REVAULT_LIBRARY`. Review whether an inherited environment variable should be allowed to override an application's explicit selection; do not silently change that trust boundary. |
| #302, #280, #282, #117 | Related but distinct Vault unlock-policy, per-identity protection, hardware backing and platform-store documentation work. Agree the security model before adding identity passwords or hardware requirements. |
| #198 | Partially superseded: the key server now uses Axum and Tower. The original issue also includes rate limiting, encoding and CLI parsing; compare each remaining clause rather than closing based on dependency names. |
| #155 | Verify and document which core/Vault operations are agentless. Do not confuse an agent-free archive API with the higher-level CLI's current session dependency. Coordinate with #279. |
| #225 | Persistent public-key directory design remains separate from the implemented short-lived exchange. Its transparency, anti-enumeration and recovery criteria remain a substantial future feature. |
| #201 | Workload identity issuance is a future authorization feature; design it with #197/#286 rather than treating it as a token-generation convenience. |
| #80 | Application roadmap overlap with #286 phone authorization and #307 browser integration; choose the intended user experience before independent implementation. |
| #24 | External-source support exists, but remote-browser file access needs end-to-end acceptance with #188/#28. A transport abstraction alone does not establish a complete browser product. |

The reviewed loader ordering and source observations describe current behavior,
not newly demonstrated vulnerabilities. Broader policy changes require their
own implementation and validation work.
