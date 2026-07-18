# Runtime result protocol

Each language runner writes tab-separated records to standard output only
after the operation's observable effects have been asserted:

```text
PASS<TAB>php<TAB>lockbox_add_file<TAB>3
ARTIFACT<TAB>php<TAB>archive-created<TAB>/artifacts/php.lbox
NATIVE<TAB>php<TAB>linux-x86_64-gnu<TAB>dynamic<TAB>/installed/path/librevault_api.so<TAB>sha256<TAB>installed
```

The fourth `PASS` field is the number of assertions made against returned
values or persisted effects. `SKIP` and `XFAIL` records fail conformance.
Required artifact records are `archive-created`, `archive-opened`,
`vault-created`, and `vault-opened`.

`revault-tool e2e verify-results` rejects missing languages, missing
operations, zero assertions, unknown symbols, absent native-install evidence,
and absent artifact interoperability checks.

The Rust source-native exception emits `SUITE<TAB>rust<TAB>name<TAB>passed`
for both public API suites and
`SOURCE<TAB>rust<TAB>target<TAB>source-native<TAB>path<TAB>sha256<TAB>installed`.
