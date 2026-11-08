# Test Traceability & Invariant Coverage Diagnostic Report

> **Werkzeug:** `test_coverage_matrix.py`  
> **Erstellungszeit:** `2026-11-07T23:15:51.141535+00:00`  
> **Status/Befunde:** `5150 Befunde erfasst`  

## Beschreibung
Maps unit tests, async tokio tests, proptests, benchmarks, and SPEC traceability markers (T-TRACE-1) across workspace crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_tests_discovered` | `5150` |
| `tests_with_spec_trace` | `160` |
| `unique_spec_invariants_covered` | `43` |
| `test_type_unit` | `3024` |
| `test_type_async` | `2011` |
| `test_type_proptest` | `86` |
| `test_type_benchmark` | `29` |

## Crate-Test-Matrix
| Crate | Tests Gesamt | Mit SPEC Trace (T-TRACE-1) | Traceability-Quote |
|---|---|---|---|
| `contextra` | 72 | 0 | 0.0 % |
| `contextra-adapt` | 142 | 0 | 0.0 % |
| `contextra-agent` | 125 | 0 | 0.0 % |
| `contextra-audit-export` | 32 | 0 | 0.0 % |
| `contextra-avv-generator` | 25 | 0 | 0.0 % |
| `contextra-bench` | 26 | 0 | 0.0 % |
| `contextra-checkpoint` | 148 | 0 | 0.0 % |
| `contextra-cognition` | 109 | 0 | 0.0 % |
| `contextra-core` | 7 | 0 | 0.0 % |
| `contextra-crypto` | 330 | 0 | 0.0 % |
| `contextra-db` | 192 | 0 | 0.0 % |
| `contextra-durable-fs` | 37 | 0 | 0.0 % |
| `contextra-engine` | 271 | 2 | 0.7 % |
| `contextra-erasure` | 94 | 62 | 66.0 % |
| `contextra-graph` | 411 | 3 | 0.7 % |
| `contextra-infer-candle` | 123 | 0 | 0.0 % |
| `contextra-infer-ollama` | 119 | 0 | 0.0 % |
| `contextra-infer-onnx` | 43 | 0 | 0.0 % |
| `contextra-ingest` | 213 | 0 | 0.0 % |
| `contextra-kvcache` | 103 | 3 | 2.9 % |
| `contextra-license` | 67 | 0 | 0.0 % |
| `contextra-mcp` | 168 | 0 | 0.0 % |
| `contextra-mvcc` | 117 | 0 | 0.0 % |
| `contextra-ports` | 53 | 2 | 3.8 % |
| `contextra-privacy` | 147 | 0 | 0.0 % |
| `contextra-py` | 91 | 86 | 94.5 % |
| `contextra-rank` | 100 | 0 | 0.0 % |
| `contextra-router` | 145 | 0 | 0.0 % |
| `contextra-sandbox` | 74 | 0 | 0.0 % |
| `contextra-simd` | 82 | 0 | 0.0 % |
| `contextra-store` | 600 | 0 | 0.0 % |
| `contextra-sys` | 31 | 0 | 0.0 % |
| `contextra-testkit` | 61 | 2 | 3.3 % |
| `contextra-text` | 211 | 0 | 0.0 % |
| `contextra-types` | 193 | 0 | 0.0 % |
| `contextra-vector` | 280 | 0 | 0.0 % |
| `contextra-wire` | 108 | 0 | 0.0 % |

## Top Abgedeckte Spezifikations-Invarianten
| Invariante / Spec-ID | Zugeordnete Tests | Beispiel-Dateien |
|---|---|---|
| `T-PY-1` | 86 | `tests.rs` |
| `T-VAULT-1` | 8 | `vault_slot_layout.rs, vault_spec_conformance.rs` |
| `T-REPORT-1` | 5 | `report_canonical_spec_tests.rs, report_spec_tests.rs` |
| `T-CRASH-1` | 3 | `crash_matrix_recovery.rs, saga_core_orchestration.rs` |
| `T-RAW-1` | 3 | `measure_real_scan_tests.rs, measure_spec_tests.rs` |
| `T-VAULT-7` | 3 | `vault_spec_conformance.rs` |
| `D-05` | 3 | `decryption_negative_tests.rs` |
| `T-CONC-1` | 2 | `tests.rs` |
| `T-CRASH-3` | 2 | `crash_matrix_recovery.rs, t_observer.rs` |
| `T-RAW-2` | 2 | `measure_real_scan_tests.rs, measure_spec_tests.rs` |
| `T-PROOF-1` | 2 | `report_canonical_spec_tests.rs` |
| `T-CLASS-1` | 2 | `report_canonical_spec_tests.rs` |
| `T-LINT-1` | 2 | `report_canonical_spec_tests.rs, special_targets_test.rs` |
| `T-STATUS-1` | 2 | `saga_core_orchestration.rs` |
| `T-REPL-1` | 2 | `special_targets_test.rs, t_saga_spec_extensions.rs` |
| `T-SUP-1` | 2 | `special_targets_test.rs, t_saga_spec_extensions.rs` |
| `T-RESTR-2` | 2 | `special_targets_test.rs, t_saga_spec_extensions.rs` |
| `T-ERID-1` | 2 | `special_targets_test.rs, t_saga_spec_extensions.rs` |
| `T-OBS-1` | 2 | `observer.rs, t_observer.rs` |
| `T-VAULT-8` | 2 | `vault_spec_conformance.rs` |
| `T-RESTORE-1` | 1 | `ledger_spec_tests.rs` |
| `T-RESTORE-2` | 1 | `ledger_spec_tests.rs` |
| `T-MUT-1` | 1 | `measure_real_scan_tests.rs` |
| `T-RAW-4` | 1 | `measure_spec_tests.rs` |
| `T-RAW-3` | 1 | `measure_spec_tests.rs` |
| `T-VERIFY-1` | 1 | `measure_spec_tests.rs` |
| `T-SLA-1` | 1 | `saga_core_orchestration.rs` |
| `T-CANCEL-1` | 1 | `saga_core_orchestration.rs` |
| `T-REPL-2` | 1 | `special_targets_test.rs` |
| `T-REPL-3` | 1 | `special_targets_test.rs` |
| `T-SUP-2` | 1 | `special_targets_test.rs` |
| `T-SUP-3` | 1 | `special_targets_test.rs` |
| `T-RESTR-1` | 1 | `special_targets_test.rs` |
| `T-RESTR-3` | 1 | `special_targets_test.rs` |
| `T-RESTR-4` | 1 | `special_targets_test.rs` |
| `T-EPOCH-1` | 1 | `t_saga_spec_extensions.rs` |
| `T-FK-1` | 1 | `vault_spec_conformance.rs` |
| `T-GRAPH-ALGO-DOUBLE-01` | 1 | `algorithm_test_doubles.rs` |
| `T-CASCADE-SUPERSESSION-01` | 1 | `supersession_characterization_tests.rs` |
| `T-CASCADE-SUPERSESSION-02` | 1 | `supersession_characterization_tests.rs` |
| `T-SIG-1` | 1 | `report_signer.rs` |
| `REQ-VFS-CONFORMANCE` | 1 | `conformance_selftest.rs` |
| `REQ-VFS-MUTANT-DETECTION` | 1 | `conformance_selftest.rs` |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra` | `crates/contextra/src/agent_memory.rs` | `361` | Test `test_fallback_id_prefix_and_monotonicity` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `390` | Test `test_fallback_id_reproducibility_from_initial_state` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `435` | Test `test_builder_defaults` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `446` | Test `test_builder_custom_configuration` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `472` | Test `test_encryption_passphrase_redaction` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `488` | Test `test_empty_passphrase_rejected` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `512` | Test `test_explicit_unencrypted_mode` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/builder.rs` | `533` | Test `test_builder_build_roundtrip` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/lib.rs` | `183` | Test `test_facade_open_and_builder_entrypoints` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `227` | Test `test_performance_profile_default` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `235` | Test `test_performance_profile_resolve_values` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `259` | Test `test_all_resolved_presets_validate_successfully` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `266` | Test `test_enforce_license_fast_ring_requires_no_gate` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `278` | Test `test_enforce_license_sovereign_ring_with_fast_gate_fails` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `294` | Test `test_enforce_license_sovereign_ring_with_sovereign_gate_succeeds` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `303` | Test `test_durability_vector_mismatch_error` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `317` | Test `test_error_display_and_from_conversion` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `35` | Test `test_agent_memory_facade_lifecycle` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `46` | Test `test_relate_n_ary_success_and_unique_ids` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `80` | Test `test_relate_n_ary_insufficient_participants` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `103` | Test `test_relate_n_ary_empty_predicate_or_role` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `135` | Test `test_relate_n_ary_nonexistent_memory_id` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `160` | Test `test_relate_n_ary_forget_participant_does_not_panic` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_idgen_test.rs` | `28` | Test `test_agent_memory_deterministic_id_generation` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/agent_memory_idgen_test.rs` | `75` | Test `test_agent_memory_fallback_id_generation` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/collection_profile_matrix.rs` | `21` | Test `test_collection_profile_matrix` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `94` | Test `test_scenario_1_attrappe_entlarvt` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `169` | Test `test_scenario_2_voller_pfad` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `257` | Test `test_scenario_3_rohdaten_needle` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `324` | Test `test_scenario_4_mehrere_level_und_neustart` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `435` | Test `test_scenario_5_collection_loeschung` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `503` | Test `test_scenario_6_fehlender_schluessel` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `542` | Test `test_scenario_7_offener_snapshot_blockiert_purge` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_e2e.rs` | `41` | Test `test_tdel_1_canary_bytes_erased_after_compaction` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deletion_e2e.rs` | `89` | Test `test_tdel_3_proof_verification_fails_with_invalid_key` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/deployment_tier_presets_valid.rs` | `14` | Test `test_all_deployment_tier_presets_valid` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `20` | Test `test_encryption_mode_redaction_and_facade` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `48` | Test `test_empty_passphrase_rejection` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `62` | Test `test_explicit_unencrypted_mode_building` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `85` | Test `test_implicit_unencrypted_mode_warning` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `132` | Test `test_encrypted_storage_roundtrip` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `17` | Test `open_without_gate_never_grants_sovereign` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `37` | Test `compliance_profile_without_signed_license_returns_error` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `72` | Test `compliance_profile_with_valid_signed_license_activates_deletion_proof` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `110` | Test `expired_signature_rejected` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `153` | Test `tampered_signature_rejected` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `196` | Test `explicit_conflicting_user_value_to_profile_returns_policy_violation` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `21` | Test `test_compliance_profile_without_gate_fails` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `57` | Test `test_compliance_profile_with_explicit_open_fast_gate_fails` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `94` | Test `test_builder_without_performance_profile_succeeds` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `134` | Test `test_compliance_profile_with_allow_all_gate_succeeds` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `71` | Test `test_a_plain_line_list_parsing` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `91` | Test `test_b_json_list_in_code_fence` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `104` | Test `test_c_more_lines_than_max_subqueries` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `117` | Test `test_d_duplicates_and_original_query_removed` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `131` | Test `test_e_garbage_and_empty_response_yields_ok_empty_vec` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `153` | Test `test_f_generator_error_propagates_as_err` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `170` | Test `test_g_unicode_multi_byte_snippet_not_split_mid_char` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/llm_query_rewriter.rs` | `194` | Test `test_h_snippet_with_embedded_instruction_isolated_in_untrusted_context` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/regulated_auto_extraction_default.rs` | `17` | Test `test_enterprise_regulated_auto_extraction_default_is_disabled` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/regulated_auto_extraction_default.rs` | `35` | Test `test_other_deployment_tiers_auto_extraction_default_is_enabled` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `21` | Test `test_proof_with_remaining_items_must_fail` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `52` | Test `test_proof_verification_with_wrong_key_fails` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `16` | Test `tenant_a_cannot_read_tenant_b` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `134` | Test `startup_ssi_tracking_check` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/type_identity_test.rs` | `8` | Test `test_facade_type_identities` (unit) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `49` | Test `vertical_path_test_3_ssi_under_write_load_b16` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `21` | Test `flush_preserves_pinned_snapshot` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `59` | Test `second_open_same_directory_rejected` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `76` | Test `open_without_gate_never_grants_sovereign` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `104` | Test `profile_durability_reaches_lsm_config` (async) -> SPEC: Untraced |
| `contextra` | `crates/contextra/tests/wiring_matrix.rs` | `127` | Test `confirmed_commit_survives_drop_reopen` (async) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `835` | Test `test_default_implementation_is_sherman_morrison` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `855` | Test `test_custom_bandit_policy_implementation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `863` | Test `test_bandit_policy_drift_penalty` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `877` | Test `test_sherman_morrison_score_and_update` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `893` | Test `test_sherman_morrison_correctness_and_precision` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `934` | Test `bench_sherman_morrison_p95_latency` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `973` | Test `test_dimension_mismatch_returns_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1018` | Test `test_cold_start_zero_theta_unit_sigma` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1026` | Test `test_score_cold_start_is_exploration_dominated` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1038` | Test `test_update_increases_theta_on_positive_reward` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1050` | Test `test_cloud_transport_penalty_reduces_score` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1062` | Test `test_drift_increases_alpha` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1071` | Test `test_repeated_drift_caps_at_max_multiplier` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1081` | Test `test_serde_backward_compatibility_for_alpha_fields` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1104` | Test `test_bandit_forgetting` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1167` | Test `test_drift_detected_after_1000_updates` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1192` | Test `test_sketched_projection_invalid_projected_dim_returns_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1211` | Test `test_drift_detected_on_nonpositive_denominator` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `133` | Test `test_temperature_clamp` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `148` | Test `test_high_temperature_shorter_half_life` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `171` | Test `test_effective_score_decays_over_time` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `183` | Test `test_effective_score_non_negative` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `195` | Test `test_should_evict_low_score_high_temp` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `207` | Test `test_should_not_evict_fresh_chunk` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `215` | Test `test_engine_equivalence_and_accessors` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `205` | Test `test_catoni_drift_detector_stable_stream` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `215` | Test `test_catoni_drift_detector_robust_to_single_outlier` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `227` | Test `test_catoni_drift_detector_detects_sustained_shift` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `248` | Test `test_ensemble_drift_watcher_requires_both` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `259` | Test `test_drift_policy_bridge_triggers_bandit_penalty` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `285` | Test `test_drift_policy_bridge_dispatches_exact_penalty_parameters` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `923` | Test `test_splitmix64_reproducibility` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `934` | Test `test_config_validation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `146` | Test `test_high_latency_causes_monotonic_decrease_bounded_by_k_min` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `173` | Test `test_low_latency_causes_increase_bounded_by_k_max` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `200` | Test `test_hard_k_min_floor_never_below_50` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `221` | Test `test_rerank_deadline_exceeded` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `288` | Test `test_insufficient_data_for_first_window_updates` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `302` | Test `test_stable_distribution_yields_stable_result` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `327` | Test `test_artificially_shifted_distribution_triggers_drift_detected` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `368` | Test `test_zero_divergence_denominator_edge_case_no_panic` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `384` | Test `test_laplace_smoothing_empty_baseline_bin_scenario` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `418` | Test `test_identical_distributions_zero_kl_divergence` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `437` | Test `test_kl_divergence_clipped_on_extreme_histogram_degeneration` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `464` | Test `test_extreme_distribution_shift_kl_contribution_clipped` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `497` | Test `test_moderate_distribution_shift_kl_unclipped` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `165` | Test `test_randomized_logging_policy_propensities` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `190` | Test `test_randomized_logging_policy_select_action` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `205` | Test `test_hit_case` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `217` | Test `test_miss_case` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `228` | Test `test_propensity_clamping` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `241` | Test `test_invalid_propensity_discarded` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `259` | Test `test_convergence_on_synthetic_dataset` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `204` | Test `test_pid_stable_at_target` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `214` | Test `test_pid_latency_too_high_reduces_pool` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `224` | Test `test_pid_latency_too_low_increases_pool` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `234` | Test `test_pid_non_finite_latency_ignored` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `255` | Test `test_pid_anti_windup_halts_integral_accumulation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `275` | Test `test_pid_clamps_to_min_max` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `291` | Test `test_high_latency_causes_monotonic_decrease_bounded_by_k_min` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `319` | Test `test_low_latency_causes_increase_bounded_by_k_max` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `347` | Test `test_oscillating_latency_anti_windup_clamp` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `362` | Test `test_hard_k_min_floor_never_below_50` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `380` | Test `test_pid_nan_latency_preserves_state_and_returns_current_pool` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `415` | Test `test_pid_dt_rate_independence` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `435` | Test `test_pid_nan_on_fresh_controller_returns_min_pool_size` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `449` | Test `test_pid_recovers_correctly_after_nan_then_valid_input` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `205` | Test `test_pid_controller_default_values` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `214` | Test `test_pid_controller_latency_series_monotonically_adjusts` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `241` | Test `test_pid_controller_clamping_bounds` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `258` | Test `test_pid_controller_anti_windup_clamping` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `275` | Test `test_pid_controller_nan_and_inf_safety` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `289` | Test `test_latency_budget_guard` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `229` | Test `test_rie_greedy_initialization` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `47` | Test `test_tracing_shadow_sink_does_not_panic` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `60` | Test `test_select_arm_non_influence_invariant` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `10` | Test `test_sherman_morrison_long_run_drift_detection` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `60` | Test `test_sherman_morrison_well_conditioned_no_false_positives` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `162` | Test `test_sherman_morrison_100k_updates_frobenius_stability` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/catoni_robust_to_outliers.rs` | `44` | Test `test_catoni_robust_to_extreme_outliers_compared_to_naive_mean` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/catoni_robust_to_outliers.rs` | `89` | Test `test_catoni_detects_true_sustained_distribution_shift` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `44` | Test `test_pid_anti_windup_bounds_integrator_strictly` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `63` | Test `test_pid_step_response_without_overshoot` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `79` | Test `test_pid_latency_controller_step_response_clamping` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `92` | Test `test_pid_resets_properly` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `102` | Test `test_pid_latency_controller_reset_clears_history` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `114` | Test `test_lyapunov_drift_monitor_detects_divergence` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `138` | Test `test_lyapunov_drift_monitor_status_str` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `152` | Test `test_catoni_drift_detector_reset` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `164` | Test `test_drift_policy_bridge_default` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `174` | Test `test_fcts_bandit_converges_on_synthetic_bernoulli_environment` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `221` | Test `test_linucb_bandit_converges_on_synthetic_environment` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `246` | Test `test_fcts_config_validation_failures` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `286` | Test `test_shadow_mode_never_alters_productive_decisions` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `329` | Test `test_latency_budget_guard_with_mock_clock` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `354` | Test `test_latency_budget_guard_default_fallback` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `362` | Test `test_decay_controller_config_default` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `370` | Test `test_decay_signal_inputs_default` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `379` | Test `test_adaptive_decay_controller_temperature_extreme_clamp` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `391` | Test `test_adaptive_decay_controller_effective_score_zero_elapsed` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `399` | Test `test_sketched_bandit_feature_gated_initialization` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `413` | Test `test_catoni_drift_detector_influence_function_bounds` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `422` | Test `test_ensemble_drift_watcher_set_baseline_updates_both_detectors` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `430` | Test `test_pid_latency_controller_with_params` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `439` | Test `test_pid_latency_controller_negative_params_fallback` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` | `18` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` | `25` | Test `prop_off_policy_positivity_verification` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` | `80` | Test `test_off_policy_error_conversion_to_contextra_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` | `94` | Test `test_bandit_default_flip_gate_decision_boundaries` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_drift_update_ring3_only.rs` | `18` | Test `test_ak18_drift_rate_assignment_restricted_to_recompute` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_drift_update_ring3_only.rs` | `62` | Test `test_ak18_recompute_signature_requires_ring3_token` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_drift_update_ring3_only.rs` | `94` | Test `test_ak18_ring3_token_not_called_in_ring0_crates` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `54` | Test `test_reproducible_arm_selection_same_seed` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `86` | Test `test_parameter_mu_convergence_linear_target` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `127` | Test `test_sherman_morrison_matches_direct_matrix_inversion` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `174` | Test `test_error_handling_and_state_non_mutation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `223` | Test `test_ring_buffer_fixed_capacity_overwriting` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/fcts_numerics.rs` | `247` | Test `test_drift_rate_recovery_linear_drift` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/ips_requires_propensity.rs` | `9` | Test `test_propensity_lower_bound_clamping_and_discard` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/ips_requires_propensity.rs` | `37` | Test `test_valid_propensity_accepted` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_anti_windup_tests.rs` | `14` | Test `test_sustained_high_load_actuator_saturated_prevents_integral_windup` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_anti_windup_tests.rs` | `34` | Test `test_spike_recovery_comparison_faster_convergence` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_anti_windup_tests.rs` | `78` | Test `test_nan_latency_handling_no_panic_returns_current_pool` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_anti_windup_tests.rs` | `99` | Test `test_unsaturated_behavior_identical_to_update` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_stability.rs` | `103` | Test `step_response_settles_within_expected_time` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_stability.rs` | `140` | Test `step_response_overshoot_bounded` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_stability.rs` | `169` | Test `no_steady_state_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/proptest_sherman_morrison_matrix_inverse_differential.rs` | `116` | Test `prop_sherman_morrison_matches_naive_reinversion` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/proptest_sherman_morrison_matrix_inverse_differential.rs` | `183` | Test `test_sherman_morrison_collinear_stability_bounded_growth` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/rie_greedy_numerics.rs` | `7` | Test `test_uncertainty_growth_after_discounting_and_update` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/rie_greedy_numerics.rs` | `35` | Test `test_dimension_mismatch_returns_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/rie_greedy_numerics.rs` | `59` | Test `test_deterministic_behavior_bit_identical_state` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/rie_greedy_numerics.rs` | `98` | Test `test_convergence_on_constant_context_and_reward` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/rie_greedy_numerics.rs` | `123` | Test `test_non_finite_input_handling` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `13` | Test `test_sketch_matrix_from_seed_bounds_validation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `36` | Test `test_bandit_profile_state_sketched_config_validation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `76` | Test `test_non_finite_variance_returns_error` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `12` | Test `test_sketched_projection_matrix_determinism_100_runs` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `35` | Test `test_sketched_projection_vector_projection_determinism` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `59` | Test `test_sketched_projection_score_and_update_determinism` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `105` | Test `test_sketched_projection_tenant_seed_isolation` (unit) -> SPEC: Untraced |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `42` | Test `test_sketched_projection_vs_sherman_morrison_score_correlation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `238` | Test `test_audit_log_in_memory_storage` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `288` | Test `test_audit_log_duplicate_step_rejected` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `326` | Test `test_audit_log_null_byte_and_empty_input_boundary_guards` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `453` | Test `test_migrate_legacy_audit_entries` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `29` | Test `test_reserve_tokens_raii_settle` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `42` | Test `test_reserve_tokens_raii_drop_refund` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `58` | Test `test_reserve_tokens_over_budget_rejected` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `72` | Test `test_reserve_tokens_sequential_reservations` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `538` | Test `test_scratchpad_builders_and_sinks` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `570` | Test `test_scratchpad_apply_edit_all_variants` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `628` | Test `test_scratchpad_checkpoint_and_reset` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `679` | Test `test_scratchpad_capacity_exceeded_returns_error` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `708` | Test `test_scratchpad_missing_label_returns_error` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `734` | Test `test_pinned_region_immutability_guards` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `285` | Test `test_agent_context_cache_directives` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `303` | Test `test_agent_context_with_clm_scratchpad` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `335` | Test `test_validate_task_id_guards` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `353` | Test `test_validate_node_id_guards` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `371` | Test `test_agent_context_telemetry_event_capacity_cap` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `394` | Test `test_agent_context_fifo_eviction` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `433` | Test `test_try_attach_event_error_message_unit` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `487` | Test `test_telemetry_100k_insertions_and_amortized_performance` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `251` | Test `test_dlq_push_list_drain` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `294` | Test `test_dlq_allocate_tx_uniqueness` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `305` | Test `test_dlq_metrics_gauge_depth` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `399` | Test `test_dlq_new_with_metrics_constructor` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine.rs` | `844` | Test `test_fallback_registry_methods` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/event_source.rs` | `261` | Test `test_background_event_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/event_source.rs` | `282` | Test `test_vec_event_source_capacity_limit` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `242` | Test `test_output_field_equals` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `283` | Test `test_output_field_contains` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `321` | Test `test_tokens_consumed_at_least` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `335` | Test `test_next_edge_equals` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `353` | Test `test_all_any_not_logic` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `388` | Test `test_describe` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/goal_condition.rs` | `432` | Test `test_parse_legacy_condition_string` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/graph.rs` | `279` | Test `test_graph_add_node_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/graph.rs` | `306` | Test `test_graph_add_edge_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `35` | Test `test_evaluate_condition_expr_grammar_and_outcomes` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `82` | Test `test_audit_before_commit_ordering` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `148` | Test `test_try_register_tool_boundary_validations` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `187` | Test `test_orchestrator_recover_orphans_succeeds` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `222` | Test `test_goal_condition_triggers_scratchpad_checkpoint_and_reset` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `67` | Test `test_tool_manifest_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `91` | Test `test_tool_registry_capability_fail_closed` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `121` | Test `test_retrieve_tool_execution` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `145` | Test `test_retrieve_tool_with_custom_retriever` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `81` | Test `test_agent_auto_checkpoint_before_step` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `119` | Test `test_agent_replay_from_checkpoint` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `164` | Test `test_agent_error_handling` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `207` | Test `test_agent_audit_log_immutable` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `247` | Test `test_crash_during_execute_recovery` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `296` | Test `test_loop_rollback_integrity` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `372` | Test `test_agent_engine_recovers_orphaned_checkpoint_on_restart` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `22` | Test `test_audit_logging_does_not_pollute_hnsw_index_count` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `74` | Test `test_vector_search_results_unaffected_by_massive_audit_logging` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `148` | Test `test_migration_removes_legacy_zero_vectors_from_hnsw` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `291` | Test `test_migration_hnsw_delete_failure_prevents_orphan_state` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `405` | Test `test_zero_vector_insertion_rejected_with_invalid_input` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `50` | Test `test_task_and_node_id_validation_boundaries` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `89` | Test `test_state_graph_boundary_validations` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `113` | Test `test_orchestrator_tool_registration_boundaries` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `141` | Test `test_background_event_and_vec_source_capacity_boundaries` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `88` | Test `test_sequential_workflow_budget_check` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `127` | Test `test_failed_step_raii_drop_refunds_tokens` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `176` | Test `test_estimated_cost_pre_execution_check` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `269` | Test `test_atomic_budget_reservation_concurrency_stress` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_goal_condition_edges.rs` | `20` | Test `test_output_field_equals_all_json_types` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_goal_condition_edges.rs` | `80` | Test `test_output_field_contains_edge_cases` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_goal_condition_edges.rs` | `112` | Test `test_tokens_consumed_at_least_boundary` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_goal_condition_edges.rs` | `126` | Test `test_next_edge_equals` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_goal_condition_edges.rs` | `150` | Test `test_logical_combinators_and_determinism` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` | `26` | Test `test_scratchpad_reset_flow_and_invariants` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `59` | Test `test_orchestrator_deterministic_clock_failed_at_secs` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `96` | Test `test_orchestrator_deterministic_clock_checkpoint_created_at` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `132` | Test `test_orchestrator_deterministic_clock_identical_runs` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `118` | Test `test_agent_auto_checkpoint_before_step` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `157` | Test `test_agent_replay_from_checkpoint` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `183` | Test `test_agent_audit_log_immutable` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `222` | Test `test_token_budget_exhaustion` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `74` | Test `test_crash_recovery_step_abort_injection` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `152` | Test `test_audit_log_before_commit_guarantee` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_clock_determinism_test.rs` | `57` | Test `test_dlq_failed_at_secs_uses_injected_clock` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `20` | Test `test_tool_timeout_creates_dead_letter` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `107` | Test `test_dlq_remove_and_string_tx_id_idempotency` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `163` | Test `test_dlq_replay_idempotency_scenarios` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `238` | Test `test_concurrent_allocate_tx_uniqueness` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `276` | Test `test_tool_retry_succeeds_on_second_attempt` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `71` | Test `test_e2e_agent_workflow` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `107` | Test `test_e2e_db_crud_roundtrip` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `159` | Test `test_stress_concurrent_agent_ops` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `80` | Test `test_event_loop_exhausted_source_exit_and_checkpointing` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `139` | Test `test_event_loop_cancellation_token_exit` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `176` | Test `test_polling_document_event_source_delta_detection` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `267` | Test `test_custom_event_source_extensibility` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `328` | Test `test_notify_push_event_source_wait_for_event` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `20` | Test `test_atomic_final_state_checkpoint` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `14` | Test `test_stategraph_complex_workflow` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `57` | Test `test_decision_node_condition_branching` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/id_gen_determinism_test.rs` | `15` | Test `test_agent_id_gen_determinism` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/loop_protection_test.rs` | `73` | Test `test_loop_protection_max_workflow_steps_termination` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/loop_protection_test.rs` | `142` | Test `test_loop_protection_budget_exhaustion_termination` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/persistence_test.rs` | `46` | Test `test_agent_persistence_and_recovery` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/proptest_workflow_tests.rs` | `35` | Test `test_state_graph_node_addition_valid_inputs_always_succeed` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/proptest_workflow_tests.rs` | `51` | Test `test_state_graph_node_addition_invalid_null_byte_always_rejected` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/proptest_workflow_tests.rs` | `72` | Test `test_state_graph_node_addition_invalid_oversized_always_rejected` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/proptest_workflow_tests.rs` | `85` | Test `test_background_event_validation_valid_inputs_always_succeed` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/regression_agt_a.rs` | `81` | Test `test_regression_zero_panic_doctrine_no_panics_on_invalid_inputs` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/regression_agt_a.rs` | `107` | Test `test_regression_unbounded_loop_termination` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/regression_agt_a.rs` | `139` | Test `test_regression_budget_exhaustion_halts_retries` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/session_concurrency_test.rs` | `77` | Test `test_parallel_agent_sessions_isolation` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/session_concurrency_test.rs` | `155` | Test `test_parallel_tool_execution_failure_dlq_routing` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/system_prompt_pinning_latency.rs` | `18` | Test `test_system_prompt_pinning_latency_under_100_micros` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/system_prompt_pinning_latency.rs` | `66` | Test `test_system_prompt_pinning_agent_flow` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_edge_serde_compat.rs` | `20` | Test `test_legacy_edge_json_deserialization` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_edge_serde_compat.rs` | `40` | Test `test_new_edge_json_roundtrip_with_goal` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_edge_serde_compat.rs` | `62` | Test `test_new_edge_json_omits_none_goal` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `15` | Test `test_stategraph_construction` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `43` | Test `test_stategraph_boundary_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `85` | Test `test_background_event_boundary_validation` (unit) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `100` | Test `test_agent_context_boundary_validation` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `187` | Test `test_pre_execution_budget_check_prevents_tool_execution` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `270` | Test `test_replay_from_restores_budget_state` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `343` | Test `test_replay_from_identifier_resolution` (async) -> SPEC: Untraced |
| `contextra-agent` | `crates/contextra-agent/tests/workflow_tests.rs` | `485` | Test `test_audit_log_field_reused_across_steps` (async) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/src/bsi_mapping.rs` | `104` | Test `test_bsi_mapping_table_exact_entries` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/src/bsi_mapping.rs` | `136` | Test `test_render_bsi_mapping_markdown` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/campaign_j27_compliance_test.rs` | `12` | Test `test_h10_gdpr_art30_processing_registry_and_canary_leak_check` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/error_and_edge_cases.rs` | `9` | Test `test_error_display_formatting` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/error_and_edge_cases.rs` | `32` | Test `test_source_collection_empty_returns_empty_vec` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/error_and_edge_cases.rs` | `41` | Test `test_empty_optional_contacts_rendering` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/golden_file_test.rs` | `9` | Test `test_golden_file_json_export` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/golden_file_test.rs` | `27` | Test `test_golden_file_markdown_export` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/golden_file_test.rs` | `45` | Test `test_byte_identical_markdown_output` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/golden_file_test.rs` | `61` | Test `test_byte_identical_json_output` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/golden_file_test.rs` | `77` | Test `test_deterministic_sorting_by_tenant_and_purpose` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/register_rendering.rs` | `7` | Test `test_json_rendering_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/register_rendering.rs` | `24` | Test `test_markdown_rendering_data_rows` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/register_rendering.rs` | `54` | Test `test_empty_entries_rendering` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `9` | Test `test_sanitize_pipes` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `16` | Test `test_sanitize_backticks` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `23` | Test `test_sanitize_newlines` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `30` | Test `test_sanitize_html_tags` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `37` | Test `test_markdown_rendering_sanitization_in_fields` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/sanitization_tests.rs` | `60` | Test `test_unicode_umlaute_preservation` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `16` | Test `test_validation_valid_entry_passes` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `23` | Test `test_validation_missing_controller_name` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `39` | Test `test_validation_missing_controller_contact` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `55` | Test `test_validation_missing_processing_purpose` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `71` | Test `test_validation_missing_legal_basis` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `87` | Test `test_validation_missing_data_subject_categories` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `103` | Test `test_validation_missing_data_categories` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `119` | Test `test_validation_missing_recipient_categories` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `135` | Test `test_validation_missing_retention_periods` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `151` | Test `test_validation_missing_retention_period_fields` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `170` | Test `test_validation_missing_toms` (unit) -> SPEC: Untraced |
| `contextra-audit-export` | `crates/contextra-audit-export/tests/validation_tests.rs` | `186` | Test `test_validation_missing_third_country_fields` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `32` | Test `test_render_avv_markdown_contains_required_sections` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `64` | Test `test_empty_subprocessors_handling` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `87` | Test `test_snapshot_rendering_output` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `128` | Test `test_art28_dsgvo_completeness_checklist` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `185` | Test `test_disclaimer_present_in_header` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `194` | Test `test_golden_file_output_matches` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `208` | Test `test_validation_empty_controller_name` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `216` | Test `test_validation_empty_processor_name` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `224` | Test `test_validation_empty_contract_date` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `232` | Test `test_validation_invalid_deletion_sla_zero` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `246` | Test `test_validation_invalid_deletion_sla_too_high` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `260` | Test `test_validation_missing_subprocessors_when_allowed` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `269` | Test `test_validation_invalid_subprocessor_empty_string` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `282` | Test `test_validation_invalid_technical_measure_empty_name` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `299` | Test `test_validation_valid_context` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `307` | Test `test_markdown_escaping_user_inputs` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `321` | Test `test_markdown_escaping_injection_prevention` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `335` | Test `test_deterministic_rendering_with_fixed_date` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `347` | Test `test_unicode_support_in_names_and_measures` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `361` | Test `test_subprocessor_change_notification_clause` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `371` | Test `test_audit_and_control_rights_clause` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `384` | Test `test_data_subject_rights_and_assistance_clauses` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `396` | Test `test_confidentiality_clause_present` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/avv_rendering.rs` | `407` | Test `test_instructions_clause_present` (unit) -> SPEC: Untraced |
| `contextra-avv-generator` | `crates/contextra-avv-generator/tests/campaign_j27_avv_test.rs` | `22` | Test `test_h11_gdpr_art28_avv_generator_technical_completeness` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/ann_benchmarks.rs` | `269` | Test `test_synthetic_vectors_and_ground_truth` (async) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/beir_eval.rs` | `357` | Test `test_ir_metrics_calculations` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `240` | Test `test_compare_metrics_happy_path_equal` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `260` | Test `test_compare_metrics_regression_exceeds_threshold` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `283` | Test `test_compare_metrics_within_threshold_passes` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `307` | Test `test_compare_metrics_improvement_emits_info` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `330` | Test `test_compare_metrics_nan_and_inf_detection` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `364` | Test `test_compare_metrics_invalid_threshold` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `379` | Test `test_compare_metrics_missing_section` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/compare.rs` | `405` | Test `test_compare_metrics_zero_baseline` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/main.rs` | `1474` | Test `test_create_synthetic_corpus_integrity` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/main.rs` | `1493` | Test `test_benchmark_scenarios_execution` (async) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/path_rag_sweep.rs` | `342` | Test `test_pad_vector_various_dimensions` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/regression_gate.rs` | `125` | Test `test_regression_gate_passes_when_equal` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/regression_gate.rs` | `146` | Test `test_regression_gate_fails_on_drop` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/regression_gate.rs` | `172` | Test `test_regression_gate_updates_baseline` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/src/regression_gate.rs` | `193` | Test `test_regression_gate_error_cases` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/benches/competitive_kv.rs` | `605` | Test `criterion_benchmark` (benchmark) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `9` | Test `test_simulated_current_10pp_below_baseline_triggers_regression` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `47` | Test `test_simulated_current_within_tolerance_passes` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `84` | Test `test_simulated_current_above_baseline_emits_info_no_regression` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `124` | Test `test_missing_metric_sections_triggers_regression` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `172` | Test `test_zero_baseline_cases` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `213` | Test `test_compare_metrics_files_errors` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/compare_baseline_test.rs` | `258` | Test `test_nan_inf_metric_values_trigger_regression` (unit) -> SPEC: Untraced |
| `contextra-bench` | `benchmarks/contextra-bench/tests/scale_100k_smoke.rs` | `9` | Test `test_scale_100k_smoke` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `628` | Test `test_orphan_registry_persists_across_drop` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `659` | Test `test_rollback_blocking_in_sync_context` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `679` | Test `test_rollback_blocking_in_async_context_returns_error` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `711` | Test `test_checkpoint_guard_for_agent_step` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `728` | Test `checkpoint_guard_CASE_uncommitted_guard_holds_state` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `747` | Test `checkpoint_guard_CASE_commit_moves_ownership` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `769` | Test `checkpoint_guard_CASE_rollback_consumed_returns_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `818` | Test `test_checkpoint_metrics_commit_duration` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `853` | Test `test_checkpoint_metrics_drop_rollback_duration` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `110` | Test `test_manifest_validation_blank_component` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `127` | Test `test_checkpoint_manifest_CASE_whitespace_component_rejected` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `143` | Test `test_checkpoint_manifest_CASE_tampered_manifest_fails_verify` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `161` | Test `test_manifest_creation_invalid_meta_name` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `176` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/manifest.rs` | `177` | Test `test_prop_manifest_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `132` | Test `checkpoint_meta_CASE_serialization_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `158` | Test `state_checkpoint_CASE_serialization_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `176` | Test `state_checkpoint_deserializes_legacy_json_without_namespace` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `189` | Test `into_workflow_state_CASE_valid_conversion` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `208` | Test `test_validate_identifier_boundary_oversized_257` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta.rs` | `219` | Test `test_validate_identifier_empty_or_whitespace` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `669` | Test `test_deprecated_global_orphan_path_warns` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `684` | Test `timestamp_ms_is_monotonic` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `693` | Test `test_instance_orphan_registry_drain_pins_and_checkpoints` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `721` | Test `test_instance_orphan_registry_clear_nonexistent` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `743` | Test `test_instance_orphan_registry_empty_path_persistence_noop` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `753` | Test `test_instance_orphan_registry_with_vfs` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `775` | Test `test_two_instance_registries_independent` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `805` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `806` | Test `prop_monotonic_timestamp_ms_increases_or_equals` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `119` | Test `test_orphan_registry_persists_across_drop` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `150` | Test `test_rollback_blocking_in_sync_context` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `170` | Test `test_rollback_blocking_in_async_context_returns_error` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `202` | Test `test_checkpoint_guard_for_agent_step` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `219` | Test `checkpoint_guard_CASE_uncommitted_guard_holds_state` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `238` | Test `checkpoint_guard_CASE_commit_moves_ownership` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `260` | Test `checkpoint_guard_CASE_rollback_consumed_returns_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `136` | Test `test_create_and_load` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `152` | Test `test_name_uniqueness` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `172` | Test `test_checkpoint_creation_rollback_on_failure` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `190` | Test `test_orphan_recovery_on_startup` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `221` | Test `test_multi_instance_orphan_isolation` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `249` | Test `test_pin_guard_unpins_checkpoint_on_storage_write_failure` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `284` | Test `test_pin_before_unpin_invariant_on_failure` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `322` | Test `test_panic_unwind_triggers_orphan_registration_and_recovery` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `373` | Test `test_checkpoint_guard_rollback_on_drop` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `393` | Test `test_checkpoint_guard_commit_prevents_rollback` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `410` | Test `test_list_checkpoints_empty_initially` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `421` | Test `test_checkpoint_not_found_returns_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `432` | Test `test_list_named_checkpoints_after_reopen` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `469` | Test `test_list_checkpoints_cache_matches_storage` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `499` | Test `test_concurrent_checkpoint_creation_is_safe` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `535` | Test `test_checkpoint_guard_dropped_outside_tokio_runtime` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `559` | Test `test_auto_rollback_tracking_and_await` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `579` | Test `test_drop_checkpoint_uses_unique_tx_and_unpins` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `602` | Test `test_next_tx_overflow_returns_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `619` | Test `test_input_validation_empty_and_oversized_names` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `673` | Test `test_create_checkpoint_CASE_unicode_and_multibyte_name` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `707` | Test `test_create_checkpoint_CASE_exact_max_len_256` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `727` | Test `test_drop_checkpoint_CASE_nonexistent_returns_ok` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `741` | Test `test_restore_checkpoint_CASE_not_found_returns_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `752` | Test `test_list_checkpoints_CASE_corrupted_storage_data_propagates_err` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `769` | Test `test_get_orphaned_checkpoints_for_namespace` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `800` | Test `test_allocate_tx_CASE_parity_with_deprecated_next_tx` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `815` | Test `test_store_skipped_rollback_count_and_checkpoint_counter` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `836` | Test `test_concurrent_checkpoint_index_atomicity` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `898` | Test `test_store_traits_roundtrip_coverage` (async) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `12` | Test `test_deprecated_global_orphan_path_warns` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `27` | Test `timestamp_ms_is_monotonic` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `36` | Test `test_instance_orphan_registry_drain_pins_and_checkpoints` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `64` | Test `test_instance_orphan_registry_clear_nonexistent` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `86` | Test `test_instance_orphan_registry_empty_path_persistence_noop` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `96` | Test `test_two_instance_registries_independent` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `126` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan/tests.rs` | `127` | Test `prop_monotonic_timestamp_ms_increases_or_equals` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta/tests.rs` | `12` | Test `checkpoint_meta_CASE_serialization_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta/tests.rs` | `37` | Test `state_checkpoint_CASE_serialization_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/meta/tests.rs` | `56` | Test `state_checkpoint_deserializes_legacy_json_without_namespace` (unit) -> SPEC: Untraced |

*... und 4650 weitere Befunde (siehe JSON-Export).*

---
*Automatisch generiert durch `test_coverage_matrix.py` für Agentic Engineering & Context Auditing.*