---
description: "The purpose, project goals, priorities and boundaries of reVault."
---

# Project goals

reVault gives people and applications a fast, secure, portable way to store,
use and share files and secrets.

A **Lockbox** brings files, directory metadata, variables and structured records
into one archive. A **Vault** manages the local identities and credentials used
to access and share those archives.

The CLI and language libraries make the
same capabilities available to people, applications and automation.

This is the canonical statement of reVault's project goals.
Release documentation states available capabilities and limitations;
engineering plans define measurable acceptance criteria for each change.

## G1. Ease of use.

A major goal of the reVault project is to provide a great user experience by:

* providing fast, simple and consistent CLI tooling with defaults that the user would
expect. The CLI is opinionated with the aim of keeping the users data safe.
* platform integration to provide SSO and reduce tedium when
working with an active archive from the CLI
* bindings for all popular languages that provide 'language native' access
to archives rather than exporting 'rustisms' up through the stack.


## G2. Safety first
The lockbox archive format is safe by default; encrypted, signed and compressed.

All meta data is encrypted and signed unless the user opts out of encryption.

The user may choose to opt out of encryption, signing and compression on a per
lockbox basis when the lockbox is created.

The archive must support read/writing of file content without the file having
to be expanded to disk.

Secrets are stored in locked memory so that swapping and hibernation do not
result in plain text being written to disk.


## G3. Large files and many files, unlimited forms and variables.
The archive format should be designed to allow the storage of very large files (TB+)
and huge numbers of files (millions +) whilst retaining speed of access to indvidiual files
and the content of files.

There should be no limit (baring disk space) to the number of variables and forms
that can be stored in an archive.

reVault also has a small memory foot print aiming to stay under 100MB RSS even
when working with large archives.


## G4. Keep ownership and access local and portable

People should be able to create, open, copy, back up and share a Lockbox without
depending on a hosted reVault service. An archive should travel with the
information needed to interpret it.

Optional services can help exchange keys or deliver archive bytes. They must
not become a prerequisite for local access to an archive whose credentials the
user already holds. Browser and remote-storage integrations should support
fetching required byte ranges and processing protected contents on the browser
or on the users local device.


## G5. Protect contents, private metadata and authority

Secure defaults should protect file contents, paths, variable names, form data
and private access metadata. Confidentiality, integrity and owner authorization
are distinct guarantees and must be described and tested separately. Explicit
unencrypted or unsigned modes must make their reduced guarantees clear.

The archive uses a hybrid classical and post-quantum protection approach.
Cryptographic mechanisms are part of the core security model and versioned
format specification.

Content marked as secret must be kept in memory.
Logs, command arguments, temporary files and language boundaries as small
and short-lived as practical.

Sharing must grant the intended access without accidentally granting
signing authority or exposing deleted or abandoned content to a
newly authorized recipient. Deleted content must be zeroed out.

## G6. Preserve data through updates, failure and damage

Successful operations must persist the intended contents and metadata. Failed
or interrupted transactions must recover to a complete old or complete new
state, with a clear publication boundary and resumable cleanup.

Damage should be contained so that intact, sufficiently authenticated content
can be recovered independently where the format permits. Recovery must state
what it can prove, distinguish complete from partial results, and never treat
readable abandoned data as committed data. Migration and compaction must verify
their results and preserve the original until replacement is safe.


## G7. Make secure archives practical for everyday workloads

Creating, opening, listing, reading, seeking, updating and extracting Lockboxes
should be fast enough for interactive use, application access and bulk work.
Applications should use selected entries and file ranges without unpacking the
whole archive. Small updates should avoid work proportional to unrelated data
where the security and durability contract permits.

Combine effective compression with bounded memory, reasonable storage overhead,
space reuse and useful damage boundaries.


## G8. Make safe use, sharing and automation straightforward

Provide understandable operations for files, variables, forms, mirrors and
credentials. A mirror is a managed one-way update into a Lockbox, with clear
ownership, deletion policies, previews and refusal conditions.

Profiles, Contacts, password access and recovery material should make access
manageable without exposing raw keys during ordinary use. Support interactive
use and unattended workflows with explicit session and credential boundaries,
useful errors, progress and cancellation where operations can take time.

On desktops the tooling (CLI and API) provide integration with platform
credential stores to provide SSO.
When a credential store is unavailable, supported explicit credential flows must remain
usable without silently weakening protection or hanging on an unavailable prompt.

Installation and core CLI use must work on both desktop and headless systems.

The CLI tooling and API must not make a graphical session or its
development libraries a requirement for ordinary installation or use.

The CLI and libraries should expose consistent domain operations. Callers
should not have to implement archive allocation, transaction recovery or
cryptography themselves.

## G9. Keep formats interoperable and upgrades trustworthy

Use one documented archive model across the CLI and language bindings, backed
by the shared Rust engine. Support native platforms and WebAssembly with
explicitly documented platform capabilities.

A compatibility line is a group of releases that can read a specific
archives format version and saved Vault data. Compatibility must work in both directions:
newer releases must read data written by older releases, and older releases
must read data written by newer releases for the supported archive format.

The tooling can only read archives with the archive format version it supports.
When it detects an older archive format version, it must prompt the user to
migrate the archive. When it detects a newer archive format version, it must
prompt the user to upgrade their tooling.

Any archive or Vault format change that breaks this guarantee requires a new
compatibility line and a tested way to migrate existing data. Compatibility
must be demonstrated by reading each other's saved data.

## G10. Keep the implementation understandable and verifiable

Maintain a shared Rust implementation and a pure-Rust core dependency strategy.
Keep necessary platform and binding boundaries narrow, documented and tested.
Avoid multiple independent implementations of the archive format.

Use explicit storage ownership, bounded parsing and queues, reproducible
measurements, failure injection and independently reopened persisted results.
Prefer a design whose security and recovery properties can be explained and
tested over accumulated special cases. Each optimization should address a
measured gap in a project goal.

## G11. Performance objectives

Unencrypted reads and writes should be competitive with ZIP, Encrypted reads and
writes should be competative with PGP.

The objective is parity, with an aspiration to be at least 10% faster. Measure first
reads separately from repeated reads of cached decoded data.

Security guarentees must not be compromised for performance.

Release plans define the workloads, baselines and statistical tests; no single
microbenchmark establishes success.

Using the CLI tools should consume no more than 100MB RSS even on the largest
of archives.

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
* provide a hosted enterprise secret-management service
* save space by compressing files together or storing shared content only once
  at the expense of easy access, privacy, safe updates or recovery after damage.

These boundaries do not preclude future integrations. Such work needs its own
scope and must not displace unfinished core guarantees.

## addendum
The core rust archive library aims to be pure rust with as few unsafe blocks as
possible.
Any unsafe blocks must be rigourously tested.

No C dependancies are allowed.

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
