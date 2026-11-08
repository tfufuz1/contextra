# Micro-Granular Rust AST & Item Inspection Report

> **Werkzeug:** `rust_ast_inspector.py`  
> **Erstellungszeit:** `2026-11-07T23:05:32.584335+00:00`  
> **Status/Befunde:** `13395 Befunde erfasst`  

## Beschreibung
Comprehensive item inventory covering structs, enums, traits, implementations, functions, visibilities, and qualifiers across workspace crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_crates_scanned` | `37` |
| `total_rust_files_scanned` | `955` |
| `total_items_discovered` | `13395` |
| `public_items` | `7482` |
| `async_functions` | `1651` |
| `unsafe_items` | `43` |
| `count_struct` | `1253` |
| `count_enum` | `278` |
| `count_trait` | `141` |
| `count_impl` | `1622` |
| `count_fn` | `9393` |
| `count_type` | `115` |
| `count_const` | `593` |

## Crate-Item-Übersicht
| Crate | Datentypen (Struct/Enum/Trait) | Funktionen | Gesamt-Items |
|---|---|---|---|
| `contextra` | 20 | 88 | 138 |
| `contextra-adapt` | 48 | 228 | 341 |
| `contextra-agent` | 44 | 222 | 309 |
| `contextra-audit-export` | 10 | 20 | 36 |
| `contextra-avv-generator` | 3 | 24 | 30 |
| `contextra-bench` | 20 | 78 | 111 |
| `contextra-checkpoint` | 25 | 304 | 365 |
| `contextra-cognition` | 49 | 230 | 323 |
| `contextra-core` | 0 | 0 | 0 |
| `contextra-crypto` | 68 | 379 | 542 |
| `contextra-db` | 19 | 85 | 122 |
| `contextra-durable-fs` | 19 | 80 | 118 |
| `contextra-engine` | 95 | 694 | 968 |
| `contextra-erasure` | 178 | 554 | 955 |
| `contextra-graph` | 114 | 736 | 1018 |
| `contextra-infer-candle` | 36 | 206 | 294 |
| `contextra-infer-ollama` | 37 | 209 | 275 |
| `contextra-infer-onnx` | 12 | 113 | 145 |
| `contextra-ingest` | 111 | 509 | 747 |
| `contextra-kvcache` | 47 | 253 | 360 |
| `contextra-license` | 6 | 39 | 54 |
| `contextra-mcp` | 35 | 232 | 317 |
| `contextra-mvcc` | 18 | 219 | 276 |
| `contextra-ports` | 143 | 538 | 827 |
| `contextra-privacy` | 48 | 206 | 307 |
| `contextra-py` | 8 | 164 | 190 |
| `contextra-rank` | 34 | 169 | 251 |
| `contextra-router` | 39 | 232 | 327 |
| `contextra-sandbox` | 15 | 60 | 100 |
| `contextra-simd` | 5 | 79 | 92 |
| `contextra-store` | 105 | 906 | 1175 |
| `contextra-sys` | 8 | 74 | 91 |
| `contextra-testkit` | 17 | 133 | 184 |
| `contextra-text` | 42 | 366 | 477 |
| `contextra-types` | 92 | 333 | 587 |
| `contextra-vector` | 61 | 463 | 640 |
| `contextra-wire` | 41 | 168 | 303 |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra` | `crates/contextra/src/agent_memory.rs` | `60` | [FN] private next_fallback_id |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `70` | [STRUCT] pub MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `72` | [IMPL] inherent MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `74` | [FN] pub new |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `79` | [FN] pub as_str |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `84` | [IMPL] trait_impl From for MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `85` | [FN] private from |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `90` | [IMPL] trait_impl From for MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `91` | [FN] private from |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `96` | [IMPL] trait_impl AsRef for MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `97` | [FN] private as_ref |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `102` | [IMPL] trait_impl fmt::Display for MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `103` | [FN] private fmt |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `113` | [STRUCT] pub RelationId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `115` | [IMPL] trait_impl fmt::Display for RelationId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `116` | [FN] private fmt |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `124` | [STRUCT] pub Memory |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `137` | [IMPL] trait_impl From for Memory |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `138` | [FN] private from |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `158` | [STRUCT] pub AgentMemory |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `163` | [IMPL] inherent AgentMemory |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `166` | [FN] pub new |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `175` | [FN] pub new_arc |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `184` | [FN] pub new_with_id_gen |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `193` | [FN] pub with_id_gen |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `200` | [FN] pub engine |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `228` | [FN] pub remember |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `254` | [FN] pub recall |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `266` | [FN] pub forget |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `280` | [FN] pub relate |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `313` | [FN] pub relate_n_ary |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `362` | [FN] private test_fallback_id_prefix_and_monotonicity |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `391` | [FN] private test_fallback_id_reproducibility_from_initial_state |
| `contextra` | `crates/contextra/src/boot.rs` | `17` | [ENUM] pub BootPhase |
| `contextra` | `crates/contextra/src/boot.rs` | `41` | [FN] pub execute_boot_sequence |
| `contextra` | `crates/contextra/src/builder.rs` | `46` | [ENUM] pub EncryptionMode |
| `contextra` | `crates/contextra/src/builder.rs` | `53` | [IMPL] trait_impl std::fmt::Debug for EncryptionMode |
| `contextra` | `crates/contextra/src/builder.rs` | `54` | [FN] private fmt |
| `contextra` | `crates/contextra/src/builder.rs` | `64` | [TRAIT] pub ContextraConfigExt |
| `contextra` | `crates/contextra/src/builder.rs` | `66` | [FN] private is_encrypted |
| `contextra` | `crates/contextra/src/builder.rs` | `69` | [IMPL] trait_impl ContextraConfigExt for ContextraConfig |
| `contextra` | `crates/contextra/src/builder.rs` | `70` | [FN] private is_encrypted |
| `contextra` | `crates/contextra/src/builder.rs` | `83` | [STRUCT] pub ContextraBuilder |
| `contextra` | `crates/contextra/src/builder.rs` | `95` | [IMPL] trait_impl std::fmt::Debug for ContextraBuilder |
| `contextra` | `crates/contextra/src/builder.rs` | `96` | [FN] private fmt |
| `contextra` | `crates/contextra/src/builder.rs` | `113` | [IMPL] trait_impl Clone for ContextraBuilder |
| `contextra` | `crates/contextra/src/builder.rs` | `114` | [FN] private clone |
| `contextra` | `crates/contextra/src/builder.rs` | `129` | [IMPL] inherent ContextraBuilder |
| `contextra` | `crates/contextra/src/builder.rs` | `140` | [FN] pub new |
| `contextra` | `crates/contextra/src/builder.rs` | `162` | [FN] pub with_storage_path |
| `contextra` | `crates/contextra/src/builder.rs` | `172` | [FN] pub with_max_elements |
| `contextra` | `crates/contextra/src/builder.rs` | `182` | [FN] pub with_distance_metric |
| `contextra` | `crates/contextra/src/builder.rs` | `192` | [FN] pub with_encryption |
| `contextra` | `crates/contextra/src/builder.rs` | `214` | [FN] pub with_encryption_passphrase |
| `contextra` | `crates/contextra/src/builder.rs` | `221` | [FN] pub is_encrypted |
| `contextra` | `crates/contextra/src/builder.rs` | `227` | [FN] pub with_embedding_backend |
| `contextra` | `crates/contextra/src/builder.rs` | `238` | [FN] pub with_consolidation |
| `contextra` | `crates/contextra/src/builder.rs` | `246` | [FN] pub with_embedder |
| `contextra` | `crates/contextra/src/builder.rs` | `253` | [FN] pub with_license_gate |
| `contextra` | `crates/contextra/src/builder.rs` | `264` | [FN] pub with_signed_license |
| `contextra` | `crates/contextra/src/builder.rs` | `294` | [FN] pub with_performance_profile |
| `contextra` | `crates/contextra/src/builder.rs` | `306` | [FN] pub with_config |
| `contextra` | `crates/contextra/src/builder.rs` | `337` | [FN] pub build |
| `contextra` | `crates/contextra/src/builder.rs` | `422` | [IMPL] trait_impl Default for ContextraBuilder |
| `contextra` | `crates/contextra/src/builder.rs` | `423` | [FN] private default |
| `contextra` | `crates/contextra/src/builder.rs` | `436` | [FN] private test_builder_defaults |
| `contextra` | `crates/contextra/src/builder.rs` | `447` | [FN] private test_builder_custom_configuration |
| `contextra` | `crates/contextra/src/builder.rs` | `473` | [FN] private test_encryption_passphrase_redaction |
| `contextra` | `crates/contextra/src/builder.rs` | `489` | [FN] private test_empty_passphrase_rejected |
| `contextra` | `crates/contextra/src/builder.rs` | `513` | [FN] private test_explicit_unencrypted_mode |
| `contextra` | `crates/contextra/src/builder.rs` | `534` | [FN] private test_builder_build_roundtrip |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `39` | [ENUM] pub KvDeleteMode |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `44` | [IMPL] trait_impl Default for KvDeleteMode |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `45` | [FN] private default |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `54` | [CONST] pub DEFAULT_AUTO_EXTRACTION_MODE |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `62` | [CONST] pub ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `70` | [STRUCT] pub LsmTuning |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `82` | [STRUCT] pub CollectionProfile |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `92` | [ENUM] pub CollectionProfileError |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `99` | [IMPL] inherent CollectionProfile |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `109` | [FN] pub validate |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `123` | [FN] pub validate_with_license |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `138` | [ENUM] pub DeploymentTier |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `145` | [IMPL] inherent DeploymentTier |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `150` | [FN] pub resolve |
| `contextra` | `crates/contextra/src/lib.rs` | `92` | [FN] pub builder |
| `contextra` | `crates/contextra/src/lib.rs` | `134` | [FN] pub open |
| `contextra` | `crates/contextra/src/lib.rs` | `166` | [FN] pub open_with_config |
| `contextra` | `crates/contextra/src/lib.rs` | `184` | [FN] private test_facade_open_and_builder_entrypoints |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `37` | [ENUM] pub DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `43` | [IMPL] trait_impl Default for DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `44` | [FN] private default |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `51` | [ENUM] pub DurabilityConfigError |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `60` | [IMPL] trait_impl From for contextra_types::ContextraError |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `61` | [FN] private from |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `66` | [IMPL] inherent DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `68` | [FN] pub validate_against_features |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `100` | [ENUM] pub PerformanceProfile |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `106` | [IMPL] trait_impl Default for PerformanceProfile |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `108` | [FN] private default |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `116` | [STRUCT] pub ResolvedProfileConfig |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `126` | [ENUM] pub PerformanceProfileError |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `135` | [IMPL] inherent PerformanceProfile |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `143` | [FN] pub resolve |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `167` | [IMPL] inherent ResolvedProfileConfig |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `173` | [FN] pub enforce_license |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `186` | [FN] pub validate |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `205` | [STRUCT] private MockLicenseGate |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `209` | [IMPL] trait_impl LicenseGate for MockLicenseGate |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `210` | [FN] private check_ring |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `218` | [FN] private authorize |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `228` | [FN] private test_performance_profile_default |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `236` | [FN] private test_performance_profile_resolve_values |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `260` | [FN] private test_all_resolved_presets_validate_successfully |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `267` | [FN] private test_enforce_license_fast_ring_requires_no_gate |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `279` | [FN] private test_enforce_license_sovereign_ring_with_fast_gate_fails |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `295` | [FN] private test_enforce_license_sovereign_ring_with_sovereign_gate_succeeds |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `304` | [FN] private test_durability_vector_mismatch_error |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `318` | [FN] private test_error_display_and_from_conversion |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `332` | [IMPL] trait_impl From for contextra_db::DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `333` | [FN] private from |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `342` | [IMPL] trait_impl From for DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `343` | [FN] private from |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `40` | [STRUCT] pub LlmQueryRewriter |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `47` | [IMPL] inherent LlmQueryRewriter |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `53` | [FN] pub new |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `64` | [FN] pub with_max_subqueries |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `71` | [FN] pub with_max_context_results |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `78` | [FN] pub with_max_snippet_chars |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `84` | [FN] pub rewrite_query |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `93` | [IMPL] trait_impl QueryRewriter for LlmQueryRewriter |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `105` | [FN] private rewrite |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `129` | [FN] private truncate_str_chars |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `140` | [FN] private extract_snippet |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `161` | [FN] private build_prompt |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `194` | [FN] private clean_line |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `220` | [FN] private strip_code_fences |
| `contextra` | `crates/contextra/src/query_rewriter.rs` | `235` | [FN] private parse_response |
| `contextra-adapt` | `crates/contextra-adapt/examples/adapt_standalone.rs` | `11` | [FN] private main |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `31` | [CONST] pub SHERMAN_MORRISON_REFACTORIZATION_INTERVAL |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `36` | [ENUM] pub BanditError |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `61` | [IMPL] trait_impl From for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `62` | [FN] private from |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `99` | [STRUCT] pub AlignedF32Vec |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `101` | [IMPL] trait_impl std::ops::Deref for AlignedF32Vec |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `102` | [TYPE] private Target |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `103` | [FN] private deref |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `108` | [IMPL] trait_impl std::ops::DerefMut for AlignedF32Vec |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `109` | [FN] private deref_mut |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `114` | [IMPL] inherent AlignedF32Vec |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `116` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `121` | [FN] pub zeros |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `130` | [ENUM] pub BanditImplementation |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `152` | [STRUCT] pub SketchMatrix |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `163` | [IMPL] inherent SketchMatrix |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `166` | [FN] pub derive_seed |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `178` | [FN] pub from_seed |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `209` | [FN] pub project |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `220` | [TRAIT] pub BanditPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `222` | [FN] private apply_drift_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `225` | [FN] private default_gamma |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `229` | [FN] private default_drift_gamma |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `233` | [FN] private default_drift_decay_window |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `237` | [FN] private default_alpha_base |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `241` | [FN] private default_alpha_max_multiplier |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `250` | [FN] private dot_product_f32 |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `299` | [FN] private fused_row_update |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `343` | [STRUCT] pub BanditProfileState |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `395` | [IMPL] trait_impl BanditPolicy for BanditProfileState |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `396` | [FN] private apply_drift_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `412` | [IMPL] inherent BanditProfileState |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `415` | [FN] pub cold_start |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `449` | [FN] pub ensure_sketched_state |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `494` | [FN] private ensure_inv_a |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `506` | [FN] private ensure_work_buf |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `514` | [FN] pub expected_dim |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `539` | [FN] pub score |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `683` | [FN] pub update |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `826` | [FN] pub on_drift_detected |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `836` | [FN] private test_default_implementation_is_sherman_morrison |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `845` | [STRUCT] private TestMockPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `849` | [IMPL] trait_impl BanditPolicy for TestMockPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `850` | [FN] private apply_drift_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `856` | [FN] private test_custom_bandit_policy_implementation |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `864` | [FN] private test_bandit_policy_drift_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `878` | [FN] private test_sherman_morrison_score_and_update |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `894` | [FN] private test_sherman_morrison_correctness_and_precision |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `936` | [FN] private bench_sherman_morrison_p95_latency |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `974` | [FN] private test_dimension_mismatch_returns_error |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1019` | [FN] private test_cold_start_zero_theta_unit_sigma |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1027` | [FN] private test_score_cold_start_is_exploration_dominated |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1039` | [FN] private test_update_increases_theta_on_positive_reward |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1051` | [FN] private test_cloud_transport_penalty_reduces_score |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1063` | [FN] private test_drift_increases_alpha |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1072` | [FN] private test_repeated_drift_caps_at_max_multiplier |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1082` | [FN] private test_serde_backward_compatibility_for_alpha_fields |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1105` | [FN] private test_bandit_forgetting |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1168` | [FN] private test_drift_detected_after_1000_updates |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1193` | [FN] private test_sketched_projection_invalid_projected_dim_returns_error |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1212` | [FN] private test_drift_detected_on_nonpositive_denominator |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `20` | [STRUCT] pub DecaySignalInputs |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `33` | [IMPL] trait_impl Default for DecaySignalInputs |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `34` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `46` | [STRUCT] pub DecayControllerConfig |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `55` | [IMPL] trait_impl Default for DecayControllerConfig |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `56` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `67` | [STRUCT] pub AdaptiveDecayController |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `71` | [IMPL] inherent AdaptiveDecayController |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `73` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `78` | [FN] pub with_defaults |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `83` | [FN] pub config |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `89` | [FN] pub system_temperature |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `96` | [FN] pub effective_half_life |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `103` | [FN] pub effective_score |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `119` | [FN] pub should_evict |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `134` | [FN] private test_temperature_clamp |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `149` | [FN] private test_high_temperature_shorter_half_life |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `172` | [FN] private test_effective_score_decays_over_time |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `184` | [FN] private test_effective_score_non_negative |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `196` | [FN] private test_should_evict_low_score_high_temp |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `208` | [FN] private test_should_not_evict_fresh_chunk |
| `contextra-adapt` | `crates/contextra-adapt/src/decay_controller.rs` | `216` | [FN] private test_engine_equivalence_and_accessors |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `13` | [ENUM] pub DriftSignal |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `27` | [TRAIT] pub DriftDetector |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `29` | [FN] private observe |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `31` | [FN] private is_drifting |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `40` | [STRUCT] pub CatoniDriftDetector |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `53` | [IMPL] trait_impl Default for CatoniDriftDetector |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `54` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `59` | [IMPL] inherent CatoniDriftDetector |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `61` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `72` | [FN] pub reset |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `78` | [IMPL] trait_impl DriftDetector for CatoniDriftDetector |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `79` | [FN] private observe |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `100` | [FN] private is_drifting |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `110` | [STRUCT] pub EnsembleDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `117` | [IMPL] trait_impl Default for EnsembleDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `118` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `126` | [IMPL] inherent EnsembleDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `128` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `133` | [FN] pub observe_and_decide |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `140` | [FN] pub set_baseline |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `154` | [STRUCT] pub DriftPolicyBridge |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `165` | [IMPL] trait_impl Default for DriftPolicyBridge |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `166` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `176` | [IMPL] inherent DriftPolicyBridge |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `178` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `190` | [FN] pub observe_and_react |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `206` | [FN] private test_catoni_drift_detector_stable_stream |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `216` | [FN] private test_catoni_drift_detector_robust_to_single_outlier |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `228` | [FN] private test_catoni_drift_detector_detects_sustained_shift |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `249` | [FN] private test_ensemble_drift_watcher_requires_both |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `261` | [FN] private test_drift_policy_bridge_triggers_bandit_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `287` | [FN] private test_drift_policy_bridge_dispatches_exact_penalty_parameters |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `290` | [STRUCT] private RecordingPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `294` | [IMPL] trait_impl BanditPolicy for RecordingPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/drift.rs` | `295` | [FN] private apply_drift_penalty |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `29` | [ENUM] pub FcTsError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `46` | [IMPL] trait_impl From for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `47` | [FN] private from |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `66` | [TRAIT] pub FcTsRng |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `68` | [FN] private next_u64 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `71` | [FN] private next_f32 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `77` | [FN] private next_standard_normal |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `90` | [STRUCT] pub SplitMix64 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `94` | [IMPL] inherent SplitMix64 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `96` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `101` | [IMPL] trait_impl FcTsRng for SplitMix64 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `102` | [FN] private next_u64 |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `114` | [STRUCT] pub FcTsConfig |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `131` | [IMPL] trait_impl Default for FcTsConfig |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `132` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `145` | [IMPL] inherent FcTsConfig |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `147` | [FN] pub validate |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `188` | [STRUCT] pub Ring3Token |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `198` | [FN] pub ring3_background_task_token |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `205` | [STRUCT] pub FlowCorrectedThompsonBandit |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `241` | [IMPL] trait_impl std::fmt::Debug for FlowCorrectedThompsonBandit |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `242` | [FN] private fmt |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `257` | [IMPL] inherent FlowCorrectedThompsonBandit |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `259` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `291` | [FN] pub with_shadow_sink |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `300` | [FN] pub shadow_sink |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `305` | [FN] pub config |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `310` | [FN] pub mu |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `315` | [FN] pub drift_rate |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `320` | [FN] pub inv_a |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `335` | [FN] pub update_with_flow |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `448` | [FN] pub sample_score |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `479` | [FN] pub baseline_score |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `505` | [FN] pub select_arm |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `532` | [FN] pub recompute_drift_rate_from_window |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `560` | [FN] private build_drift_regression_system |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `605` | [FN] private solve_conjugate_gradient |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `658` | [STRUCT] pub DiagonalApproximationBandit |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `663` | [IMPL] inherent DiagonalApproximationBandit |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `665` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `683` | [FN] pub from_pairs |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `692` | [FN] pub propensity |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `699` | [TRAIT] pub OffPolicyCompatibility |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `704` | [FN] private verify_positivity |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `713` | [ENUM] pub OffPolicyError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `719` | [IMPL] trait_impl From for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `720` | [FN] private from |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `728` | [STRUCT] pub FcTsSamplingDistribution |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `733` | [IMPL] inherent FcTsSamplingDistribution |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `735` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `753` | [FN] pub from_pairs |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `762` | [FN] pub probability |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `767` | [IMPL] trait_impl OffPolicyCompatibility for FcTsSamplingDistribution |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `768` | [FN] private verify_positivity |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `787` | [STRUCT] pub FcTsArmSet |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `792` | [IMPL] inherent FcTsArmSet |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `794` | [FN] pub with_shadow_sink |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `809` | [FN] pub select_arm |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `850` | [IMPL] trait_impl OffPolicyCompatibility for FcTsArmSet |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `851` | [FN] private verify_positivity |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `878` | [CONST] pub MIN_BANDIT_SHADOW_SAMPLES |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `880` | [CONST] pub MAX_CUMULATIVE_REGRET_THRESHOLD |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `885` | [STRUCT] pub BanditShadowReport |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `898` | [TRAIT] pub BanditDefaultFlipGate |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `903` | [FN] private should_flip |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `909` | [STRUCT] pub DefaultBanditFlipGate |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `911` | [IMPL] trait_impl BanditDefaultFlipGate for DefaultBanditFlipGate |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `912` | [FN] private should_flip |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `924` | [FN] private test_splitmix64_reproducibility |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `935` | [FN] private test_config_validation |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `21` | [STRUCT] pub RerankPidController |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `26` | [IMPL] trait_impl Default for RerankPidController |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `27` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `33` | [IMPL] inherent RerankPidController |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `35` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `52` | [FN] pub kp |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `57` | [FN] pub ki |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `62` | [FN] pub kd |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `67` | [FN] pub target_p95_latency_ms |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `72` | [FN] pub k_pool |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `79` | [FN] pub k_min |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `84` | [FN] pub k_max |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `89` | [FN] pub update |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `102` | [STRUCT] pub RerankDeadline |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `108` | [IMPL] inherent RerankDeadline |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `110` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `115` | [IMPL] inherent RerankDeadline |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `117` | [FN] pub new_with_clock |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `122` | [FN] pub start_nanos |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `127` | [FN] pub deadline_exceeded_nanos |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `137` | [FN] pub deadline_exceeded |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `148` | [FN] private test_high_latency_causes_monotonic_decrease_bounded_by_k_min |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `175` | [FN] private test_low_latency_causes_increase_bounded_by_k_max |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `202` | [FN] private test_hard_k_min_floor_never_below_50 |
| `contextra-adapt` | `crates/contextra-adapt/src/homeostat.rs` | `222` | [FN] private test_rerank_deadline_exceeded |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `28` | [CONST] private NUM_BINS |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `44` | [CONST] private MAX_BIN_KL_CONTRIBUTION |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `48` | [STRUCT] pub DriftReason |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `58` | [ENUM] pub LyapunovResult |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `75` | [IMPL] inherent LyapunovResult |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `77` | [FN] pub is_drift_detected |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `84` | [FN] pub apply_drift_if_detected |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `102` | [STRUCT] pub LyapunovDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `113` | [IMPL] trait_impl Default for LyapunovDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `114` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `119` | [IMPL] inherent LyapunovDriftWatcher |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `121` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `134` | [FN] pub observe_score |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `139` | [FN] pub analyze |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `146` | [FN] pub latest_result |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `151` | [FN] pub status_str |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `168` | [FN] pub set_baseline |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `176` | [FN] pub update |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `229` | [FN] private compute_kl_divergence |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `263` | [FN] private compute_lyapunov_exponent |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `289` | [FN] private test_insufficient_data_for_first_window_updates |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `303` | [FN] private test_stable_distribution_yields_stable_result |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `328` | [FN] private test_artificially_shifted_distribution_triggers_drift_detected |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `369` | [FN] private test_zero_divergence_denominator_edge_case_no_panic |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `385` | [FN] private test_laplace_smoothing_empty_baseline_bin_scenario |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `419` | [FN] private test_identical_distributions_zero_kl_divergence |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `438` | [FN] private test_kl_divergence_clipped_on_extreme_histogram_degeneration |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `465` | [FN] private test_extreme_distribution_shift_kl_contribution_clipped |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `498` | [FN] private test_moderate_distribution_shift_kl_unclipped |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `22` | [STRUCT] pub RandomizedLoggingPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `29` | [IMPL] inherent RandomizedLoggingPolicy |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `31` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `39` | [FN] pub compute_propensities |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `53` | [FN] pub select_action |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `74` | [STRUCT] pub OffPolicyStats |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `81` | [STRUCT] pub OffPolicyEvaluator |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `87` | [IMPL] inherent OffPolicyEvaluator |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `92` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `110` | [FN] pub observe |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `134` | [FN] pub estimate |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `139` | [FN] pub cumulative_ips |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `144` | [FN] pub samples |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `149` | [FN] pub discarded |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `154` | [FN] pub stats |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `166` | [FN] private test_randomized_logging_policy_propensities |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `191` | [FN] private test_randomized_logging_policy_select_action |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `206` | [FN] private test_hit_case |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `218` | [FN] private test_miss_case |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `229` | [FN] private test_propensity_clamping |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `242` | [FN] private test_invalid_propensity_discarded |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `260` | [FN] private test_convergence_on_synthetic_dataset |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `26` | [CONST] pub PID_MIN_POOL_SIZE_DEFAULT |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `29` | [CONST] pub PID_MAX_POOL_SIZE_DEFAULT |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `40` | [TRAIT] pub AntiWindupController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `49` | [FN] private update_with_anti_windup |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `64` | [STRUCT] pub PidController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `84` | [IMPL] trait_impl Default for PidController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `85` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `95` | [IMPL] inherent PidController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `99` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `125` | [FN] pub update |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `154` | [FN] pub current_pool_size |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `159` | [FN] pub reset |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `166` | [IMPL] trait_impl AntiWindupController for PidController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `167` | [FN] private update_with_anti_windup |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `205` | [FN] private test_pid_stable_at_target |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `215` | [FN] private test_pid_latency_too_high_reduces_pool |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `225` | [FN] private test_pid_latency_too_low_increases_pool |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `235` | [FN] private test_pid_non_finite_latency_ignored |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `256` | [FN] private test_pid_anti_windup_halts_integral_accumulation |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `276` | [FN] private test_pid_clamps_to_min_max |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `292` | [FN] private test_high_latency_causes_monotonic_decrease_bounded_by_k_min |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `320` | [FN] private test_low_latency_causes_increase_bounded_by_k_max |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `348` | [FN] private test_oscillating_latency_anti_windup_clamp |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `363` | [FN] private test_hard_k_min_floor_never_below_50 |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `381` | [FN] private test_pid_nan_latency_preserves_state_and_returns_current_pool |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `416` | [FN] private test_pid_dt_rate_independence |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `436` | [FN] private test_pid_nan_on_fresh_controller_returns_min_pool_size |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `450` | [FN] private test_pid_recovers_correctly_after_nan_then_valid_input |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `18` | [CONST] pub DEFAULT_TARGET_LATENCY_MS |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `22` | [CONST] pub MIN_SCALING_FACTOR |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `25` | [CONST] pub MAX_SCALING_FACTOR |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `28` | [CONST] pub MAX_INTEGRAL |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `38` | [STRUCT] pub PidLatencyController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `53` | [IMPL] trait_impl Default for PidLatencyController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `54` | [FN] private default |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `59` | [IMPL] inherent PidLatencyController |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `61` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `79` | [FN] pub with_params |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `105` | [FN] pub compute_adjustment |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `143` | [FN] pub reset |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `151` | [STRUCT] pub LatencyBudgetGuard |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `157` | [IMPL] inherent LatencyBudgetGuard |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `159` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `164` | [IMPL] inherent LatencyBudgetGuard |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `166` | [FN] pub new_with_clock |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `182` | [FN] pub elapsed_ms |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `191` | [FN] pub is_exceeded |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `196` | [FN] pub budget_ms |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `206` | [FN] private test_pid_controller_default_values |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `215` | [FN] private test_pid_controller_latency_series_monotonically_adjusts |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `242` | [FN] private test_pid_controller_clamping_bounds |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `259` | [FN] private test_pid_controller_anti_windup_clamping |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `276` | [FN] private test_pid_controller_nan_and_inf_safety |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `290` | [FN] private test_latency_budget_guard |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `12` | [ENUM] pub RieGreedyError |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `29` | [IMPL] trait_impl From for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `30` | [FN] private from |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `52` | [STRUCT] pub RieGreedyProfile |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `65` | [IMPL] inherent RieGreedyProfile |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `67` | [FN] pub new |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `96` | [FN] pub mu_hat |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `114` | [FN] pub predict |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `142` | [FN] pub update |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `215` | [FN] pub trace_precision_inv |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `230` | [FN] private test_rie_greedy_initialization |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `11` | [STRUCT] pub ShadowDiscrepancy |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `21` | [TRAIT] pub ShadowSink |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `23` | [FN] private record |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `29` | [STRUCT] pub TracingShadowSink |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `31` | [IMPL] trait_impl ShadowSink for TracingShadowSink |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `32` | [FN] private record |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `48` | [FN] private test_tracing_shadow_sink_does_not_panic |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `61` | [FN] private test_select_arm_non_influence_invariant |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `68` | [STRUCT] private MockRecordingSink |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `72` | [IMPL] trait_impl ShadowSink for MockRecordingSink |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `73` | [FN] private record |
| `contextra-agent` | `crates/contextra-agent/examples/minimal_pipeline.rs` | `13` | [STRUCT] private EchoTool |
| `contextra-agent` | `crates/contextra-agent/examples/minimal_pipeline.rs` | `15` | [IMPL] trait_impl AgentTool for EchoTool |
| `contextra-agent` | `crates/contextra-agent/examples/minimal_pipeline.rs` | `16` | [FN] private name |
| `contextra-agent` | `crates/contextra-agent/examples/minimal_pipeline.rs` | `20` | [FN] private execute |
| `contextra-agent` | `crates/contextra-agent/examples/minimal_pipeline.rs` | `35` | [FN] private main |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `22` | [STRUCT] pub AuditEntry |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `43` | [STRUCT] pub MigrationStats |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `51` | [STRUCT] pub AuditLog |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `56` | [FN] pub validate_audit_payload_and_error |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `91` | [IMPL] inherent AuditLog |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `93` | [FN] pub new |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `98` | [FN] pub append_to |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `114` | [FN] pub append |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `119` | [FN] pub replay_task |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `143` | [FN] private migrate_single_legacy_audit_entry |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `202` | [FN] pub migrate_legacy_audit_entries |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `239` | [FN] private test_audit_log_in_memory_storage |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `289` | [FN] private test_audit_log_duplicate_step_rejected |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `327` | [FN] private test_audit_log_null_byte_and_empty_input_boundary_guards |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `454` | [FN] private test_migrate_legacy_audit_entries |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `17` | [FN] pub reserve_tokens |

*... und 12895 weitere Befunde (siehe JSON-Export).*

---
*Automatisch generiert durch `rust_ast_inspector.py` für Agentic Engineering & Context Auditing.*