# Lockbox, GPG, and ZIP benchmark

Run date: 2026-08-30

Release source: `revault_cli-v0.0.8` / `revault-api-v0.3.13` release candidate

Host: AMD Ryzen 7 3700X (16 logical CPUs), Linux kernel `7.0.0-30-generic`

## Executive summary

This run uses the released pure-Rust `zstd-complete 0.1.0` crate from
crates.io. The workspace lockfile records crates.io checksum
`989d8ad3ab75a470a7cf02f8e5194f203cb8b32ac72920c6f8a599a8817b9699`.
No local Cargo patch or companion checkout was used. reVault parallelizes
independent compression frames with its existing worker policy and does not
enable `zstd-complete`'s optional internal multithreading feature.

Lockbox produced substantially smaller compressible output than GPG's ZLIB
level 6 and ZIP's Deflate level 6. It was faster to create than GPG for all
three workloads, but slower to extract. ZIP was fastest on the single-file
cases because it performs no encryption, authentication, password derivation,
or signing. Every extracted payload passed checksum verification.

## Results

Each operation was repeated five times. The table reports the median
wall-clock time and final archive size. Ratios are Lockbox divided by the
comparator; lower is better.

| Workload | Input | Lockbox size | GPG size | ZIP size | Lockbox create | GPG create | ZIP create | Lockbox extract | GPG extract | ZIP extract |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| large-text | 16.00 MiB | 22.31 KiB | 55.81 KiB | 55.86 KiB | 158.5 ms | 176.4 ms | 64.9 ms | 136.5 ms | 128.8 ms | 45.8 ms |
| large-randomish | 16.00 MiB | 16.02 MiB | 16.01 MiB | 16.00 MiB | 303.5 ms | 546.1 ms | 386.7 ms | 211.3 ms | 165.8 ms | 79.8 ms |
| small-tree | 1.95 MiB | 1.98 MiB | 1.98 MiB | 2.19 MiB | 184.3 ms | 197.0 ms | 162.4 ms | 271.1 ms | 237.2 ms | 192.5 ms |

Derived ratios:

| Workload | Size L/GPG | Size L/ZIP | Create L/GPG | Create L/ZIP | Extract L/GPG | Extract L/ZIP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| large-text | 0.40x | 0.40x | 0.90x | 2.44x | 1.06x | 2.98x |
| large-randomish | 1.00x | 1.00x | 0.56x | 0.78x | 1.27x | 2.65x |
| small-tree | 1.00x | 0.90x | 0.94x | 1.13x | 1.14x | 1.41x |

The directory workload is a deterministic set of 2,000 files of 1,024 bytes
each (1.95 MiB logical input). Lockbox, GPG/tar, and ZIP each produced 2,000
extracted files whose sorted path/content SHA-256 manifests matched the source
manifest exactly.

## Archive compression comparison

The release xtask also compared Lockbox against encrypted tar streams using
GPG's default compression, GPG ZLIB level 9, and external zstd levels 1 and 19
with GPG compression disabled. Each row is one end-to-end creation run;
Lockbox time is the import phase after creation of the empty signed archive.

