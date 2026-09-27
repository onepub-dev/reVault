# Whole-layout cost feasibility model

2026-09-27. This is a read-only model, not a persisted archive or an implemented
publication protocol. It addresses the [structural review](../../archive_v4_evaluation.md#whole-format-eligibility-review)
before committing to another complete prototype.

## Fixed question and method

Can the actual packed-C file corpus fit the unchanged small-file size budget
with compact descriptors, independently bounded fragments and shared control
regions? Reject the proposed geometry if its real metadata or payload does not
fit; passing a size estimate only justifies a protocol experiment.

Read and fully authenticate the existing archive. Preserve every file identity,
logical digest, length, restart unit, fragment codec, stored length and stored-byte
digest. Reconstruct omitted per-fragment file identity, ordinal, logical offset and
length from the committed file header and order, and compare the complete original
64-byte descriptor. This preserves the existing fragment authentication context.
Copy encoded fragments into private bounded buffers; never rewrite the source.

New groups are bounded by stored bytes and 4,096 members, while each decoded
fragment retains its 64/256 KiB profile bound. Their aggregate decoded size is not
an allocation or decompression request. Preserve independent fragment protection;
share pack extent and padding digests in one pack table. Pad physical groups with
the existing codec, including authenticated encrypted-zero padding in protected
modes. Do not replace these guarantees with a shared compression or AEAD frame.

The private catalogue stores paths, stable identity, logical digest, length, unit,
fragment references and one physical-pack record per group. It also reserves
explicit file-kind and 32-bit permission fields; permission `0644` is synthetic
because current C cannot carry actual permissions. Empty and multiframe files are
included in the 16-mode source-preservation test. Variables, forms, links and access
semantics are not implemented by this file-only model.

Metadata compression uses the existing corrected pure-Rust encoder and a bounded
decoder, verifying byte-identical decoding. This evaluates metadata compression
separately from payload compression. Limits: 1,024 files, 4,096 fragments, 1 MiB
constructed catalogue; reject before private-buffer growth. Inline eligibility
requires at most 64 KiB decoded metadata and at most 48 KiB encoded data including
a conservative 60-byte page-header/authentication budget. Larger catalogues need
an overflow-tree design; this model does not assign them a false complete size.

## Proposed geometry to test

Two distinct aligned 64 KiB control regions would each contain:

| Per-region reservation | Bytes | Still requires protocol implementation |
| --- | ---: | --- |
| Publication/owner proof | 8,192 | Role-bound commitments and mirror publication ordering |
| Preparation control | 4,096 | Bounded overflow for the existing maximum reservation workload; no reduced reservation guarantee |
| Public credential bootstrap | 4,096 | Existing password/contact wrapping, owner pinning and authenticated overflow |
| Private catalogue/root | 49,152 | Independent role-bound protection, copy-on-write overflow and secure retirement |

This yields a 131,072-byte control footprint only when the catalogue fits inline.
It does not prove that all controls fit, that an overflow journal works, or that
mutating subrecords in a shared failure region is crash-safe. A real implementation
must preserve the old publication and private metadata until replacement is durable;
retired inline metadata must be zeroed without touching live anchors/journals.
Do not count an attractive size estimate as an ownership or durability proof.

The declared observations use the retained small plaintext corpus, 8 MiB raw and
compressed corpora, 8 MiB encrypted-signed compressed corpus, and a newly created
small encrypted-signed corpus using identical source files. There is one size
projection per fixture; no CPU/RSS comparison or confidence interval is claimed.
Use the resource probe with `REVAULT_CANDIDATE_PHASE=cost-model` and unit 262144.

The proposed next experiment combines this record/pack model with actual public
bootstrap, control ownership, bounded metadata caching and the existing normal-open
verification contract. Keep the current C checkpoint as the control. No format
selection, new release line activation, altered size budget or security relaxation
follows from this model.


## Executed projections

Frozen source `1b60c0d4`, executable SHA-256
`41a7e0e8c37ed6b10422189aa2eeb0da03cf6157760bbe65d6181d0e29c1dd13`.
The 16-mode source-preservation/binding test and strict Clippy pass after the
commit hook. All five declared model runs pass descriptor reconstruction and
bounded metadata decoding. The new protected fixture also independently verifies
its persisted bytes against the same small-file source before projection.

| Corpus | Catalogue decoded / encoded bytes | Packs / padded payload bytes | Projected compacted total |
| --- | ---: | ---: | ---: |
| 512 × 4 KiB, compressed plaintext | 61,478 / 29,385 | 1 / 196,608 | 327,680 |
| Same source, encrypted-signed | 61,482 / 37,733 | 1 / 196,608 | 327,680 |
| 8 MiB raw plaintext | 15,194 / 5,089 | 128 / 8,388,608 | 8,519,680 |
| 8 MiB compressed plaintext | 1,593 / 1,446 | 1 / 65,536 | 196,608 |
| Same large source, encrypted-signed | 1,593 / 1,456 | 1 / 65,536 | 196,608 |

The retained case files identify the exact corpora; the compressed large source
is the predeclared compressible workload, not an incompressible-data claim. The
small plaintext projection is **1.382×** the 237,078-byte ZIP control, below the
proposed 1.50× limit. This demonstrates geometric feasibility only. The real
current-C compacted archive remains 1,441,792 bytes and fails that gate. A5 stays
incomplete until an actual complete archive preserves the required contracts.

For the small corpus, decoded fragments remain 4 KiB while one physical pack
contains 2 MiB of logical contents. For the large compressed corpus, fragments
remain 256 KiB while one 64 KiB physical allocation contains 8 MiB logically.
Single-fragment corruption remains independently authenticatable in the model,
but losing that entire physical allocation loses more logical contents than the
old aggregate-limited packs. This is an explicit compression/damage-locality trade-off,
not proof of equivalent logical loss per damaged physical region. The next damage
matrix must report both physical bytes damaged and logical files/ranges affected.
No logical-loss bound is silently inferred from the random-read decode bound.

Proceed only with the bounded shared-control/catalogue protocol experiment:
implement credential bootstrap with the existing wrapping algorithms; preserve
owner-selected publication and copy-on-write metadata; prove overflow reservation
ownership and retirement of inline private metadata; then cover public record
semantics and the declared performance matrix. This result does not authorize
skipping eager signed-plaintext verification, padding authentication, erasure or
metadata failure-region separation. No model bytes are activated as a format.
