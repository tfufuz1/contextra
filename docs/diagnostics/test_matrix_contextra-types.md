# Test Traceability & Invariant Coverage Diagnostic Report

> **Werkzeug:** `test_coverage_matrix.py`  
> **Erstellungszeit:** `2026-11-07T23:04:46.712570+00:00`  
> **Status/Befunde:** `193 Befunde erfasst`  

## Beschreibung
Maps unit tests, async tokio tests, proptests, benchmarks, and SPEC traceability markers (T-TRACE-1) across workspace crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_tests_discovered` | `193` |
| `tests_with_spec_trace` | `0` |
| `unique_spec_invariants_covered` | `0` |
| `test_type_unit` | `186` |
| `test_type_proptest` | `7` |

## Crate-Test-Matrix
| Crate | Tests Gesamt | Mit SPEC Trace (T-TRACE-1) | Traceability-Quote |
|---|---|---|---|
| `contextra-types` | 193 | 0 | 0.0 % |

## Top Abgedeckte Spezifikations-Invarianten
| Invariante / Spec-ID | Zugeordnete Tests | Beispiel-Dateien |
|---|---|---|

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-types` | `crates/contextra-types/src/error.rs` | `510` | Test `contextra_error_display_no_panic` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `574` | Test `test_invalid_input_helper` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `583` | Test `test_capability_unsupported_helper` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `599` | Test `test_from_try_from_slice_error` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `614` | Test `test_error_display_all_variants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `753` | Test `test_from_conversions` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `783` | Test `test_result_alias` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `803` | Test `test_io_error_message_preserved` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `835` | Test `test_json_error_message_preserved` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `865` | Test `test_wal_corruption_fields_not_transposed` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `895` | Test `test_checksum_mismatch_fields_not_transposed` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `929` | Test `test_transaction_timeout_fields_not_transposed` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `962` | Test `test_memory_budget_exceeded_fields_not_transposed` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `993` | Test `test_hnsw_connectivity_degraded_preserves_ratio` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1019` | Test `test_error_constructor_helpers` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1048` | Test `test_dto_with_details_override` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1064` | Test `test_model_load_and_orphaned_vector_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1093` | Test `test_step_id_and_cache_directive` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1112` | Test `test_pin_budget_exceeded_display_and_helper` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1125` | Test `test_cross_device_link_display_and_helper` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1144` | Test `test_is_occ_conflict` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `388` | Test `test_dto_exhaustive_match_coverage` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `551` | Test `test_dto_details_serialization` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `569` | Test `test_dto_constructors_and_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/model_fingerprint.rs` | `67` | Test `test_model_fingerprint_new_and_serde` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/retrieval_strategy.rs` | `60` | Test `test_retrieval_strategy_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/retrieval_strategy.rs` | `77` | Test `test_retrieval_strategy_serde_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/schema.rs` | `110` | Test `test_manifest_schema_version_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/schema.rs` | `124` | Test `test_doc_id_width_bytes_len` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/tombstone.rs` | `86` | Test `test_roundtrip_property` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/tombstone.rs` | `108` | Test `test_negative_cases` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/tombstone.rs` | `129` | Test `test_flags_tombstone_detection` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/tombstone.rs` | `138` | Test `test_edge_case_already_tombstoned_seq` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `459` | Test `test_resource_tracker_basic` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `475` | Test `test_budget_exceeded` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `495` | Test `test_capacity_threshold` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `507` | Test `test_concurrent_consumption` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `532` | Test `test_release_memory_underflow_safety` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `552` | Test `test_consume_memory_overflow_prevention` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `567` | Test `test_token_budget_for_model` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `586` | Test `test_token_budget_edge_cases` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `613` | Test `test_token_budget_strategies` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `631` | Test `test_reservation_successful_settle` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `645` | Test `test_reservation_drop_refund` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `661` | Test `test_reservation_over_budget_leaves_consumed_unchanged` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `672` | Test `test_reservation_concurrent_double_spend` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `205` | Test `test_filter_serialization` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `223` | Test `test_filter_variants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `241` | Test `test_filter_expr_evaluate_all_operators` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `365` | Test `test_filter_evaluate_null_and_empty_meta` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `392` | Test `test_nested_filter_expression` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `225` | Test `test_importance_score_clamping_and_nan` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `235` | Test `test_decay_factor_none` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `243` | Test `test_decay_factor_exponential` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `260` | Test `test_decay_factor_step_floor` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `276` | Test `test_decay_factor_out_of_order_tx` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `285` | Test `test_memory_importance_effective_score` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/importance.rs` | `298` | Test `test_decay_factor_wall_clock_exponential` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `640` | Test `test_fusion_weights_valid` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `647` | Test `test_fusion_weights_invalid_sum` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `658` | Test `test_fusion_weights_default_is_balanced` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `676` | Test `test_hybrid_query_builder_default_uses_balanced_weights` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `689` | Test `test_hybrid_query_builder_happy_path` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `708` | Test `test_hybrid_query_builder_custom_weights` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `719` | Test `test_hybrid_query_builder_defaults` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `731` | Test `test_token_budget_edge_cases` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `744` | Test `test_fusion_weights_negative_or_infinite` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `759` | Test `test_fusion_weights_nan` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `770` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `771` | Test `prop_fusion_weights_never_panics` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `791` | Test `test_context_window_serialization` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `819` | Test `test_combined_text_with_prefix` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `837` | Test `test_combined_text_without_prefix_returns_content` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `852` | Test `test_combined_text_empty_prefix_returns_content` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `867` | Test `test_serde_backward_compat_no_prefix_field` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `874` | Test `test_combined_token_count_with_prefix` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `900` | Test `test_hybrid_query_builder_all_options` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `16` | Test `test_core_distance_dimension_mismatch` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `25` | Test `test_embedding_norm_and_normalize` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `42` | Test `test_distance_metrics_f32` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `58` | Test `test_distance_metrics_u8` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `89` | Test `test_distance_metrics_u8_overflow` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `142` | Test `test_u8_and_f32_distance_metrics_ranking_and_value_parity` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `208` | Test `test_distance_metrics_empty_and_single_element` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `245` | Test `test_distance_metric_dimension_mismatch` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `270` | Test `test_embedding_normalize_edge_cases` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `12` | Test `test_tenant_id_defaults_and_constants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `28` | Test `test_tenant_id_collections_btreemap` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `50` | Test `test_tenant_id_serde_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `62` | Test `test_doc_id_from_key_no_panic` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `77` | Test `test_doc_id_determinism` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `86` | Test `test_tenant_and_collection_id` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `100` | Test `test_doc_id_multibyte_unicode_keys` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `118` | Test `test_entity_id_methods` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `140` | Test `doc_id_valid_key_is_ok` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `147` | Test `doc_id_from_empty_returns_err` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `153` | Test `test_entity_id_from_key_empty_err` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `160` | Test `tx_id_ordering_is_consistent` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `170` | Test `test_tx_id_internal` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `183` | Test `test_tx_id_is_valid_origin` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `203` | Test `test_tx_id_range_boundary_exhaustion_simulation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `295` | Test `test_doc_id_from_key_collisions_and_distribution` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `315` | Test `test_doc_id_and_entity_id_unicode_keys` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `338` | Test `test_tx_id_invalid_sentinel_and_conversion_boundary` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `351` | Test `test_tx_id_ranges_and_internal_boundary_checks` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `366` | Test `test_tx_id_system_range_wraparound_safety` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `415` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `416` | Test `prop_tx_id_range_isolation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `425` | Test `prop_tx_id_overflow_isolation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `433` | Test `test_tenant_id_system_reserved` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `439` | Test `test_tenant_id_valid` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `448` | Test `test_tenant_id_system_constant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `455` | Test `test_tenant_id_hardening` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `462` | Test `test_tenant_id_try_from_boundary_and_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `12` | Test `test_workflow_state_hex_serde_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `30` | Test `test_expiry_metadata_key_constant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `36` | Test `test_serialization_roundtrips` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `69` | Test `test_entity_and_edge` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `127` | Test `test_entity_and_edge_try_new_validation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `142` | Test `test_memory_type_defaults_and_serde` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `175` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `226` | Test `test_entity_try_new_invalid_inputs` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `240` | Test `test_edge_try_new_invalid_inputs` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/campaign_r4_r10_oracle.rs` | `11` | Test `test_doc_id_canonical_bit_decomposition_oracle` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/campaign_r4_r10_oracle.rs` | `25` | Test `test_tx_id_monotonicity_oracle` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/campaign_r4_r10_oracle.rs` | `36` | Test `test_tenant_id_system_reservation_oracle` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `17` | Test `test_cosine_distance_identical_and_opposite_vectors` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `38` | Test `test_euclidean_distance_properties` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `52` | Test `test_dot_product_distance_properties` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `63` | Test `test_embedding_normalization_invariants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `79` | Test `test_distance_metrics_non_finite_inputs_rejected` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `97` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `98` | Test `prop_distance_metric_symmetry_and_non_negativity` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `115` | Test `prop_embedding_normalize_is_idempotent` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `18` | Test `test_request_context_creation_and_fields` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `34` | Test `test_saga_record_and_scan_methods` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `57` | Test `test_tenant_scoped_wrapper_and_checked_extraction` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `88` | Test `test_tombstone_bit_manipulations_and_semantics` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `107` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `108` | Test `prop_tombstone_bit_invariants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `120` | Test `prop_tenant_scoped_isolation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_dto_and_schema_suite.rs` | `17` | Test `test_error_dto_new_and_with_details` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_dto_and_schema_suite.rs` | `43` | Test `test_error_dto_display_and_formatting` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_dto_and_schema_suite.rs` | `55` | Test `test_hnsw_deletion_error_variants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/error_dto_and_schema_suite.rs` | `85` | Test `test_manifest_schema_version_and_doc_id_width_properties` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/filter_expressions_suite.rs` | `17` | Test `test_filter_equality_and_inequality_operators` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/filter_expressions_suite.rs` | `53` | Test `test_filter_numeric_comparisons` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/filter_expressions_suite.rs` | `98` | Test `test_filter_set_and_contains_operators` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/filter_expressions_suite.rs` | `143` | Test `test_filter_logical_combinators_and_not` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/filter_expressions_suite.rs` | `183` | Test `test_filter_null_and_missing_field_evaluations` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `13` | Test `test_doc_id_wire_golden_bytes` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `45` | Test `test_tx_id_wire_golden_bytes` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `55` | Test `test_tenant_id_wire_golden_bytes` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `17` | Test `test_importance_score_clamping_boundary_cases` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `39` | Test `test_decay_function_none_variant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `53` | Test `test_decay_function_exponential_variant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `83` | Test `test_decay_function_wallclock_exponential` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `102` | Test `test_decay_function_step_floor_variant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/importance_decay_suite.rs` | `120` | Test `test_memory_importance_effective_score_decay` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `18` | Test `test_model_fingerprint_hash_uniqueness_and_equality` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `38` | Test `test_retrieval_strategy_variants_and_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `63` | Test `test_cache_directive_variants_and_serde` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `83` | Test `test_config_fingerprint_builder_and_extractors` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `101` | Test `test_token_budget_reservations_and_concurrent_drops` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `123` | Test `test_resource_tracker_and_budget_backpressure` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `145` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `146` | Test `prop_config_fingerprint_bit_exact_floats` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `158` | Test `prop_model_fingerprint_serde_roundtrip` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `29` | Test `proptest_block` (proptest) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `30` | Test `prop_doc_id_from_key_properties` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `52` | Test `prop_entity_id_from_key_properties` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `74` | Test `prop_tx_id_ordering_and_display` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `96` | Test `prop_collection_id_invariants` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `111` | Test `prop_importance_score_clamping` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `123` | Test `prop_token_budget_consumption` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `150` | Test `test_entity_construction_and_validation` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `164` | Test `test_edge_construction_and_builder_chain` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `199` | Test `test_context_chunk_text_and_token_methods` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `237` | Test `test_query_rewrite_output_methods` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `257` | Test `test_schema_versions_and_doc_id_width` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `279` | Test `test_memory_type_helpers` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `295` | Test `test_all_error_variant_constructors_and_helpers` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `381` | Test `test_retrieval_strategy_and_step_id` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `18` | Test `test_exhaustive_serde_json_strings_and_bytes` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `62` | Test `test_serde_invalid_json_payloads_rejected` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `80` | Test `test_bincode_binary_payload_sizes` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `109` | Test `test_serde_complex_nested_structures` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `173` | Test `test_workflow_state_hex_serialization` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `190` | Test `test_entity_and_edge_attribute_maps` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `8` | Test `test_tenant_scoped_new_and_tenant_id_getter` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `17` | Test `test_into_inner_checked_matching_tenant` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `29` | Test `test_into_inner_checked_mismatched_tenant_does_not_leak_value` (unit) -> SPEC: Untraced |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `56` | Test `test_map_preserves_tenant_id` (unit) -> SPEC: Untraced |

---
*Automatisch generiert durch `test_coverage_matrix.py` für Agentic Engineering & Context Auditing.*