| Fixture | Tool | Logical bytes | Output bytes | Seconds | Max RSS KiB |
| --- | --- | ---: | ---: | ---: | ---: |
| repeated-small | Lockbox | 104,857,600 | 75,072 | 0.82 | 77,480 |
| repeated-small | GPG default | 104,857,600 | 465,438 | 1.73 | 4,800 |
| repeated-small | GPG ZLIB 9 | 104,857,600 | 194,301 | 0.68 | 5,032 |
| repeated-small | zstd 1 + GPG | 104,857,600 | 51,773 | 0.14 | 18,388 |
| repeated-small | zstd 19 + GPG | 104,857,600 | 47,028 | 0.39 | 438,164 |
| text-tree | Lockbox | 30,193,763 | 1,354,048 | 0.91 | 77,480 |
| text-tree | GPG default | 30,193,763 | 2,494,083 | 0.35 | 4,972 |
| text-tree | GPG ZLIB 9 | 30,193,763 | 2,012,028 | 0.56 | 5,112 |
| text-tree | zstd 1 + GPG | 30,193,763 | 1,773,153 | 0.17 | 19,436 |
| text-tree | zstd 19 + GPG | 30,193,763 | 1,124,918 | 24.97 | 117,592 |
| mixed-tree | Lockbox | 21,947,435 | 16,865,600 | 0.95 | 99,732 |
| mixed-tree | GPG default | 21,947,435 | 17,019,545 | 0.59 | 4,904 |
| mixed-tree | GPG ZLIB 9 | 21,947,435 | 16,939,966 | 0.68 | 5,000 |
| mixed-tree | zstd 1 + GPG | 21,947,435 | 16,985,787 | 0.22 | 24,068 |
| mixed-tree | zstd 19 + GPG | 21,947,435 | 16,866,941 | 5.57 | 126,060 |
| high-entropy | Lockbox | 67,108,880 | 67,144,000 | 1.45 | 97,328 |
| high-entropy | GPG default | 67,108,880 | 67,299,412 | 1.69 | 4,848 |
| high-entropy | GPG ZLIB 9 | 67,108,880 | 67,177,057 | 1.87 | 5,004 |
| high-entropy | zstd 1 + GPG | 67,108,880 | 67,174,412 | 0.46 | 23,816 |
| high-entropy | zstd 19 + GPG | 67,108,880 | 67,172,213 | 8.08 | 384,088 |
| revault-source | Lockbox | 3,126,235 | 664,896 | 0.73 | 77,208 |
| revault-source | GPG default | 3,126,235 | 680,060 | 0.19 | 4,964 |
| revault-source | GPG ZLIB 9 | 3,126,235 | 597,354 | 0.32 | 4,864 |
| revault-source | zstd 1 + GPG | 3,126,235 | 663,680 | 0.15 | 8,632 |
| revault-source | zstd 19 + GPG | 3,126,235 | 449,795 | 1.08 | 90,708 |

Lockbox was smaller than GPG default on every fixture, smaller than GPG ZLIB 9
on four of five, and within 1,216 bytes of `zstd -1 | gpg` on the current
reVault source tree. External zstd is not a security-equivalent comparison by
itself; the pipeline includes GPG for encryption, while Lockbox additionally
provides its page-authenticated random-access format and signed commit chain.

## Methodology

The benchmark is `rust/revault_lockbox_api/benches/pgp_compare.rs`.

- Lockbox uses the core API directly, `WorkloadProfile::BulkImport` for create
  and `WorkloadProfile::ExtractMany` for extraction, password protection, and
  owner signing.
- GPG uses symmetric AES-256 with `--no-symkey-cache`, loopback passphrase
  entry, and ZLIB compression level 6. Directory measurements include the
  required `tar` creation and extraction.
- ZIP uses Info-ZIP `zip` 3.0 with `-6 -X` and `unzip` 6.00. ZIP is not
  encrypted and does not provide Lockbox's authenticity or confidentiality.
- Corpora are generated from fixed seeds. The text and random input SHA-256
  values are respectively
  `d60bbf727227eef8d731acc999b786ac04a69006646681824d1f416a5919381d` and
  `1b60184e8c6578591892aed7dfd4f020ab18a54a34d914280c5f2fefc1f9c04f`.
- Extracted final-iteration payloads passed SHA-256 verification for every
  format and workload.

Tool versions:

```text
rustc 1.88.0 (6b00bc388 2025-06-23)
cargo 1.88.0 (873a06493 2025-05-10)
gpg 2.4.8 / libgcrypt 1.12.0
zip 3.0
unzip 6.00
tar 1.35
```

## Reproduction

Run from `rust/`:

```sh
cargo bench -p revault_lockbox_api --bench pgp_compare -- \
  --iterations 5 \
  --root target/lockbox-zstd-complete-0.1.0 \
  --output target/lockbox-zstd-complete-0.1.0/results.md

cargo xtask compare-archive-compression \
  target/archive-comparison-zstd-complete-0.1.0
```

The command leaves disposable corpora and archives under `target/`. No local
dependency override is required.
