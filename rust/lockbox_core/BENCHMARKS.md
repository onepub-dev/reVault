# Lockbox Core Benchmarks

This package keeps two benchmark entry points:

- `benches/performance.rs` contains Criterion microbenchmarks for core archive
  operations.
- `benches/pgp_compare.rs` is a manual comparison against GnuPG symmetric
  encryption.

## PGP Comparison

Run from the workspace root:

```sh
cargo bench -p lockbox_core --bench pgp_compare -- \
  --iterations 3 \
  --root target/lockbox-gpg-bench \
  --output target/lockbox-gpg-bench/results.md
```

Cargo runs bench binaries from the package directory, so relative `--root` and
`--output` paths are relative to `lockbox_core/`.

The benchmark generates deterministic corpora and compares:

- lockbox create using the core API with `WorkloadProfile::BulkImport`;
- lockbox extract using the core API with `WorkloadProfile::ExtractMany`;
- GPG symmetric encryption using `--no-symkey-cache --cipher-algo AES256
  --compress-algo ZLIB --compress-level 6`;
- `tar + gpg` and `gpg + tar` for directory scenarios, because GPG does not
  archive directory trees by itself.

Use `--lockbox-only` when profiling lockbox code without running GPG:

```sh
perf record -g -o lockbox_core/target/lockbox-gpg-bench/perf-large-randomish.data -- \
  cargo bench -p lockbox_core --bench pgp_compare -- \
  --iterations 5 \
  --scenario large-randomish \
  --lockbox-only \
  --root target/lockbox-gpg-bench-prof \
  --output target/lockbox-gpg-bench-prof/results.md
```

## Sample Result

Machine: AMD Ryzen 7 3700X, 16 hardware threads, Linux 7.0.0-22-generic.

Tooling:

- `gpg` 2.4.8, libgcrypt 1.12.0
- Rust bench profile, optimized build

Command:

```sh
cargo bench -p lockbox_core --bench pgp_compare -- \
  --iterations 3 \
  --root target/lockbox-gpg-bench-final \
  --output target/lockbox-gpg-bench-final/results.md
```

| Scenario | Logical bytes | Lockbox size | PGP size | Lockbox create | PGP create | Lockbox extract | PGP extract | Size ratio L/PGP | Create ratio L/PGP | Extract ratio L/PGP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| large-text | 16.00 MiB | 22.09 KiB | 55.80 KiB | 153.1ms | 175.9ms | 133.6ms | 124.0ms | 0.40x | 0.87x | 1.08x |
| large-randomish | 16.00 MiB | 16.02 MiB | 16.01 MiB | 263.8ms | 543.7ms | 208.8ms | 169.9ms | 1.00x | 0.49x | 1.23x |
| small-tree | 1.95 MiB | 1.98 MiB | 1.98 MiB | 176.4ms | 196.2ms | 245.1ms | 231.9ms | 1.00x | 0.90x | 1.06x |

Interpretation:

- Compression is at parity or better. The compressible text corpus is much
  smaller in lockbox because lockbox uses zstd while the PGP baseline uses
  GPG's ZLIB support.
- Create performance is at parity or better across the sampled scenarios.
- Extract performance is close to parity for text and many-small-file
  workloads, and about 23% slower for incompressible 16 MiB data on this
  machine.

The large-randomish lockbox-only profile did not show a cheap accidental
bottleneck. Top user-space samples were Argon2 password opening, page payload
zeroization, page encode/decode, SHA-256 checksums, and ChaCha20-Poly1305.
Those costs are expected from the current security model; no security-reducing
optimization was made.
