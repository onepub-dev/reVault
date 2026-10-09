# Design ideas and historical discussion

The previous running narrative mixed old implementation descriptions with
proposals. It has been retired. Product requirements live in the
[project goals](../manual/project-goals.md); v4 decisions and acceptance criteria
belong with the [implementation branch](archive_v4_plan.md).

The following ideas from that discussion remain possible future work, not
approved scope or implementation claims:

| Idea | Questions to resolve before adoption |
| --- | --- |
| Async operations and bounded pipelines | Ownership, byte-based backpressure, persistence ordering and measured benefit. |
| Passkey access | Supported platforms, a recoverable local-unlock design and the credential threat model. |
| GPG interoperability | Format/trust boundaries, explicit recipient mapping and plaintext handling. |
| Compaction headroom controls | Space preflight, alternate output destinations and recovery guarantees. |
| Resource limits for hostile archives | Bounded allocation, scan/slot work, cancellation and refusal tests. |

Reassess each proposal against the current implementation before treating it
as missing functionality. Credential-store integration and secret-memory work
have progressed since the original discussion.

The [complete previous discussion](https://github.com/onepub-dev/reVault/blob/bc85ea5c3957cf3421da713f5dd4b16965771dce/docs/design_discussion.md)
preserves the original recommendations and references. They are historical,
not an approved design or a current security assessment.
