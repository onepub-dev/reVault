64 MiB raw transition fixture comparison, 2026-10-02

The transition fixture was built by the isolated harness using the repository-pinned Rust 1.88 toolchain, from the exact retained 64 MiB raw source. The harness creates an empty typed tree and performs one authenticated update through the existing tree_image::update_files path. tree-construction.json records logical_fragment_unit=65536; the exported dense.lbox is 68,288,512 bytes. `transition-fixture-inputs-before.sha256` and measurement before/after manifests verify source and retained C/ZIP controls were unchanged.

The retained C candidate was audited read-only with the cost-model phase. It reports extent_unit=262144, 1024 fragments and physical packs, and largest_fragment_decoded_bytes=65536. The C smoke also reports extent_unit=262144. Thus the C and transition-tree actual logical fragment sizes agree at 65536 bytes; the harness create hint of 262144 is not being presented as the raw fragment unit.

Each run uses 30 paired samples, three warmups, one pass, CPU 2 affinity, warm cache. Raw-range was measured after the cost-model audit. Raw-stream completed just before the audit request arrived and is preserved in raw-stream-before-cost-model/; the audit then confirmed the unchanged C archive's units match the transition-tree unit. No duplicate stream run was performed. Both runner logs report unchanged inputs. `paired-results.tsv` contains medians and paired 95% intervals; ratios are tree/C.

This is a transition-created typed tree versus the retained post-compaction C archive. It is not a fresh-tree export comparison, a migration claim, a 64 MiB compressed case, cold-cache data, or full A3/A5 qualification.
