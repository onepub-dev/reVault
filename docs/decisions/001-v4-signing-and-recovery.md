# 001: Owner authorization and independent recovery

Status: proposed, 2026-09-26. Author: Codex. Product/security reviewer: unassigned.
No wire change activated. Goals G2/G3/G4; acceptance A2/A3/A7.

## Problem

Native salvage currently opens a reconstructed snapshot, then checks that a
recovered entry equals an entry in its verified TOC. Signed plaintext open hashes
all live contents. One damaged object therefore prevents proving an intact
neighbour. Truncating tail control data can remove the selected publication proof.
An unauthenticated recovery manifest cannot safely bypass either dependency.

## Recommendation

Separate owner-authorized **membership** from independent **content verification**.
Evaluate a signed commit commitment over canonical metadata and per-object content
commitments. Include archive identity, sequence, logical identity, lengths,
permissions, frame order, codec/protection descriptors and relevant non-file
records. Recovery should verify the selected commit and one object's proof without
opening unrelated content. Bind descriptors and payload to the same owner proof
in encrypted signed mode, including against a content-key holder.

Keep eager normal-open verification for now. This avoids silently changing when
existing callers learn about damage. Use the same commitments to verify all objects
at open and to verify one object during salvage. A later lazy-open API requires a
separate decision with explicit semantics, not a benchmark-only toggle.

Compare a flat authenticated object table against a tree with per-object proofs.
The former is simpler but loses authority if the table is damaged; the latter can
improve proof survival but adds update/storage complexity. Measure both metadata
rewrite and recovery costs before selecting an encoding.

## Alternatives and limits

* Keep whole-snapshot verification: lowest protocol change, but fails independent
  recovery and retains payload-proportional signed-open cost.
* Trust scanned descriptors/checksums: rejected; cannot establish owner authority,
  publication or deletion freshness.
* Sign each content object alone: insufficient without committed membership;
  signatures on deleted/abandoned content remain mathematically valid.
* Signed membership plus recoverable proof redundancy: recommended experiment.
  Specify which independent damage regions it tolerates. A lost final auth record
  requires a durable redundant publication proof, not merely more payload hashes.

If all publication proofs are lost, fail closed with an explicit report. A previous
valid generation may be offered as historical salvage only when clearly labeled;
it must not silently resurrect deletions as the current state. Full-file rollback
cannot be detected without external freshness state.

## Required evidence

A concrete byte/proof graph, publication ordering and tear analysis; adversarial
substitution tests in all modes; neighbour/TOC/tail damage matrix; eager-open and
small-update costs; conformance vectors; storage/security review. Existing native
failures stay visible until the selected contract explains and tests their expected
outcomes. Do not merely change expected recovered-file counts to obtain green tests.
