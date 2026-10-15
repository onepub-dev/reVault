# Advisory database evidence reconciliation

Initial clean-DB evidence used a JSON array of path/byte-count/SHA-256 records, with file digest `51e7cc39b344143bc82cfc9abdb103405880c90bc99bfece3780e006999853bc`. The later audit used `sha256sum`-style lines with digest `74d2a0ef2db9e2b57f5137b16bd7aab654102df2f5a446269d22b64d24852d4b`.

Independent normalization of the original JSON to lines `SHA256  path` produced exactly `74d2a0ef2db9e2b57f5137b16bd7aab654102df2f5a446269d22b64d24852d4b`, matching the later `advisory-manifest.txt`. The manifest hash difference reflects serialization, not differing tracked file hash/path pairs.

The actual initial `/tmp/revault-final-audit-7cb57d5/clean-db-attempt/workspace.stdout.json` reports 1,309 advisories, matching the later audit. A report of 1,308 for that initial clean scan was incorrect. Both scans identify the same database revision `d8325cf4f578e2df4cc5b98a30e9f46b3b88e9f0`; clean status is recorded by their respective executors.

The old workspace finding and new clean audit differ because the workspace dependency was corrected from ruzstd 0.8.3 to 0.9.1, not because the advisory was removed. No audit rerun is needed merely to reconcile these representations.
