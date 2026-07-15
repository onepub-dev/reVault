# Rust Development Guidance

This project follows local security and storage invariants first, then general
Rust idioms.

## References

Use these as the baseline:

- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Clippy documentation: https://doc.rust-lang.org/clippy/
- Clippy lint list: https://rust-lang.github.io/rust-clippy/master/

For a published skill-style reference, use the OpenClaw Rust skill as a light
checklist for ownership, borrowing, strings, errors, iterators, concurrency, and
unsafe pitfalls:

- https://playbooks.com/skills/openclaw/skills/rust

Do not import a generic implementation skill wholesale. It will miss project
rules around page-cache ownership, secret ownership, COW redaction, and recovery.

## Required Checks

Run from `rust/`:

```text
cargo xtask check-required
```

This task runs formatting, hard Clippy, and tests for the active Rust crates:
`revault_lockbox_api`, `revault_cli`, and `revault_vault_api`.

When a new Rust crate becomes part of the supported build, add it to
the `check-required` and `clippy-advisory` xtasks, and the CI workflow.

`revault_migration_format`, both historical v1 exporters, and
`revault_migration` are supported release crates and must be included in those
checks alongside the lockbox, vault, and CLI crates.

Hard Clippy means:

```text
cargo clippy --workspace \
  --exclude revault_bindings \
  --exclude revault_wasm_bindings \
  --exclude revault_wire \
  --exclude revault_tooling \
  --all-targets -- -D warnings
```

That treats Clippy's default lint groups as errors. It is required, but it is
not the strongest possible Clippy policy.

## API Docs

Generate the public `revault_lockbox_api` API docs from `rust/`:

```text
cargo xtask generate-api-docs
```

The generated entry point is `rust/target/doc/revault_lockbox_api/index.html`. The
generated HTML is build output and is not committed.

## Advisory Clippy

Run from `rust/`:

```text
cargo xtask clippy-advisory
```

This enables `clippy::pedantic`, `clippy::nursery`, and `clippy::cargo` as
warnings, with noisy metadata/visibility lints disabled. Treat the output as
review input, especially for public API, parser, crypto, page-cache, unsafe, and
compression work.

Do not require the advisory pass to be warning-free yet. Current useful warning
categories include unchecked integer casts, unnecessary `Result` wrappers,
large stack arrays in tests, missing public API error docs, and some avoidable
clones. Current noisy categories include style preferences such as `const fn`
candidates and long test functions.

Do not enable `clippy::restriction` wholesale. It is a collection of policy
lints, not an idiomatic-Rust profile. Cherry-pick restriction lints only when
they encode an actual project rule.

## Local Rules

- All normal page reads and writes go through the page cache.
- Passwords are owned and passed as `SecretString`.
- Long-lived secret bytes use `SecretVec` or a more specific secret wrapper.
- Keep `unsafe` blocks tiny, documented, and behind safe wrappers.
- Avoid `unwrap()`/`expect()` in production code unless the invariant is local
  and explicit.
- Public API changes need direct API tests, not only CLI coverage.
- Persisted-format changes must follow
  [Format versioning and migrations](format_versioning_and_migrations.md).
  Current APIs read only their current native format; historical readers belong
  in exact-version migration exporters published through crates.io.
