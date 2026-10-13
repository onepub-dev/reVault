# Default API suite ignored-test classification

The completed unfiltered default API suite reported 18 ignored tests. Exact output is preserved in `ignored-tests.raw.txt`. This is an inventory/classification only; no ignored test was executed.

Four compression regressions are explicitly scheduled by `.github/workflows/performance.yml:49` (`cargo test --release -p revault_lockbox_api compression_regression_tests -- --ignored --nocapture`):

- `compression_regression_tests::moderately_large_zero_file_uses_few_fixed_pages`
- `compression_regression_tests::one_gib_high_entropy_file_avoids_excessive_expansion`
- `compression_regression_tests::one_gib_zero_file_compression_does_not_regress`
- `compression_regression_tests::repeated_small_files_keep_meaningful_compression`

The remaining 14 are intentionally separate resource/qualification/maintenance probes rather than absent ordinary correctness coverage:

- `file_format::allocation_map::tests::allocation_aging_resource_probe` — manual file-backed allocator CPU/RSS/physical accounting.
- `file_format::authenticated_index::tests::authenticated_index_resource_observation` — explicit release-mode 100,000-entry file-backed resource observation.
- `file_format::candidate_files::resource_probe::candidate_file_resource_probe` — common-runner C CPU/RSS/extent-size evaluation (also named by `benches/evaluation/candidate_driver.rs`).
- `file_format::candidate_files::tests::aging::mixed_file_aging_preserves_contents_and_bounds_physical_growth` — explicit four-mode, 1,000-cycle qualification.
- `file_format::candidate_files::tests::tree_tests::forms::aggregate::typed_form_aggregate_guarded_snapshot_probe` — fresh-process aggregate endpoint probe.
- `file_format::candidate_files::tests::tree_tests::forms::aggregate::typed_form_selected_source_aggregate_probe` — fresh-process selected-source aggregate endpoint probe.
- `file_format::candidate_files::tests::tree_tests::forms::field_aggregate::typed_form_field_upgrade_aggregate_probe` — fixed fresh-process cross-record aggregate probe.
- `file_format::candidate_files::tests::tree_tests::fresh::fresh_tree_export_streams_sixty_four_mib_without_dense_intermediate` — manual file-backed 64 MiB streaming probe.
- `file_format::candidate_files::tests::tree_tests::mutation::dense::typed_dense_payload_thousand_cycle_qualification` — bounded 1,000-cycle qualification requiring `REVAULT_TYPED_AGING_GROUP=0..3`.
- `file_format::candidate_files::tests::tree_tests::variables::prepared::prepared_staging_aggregate_guarded_source_probe` — fresh-process guarded aggregate staging probe.
- `file_format::candidate_files::tests::tree_tests::variables::resource::typed_variable_resource_probe` — dedicated fresh-process resource runner.
- `file_format::preparation_journal::tests::preparation_journal_resource_probe` — manual fresh-process journal CPU/RSS and I/O experiment.
- `file_format::publication_anchor::shared::tree::tests::abort::commit::authenticated_overflow_commit_resource_observation` — bounded serial resource observation, separate from regressions.
- `fixture_artifact_tests::write_fixture_artifacts` — mutates checked-in artifacts only when `LOCKBOX_UPDATE_FIXTURES=1`; maintenance-only.

These ignored names and reasons were read from test attributes / full-run output. I did not run the separate resource or fixture-writing tasks because the full-suite authorization does not blanket-authorize them. The default full suite still completed with 711 ordinary tests passed, zero failures, and these 18 explicit ignores.
