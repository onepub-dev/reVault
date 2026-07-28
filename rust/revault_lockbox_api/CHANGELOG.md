# Changelog

## 0.0.4

- Added persistent mirror-project metadata and core-enforced exclusive subtree
  ownership. Ordinary mutation APIs cannot change managed paths; trusted
  mirror orchestration receives a mutation scope that cannot escape its
  project destination. The metadata uses existing encrypted variables and does
  not change the archive format.
- Variable path components may use a single leading dot for encrypted
  hidden/internal metadata namespaces. The unsafe `.` and `..` components
  remain invalid.
- Normal variables and form fields can now be promoted to secret storage.
  Form-field promotion creates a new definition revision and upgrades existing
  values across records of that form type; secret-to-normal changes remain
  prohibited in place.
- Raised the minimum supported Rust version from 1.88 to 1.95.
- Removed the `sysinfo` dependency. Automatic page-cache sizing now uses a
  conservative platform default, and Windows stale-lock detection uses native
  process APIs.
- Renamed vault identity access labels to profile access labels. Named access
  entries now use the `profile:` prefix; the former `identity:` prefix is not
  retained.
- Added stable archive-format probing and actionable unsupported-version errors.
- Added narrowly scoped migration APIs for streaming logical archive contents
  and access material into a new native archive. Imported archives create a new
  commit/signature chain; old public commit and signature records are not
  preserved.
