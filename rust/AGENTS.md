When the vault or lockbox format changes we MUST create and publish a new set of migration scripts that upgrade the vault and/or lockboxes as appropriate.

When prepareing for a release we need to ensure that the set of language bindings have been updated to reflect any modifications to the apis.

Our aim is a pure rust implementation so dependencies and transient dependencies that contain C code or other unsafe languages should not be permitted.

Any script tools must be written rust.

After large structural codebase changes, and as part of pre-release review,
consider running RepoWise from `rust/` to identify concentrated complexity,
duplication, architectural coupling, and dead code:

```console
repowise health . --module revault_lockbox_api --refactoring-targets --no-workspace
repowise dead-code revault_lockbox_api --safe-only --no-workspace
```

Treat RepoWise findings as review inputs rather than automatic refactoring
instructions. Prefer cohesive Rust domain types that own related state and
behaviour over collections of unrelated helper functions, and verify coverage
before accepting an `untested` finding.
