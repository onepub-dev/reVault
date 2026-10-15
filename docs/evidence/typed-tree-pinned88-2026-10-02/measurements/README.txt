Pinned-toolchain read comparison, 2026-10-02

The candidate tree reader and candidate tree fixture driver were built with the repository-pinned Rust 1.88.0 toolchain. Both were hash-verified before and after the batch. The packed-C driver, ZIP runner, and their retained archives are the frozen comparison inputs. See binaries-final.sha256 and the corresponding compiler records in the parent evidence directory.

Six cases each ran 30 measured paired samples, three warmup pairs, one pass, one worker, fresh process/handle, warm OS cache, CPU 2 affinity. Every runner result reports retained_inputs_unchanged=true. The six per-case command logs, raw samples.jsonl, and fixed-seed bootstrap summary.json are preserved. paired-results.tsv extracts medians and paired 95% intervals for total/open/read/first-byte/CPU/RSS; ratio is typed-tree / packed-C.

Cases: 512 x 4 KiB mixed compressed plain; 8 MiB random raw plain; 8 MiB patterned compressed plain; 4 KiB midpoint range in 8 MiB random raw plain; 512 x 4 KiB mixed compressed encrypted-signed; 8 MiB patterned compressed encrypted-signed. Units were 262144 bytes except raw stream and range at 65536 bytes.

Protected C control archives/public keys were copied into protected-c-controls and hash-checked against their original retained roots before and after. Source files in the new controls were copied from the corresponding existing ZIP corpora; C and tree stream/range byte-verification smokes passed. The protected ZIP comparison inputs are unsigned/unencrypted and are weaker-security descriptive baselines; the meaningful protected comparison is packed-C versus encrypted-signed typed-tree, with the same logical source bytes and normal integrity checks enabled.

Host-level context files record load/process state before and after. FDB log collectors and a persistent lbx agent were present; no cargo/rustc or HMB test/runner process was visible. Results are descriptive and limited to these cases, machine state, warm cache, and frozen drivers. They do not establish cold-cache behavior, native activation, migration, or full A3/A5 qualification.
