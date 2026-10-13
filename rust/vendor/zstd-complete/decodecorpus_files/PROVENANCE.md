# Ordinary decoder fixtures

The four additional files z000068, z000068.zst, z000088 and z000088.zst are
copied byte-for-byte from tracked, unmodified ruzstd/decodecorpus_files in
bsutton/zstd-rs, with recorded upstream history commit
eff39ae489ba87e3868293527d6337aebede818f (2025-10-15).
They are ordinary deterministic decoder regression fixtures, not an external
crash corpus. Existing LICENSES/ruzstd-MIT.txt and THIRD_PARTY_NOTICES.md apply.

SHA-256 identities:
- z000088: `8a227a9e3e43e0dd49eabb7b0973df8886bf400ffe8b6e2ea032f82724d2d23a`
- z000088.zst: `c349f02daef13a53874cf07675b03e8a451ca76c80a9d447c95bb8173df9a509`
- z000068: `feb315e87872b370513b66b6f62713d4f20ad3a60a303aefad90ee7d4942fed9`
- z000068.zst: `8ef3284106189035ed3c28ce754633830741a7776380d6f974a368a0264f0d38`

The package retains these and the existing dictionary/decoder fixtures needed
by its ordinary tests. External fuzz artifacts remain an explicit optional
test prerequisite and are not included.
The same tracked upstream history also supplies the ordinary abc.txt sibling:
SHA-256 `be609acbce9d8685b80a68d0aa4163b1e1339a018e5f3ff9b32e18e2ab47fbbf`. Independent Zstandard CLI 1.5.7 decoding of the existing
vendor abc.txt.zst produced these exact bytes. The compressed fixture remains
unchanged. All five included .zst corpus entries now have their required ordinary
plaintext siblings; the corpus test ignores this non-.zst provenance document.
