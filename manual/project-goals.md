---
description: "The purpose, project goals, priorities and boundaries of reVault."
---

# Project goals

reVault gives people and applications a fast, secure, portable way to store,
use and share files and secrets under their own control.

A **Lockbox** brings files, directory metadata, variables and structured records
into one archive. A **Vault** manages the local identities and credentials used
to access and share those archives. The CLI and language libraries make the
same capabilities available to people, applications and automation.

This is the canonical statement of reVault's project goals. It describes the
outcomes we intend to deliver, not a claim that every current release achieves
them. Release documentation states available capabilities and limitations;
engineering plans define measurable acceptance criteria for each change.

## G1. Keep ownership and access local and portable

People should be able to create, open, copy, back up and share a Lockbox without
depending on a hosted reVault service. An archive should travel with the
information needed to interpret it, while its credentials remain under the
user's control.

Optional services can help exchange keys or deliver archive bytes. They must
not become a prerequisite for local access to an archive whose credentials the
user already holds. Browser and remote-storage integrations should support
fetching required byte ranges and processing protected contents locally.

## G2. Protect contents, private metadata and authority

Secure defaults should protect file contents, paths, variable names, form data
and private access metadata. Confidentiality, integrity and owner authorization
are distinct guarantees and must be described and tested separately. Explicit
unencrypted or unsigned modes must make their reduced guarantees clear.

Preserve the project's hybrid classical and post-quantum protection approach.
Cryptographic mechanisms and changes belong in the security model and versioned
format specification, rather than being chosen as performance shortcuts.

Keep secret exposure in memory, logs, command arguments, temporary files and
language boundaries as small and short-lived as practical. Sharing must grant
the intended access without accidentally granting signing authority or exposing
deleted or abandoned content to a newly authorized recipient.

## G3. Preserve data through updates, failure and damage

Successful operations must persist the intended contents and metadata. Failed
or interrupted transactions must recover to a complete old or complete new
state, with a clear publication boundary and resumable cleanup.

Deletion, replacement and abandoned writes must account for their entire
physical storage. Repeated mirror updates must not leak payload copies, damage
neighbouring records or accumulate unexplained allocations. Reusable space must
be safe to reuse.

Damage should be contained so that intact, sufficiently authenticated content
can be recovered independently where the format permits. Recovery must state
what it can prove, distinguish complete from partial results, and never treat
readable abandoned data as committed data. Migration and compaction must verify
their results and preserve the original until replacement is safe.

## G4. Make secure archives practical for everyday workloads

Creating, opening, listing, reading, seeking, updating and extracting Lockboxes
should be fast enough for interactive use, application access and bulk work.
Applications should use selected entries and file ranges without unpacking the
whole archive. Small updates should avoid work proportional to unrelated data
where the security and durability contract permits.

Combine effective compression with bounded memory, reasonable storage overhead,
space reuse and useful damage boundaries. Measure both fresh archives and
archives aged by repeated updates. Default operation should perform well without
requiring callers to discover a special cache profile.

Unencrypted reads should be competitive with ZIP; the current improvement
objective is parity, with an aspiration to be at least 10% faster. Measure first
reads separately from repeated reads of cached decoded data. Protected modes
must have explicit performance budgets that retain their security guarantees.
Release plans define the workloads, baselines and statistical tests; no single
microbenchmark establishes success.

## G5. Make safe use, sharing and automation straightforward

Provide understandable operations for files, variables, forms, mirrors and
credentials. A mirror is a managed one-way update into a Lockbox, with clear
ownership, deletion policies, previews and refusal conditions.

Profiles, Contacts, password access and recovery material should make access
manageable without exposing raw keys during ordinary use. Support interactive
use and unattended workflows with explicit session and credential boundaries,
useful errors, progress and cancellation where operations can take time.

Installation and core CLI use must work on both desktop and headless systems.
Optional desktop credential stores must not make a graphical session or its
development libraries a requirement for ordinary installation or use. When a
credential store is unavailable, supported explicit credential flows must remain
usable without silently weakening protection or hanging on an unavailable prompt.

The CLI and libraries should expose consistent domain operations. Callers
should not have to implement archive allocation, transaction recovery or
cryptography themselves.

## G6. Keep formats interoperable and upgrades trustworthy

Use one documented archive model across the CLI and language bindings, backed
by the shared Rust engine. Support native platforms and WebAssembly with
explicitly documented platform capabilities.

Every release in a compatibility line must read archives and persisted Vault
formats written by every other release in that line, including an older
release reading newer output. Historical support outside a line must be stated
and verified through retained fixtures and migration tooling.

Breaking persisted changes require a new compatibility line and a tested
migration path. Package version declarations, unchanged binding signatures and
a successful build are not evidence of interoperability.

## G7. Keep the implementation understandable and verifiable

Maintain a shared Rust implementation and a pure-Rust core dependency strategy.
Keep necessary platform and binding boundaries narrow, documented and tested.
Avoid multiple independent implementations of the archive format.

Use explicit storage ownership, bounded parsing and queues, reproducible
measurements, failure injection and independently reopened persisted results.
Prefer a design whose security and recovery properties can be explained and
tested over accumulated special cases. Each optimization should address a
measured gap in a project goal.

## Priorities and boundaries

Correctness, the selected security guarantees and promised compatibility are
constraints. Performance, compression, memory, recovery granularity and
implementation complexity are trade-offs to evaluate within those constraints.
Changing a guarantee requires an explicit documented product decision; a faster
benchmark does not authorize it.

reVault does not promise to:

* replace independent backups or recover encrypted data without credentials;
* erase previous copies, filesystem snapshots or device-level remnants;
* revoke plaintext or keys that a recipient has already retained;
* protect secrets after an authorized application or compromised user session
  exposes them;
* provide a hosted enterprise secret-management service, distributed filesystem
  or general-purpose database as part of the core archive project;
* make whole-archive solid compression or content deduplication the default
  regardless of their effects on access, privacy, updates and recovery.

These boundaries do not preclude future integrations. Such work needs its own
scope and must not displace unfinished core guarantees.

## Applying these goals

Design proposals and implementation plans should identify the goal IDs they
serve, the observable acceptance criteria, and any trade-offs. Experimental
results describe particular revisions and workloads; they do not silently amend
these goals. Keep implementation status and release gates in the relevant plan
so this page remains a stable statement of purpose.

See [Versions and compatibility](develop-with-revault/compatibility.md),
[Transactions and recovery](develop-with-revault/transactions.md), and
[Keeping secrets a secret](protect-and-share/keeping-secrets-a-secret.md) for
the current operational contracts and limitations.
