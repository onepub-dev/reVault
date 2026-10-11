# Isolated integration campaign diagnosis

The exact default-feature release test
`file_format::candidate_files::tests::tree_tests::forms::fields::typed_form_field_upgrade_guarded_atomic_recovery_faults`
passed once: 2,232 mutation-fault cases and 672 interrupted-recovery cases,
one test passed, zero failed, 710 filtered out. Cargo separately reported a
1 minute 8 second rebuild and 578.56 seconds of test execution. The bounded
invocation used a 1,200-second timeout with a 30-second kill grace and exited 0.
See [command, output and terminal result](raw/test-transcript.txt) and
[source identity](raw/identity.txt), recorded at `574ebafc` after restoring the
rejected lifetime line. One existing dead-code warning was recorded.

This diagnoses one long-running campaign observed during the earlier
[incomplete API integrations](../credential-open-2026-10-12/raw/integration/).
Its observed duration makes cumulative suite cost plausible; it does not prove
the cause of either prior timeout, establish full-suite completion or cover the
native-block feature configuration. No assertion was removed, no test was
ignored, and no retry or identical full-suite rerun followed. It is validation
of the retained batch, not qualification of the rejected lifetime experiment.
The new public ZIP comparison ran only after this test process terminated.
