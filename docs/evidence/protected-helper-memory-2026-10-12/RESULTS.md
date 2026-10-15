# Protected writes including the persistent helper

All eight public create/open/add lifecycles completed in 125.818 seconds: one
excluded preparation and three measured writes for each pinned 1 GiB corpus.
Independent extraction verified every output byte, tree and mode. Frozen
identities matched afterward; all owned agents and private Vault contexts were
removed. This screen reused retained binary `4142dd04`, with its original
build provenance and the explicit source mapping to checkpoint `a9674d96`.
It was not rebuilt at that checkpoint.

| Corpus | Median CLI elapsed (range) | Maximum measured CLI RSS | Maximum measured CLI + agent bound |
| --- | ---: | ---: | ---: |
| Source/Zstd | 3.30 s (3.29–3.36) | 87,310,336 B | 95,604,736 B |
| Random | 5.06 s (4.96–5.15) | 78,159,872 B | 86,630,400 B |

The largest preparation bound was 94,150,656 bytes. Thus every measured and
preparation bound was below 100,000,000 bytes. Three measured repetitions provide
descriptive medians/ranges, not a speed comparison or statistical qualification.

Each bound adds a command's GNU time peak RSS to the persistent agent's VmHWM
read immediately afterward, then takes the maximum across create/open/add.
Fresh agents were used for each sample; their HWM includes agent startup and
synthetic Vault initialization. This conservative sum of peaks is not an observed
simultaneous peak or a unique-physical-memory total; shared pages may count twice.
Normal per-command credential work remained in elapsed time. Archive drain was
separate; reported 0.00 seconds reflects timer resolution, not zero work or a
crash-durability guarantee.

Agent identity was verified using the exact private socket inode, executable
hash and PID start-time. Only numeric process memory and identity were captured;
no process arguments or environments were read. The frozen child inventory
checked `/proc/PID/task/PID/children`, covering the main thread only. It did not
enumerate every thread's children or establish a complete transient process
census. Consequently these results support the observed CLI-plus-agent scope,
not unrestricted process-tree or many-file acceptance. Subsequent observation
will enumerate all task child lists.

No runtime, worker, security or format policy changed. No tests or builds were
repeated for this measurement-only checkpoint. The fixed protocol is preserved
unchanged in `PROTOCOL.txt`; exact samples, identities, cleanup records and the
descriptive summary are in `raw/`. Next is current many-file extraction evidence
including compressed data; the earlier rejected worker-cache policy stays absent.
