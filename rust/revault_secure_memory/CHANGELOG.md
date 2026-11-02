# Changelog

## 0.1.1

- Publishes the Android changes already qualified in the bindings 0.6.2 native
  carriers: page-sized allocation chunks and fewer size classes to respect the
  locked-memory allowance on 16 KiB devices.
- Applies Android dump-exclusion advice and reports the supported Android
  dump/fork protections.
- Rust source consumers with an existing lockfile should run
  `cargo update -p revault_secure_memory --precise 0.1.1`. The already-published
  0.6.2 facade accepts this compatible patch, but its original dependency minimum
  remains 0.1.0. No existing crate version or archive format is replaced.

## 0.0.3

- Restored the minimum supported Rust version to Rust 1.88.
- Updated secure string handling and its public tests for the final profile-based
  API release.
