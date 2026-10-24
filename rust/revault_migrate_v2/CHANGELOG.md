# Changelog

## 0.1.1

- Publish recovery-directory protocol 1 support, including verified alternate
  file ranges and private recovery diagnostics for `lockbox doctor recover`.
  The public CLI validates the exported directory and creates the recovered
  archive; this exporter continues to read historical format 2.
- Retain the original frozen-reader provenance and licenses. Ordinary archive
  and Vault migration capability versions remain unchanged.

## 0.1.0

- Initial combined format-2 archive and Vault exporter package.
