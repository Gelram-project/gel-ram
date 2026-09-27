# Property-to-test map

Each row names a documented property and the tests that exercise it. On every
platform the [CI evidence collector](CI-EVIDENCE.md) checks that each named
test ran and passed exactly once in the debug workspace pass, or is a declared
Unix-only test that is absent elsewhere, and writes one `PROPERTY` line per row
to its report; any other result withholds COMPLETE. A test name that is not
unique in the workspace fails that check, so every name here identifies one
test.

A passing row shows that these tests ran and passed. It does not show that they
cover every aspect of the property, and properties without a row are not
claimed to be untested. Checks that are not tests (the mutation matrix inside
`xtask verify`, the network-isolation and full-disk steps of the CI workflow,
the binary package smoke run) are listed under the table.

| ID | Property | Source | Tests |
|---|---|---|---|
| P01 | Batch mode: a HIT or UNKNOWN is a data record with exact quotes, and the exit code is 0 | [batch](EVIDENCE-BATCH.md) | `hits_and_unknown_are_data_with_exact_quotes_and_exit_zero` |
| P02 | Batch mode: a skipped over-long line makes find INCOMPLETE and the exit code 3 | [batch](EVIDENCE-BATCH.md) | `a_skipped_long_line_is_incomplete_with_exit_three` |
| P03 | Batch mode: the first ERROR stops the batch and later commands are counted as not run | [batch](EVIDENCE-BATCH.md) | `the_first_error_stops_the_batch_and_the_rest_is_counted` |
| P04 | Batch mode: over-long and non-UTF-8 command lines are errors, not silent skips | [batch](EVIDENCE-BATCH.md) | `over_long_and_non_utf8_commands_are_errors_not_silent_skips` |
| P05 | clear forgets the collection in memory only; saved snapshots still reopen by pin | [batch](EVIDENCE-BATCH.md) | `clear_forgets_memory_only_and_saved_snapshots_still_reopen` `saved_collections_reopen_by_pin_in_batch_mode` |
| P06 | A backup without a committed manifest is INCOMPLETE and cannot be restored | [backup](BACKUP.md) | `backup::tests::every_interrupted_state_is_incomplete_and_refused` `an_unfinished_backup_is_not_restorable_and_usage_errors_write_no_result` |
| P07 | Restore writes only to a path that does not exist | [backup](BACKUP.md) | `backup::tests::restore_never_replaces_an_existing_target` |
| P08 | A failed restore is classified by what is at the target: not published or published unconfirmed | [backup](BACKUP.md) | `backup::tests::a_failed_publication_is_classified_by_what_is_at_the_target` |
| P09 | Tampered, foreign or unexpected backup contents are not restorable | [backup](BACKUP.md) | `backup::tests::tampering_foreign_backups_and_unexpected_files_are_not_restorable` `backup::tests::the_manifest_parser_accepts_only_its_own_format` |
| P10 | Withdraw keeps the backed-up bytes and makes restore refuse | [backup](BACKUP.md) | `backup::tests::withdrawn_backups_keep_their_bytes_and_refuse_restore` |
| P11 | Delete removes only the file matching the pin; create refuses a wrong pin and an existing directory | [backup](BACKUP.md) | `backup::tests::delete_removes_only_the_pinned_file` `backup::tests::creation_refuses_a_wrong_pin_and_an_existing_directory` |
| P12 | The backup command line keeps distinct exit codes and accepts paths with spaces | [backup](BACKUP.md) | `full_cycle_with_spaces_in_paths_and_distinct_exit_codes` |
| P13 | Network isolation is reported only for loopback alone with no route | [isolated reproduction](REPRODUCE-ISOLATED.md) | `isolation::tests::a_host_with_network_interfaces_is_not_isolated` `isolation::tests::a_route_through_another_device_fails_even_with_loopback_listed_alone` `isolation::tests::loopback_alone_without_routes_passes` |
| P14 | Only a network-unreachable error counts as a blocked connection; reports count interfaces instead of naming them | [isolated reproduction](REPRODUCE-ISOLATED.md) | `isolation::tests::only_network_unreachable_counts_as_blocked` `isolation::tests::shared_reports_count_interfaces_instead_of_naming_them` |
| P15 | In strict reproduction a skipped step is a failure | [reproduction](REPRODUCE.md) | `reproduction::tests::a_skip_passes_only_outside_strict_mode` |
| P16 | The grep comparison calls answers the same only with equal counts and contained lines; percentiles are nearest-rank | [grep comparison](BENCHMARK-GREP.md) | `bench_compare::tests::only_equal_counts_with_contained_lines_are_the_same_answer` `bench_compare::tests::nearest_rank_percentiles` |
| P17 | The binary package check reads license files from the reviewed inventory, keeps only registry crates and scans both path separators | [binaries](BINARIES.md) | `package::tests::inventory_reads_license_files_and_skips_the_archive` `package::tests::tree_lines_keep_only_registry_crates` `package::tests::needles_cover_both_separators_and_ignore_short_paths` |
| P18 | The mutation campaign uses a memory sample only after it passed and compares timings in pairs | [mutation comparison](MUTATION-COMPARISON.md) | `mutation_campaign::tests::a_memory_row_is_read_only_after_a_passing_sample` `mutation_campaign::tests::nearest_rank_and_pairing` `memory_tests::proc_units_and_missing_values_are_checked` |
| P19 | Every byte change and truncation of a collection snapshot is rejected with the original pin, and a malformed structure is rejected even with a matching pin | [collection builder](COLLECTION-BUILDER.md) | `every_byte_mutation_and_truncation_rejected_with_original_pin` `malformed_structure_rejected_even_with_matching_pin` |
| P20 | A rejected change leaves the collection unchanged | [collection builder](COLLECTION-BUILDER.md) | `rejected_mutations_are_transactional` `exhausted_revision_and_id_counters_reject_without_mutation` |
| P21 | Replace, delete and add invalidate earlier hits | [update and restart](UPDATE-RESTART-SCENARIO.md) | `replace_delete_and_unrelated_add_invalidate_hits` |
| P22 | A search that skipped an over-long line is INCOMPLETE, and result caps do not hide total counts | [batch](EVIDENCE-BATCH.md) | `skipped_lines_are_incomplete_and_empty_bank_validates_query` `search_caps_do_not_hide_total_counts` |
| P23 | Publication never replaces an existing file or directory, and competing writers have exactly one winner | [publication faults](PUBLICATION-FAULT-TESTS.md) | `existing_file_is_never_overwritten` `existing_directory_is_not_replaced` `competing_writers_have_exactly_one_winner` `concurrent_publish_has_one_winner` |
| P24 | I/O failures, SIGKILL and permission denial keep the previous snapshot readable | [publication faults](PUBLICATION-FAULT-TESTS.md) | `bundle::publication_fault_tests::all_io_failures_preserve_old_snapshot_and_expose_only_complete_new_file` `sigkill_after_save_ack_keeps_a_readable_snapshot` `sigkill_at_observable_publication_never_installs_a_partial_snapshot` `kernel_permission_denial_keeps_previous_snapshot_and_leaves_no_file` |
| P25 | After an update and restart, stale citations are refused, old snapshots stay unchanged and a corrupted copy is refused without replacing live documents | [update and restart](UPDATE-RESTART-SCENARIO.md) | `complete_update_restart_and_corruption_scenario` `corrupt_load_refusal_retains_current_collection` `well_hashed_invalid_snapshot_does_not_replace_live_documents` |
| P26 | Control characters in documents never reach the terminal as escapes | [security](../SECURITY.md) | `controls_in_document_never_become_terminal_escape` `tests::terminal_controls_are_escaped` `cli_does_not_emit_terminal_escape_from_text` |
| P27 | Search matches whole tokens, never substrings, and never across lines or documents | [document readout](DOCUMENT-READOUT.md) | `document::tests::whole_tokens_not_substrings_and_no_cross_line_inference` `no_cross_document_phrase` |
| P28 | Original Unicode offsets survive normalization, and sigma variants match without rewriting the source | [document readout](DOCUMENT-READOUT.md) | `document::tests::original_unicode_offsets_survive_normalization` `document::tests::sigma_variants_match_without_rewriting_source` |
| P29 | Quote context is bounded and keeps negation, condition, unit and heading | [quote context](QUOTE-CONTEXT.md) | `context::tests::preserves_negation_condition_unit_and_heading_across_mixed_lines` `context::tests::long_context_is_bounded_and_honestly_truncated` |
| P30 | Every header bit flip, payload byte flip and truncation of a .gel file is rejected, and an oversized record count is rejected before allocation | [security](../SECURITY.md) | `tests::every_single_header_bit_flip_is_rejected` `tests::every_payload_byte_flip_is_rejected` `tests::every_truncation_length_is_rejected` `tests::oversized_record_count_is_rejected_before_allocation` |
| P31 | A printed PASS never overrides a failed exit status, and a native failure stops later steps | [CI evidence](CI-EVIDENCE.md) | `ci_evidence::tests::printed_success_never_overrides_exit_failure_or_missing_evidence` `first_and_middle_native_failure_stop_later_success` |
| P32 | The film recorder fails closed | [recorder safety](RECORDER-SAFETY.md) | `recorder_existing_log_missing_executable_and_eof_fail_closed` `record_support::tests::every_utf8_byte_split_is_safe` |
| P33 | The dependency inventory matches every locked external package | [dependency inventory](DEPENDENCY-INVENTORY.md) | `inventory_matches_every_locked_external_package` `rejects_stale_extra_duplicate_or_missing_inventory_entries` |
| P34 | The file gate rejects symlinks and executable files, and a reviewed asset must have its exact bytes and path | [gates](GATES.md) | `tests::rust_only_gate_rejects_symlinks_and_executable_files` `tests::documentation_png_gate_requires_exact_reviewed_bytes_and_path` |
| P35 | The claims registry and its documentation agree, and an open row cannot declare itself passed | [claims](CLAIMS.md) | `claims::tests::documentation_and_registry_agree` `claims::tests::every_open_row_uses_an_allowed_mode` |

Checks outside the table:

- The format mutation matrix runs inside `xtask verify` and must equal its
  recorded report ([mutation matrix](MUTATION-MATRIX.md)).
- The CI workflow runs the network-isolation check twice: it must fail on the
  networked runner and pass inside a network namespace
  ([isolated reproduction](REPRODUCE-ISOLATED.md)).
- The CI workflow publishes a snapshot into a 1 MiB filesystem that is
  physically full ([publication faults](PUBLICATION-FAULT-TESTS.md)).
- The binaries workflow runs the packaged programs on each platform
  ([binaries](BINARIES.md)).
