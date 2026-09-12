# Lockbox v4 transactions and recovery

The maintained protocol specification is now part of the GitBook manual:
[Transactions and recovery](../manual/develop-with-revault/transactions.md).
It describes preparation, the publication boundary, post-publication cleanup,
sealing, rollback, truncation, compaction and failure handling.

## Implementation map

| Responsibility | Source |
| --- | --- |
| Authenticated header slots and publication | [header_v2.rs](../rust/revault_lockbox_api/src/file_format/header_v2.rs) (historical filename; implements v4) |
| Conservative reservation journal and rollback | [preparation.rs](../rust/revault_lockbox_api/src/lockbox/preparation.rs) |
| Commit construction, publication, savepoints and abort | [commit.rs](../rust/revault_lockbox_api/src/lockbox/commit.rs) |
| Cleanup validation and checkpoints | [lockbox.rs](../rust/revault_lockbox_api/src/lockbox.rs) |
| Live allocation inventory and retirement | [accounting.rs](../rust/revault_lockbox_api/src/lockbox/accounting.rs) |
| Authenticated tail truncation | [tail_reclamation.rs](../rust/revault_lockbox_api/src/lockbox/tail_reclamation.rs) |
| Independently verified atomic compaction | [lockbox_rewrite.rs](../rust/revault_lockbox_api/src/lockbox/lockbox_rewrite.rs) |
| Recovery status and progress types | [transaction_recovery.rs](../rust/revault_lockbox_api/src/model/transaction_recovery.rs) |

## Regression coverage

Core tests exercise repeated file/variable/form replacement, physical allocation
bounds, abandoned streamed writes, invalid authenticated boundaries, storage
operation failures, cancellation/resume, loss of either header slot, and real
process termination during preparation, rollback, truncation and compaction.
CLI tests separately reopen and read persisted content after mirror lifecycles,
compaction and migration. See the tests alongside the implementation modules,
[CLI tests](../rust/revault_cli/tests), and
[migration tests](../rust/revault_migration/tests).

The public ABI remains unchanged. The maintenance exclusions and package release
requirements are documented in [bindings](../bindings/README.md#v4-storage-and-transaction-maintenance).
