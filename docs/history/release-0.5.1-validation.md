# 0.5.1 release validation scope

This release integrates the previously published 0.5.0 format-4 work into
`main`, repairs migration selection and diagnostics, consolidates historical
exporters, and renames the current secure-memory utility. It does not introduce
a new archive compatibility line.

The secure-memory implementation is copied under its accurate package name;
the historical readers retain documented source provenance. Migration changes
are checked through packaged exporter protocols and public CLI lifecycles,
including preserved Vault records and bulk migration behavior. Published CLI
0.5.0 is the pinned baseline for bidirectional archive and Vault compatibility.

No new steady-state archive compression or allocation algorithm is introduced
by this consolidation. Repeating the full ZIP/PGP comparison, profiling,
bootstrap sampling, and RSS observation campaigns is therefore not a release
requirement. Earlier observations retain their original scope and limitations;
this decision does not turn incomplete performance goals into passing results.

Focused local checks qualify the changed code and new dependency packages.
The release candidate owns the broad source, platform, and installed-binding
checks. Candidate preparation also installs the exact published exporter
versions and runs persisted migration checks with those binaries. Promotion
reuses that successful candidate's verified artifacts and evidence.

Publication status and the authoritative candidate results are recorded by the
release workflows; this scope note alone does not establish release readiness.
