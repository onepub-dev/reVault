# Current protected-write baseline

Eight public write lifecycles completed on checkpoint `b5f82693`: one excluded
preparation and three measured writes for each pinned 1 GiB corpus. All output
bytes, modes and trees were independently verified through public extraction;
source/executable identities matched afterward, and private Vault contexts and
agents were cleaned up. This is descriptive evidence, not a paired comparison
with earlier ZIP reports.

| Corpus | Median create + open + add | Measured range | Maximum measured subprocess RSS |
| --- | ---: | ---: | ---: |
| Source/Zstd | 3.89 s | 3.82–4.00 s | 102,912,000 bytes |
| Random | 8.31 s | 8.26–8.32 s | 87,584,768 bytes |

The compressed preparation run reached 110,874,624 bytes. The measured compressed
maximum still exceeds the 100 MB target. RSS is the maximum of the three
sequential CLI subprocess peaks and excludes the persistent agent; it does not
establish total process-tree memory. Archive drain is reported separately.
GNU time rounded these drain measurements to 0.00 seconds, which does not mean
zero work or establish crash durability.

The CLI used ChaCha20-Poly1305 encryption, owner signing through a synthetic
default Vault profile, Zstd level 3, default padding and Auto write jobs.
Vault initialization and agent startup were excluded; normal per-command Vault
opening/KDF and explicit archive open remained in timing. Source files retained
mode 0664; archives stored and default extraction restored mode 0600.

The first attempt stopped at its storage cap before any write or sample.
Cleanup initially mistook a directory symlink for an independent retained source:
one backing file was removed, then restored to its pinned hash and mode from an
untouched verified copy. A later retarget helper failed while logging, before
payload removal; its partial state and recovery were recorded. The final
lead-reviewed helper verified all 16 logical source paths before and after
removing 12 redundant backing files, preserving canonical sources, archives,
manifests and evidence. The fresh attempt then fit the unchanged 24 GiB cap.
The [incident report](raw/preparation/incident-and-cleanup-report.txt) links the
original records; no failed timing samples were replaced.

The preserved current CLI hash is
`9ab29f4cf50054ed9c816ae6e1bc464b8ff053fb0a2e83775a2d8da5a85f9312`.
The identity record under `raw` pins the protocol and inputs; the eight samples
and descriptive summary are preserved there. A separate paired trial will assess direct encrypted final-page
assembly; this baseline alone establishes no candidate improvement.
