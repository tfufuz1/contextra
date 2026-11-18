# Dead Code & Unused Item Microprobe Diagnostic

> **Werkzeug:** `dead_code_microprobe.py`  
> **Erstellungszeit:** `2026-11-18T18:01:33.875379+00:00`  
> **Status/Befunde:** `502 Befunde erfasst`  

## Beschreibung
Identifies unreferenced private functions, types, constants, #[allow(dead_code)] suppressions, and deprecated items across crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `dead_code_candidates` | `421` |
| `allow_dead_code_attributes` | `47` |
| `deprecated_attributes` | `34` |
| `total_findings` | `502` |

## Zusammenfassung der technischen Schulden
| Kategorie | Anzahl | Beschreibung |
|---|---|---|
| Unused Private Items | 421 | Private Funktionen, Structs oder Consts ohne Aufrufer im Crate |
| `#[allow(dead_code)]` | 47 | Unterdrückte Dead-Code-Warnungen |
| `#[deprecated]` | 34 | Veraltete Relikte zur Entfernung |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra` | `crates/contextra/src/builder.rs` | `247` | Deprecated item marked for removal |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `125` | Deprecated item marked for removal |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `133` | Deprecated item marked for removal |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `176` | Deprecated item marked for removal |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `209` | Private function 'check_ring' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `17` | Private function 'tenant_a_cannot_read_tenant_b' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `131` | Private function 'startup_ssi_tracking_check' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/type_identity_test.rs` | `48` | Private function 'check_err' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/type_identity_test.rs` | `51` | Private function 'check_metric' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/type_identity_test.rs` | `54` | Private function 'check_doc_id' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `50` | Private function 'vertical_path_test_3_ssi_under_write_load_b16' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `22` | Private function 'flush_preserves_pinned_snapshot' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `60` | Private function 'second_open_same_directory_rejected' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `77` | Private function 'open_compliance_profile_succeeds_without_license' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `96` | Private function 'profile_durability_reaches_lsm_config' appears unused in crate (freq=1) |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `119` | Private function 'confirmed_commit_survives_drop_reopen' appears unused in crate (freq=1) |
| `contextra-bench` | `benchmarks/contextra-bench/src/main.rs` | `890` | Private function 'update_current_metrics_locomo' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `95` | Private function 'default_id_width_bits' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `16` | Private function 'create_named_checkpoint' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `29` | Private function 'restore_named_checkpoint' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `33` | Private function 'drop_named_checkpoint' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `37` | Private function 'list_named_checkpoints' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `45` | Private function 'take_snapshot' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | `55` | Private function 'restore' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/checkpoint_systematic_crash.rs` | `137` | Private function 'systematic_crash_at_every_checkpoint_io_point' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/checkpoint_systematic_crash.rs` | `200` | Private function 'crash_during_checkpoint_deletion_leaves_clean_state' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/golden_corpus_test.rs` | `146` | Private function 'regen_golden_corpus' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/golden_generator.rs` | `7` | Private function 'generate_golden_files' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_proptest.rs` | `137` | Private function 'prop_guard_random_lifecycle_sequences' appears unused in crate (freq=1) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_proptest.rs` | `177` | Private function 'prop_manifest_checksum_integrity' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/campaign_mut_j17.rs` | `12` | Private function 'killer_test_kdf_params_new_for_test' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/campaign_mut_j17.rs` | `22` | Private function 'killer_test_kdf_header_to_bytes_exact_length_and_magic' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/campaign_mut_j17.rs` | `46` | Private function 'killer_test_kdf_header_from_bytes_boundary_validations' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/campaign_mut_j17.rs` | `85` | Private function 'killer_test_key_manager_kv_cipher_seal_open_roundtrip' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/campaign_mut_j17.rs` | `112` | Private function 'killer_test_kv_segment_cipher_kv_cipher_seal_open' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/crypto_property_and_invariants.rs` | `20` | Private function 'prop_aead_seal_open_roundtrip' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/crypto_property_and_invariants.rs` | `37` | Private function 'prop_bit_mutation_rejects' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs` | `16` | Private function 'eviction_worker_drop_does_not_starve_other_tasks_indefinitely' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/golden_corpus.rs` | `242` | Private function 'generate_golden_files' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/kv_segment_proptests.rs` | `22` | Private function 'prop_kv_segment_creation' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/kv_segment_proptests.rs` | `38` | Private function 'prop_tenant_isolation_strictness' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/kv_segment_proptests.rs` | `79` | Private function 'prop_segment_zeroize_wipes_all_bytes' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptest_wal_hmac_v2_collision.rs` | `86` | Private function 'v2_hmac_is_not_length_prefixed_documented_gap' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptest_wal_hmac_v2_collision.rs` | `141` | Private function 'v3_hmac_is_length_prefixed_no_collision' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptest_wal_hmac_v2_collision.rs` | `214` | Private function 'prop_v2_arbitrary_boundary_shift_collides' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `18` | Private function 'prop_encrypt_decrypt_roundtrip' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `26` | Private function 'prop_ciphertext_bit_flip_authenticity_failure' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `44` | Private function 'prop_kv_segment_cipher_roundtrip' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `65` | Private function 'prop_kv_segment_cipher_mismatch_fails_decrypt' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `85` | Private function 'prop_kv_segment_cipher_freshness_nonce_and_ciphertext' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `103` | Private function 'prop_encrypted_wal_roundtrip' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/proptests.rs` | `115` | Private function 'prop_integrity_verifier_v3_valid_and_tampered' appears unused in crate (freq=1) |
| `contextra-crypto` | `crates/contextra-crypto/tests/wal_completeness_tests.rs` | `40` | Private function 'prop_wal_chain_completeness_equivalence' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/src/homeostat.rs` | `31` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/background_workers/expiry_workers.rs` | `74` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/background_workers/expiry_workers.rs` | `113` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/background_workers/orphan_workers.rs` | `188` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/transaction/compensating_actions.rs` | `205` | Explicit #[allow(dead_code)] annotation |
| `contextra-db` | `crates/contextra-db/src/transaction/db_transaction_rollback.rs` | `14` | Explicit #[allow(dead_code)] annotation |
| `contextra-db` | `crates/contextra-db/src/collection/mod.rs` | `103` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/crud/auto_extraction.rs` | `85` | Explicit #[allow(dead_code)] annotation |
| `contextra-db` | `crates/contextra-db/src/collection/search/basic.rs` | `20` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/basic.rs` | `38` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/basic.rs` | `55` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/basic.rs` | `101` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/filtered.rs` | `178` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/filtered.rs` | `197` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/hybrid/mod.rs` | `20` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/hybrid/mod.rs` | `42` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/hybrid/mod.rs` | `65` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/hybrid/mod.rs` | `90` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/src/collection/search/hybrid/query.rs` | `42` | Deprecated item marked for removal |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `54` | Explicit #[allow(dead_code)] annotation |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `91` | Private function 'provider_name' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `97` | Private function 'embedding_dim' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `134` | Private function 'embed_batch' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `200` | Private function 'chaos_gpu_busy_disk_full_no_orphaned_embedding' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/fusion_edge_cases_test.rs` | `603` | Private function 'prop_replicator_weight_sum_invariant_preserved_through_fusion' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/proptest_search_invariants.rs` | `51` | Private function 'prop_rrf_bounded_by_k' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/proptest_search_invariants.rs` | `68` | Private function 'prop_e2e_search_bounded_by_k' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/proptest_search_invariants.rs` | `141` | Private function 'prop_bm25_score_monotonic_in_tf' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/proptest_search_invariants.rs` | `175` | Private function 'prop_rrf_scores_sorted_descending' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/proptest_search_invariants.rs` | `202` | Private function 'prop_rrf_empty_inputs_never_panic' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `55` | Private function 'read_search_rs' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `75` | Private function 'proof_search_never_exceeds_k' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `144` | Private function 'proof_search_returns_nonzero_results_for_matching_query' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `180` | Private function 'proof_search_with_k_zero_returns_empty' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/search_result_bound.rs` | `211` | Private function 'proof_usize_max_removed_from_search_path' appears unused in crate (freq=1) |
| `contextra-db` | `crates/contextra-db/tests/split_brain_vector_orphan_test.rs` | `364` | Private function 'prop_no_dangling_search_result_under_random_insert_delete_sequence' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `6` | Deprecated item marked for removal |
| `contextra-engine` | `crates/contextra-engine/tests/chunker_hardening_tests.rs` | `136` | Private function 'prop_source_span_matches_unsynthesized_content' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/decay_and_expiry.rs` | `128` | Private function 'monotonic_nanos' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/metrics_instrumentation.rs` | `19` | Private function 'record_counter' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/metrics_instrumentation.rs` | `23` | Private function 'record_gauge' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/metrics_instrumentation.rs` | `27` | Private function 'record_histogram' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/proof_chunker_properties.rs` | `71` | Private function 'prop_chunker_arbitrary_utf8_safety' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/proof_temporal_filter_vs_rank.rs` | `160` | Private function 'prop_temporal_filter_differential_proptest' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/query_builder_scope_acorn.rs` | `93` | Private function 'search_knn_acorn' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/s64_public_surface.rs` | `12` | Private function '_pin_db_consumer_surface' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/s64_public_surface.rs` | `34` | Private function '_pin_language_consumer_surface' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/s64_public_surface.rs` | `42` | Private function '_pin_engine_public_surface' appears unused in crate (freq=1) |
| `contextra-engine` | `crates/contextra-engine/tests/write_skew_via_engine_api.rs` | `125` | Private function 'prop_concurrent_engine_transactions_serializable' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `247` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `275` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `227` | Explicit #[allow(dead_code)] annotation |
| `contextra-erasure` | `crates/contextra-erasure/src/measure.rs` | `36` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/measure.rs` | `45` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/measure.rs` | `62` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/measure.rs` | `96` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/measure.rs` | `116` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/src/measure/scanner_flow.rs` | `186` | Deprecated item marked for removal |
| `contextra-erasure` | `crates/contextra-erasure/tests/crash_matrix_recovery.rs` | `211` | Private function 't_crash_1_fm_matrix_recovery_step_progression' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/crash_matrix_recovery.rs` | `272` | Private function 't_crash_3_fm_3_disk_reserve_pending' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | `16` | Private function 'verify_external_accepts_valid_proof' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | `46` | Private function 'verify_external_rejects_tampered_timestamp' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | `80` | Private function 'verify_external_rejects_tampered_deleted_keys_hash' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | `114` | Private function 'verify_external_rejects_wrong_public_key' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | `142` | Private function 'verify_external_rejects_legacy_hmac_proof' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/deletion_proof_prefix_collision_proptest.rs` | `39` | Private function 'no_prefix_collision' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `66` | Private function 't_gate_1_immediate_rejection_after_persist' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `93` | Private function 't_gate_2_linearized_read_and_gate_epoch' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `131` | Private function 't_gate_3_state_table_per_read_path' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `160` | Private function 't_gategroup_1_concurrent_erases_share_barrier_phase' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `214` | Private function 't_ver_1_replay_filter_and_78_byte_matcher' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/golden_erasure_test.rs` | `119` | Private function 'generate_golden_files' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `179` | Private function 't_crash_1_durable_journal_roundtrip_and_fsync' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `225` | Private function 't_crash_2_torn_tail_recovery' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `264` | Private function 't_crash_4_recover_open_sagas_conservative_gate' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `297` | Private function 't_jrn_3_reduction_after_completed_r13' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `366` | Private function 't_erid_1_p13_no_plaintext_keys_or_ids_in_journal' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_erasure_models.rs` | `48` | Private function 'loom_verify_fence_writer_mutual_exclusion' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_erasure_models.rs` | `73` | Private function 'loom_tests_disabled_without_cfg' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_gate.rs` | `70` | Private function 't_conc_1_loom_gate_linearized_reader' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_gate.rs` | `106` | Private function 't_conc_1_stress_concurrent_reads_and_gate_linearization' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/measure_real_scan_tests.rs` | `207` | Private function 't_raw_2_missing_positive_control_fails_closed' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/measure_real_scan_tests.rs` | `234` | Private function 't_raw_1_and_4_scan_finds_needle_in_tmp_and_old_segments' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/measure_real_scan_tests.rs` | `290` | Private function 't_mut_1_and_t_keycov_1_mk3_broken_slot_detected' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/measure_real_scan_tests.rs` | `342` | Private function 't_raw_1_and_5_structure_decoder_injection' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p1_key_hierarchy_and_purge.rs` | `26` | Private function 'kek_destruction_makes_all_deks_undecryptable' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p1_key_hierarchy_and_purge.rs` | `77` | Private function 'dek_never_plaintext_outside_zeroized_memory' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p1_key_hierarchy_and_purge.rs` | `94` | Private function 'epoch_isolation' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p1_key_hierarchy_and_purge.rs` | `134` | Private function 'purge_keys_idempotent' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p1_key_hierarchy_and_purge.rs` | `160` | Private function 'inv_no_resurrection_1_purge_keys_survives_double_call_without_resurrection' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p2_vault_rewrite_tests.rs` | `21` | Private function 'swap_error_leaves_no_plaintext' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p2_vault_rewrite_tests.rs` | `141` | Private function 'rewrite_crash_at_every_step_never_exposes_plaintext' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p2_vault_rewrite_tests.rs` | `219` | Private function 'rewrite_io_error_poisons_handle_permanently' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p2_vault_rewrite_tests.rs` | `281` | Private function 'manifest_matches_journal_after_recovery' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p3_purge_and_scope_tests.rs` | `158` | Private function 'purge_phase_transition_matrix_is_exhaustive' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p3_purge_and_scope_tests.rs` | `221` | Private function 'inv_era_3_single_flight_after_simulated_crash_between_two_starts' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p3_purge_and_scope_tests.rs` | `383` | Private function 'stale_lock_is_recovered_without_double_purge' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p5_wal_sink_tests.rs` | `45` | Private function 'inv_era_6_wal_read_after_scope_purge_has_no_plaintext' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p5_wal_sink_tests.rs` | `91` | Private function 'dispose_is_idempotent_and_resumable' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p5_wal_sink_tests.rs` | `124` | Private function 'other_scopes_wal_untouched' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p5_wal_sink_tests.rs` | `159` | Private function 'sink_error_is_fail_closed' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p5_wal_sink_tests.rs` | `209` | Private function 'wal_sink_error_variants_and_display' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p6_ledger_durability.rs` | `89` | Private function 'inv_era_4_key_destruction_is_anchored_before_saga_end' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p6_ledger_durability.rs` | `118` | Private function 'open_missing_ledger_fails_closed' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p6_ledger_durability.rs` | `142` | Private function 'open_corrupted_ledger_fails_closed' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p6_ledger_durability.rs` | `204` | Private function 'broken_hash_chain_detected' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p6_ledger_durability.rs` | `330` | Private function 'atomic_batch_append_crash_leaves_no_partial_batch' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `28` | Private function 'inv_era_7_report_binds_witness_chain' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `56` | Private function 'tamper_removed_reordered_modified_witness_rejected' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `119` | Private function 'canonical_form_is_deterministic' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `172` | Private function 'report_without_verified_witness_refused' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `214` | Private function 'report_text_contains_no_binding_guarantee_phrases' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p7_report_chain_tests.rs` | `251` | Private function 'existing_external_verify_test_still_green' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `27` | Private function 'positive_control_finds_known_residue' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `41` | Private function 'negative_control_after_erasure_finds_nothing' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `73` | Private function 'scan_without_working_positive_control_is_invalid' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `100` | Private function 'scanner_error_never_reported_as_clean' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `136` | Private function 'inv_class_measured_1_no_witness_without_positive_control_and_zero_residue' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `164` | Private function 'inv_no_resurrection_1_erased_entry_cannot_reappear' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `191` | Private function 'inv_vault_failclosed_1_scanner_error_fails_closed' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/p8_residue_scan_tests.rs` | `217` | Private function 'inv_claim_bind_1_no_legal_warranty_wording' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/proof_fail_closed_and_consistency.rs` | `97` | Private function 'prop_wal_sink_monotonicity_and_durability' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `403` | Private function 't_crash_1_orchestration_full_flow_steps_1_to_6' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `429` | Private function 't_sla_1_orchestration_batched_returns_gated' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `453` | Private function 't_status_1_retention_matrix_refusal' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `475` | Private function 't_restr_1_retention_matrix_restricted_mode' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `500` | Private function 't_cancel_1_caller_abort' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/saga_core_orchestration.rs` | `525` | Private function 't_crash_1_witness_missing_residue_pending' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/scope_codec_spec_tests.rs` | `149` | Private function 't_crash_1_scope_binary_codec_roundtrip' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/scope_codec_spec_tests.rs` | `168` | Private function 't_erid_1_scope_file_persistence_threshold' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_gate.rs` | `9` | Private function 't_gate_1_erased_set_checks_and_compaction_policy' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_gate.rs` | `64` | Private function 't_restr_1_lift_restriction' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_measure.rs` | `17` | Private function 't_raw_2_positive_control_and_witness_generation' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_measure.rs` | `50` | Private function 't_raw_2_positive_control_genuine_scan_and_mutants' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_measure.rs` | `72` | Private function 't_raw_2_typed_scanner_flow' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_observer.rs` | `27` | Private function 't_obs_1_2_3_observer_lifecycle' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_observer.rs` | `50` | Private function 't_crash_3_disk_reserve_check' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_report_ledger.rs` | `15` | Private function 't_report_1_canonical_json_and_signature' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_report_ledger.rs` | `30` | Private function 't_restore_1_ledger_and_guard' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_report_ledger.rs` | `57` | Private function 't_report_residual_risks_and_appendix_d' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_saga_journal.rs` | `18` | Private function 't_crash_1_and_idempotence' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_saga_journal.rs` | `48` | Private function 't_driver_linearization_and_advance' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_saga_journal.rs` | `84` | Private function 't_cancel_1_and_journal_persistence' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_saga_journal.rs` | `104` | Private function 't_crash_matrix_recovery_table' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_sealer_keyring.rs` | `13` | Private function 't_fk_1_keyring_create_open_destroy' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_sealer_keyring.rs` | `40` | Private function 't_vault_2_and_4_sealer_seal_open_evict' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_vault_core.rs` | `17` | Private function 't_vault_1_slot_destruction_and_verification' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_vault_core.rs` | `47` | Private function 't_vault_3_constants_and_layout' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_vault_core.rs` | `55` | Private function 't_vault_5_probing_and_load_factor' appears unused in crate (freq=1) |
| `contextra-erasure` | `crates/contextra-erasure/tests/t_vault_core.rs` | `94` | Private function 't_vault_1_and_t_crash_5_slot_tombstone_verification' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/ppr/tests/algo_tests.rs` | `838` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/ppr/tests/algo_tests.rs` | `853` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/csr/types.rs` | `18` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/csr/types.rs` | `20` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/csr/types.rs` | `22` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/csr/types.rs` | `24` | Explicit #[allow(dead_code)] annotation |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `247` | Private function 'default_tl_hfd_sigma' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `248` | Private function 'default_tl_hfd_delta' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `249` | Private function 'default_tl_hfd_gamma' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `250` | Private function 'default_tl_hfd_max_iterations' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `251` | Private function 'default_tl_hfd_max_top_k_expansion' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `252` | Private function 'default_tl_hfd_max_hyperedge_sort_size' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/types.rs` | `291` | Private function 'default_warn_on_non_convergence' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/path_rag/k_path.rs` | `75` | Private function 'eq' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner/mod.rs` | `174` | Private function 'deref_mut' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/csr_complexity_bench.rs` | `14` | Private function 'bench_single_compaction_scaling' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/csr_complexity_bench.rs` | `83` | Private function 'bench_amortized_1m_edge_inserts' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/golden_generator.rs` | `11` | Private function 'generate_golden_files' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/hyperedge_suggest_tests.rs` | `313` | Private function 'prop_candidates_participants_bounds' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/hyperedge_suggest_tests.rs` | `349` | Private function 'prop_output_candidates_are_canonically_sorted' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/hyperedge_suggest_tests.rs` | `368` | Private function 'prop_idempotency_of_candidate_computation' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/leiden_reference_validation.rs` | `125` | Private function 'leiden_on_karate_club_matches_reference' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/leiden_reference_validation.rs` | `158` | Private function 'leiden_is_deterministic_with_fixed_seed' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/persistence_test.rs` | `123` | Private function 'edges_survive_restart' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_csr_invariants.rs` | `21` | Private function 'prop_1_adjacency_after_compact_has_no_tombstones' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_csr_invariants.rs` | `129` | Private function 'prop_2_edge_weights_non_negative' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_csr_invariants.rs` | `192` | Private function 'prop_3_source_doc_ids_not_none_after_compact' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_csr_invariants.rs` | `262` | Private function 'prop_4_compact_is_idempotent' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_csr_invariants.rs` | `336` | Private function 'prop_5_node_count_monotonically_non_decreasing' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_diffusion.rs` | `128` | Private function 'prop_tl_hfd_diffusion_objective_is_monotonically_non_increasing' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_diffusion.rs` | `194` | Private function 'prop_tl_hfd_diffusion_matches_bruteforce_reference_small_graph' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_lovasz.rs` | `77` | Private function 'prop_lovasz_extension_is_nonnegative' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_lovasz.rs` | `90` | Private function 'prop_lovasz_extension_matches_bruteforce_reference' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_lovasz.rs` | `111` | Private function 'prop_lovasz_extension_max_min_are_distinct_or_none' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_lovasz.rs` | `149` | Private function 'prop_truncate_participants_preserves_min_and_top_k' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/proptest_tl_hfd_lovasz.rs` | `209` | Private function 'prop_truncate_participants_result_length_bounded' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/source_doc_ids_populated.rs` | `13` | Private function 'proof_source_doc_ids_set_after_compact' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/source_doc_ids_populated.rs` | `64` | Private function 'proof_source_doc_ids_empty_before_first_compact' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/source_doc_ids_populated.rs` | `101` | Private function 'proof_source_doc_ids_consistent_after_multiple_compacts' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/stress_dag.rs` | `20` | Private function 'stress_session_dag_deadlock_freedom' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/type_parity.rs` | `103` | Private function 'prop_entity_id_bincode_parity' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/type_parity.rs` | `111` | Private function 'prop_doc_id_bincode_parity' appears unused in crate (freq=1) |
| `contextra-graph` | `crates/contextra-graph/tests/type_parity.rs` | `119` | Private function 'prop_from_key_parity' appears unused in crate (freq=1) |
| `contextra-ids` | `crates/contextra-ids/src/lib.rs` | `89` | Deprecated item marked for removal |
| `contextra-ids` | `crates/contextra-ids/src/lib.rs` | `436` | Deprecated item marked for removal |
| `contextra-ids` | `crates/contextra-ids/src/lib.rs` | `138` | Private function 'try_from' appears unused in crate (freq=1) |
| `contextra-mcp` | `crates/contextra-mcp/src/explain.rs` | `22` | Private function 'default_collection' appears unused in crate (freq=1) |
| `contextra-mcp` | `crates/contextra-mcp/src/prompt_injection.rs` | `19` | Private function 'now_unix_nanos' appears unused in crate (freq=1) |
| `contextra-mcp` | `crates/contextra-mcp/tests/mcp_test.rs` | `703` | Private function 'mcp_parse_error_returns_rpc_32700' appears unused in crate (freq=1) |
| `contextra-mcp` | `crates/contextra-mcp/tests/mcp_test.rs` | `723` | Private function 'mcp_method_not_found_returns_32601' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/golden_corpus.rs` | `204` | Private function 'generate_golden_mvcc' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_coarsening_proptest.rs` | `18` | Private function 'prop_ssi_coarsening_zero_false_negatives' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_determinism_proptest.rs` | `79` | Private function 'prop_ssi_validation_is_100_percent_deterministic' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_forget_and_bounds.rs` | `15` | Private function 'forget_from_removes_keys_of_failed_batch' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_forget_and_bounds.rs` | `43` | Private function 'committed_writes_bounded' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_forget_and_bounds.rs` | `72` | Private function 'phantom_prefix_conflict' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_hardening.rs` | `157` | Private function 'prop_prune_equivalence_above_watermark' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_metrics_observability.rs` | `52` | Private function 'record_counter' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_metrics_observability.rs` | `76` | Private function 'record_histogram' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/tx_buffer_byte_budget.rs` | `15` | Private function 'tx_buffer_byte_budget_exceeded' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/tx_buffer_byte_budget.rs` | `52` | Private function 'tx_buffer_byte_budget_freed_on_drain_discard_reap' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/visibility_proptest.rs` | `23` | Private function 'prop_mvcc_visibility_matches_reference_10k' appears unused in crate (freq=1) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/visibility_proptest.rs` | `74` | Private function 'prop_mvcc_visibility_explicit_boundary_cases' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/conformance.rs` | `7` | Explicit #[allow(dead_code)] annotation |
| `contextra-ports` | `crates/contextra-ports/src/key_material.rs` | `37` | Private function 'fill_key_bytes' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/license.rs` | `158` | Private function '_doctest_authorized_ring_private_fields' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/src/storage.rs` | `185` | Private function 'flush' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/tests/generate_golden_env.rs` | `9` | Private function 'generate_golden_env_data' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/tests/plugin_registry_dependency_cycle.rs` | `78` | Private function 'prop_dependency_cycle_detected_and_no_plugins_activated' appears unused in crate (freq=1) |
| `contextra-ports` | `crates/contextra-ports/tests/plugin_registry_dependency_cycle.rs` | `143` | Private function 'prop_dag_dependencies_activated_successfully' appears unused in crate (freq=1) |
| `contextra-privacy` | `crates/contextra-privacy/src/egress_gateway.rs` | `397` | Deprecated item marked for removal |
| `contextra-privacy` | `crates/contextra-privacy/src/egress_gateway.rs` | `429` | Deprecated item marked for removal |
| `contextra-rank` | `crates/contextra-rank/src/fusion/types.rs` | `20` | Explicit #[allow(dead_code)] annotation |
| `contextra-rank` | `crates/contextra-rank/src/calibration/isotonic.rs` | `65` | Explicit #[allow(dead_code)] annotation |
| `contextra-rank` | `crates/contextra-rank/src/calibration/isotonic.rs` | `67` | Explicit #[allow(dead_code)] annotation |
| `contextra-rank` | `crates/contextra-rank/src/calibration/isotonic.rs` | `90` | Private function '_default_clock' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/calibration_deep_tests.rs` | `284` | Private function 'prop_platt_scaler_bounded_output' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/calibration_drift_property_tests.rs` | `11` | Private function 'prop_isotonic_pava_monotonicity' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/calibration_drift_property_tests.rs` | `42` | Private function 'prop_platt_scaling_range_bounded' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/calibration_drift_property_tests.rs` | `54` | Private function 'prop_drift_detector_stationary_no_false_alarms' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs` | `71` | Private function 'prop_isotonic_ece_non_negative_and_bounded' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/fusion_deep_property_tests.rs` | `30` | Private function 'prop_mrrf_weights_renormalized' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/fusion_deep_property_tests.rs` | `97` | Private function 'prop_score_normalized_minmax_clamping' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/fusion_deep_property_tests.rs` | `135` | Private function 'prop_global_fusion_strategy_community_cap' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/fusion_tie_break_proof.rs` | `122` | Private function 'prop_fusion_tie_break_alphabetical' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/golden_tests.rs` | `184` | Private function 'regen_fixtures' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/isotonic_mutation_hardening_test.rs` | `162` | Private function 'prop_pava_monotonicity_holds_for_arbitrary_unordered_input' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rank_invariants.rs` | `332` | Private function 'prop_inv_rank_monotonic_1' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rank_invariants.rs` | `358` | Private function 'prop_rrf_commutativity' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `28` | Private function 'prop_rrf_empty_lists_return_empty' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `36` | Private function 'prop_rrf_monotonicity_single_list' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `57` | Private function 'prop_rrf_permutation_invariance' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `102` | Private function 'prop_rrf_k_boundary_values' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `128` | Private function 'prop_rrf_duplicate_ids_in_single_list' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `144` | Private function 'prop_weighted_rrf_permutation_invariance_arbitrary_docs' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_property_tests.rs` | `195` | Private function 'prop_weighted_rrf_tie_break_ascending_id' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_reference.rs` | `46` | Private function 'rrf_known_values' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_reference.rs` | `65` | Private function 'rrf_with_disjoint_lists' appears unused in crate (freq=1) |
| `contextra-rank` | `crates/contextra-rank/tests/rrf_reference.rs` | `90` | Private function 'rrf_with_three_lists' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `36` | Private function 'simd_vs_scalar_cosine_f32' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `59` | Private function 'simd_vs_scalar_euclidean_f32' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `82` | Private function 'simd_vs_scalar_dot_product_f32' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `105` | Private function 'simd_vs_scalar_cosine_f32_bytes' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `128` | Private function 'simd_vs_scalar_euclidean_f32_bytes' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `151` | Private function 'simd_vs_scalar_dot_product_f32_bytes' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `174` | Private function 'simd_vs_scalar_dot_product_u8' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `195` | Private function 'simd_vs_scalar_euclidean_sq_u8' appears unused in crate (freq=1) |
| `contextra-simd` | `crates/contextra-simd/tests/simd_cross_validation.rs` | `216` | Private function 'simd_vs_scalar_cosine_similarity_parts_u8' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `119` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `138` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `148` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `159` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `175` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/tenant_codec.rs` | `187` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/manifest/core.rs` | `16` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/manifest/core.rs` | `18` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `9` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/wal/hmac.rs` | `447` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/wal/replay.rs` | `169` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/compat/scanner/memtable.rs` | `32` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `568` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `341` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `346` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `385` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `400` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/maintenance.rs` | `22` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/support/mod.rs` | `6` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/support/mod.rs` | `16` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/support/mod.rs` | `30` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/support/mod.rs` | `47` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/support/mod.rs` | `59` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `14` | Explicit #[allow(dead_code)] annotation |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `200` | Private function 'poll_read' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `217` | Private function 'poll_write' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `225` | Private function 'poll_flush' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `231` | Private function 'poll_shutdown' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `240` | Private function 'start_seek' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/wal/fs.rs` | `255` | Private function 'poll_complete' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `569` | Private function '_check_partial_cmp' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `574` | Private function '_verify_partial_cmp' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/amplification_benchmark.rs` | `103` | Private function 'run_amplification_benchmark' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/budget_drift.rs` | `38` | Private function 'prop_memtable_budget_drift_is_zero' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/campaign_fuzzregress_memtable_concurrent.rs` | `15` | Private function 'campaign_fuzzregress_memtable_concurrent_out_of_order_seq' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/campaign_prop_lsm_ops.rs` | `244` | Private function 'prop_campaign_lsm_differential_simulation' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/campaign_prop_lsm_ops.rs` | `377` | Private function 'prop_counter_probe_scan_ordering_inversion' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/campaign_prop_lsm_ops.rs` | `401` | Private function 'prop_counter_probe_uncommitted_rollback_leakage' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/campaign_prop_lsm_ops.rs` | `421` | Private function 'prop_counter_probe_prefix_bound_overflow' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/chaos_bitflip_sstable.rs` | `174` | Private function 'prop_chaos_sstable_datablock_bitflip' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/chaos_bitflip_sstable.rs` | `257` | Private function 'prop_chaos_sstable_bloomfilter_bitflip' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/chaos_bitflip_sstable.rs` | `329` | Private function 'prop_chaos_sstable_index_bitflip' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/chaos_bitflip_sstable.rs` | `407` | Private function 'prop_chaos_sstable_trailer_bitflip' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/compaction_overlap_scan_and_tombstone.rs` | `115` | Private function 'prop_compaction_overlap_scan_and_tombstone' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/crash_kill_durability.rs` | `59` | Private function 'repro_bug_tx_end_duplicate_seq_no_wal_replay' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/crash_kill_durability.rs` | `103` | Private function 'child_writer' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/golden_corpus.rs` | `172` | Private function 'generate_golden_corpus' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit_handoff.rs` | `19` | Private function 'commit_mutex_handoff_no_lost_write' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit_handoff.rs` | `50` | Private function 'commit_mutex_handoff_preserves_commit_order' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/loom_guard_rollback.rs` | `60` | Private function 'loom_guard_drop_against_concurrent_append' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/manifest_corruption.rs` | `6` | Private function 'scenario1_mid_file_corruption_returns_err' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/manifest_corruption.rs` | `55` | Private function 'scenario2_tail_truncation_is_recoverable' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/manifest_corruption.rs` | `97` | Private function 'scenario3_prevent_sstable_resurrection' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/manifest_corruption.rs` | `141` | Private function 'scenario4_crc_header_corruption_returns_err' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/manifest_corruption.rs` | `177` | Private function 'scenario5_replace_mid_file_corruption_returns_err' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/model_based_mvcc_flush.rs` | `56` | Private function 'prop_model_based_mvcc_flush_simulation' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/proptest_model_based.rs` | `136` | Private function 'prop_model_based_lsm_simulation' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/proptest_sstable_zero_copy_block_decode.rs` | `74` | Private function 'proptest_sstable_zero_copy_block_decode' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/tenant_codec_properties.rs` | `8` | Private function 'prop_tenant_codec_chunk_key_injective' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/tenant_scoped_storage.rs` | `131` | Private function 'scan_prefix_never_crosses_tenant_boundary' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/toctou_put_if_absent.rs` | `76` | Private function 'proof_trait_default_removed' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/toctou_put_if_absent.rs` | `91` | Private function 'proof_put_if_absent_default_trait_removed' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/toctou_put_if_absent.rs` | `106` | Private function 'proof_put_if_absent_sees_uncommitted_staged_write' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/toctou_put_if_absent.rs` | `143` | Private function 'proof_put_if_absent_atomicity_under_contention' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/toctou_put_if_absent.rs` | `204` | Private function 'proof_put_if_absent_atomicity' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_legacy_key_tests.rs` | `11` | Private function 'record_debug' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_lifecycle_crash_point_matrix.rs` | `212` | Private function 'wal_crash_matrix_full_command_coverage' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_poison_causes.rs` | `16` | Private function 'fsync_fail_poisons_irreversibly' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_poison_causes.rs` | `64` | Private function 'set_len_fail_poisons' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_poison_causes.rs` | `113` | Private function 'rollback_contention_poisons_handle' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_poison_causes.rs` | `150` | Private function 'recover_from_poison_rejects_fsync_cause' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_poison_causes.rs` | `174` | Private function 'recover_from_poison_accepts_torn_write' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `29` | Private function 'spec_n12_wal_tail_truncation_detection_after_sstable_flush' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `113` | Private function 'spec_n12_cross_segment_hmac_chain_continuity' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `161` | Private function 'spec_n12_manifest_periodic_hwm_checkpoint_verification' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `197` | Private function 'spec_n13_v4_frame_header_crc_bitflip_fails_hard_middle_entry' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `247` | Private function 'spec_n13_v4_clean_torn_tail_distinguished_from_middle_corrupted_header' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_spec_pending.rs` | `294` | Private function 'spec_n13_v2_v3_to_v4_transparent_migration_and_compatibility' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_systematic_crash.rs` | `21` | Private function 'systematic_crash_at_every_wal_io_point' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_systematic_crash.rs` | `149` | Private function 'crash_exactly_at_fsync_after_commit_marker' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `26` | Private function 'proof_hmac_chain_valid_after_truncate_and_rewrite' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `85` | Private function 'proof_size_counter_consistent_after_truncate' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `123` | Private function 'proof_flusher_actor_exclusive_after_single_consumer_refactor' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `138` | Private function 'proof_concurrent_flush_and_truncate_no_panic' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `189` | Private function 'proof_wal_size_counter_consistent_after_failed_truncate' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/wal_truncate_ordering.rs` | `234` | Private function 'proof_wal_truncate_ordering_under_concurrent_flush' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `388` | Private function 'run_differential_large' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `651` | Private function 'task_op_strategy' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `666` | Private function 'run_concurrent_differential' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `790` | Private function 'differential_lsm_vs_redb' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `803` | Private function 'differential_redb_under_memory_pressure_large_values' appears unused in crate (freq=1) |
| `contextra-store` | `crates/contextra-store/tests/differential/redb_operation_sequence.rs` | `817` | Private function 'differential_redb_concurrent_compaction_under_load' appears unused in crate (freq=1) |
| `contextra-sys` | `crates/contextra-sys/src/mmap.rs` | `179` | Private function 'as_ref' appears unused in crate (freq=1) |
| `contextra-sys` | `crates/contextra-sys/src/vault.rs` | `201` | Private function 'deref_mut' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/src/wand.rs` | `24` | Private function 'eq' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/src/compat/scanner.rs` | `196` | Private function 'layer' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/src/compat/scanner.rs` | `200` | Private function 'scan_units' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/alloc_profiler.rs` | `242` | Private function 'run_profile_10k_documents' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/alloc_profiler.rs` | `510` | Private function 'alloc_budget_gate' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/audit_inspection.rs` | `15` | Private function 'inspect_20_compounds' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/audit_inspection.rs` | `56` | Private function 'inspect_umlaut_bimap' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/core_smoke.rs` | `127` | Private function 'core_smoke' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/golden_oracle_tests.rs` | `25` | Private function 'generate_golden_corpus' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/ids_env_diff_test.rs` | `15` | Private function 'monotonic_nanos' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/index_purge_proptests.rs` | `157` | Private function 'prop_compound_splitter_umlauts_and_sz_never_panic' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/index_purge_proptests.rs` | `167` | Private function 'prop_compound_splitter_hyphenated_never_panic' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/index_purge_proptests.rs` | `176` | Private function 'prop_tokenizer_case_and_normalization_determinism' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/pipeline_analyzer_tests.rs` | `131` | Private function 'prop_pipeline_arbitrary_utf8_never_panics_and_offsets_valid' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/pipeline_analyzer_tests.rs` | `153` | Private function 'prop_normalization_idempotency_arbitrary_utf8' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/proptest_bm25_invariants.rs` | `211` | Private function 'prop_1_bm25_score_strictly_positive_for_present_terms' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/proptest_bm25_invariants.rs` | `230` | Private function 'prop_2_bm25_score_monotone_increasing_in_tf' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/proptest_bm25_invariants.rs` | `266` | Private function 'prop_3_bm25_score_monotone_decreasing_in_doc_len' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/proptest_bm25_invariants.rs` | `289` | Private function 'prop_4_no_nan_output_for_valid_inputs' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/proptest_bm25_invariants.rs` | `316` | Private function 'prop_5_tombstone_isolation_snapshot_semantics' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/snapshot_oracle_proptest.rs` | `73` | Private function 'prop_memory_index_matches_oracle' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/tombstone_read_your_writes.rs` | `205` | Private function 'proof_deleted_doc_not_in_search_results' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/tombstone_read_your_writes.rs` | `259` | Private function 'proof_tombstone_snapshot_isolation' appears unused in crate (freq=1) |
| `contextra-text` | `crates/contextra-text/tests/wand_differential.rs` | `420` | Private function 'prop_wand_differential_randomized_corpus' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `534` | Private function 'contextra_error_display_no_panic' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `774` | Private function 'prop_fusion_weights_never_panics' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `410` | Private function 'default_tl_hfd_sigma' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `413` | Private function 'default_tl_hfd_delta' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `416` | Private function 'default_tl_hfd_gamma' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `419` | Private function 'default_tl_hfd_max_iterations' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `422` | Private function 'default_tl_hfd_max_top_k_expansion' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `425` | Private function 'default_tl_hfd_max_hyperedge_sort_size' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `502` | Private function 'default_warn_on_non_convergence' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `142` | Private function 'doc_id_valid_key_is_ok' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `149` | Private function 'doc_id_from_empty_returns_err' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `162` | Private function 'tx_id_ordering_is_consistent' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `418` | Private function 'prop_tx_id_range_isolation' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `427` | Private function 'prop_tx_id_overflow_isolation' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `177` | Private function 'prop_docid_serialization' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `185` | Private function 'doc_id_from_key_deterministic' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `192` | Private function 'doc_id_from_key_never_panics' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `197` | Private function 'doc_id_empty_key_is_err' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `202` | Private function 'prop_txid_serialization' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `210` | Private function 'prop_entityid_serialization' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `100` | Private function 'prop_distance_metric_symmetry_and_non_negativity' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `117` | Private function 'prop_embedding_normalize_is_idempotent' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `110` | Private function 'prop_tombstone_bit_invariants' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `122` | Private function 'prop_tenant_scoped_isolation' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/generate_golden_ids.rs` | `17` | Private function 'generate_golden_ids_data' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `148` | Private function 'prop_config_fingerprint_bit_exact_floats' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `160` | Private function 'prop_model_fingerprint_serde_roundtrip' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `32` | Private function 'prop_doc_id_from_key_properties' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `54` | Private function 'prop_entity_id_from_key_properties' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `76` | Private function 'prop_tx_id_ordering_and_display' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `98` | Private function 'prop_collection_id_invariants' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `113` | Private function 'prop_importance_score_clamping' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `125` | Private function 'prop_token_budget_consumption' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/src/diskann/build/tombstone_wal.rs` | `44` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/src/quantize/train.rs` | `172` | Deprecated item marked for removal |
| `contextra-vector` | `crates/contextra-vector/src/hnsw/staging.rs` | `27` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/mmap_toctou_test.rs` | `78` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/mmap_toctou_test.rs` | `186` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `12` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `15` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `18` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `21` | Explicit #[allow(dead_code)] annotation |
| `contextra-vector` | `crates/contextra-vector/benches/overfetch_selectivity.rs` | `56` | Private function 'fill_bytes' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/deleted_nodes_lock_contention.rs` | `10` | Private function 'proof_no_deadlock_concurrent_delete_search' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/deleted_nodes_lock_contention.rs` | `83` | Private function 'proof_deleted_nodes_lock_hoisted_before_loop' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/deleted_nodes_lock_contention.rs` | `123` | Private function 'proof_search_throughput_not_degraded_by_deletes' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/deletion_ghost_pointer_proptest.rs` | `9` | Private function 'prop_ghost_free_deletion_removes_all_pointers' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/golden_test.rs` | `31` | Private function 'generate_golden_files' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/mmap_toctou_test.rs` | `28` | Private function 'run_subcommand_target' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/nan_validation_policy.rs` | `14` | Private function 'proof_nan_rejected_at_insert' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/nan_validation_policy.rs` | `47` | Private function 'proof_compute_distance_trusted_never_scans_nan' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/nan_validation_policy.rs` | `78` | Private function 'proof_valid_insert_after_nan_rejection' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/nan_validation_policy.rs` | `126` | Private function 'proof_validate_vector_covers_all_positions' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/nan_validation_policy.rs` | `156` | Private function 'proof_nan_in_query_rejected_at_search' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/proptest_distance_quantize.rs` | `22` | Private function 'prop_sq8_quantize_dequantize_roundtrip' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/proptest_distance_quantize.rs` | `44` | Private function 'prop_cosine_self_distance_is_zero' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/proptest_distance_quantize.rs` | `58` | Private function 'prop_euclidean_non_negative' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/proptest_distance_quantize.rs` | `71` | Private function 'prop_dimension_mismatch_returns_err' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/recall_vs_brute_force.rs` | `189` | Private function 'recall_at_10_vs_brute_force_cosine' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/recall_vs_brute_force.rs` | `201` | Private function 'recall_at_10_vs_brute_force_euclidean' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/recall_vs_brute_force.rs` | `213` | Private function 'recall_at_10_vs_brute_force_dot_product' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/recall_vs_brute_force.rs` | `225` | Private function 'recall_at_1_vs_brute_force' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/simd_numerical_audit.rs` | `170` | Private function 'prop_simd_vs_f64_parity_random_vectors' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `13` | Private function 'assert_distance_fn_object_safety' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `16` | Private function 'assert_block_transform_object_safety' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `19` | Private function 'assert_clock_object_safety' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_contract_object_safety.rs` | `22` | Private function 'assert_rng_object_safety' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_primitives.rs` | `121` | Private function 'prop_test_u32_roundtrip' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_primitives.rs` | `128` | Private function 'prop_test_to_u8_valid' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_primitives.rs` | `134` | Private function 'prop_test_to_u8_invalid' appears unused in crate (freq=1) |
| `contextra-vector` | `crates/contextra-vector/tests/wp02_primitives.rs` | `139` | Private function 'prop_test_f32_non_negative_finite' appears unused in crate (freq=1) |

*... und 2 weitere Befunde (siehe JSON-Export).*

---
*Automatisch generiert durch `dead_code_microprobe.py` für Agentic Engineering & Context Auditing.*