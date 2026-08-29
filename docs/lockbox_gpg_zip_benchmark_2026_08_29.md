# Lockbox, GPG, and ZIP benchmark

Run date: 2026-08-29

Repository revision: `revault` `26c544aecb78fbafe2845009e73eca700404bdc7`

Host: AMD Ryzen 7 3700X (16 logical CPUs), Linux kernel `7.0.0-30-generic`

## Executive summary

This run used the optimized, dirty companion checkout at
`/home/bsutton/git/zstd-rs`, branch `faithful-c-compressor-port`, source
`433d26612b5e4e1111610570a46f39b83b4a3b73`. The normal reVault lockfile does
**not** use that checkout: it resolves `ruzstd` from the Git repository at
revision `3f877ed916fd2a8687913f5c130f5f81c7dbcd2b`. A temporary Cargo patch
selected the companion source for this benchmark; the lockfile and companion
worktree were restored/left unchanged.

Lockbox produced substantially smaller compressible output than GPG's ZLIB
level 6 and ZIP's Deflate level 6. It was faster to create than GPG for all
three workloads, but slower to extract than GPG for the two single-file cases.
ZIP was fastest on the single-file cases because it performs no encryption,
authentication, password derivation, or signing. All three workloads completed
and all extracted payloads passed checksum verification.

## Results

Each operation was repeated five times. The table reports the median wall-clock
time and final archive size. Ratios are Lockbox divided by the comparator;
lower is better.

| Workload | Input | Lockbox size | GPG size | ZIP size | Lockbox create | GPG create | ZIP create | Lockbox extract | GPG extract | ZIP extract |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| large-text | 16.00 MiB | 22.31 KiB | 55.81 KiB | 55.86 KiB | 155.1 ms | 180.9 ms | 63.9 ms | 135.6 ms | 126.2 ms | 64.2 ms |
| large-randomish | 16.00 MiB | 16.02 MiB | 16.01 MiB | 16.00 MiB | 287.1 ms | 533.6 ms | 373.9 ms | 215.4 ms | 167.4 ms | 79.5 ms |
| small-tree | 1.95 MiB | 1.98 MiB | 1.98 MiB | 2.19 MiB | 186.4 ms | 207.4 ms | 162.0 ms | 236.0 ms | 229.1 ms | 170.9 ms |

Derived ratios:

| Workload | Size L/GPG | Size L/ZIP | Create L/GPG | Create L/ZIP | Extract L/GPG | Extract L/ZIP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| large-text | 0.40x | 0.40x | 0.86x | 2.43x | 1.07x | 2.11x |
| large-randomish | 1.00x | 1.00x | 0.54x | 0.77x | 1.29x | 2.71x |
| small-tree | 1.00x | 0.90x | 0.90x | 1.15x | 1.03x | 1.38x |

The directory workload is deterministic 2,000 files of 1,024 bytes each
(1.95 MiB logical input). The Lockbox file API automatically creates each
required parent directory while adding the files. Lockbox, GPG/tar, and ZIP
all completed this case; each produced 2,000 extracted files whose sorted
path/content SHA-256 manifests matched the source manifest exactly.

## Methodology

The benchmark is `rust/revault_lockbox_api/benches/pgp_compare.rs`.

- Lockbox uses the core API directly, `WorkloadProfile::BulkImport` for create
  and `WorkloadProfile::ExtractMany` for extraction, password protection, and
  owner signing.
- GPG uses symmetric AES-256 with `--no-symkey-cache`, loopback passphrase
  entry, ZLIB compression level 6. For a directory, the benchmark includes
  the `tar` create/extract steps because GPG does not archive directory trees.
- ZIP uses Info-ZIP `zip` 3.0, `-6 -X`, and `unzip` 6.00. ZIP is **not
  encrypted** in this comparison. Directory ZIP contains the files directly;
  it does not provide Lockbox's authenticity or confidentiality guarantees.
- Corpora are generated from fixed seeds by the benchmark. The text and random
  input SHA-256 values are respectively
  `d60bbf727227eef8d731acc999b786ac04a69006646681824d1f416a5919381d` and
  `1b60184e8c6578591892aed7dfd4f020ab18a54a34d914280c5f2fefc1f9c04f`.
- Extracted final-iteration payloads passed SHA-256 verification against their
  source files for all three formats and all three workloads. The small-tree
  source and each extracted format contained 2,000 files.

Tool versions:

```text
rustc 1.88.0 (6b00bc388 2025-06-23)
cargo 1.88.0 (873a06493 2025-05-10)
gpg 2.4.8 / libgcrypt 1.12.0
zip 3.0
unzip 6.00
tar 1.35
```

The local companion checkout currently requires a one-line `unsafe` wrapper
around its CPUID call to compile under Rust 1.88. For this run a source-only
temporary copy was made and that compile-compatibility wrapper was applied;
the optimized compressor sources were otherwise unchanged. No generated
payloads or benchmark results are retained in the repository.

## Reproduction

After making the companion checkout compile with the selected Rust toolchain,
run from `rust/`:

```sh
cargo bench -p revault_lockbox_api --bench pgp_compare \
  --config 'patch."https://github.com/bsutton/zstd-rs".ruzstd.path="/home/bsutton/git/zstd-rs/ruzstd"' \
  -- --iterations 5 --root target/lockbox-zstd-local \
  --output target/lockbox-zstd-local/results.md
```

The command leaves disposable corpora and archives under `target/`; remove
that benchmark directory after inspection. The benchmark source now includes
the GPG and ZIP commands, reports median sizes/times, and fails fast if a
comparator command or extraction fails.
