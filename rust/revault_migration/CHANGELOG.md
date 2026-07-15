# Changelog

## Unreleased

- Raised the minimum supported Rust version from 1.88 to 1.95.
- Added an encrypted journal frame for the generated per-migration artifact
  key, allowing direct migrations to resume without requiring a user-supplied
  migration password. The runtime key remains in `SecretVec`; it is not
  serialized through an ordinary byte field.
