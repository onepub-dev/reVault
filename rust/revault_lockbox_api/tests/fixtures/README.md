# Lockbox Fixture Artifacts

This directory separates compatibility fixtures from hostile or corrupt
fixtures.

- `golden/v1/` contains valid lockbox artifacts written by the current v1
  writer. These files protect backwards-compatible decoding behaviour.
- `adversarial/v1/` contains deliberately corrupt or malicious format
  fragments. These files protect expansion and allocation guards.

Fixture payloads are stored as `.hex` text so reviews can diff exact bytes
without adding binary churn. The normal test suite reads these files. To
rewrite them after an intentional format change, run:

```sh
LOCKBOX_UPDATE_FIXTURES=1 cargo test -p lockbox_core \
  fixture_artifact_tests::write_fixture_artifacts -- --ignored
```

Only rewrite golden fixtures for intentional compatibility changes. Do not
rewrite adversarial fixtures unless the corrupt scenario itself changes.
