# Changelog

## 0.0.4

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
