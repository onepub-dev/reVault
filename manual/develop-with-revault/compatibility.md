---
description: Manual scope, component versions and compatibility policy.
---

# Versions and compatibility

This is the <code class="expression">space.vars.manual_status</code> manual for the current development line. Its CLI examples target `revault_cli` <code class="expression">space.vars.cli_version</code>, and its operator pages target `revault_key_server` <code class="expression">space.vars.key_server_version</code>.

{% hint style="warning" %}
reVault is pre-release software. The archive format and public APIs may change. Do not keep the only copy of important data in a Lockbox, and retain tested Vault/Profile recovery material.
{% endhint %}

Check the executable actually being used:

```bash
lbx --version
revault_key_server --version
```

Package registries and a language binding's own README are authoritative for that package's latest published version. A binding and its native carrier must come from the same release; do not combine a shim from one version with a native library from another. The stable C ABI reports its ABI version through `api_abi_version()`.

## Format compatibility

Vault and Lockbox formats are versioned independently. A current CLI may require the Vault to be migrated before it can migrate an older Lockbox:

```bash
lbx doctor migrate vault --output ./vault-migrated
lbx doctor migrate lockbox old.lbox --output migrated.lbox
```

Read [Migrating between versions](../maintain-and-recover/migrating-between-versions.md) before replacing either source. Keep the pre-migration copy until the new one has been opened and checked.

## Migration checks for a release

Release preparation checks permanently retained Lockbox and Vault files from
historical writers. The repository keeps archive fixtures for formats v1–v5 and
Vault fixtures for structures v1–v3, including the historical structure-v2
containers and the structure-v3/container-v4 and current structure-v3/container-v5 combinations. Each fixture
records its producing versions, supported logical content and checksums.

Fixtures with the required owner material are migrated to the current native
format, independently reopened and compared with saved logical contents. The
immutable signed v2/v3 originals do not retain their original private owner keys:
their coverage is authenticated export plus refusal to replace the established
owner, with source bytes and destination absence checked. Separate ordinary
v2/v3 fixtures with known synthetic owners exercise successful migration,
subsequent mutation and persisted reopen. Those positive tests do not turn an
unavailable historical owner key into a successful retained-fixture migration.

The new v4 fixture includes clearly marked public test-only signing material so
its established owner can be preserved through the complete lifecycle. It must
never be used for real data. Tests also update migrated mirrors and Vault
credentials and reopen them to verify persistence. Release checks require every
supported format version and the current Vault/container combination. New
versions add fixtures; historical fixture bytes are never regenerated to pass.
The artifacts live in Git and release tags, not the published runtime library.

## Documentation for a release

The repository tag belonging to a release is the permanent documentation snapshot for that source version. The hosted manual follows the active development branch and may describe functionality newer than an installed package.

When reporting a problem, include the exact CLI or package version rather than saying only that the manual is current. Generated class/method documentation belongs to the package release and is the signature authority when it differs from the development manual.

## v4 development transition

The v4 container changes physical allocation accounting and durable transaction recovery. Vault structure version 3 remains unchanged, but its enclosing Lockbox container must migrate to v4. Older containers are read by isolated migration executables rather than the current core.

The C ABI and FlatBuffers operation schema have not changed for transaction maintenance. Every distributed native carrier, including the WASM build, must nevertheless be rebuilt with the v4 core and shipped with its matching facade package. Existing binaries do not gain v4 support from a documentation or facade-only update. See [API maintenance](apis/revault-api.md#transaction-maintenance-and-v4).
## Native descriptions and format 5

The 0.6.x line stores variable descriptions, field guidance and form-record
descriptions directly in format 5. The 0.5.x line remains format 4. A new
compatibility line is required because neither line can open the other's
containers directly. Migrate the Vault first, then its older Lockboxes. Vault
structure version remains 3; its container changes to format 5.

The first 0.6.0 candidate uses published 0.5.0 as a migration source and tests
old-client refusal of format 5. Later 0.6.x releases must use a published 0.6.x
baseline for bidirectional compatibility tests. Declaring a version does not
establish release readiness.
