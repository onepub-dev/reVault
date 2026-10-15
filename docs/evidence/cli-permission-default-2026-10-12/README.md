# CLI extraction permission default

At the user's request, ordinary and mirror CLI extraction now restore stored
file and directory permissions by default where supported. The explicit
`--no-restore-permissions` option uses host-created permissions; the existing
`--restore-permissions` flag remains accepted. Conflicting flags are refused.
Explicit-file extraction now applies the same existing restoration helper as
mirror extraction. Library `ExtractPolicy` and import defaults are unchanged.

Current ordinary add and mirror import store file mode `0600`, even when the
host source is `0664`. The default restores that stored metadata; it does not
introduce source-mode preservation during import. This distinction was observed
while preparing the separate public ZIP comparison, before any timed sample.

Four focused release filters passed: the new six-path ordinary/mirror
file/directory/whole extraction lifecycle, read-only byte-exact extraction,
CLI help, and the existing mirror file-command lifecycle. The new lifecycle
was then strengthened with public directory-mode assertions and passed again
(7.74 seconds): five executions, four unique tests. It also verifies child-only
umask 0022, opt-out modes, legacy flag compatibility, conflict refusal,
sentinel-preserving repeat refusal, overwrite, archive/source preservation and
persisted bytes. All archive setup and observations use public CLI operations.

See the unique logs and identities in [raw evidence](raw/), and the exact
[pre-format source diff](raw/pre-format-source.diff). The tracked hook formats
staged Rust only after validation; recorded test hashes refer to that tested
pre-format epoch. No broad suite, WASM build or performance claim is attached to
this CLI-only behavior change. The normative contract is documented in the
[security design](../../security_model.md#extraction-permissions).
