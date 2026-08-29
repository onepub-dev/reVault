---
description: "Manual scope, component versions and compatibility policy."
---

# Versions and compatibility

This is the <code class="expression">space.vars.manual_status</code> manual for the current `master` development line. Its CLI examples target `revault_cli` <code class="expression">space.vars.cli_version</code>, and its operator pages target `revault_key_server` <code class="expression">space.vars.key_server_version</code>.

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
lbx migrate vault --output ./vault-migrated
lbx migrate archive old.lbox --output migrated.lbox
```

Read [Migrating between versions](cli-tooling/migrating-between-versions.md) before replacing either source. Keep the pre-migration copy until the new one has been opened and checked.

## Documentation for a release

The repository tag belonging to a release is the permanent documentation snapshot for that source version. The hosted manual follows the active development branch and may describe functionality newer than an installed package.

When reporting a problem, include the exact CLI or package version rather than saying only that the manual is current. Generated class/method documentation belongs to the package release and is the signature authority when it differs from the development manual.

## Release checklist

Before publishing a component:

1. update its version and package README;
2. regenerate class/method documentation;
3. run the binding conformance examples;
4. run the manual validation job;
5. confirm migration notes and compatibility statements; and
6. retain the tagged repository documentation as the release snapshot.
