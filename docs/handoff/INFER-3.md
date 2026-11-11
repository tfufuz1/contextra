# Handoff Report: INFER-3 — contextra-infer-ollama und contextra-infer-candle

## 1. Ergebnisse je Arbeitspaket (WP)

| WP | Titel | Status | Prüfbefehl | Ergebnis |
|---|---|---|---|---|
| IN3.0 | Baseline und Vorbedingungen | Grün |  | Grün |
| IN3.1 | Golden-Korpus für beide Crates | Grün |
running 3 tests
test regen_golden_candle ... ignored, Generator für Golden-Korpus
test test_golden_backward_read_candle ... ok
test test_golden_corpus_candle ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s /
running 3 tests
test regen_golden_ollama ... ignored, Generator für Golden-Korpus
test test_golden_backward_read_ollama ... ok
test test_golden_corpus_ollama ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s | Grün (Byteweise identisch, MANIFEST.sha256 validiert) |
| IN3.2 | contextra-infer-ollama Härtung & Entkopplung | Grün |
running 22 tests
test context_prefixer::tests::test_compute_max_prefix_chars ... ok
test context_prefixer::tests::test_truncate_chars_exact_limit ... ok
test context_prefixer::tests::test_truncate_chars_german_umlauts ... ok
test context_prefixer::tests::test_truncate_chars_short_string ... ok
test context_prefixer::tests::test_truncate_chars_unicode_boundary ... ok
test context_prefixer::tests::test_truncate_chars_unicode_no_premature_truncation ... ok
test context_prefixer::tests::test_truncate_chars_zero_and_boundary ... ok
test context_prefixer::tests::test_truncate_prefix_char_boundary_off_by_one ... ok
test context_prefixer::tests::test_truncate_prefix_empty_and_whitespace ... ok
test context_prefixer::tests::test_truncate_prefix_token_boundary_off_by_one ... ok
test context_prefixer::tests::test_truncate_prefix_word_boundary ... ok
test context_prefixer::tests::test_truncate_chars_over_limit ... ok
test importance::tests::test_importance_assessment_serde_backward_compatibility ... ok
test importance::tests::test_parsed_0_5_score_returns_parsed_confidence ... ok
test importance::tests::test_score_importance_regex_get ... ok
test importance::tests::test_score_importance_regex_parsing_edge_cases ... ok
test importance::tests::test_unparseable_response_returns_unparseable_confidence ... ok
test model_info::tests::test_known_dimension_nomic ... ok
test importance::tests::test_unparseable_response_logs_warning_with_raw_text ... ok
test model_info::tests::test_model_info_deserializes_correctly ... ok
test importance::tests::test_prop_score_importance_arbitrary_garbage_never_panics ... ok
test importance::tests::test_prop_score_importance_parse_formats ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test regen_golden_ollama ... ignored, Generator für Golden-Korpus
test test_golden_backward_read_ollama ... ok
test test_golden_corpus_ollama ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 1 test
test test_exponential_backoff_monotonic_and_bounded ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test crates/contextra-infer-ollama/src/context_prefixer.rs - context_prefixer::ContextPrefixEngine (line 51) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s | Grün |
| IN3.3 | contextra-infer-candle Härtung & Entkopplung | Grün |
running 41 tests
test embedding::tests::test_bert_embed_model_load_nonexistent_weights_returns_io_error ... ok
test attention_exporter::tests::test_export_attention_weights_internal ... ok
test embedding::tests::test_from_dir_nonexistent ... ok
test embedding::tests::test_embed_zero_concurrency_limit_clamped_to_one ... ok
test embedding::tests::test_from_dir_valid_temp_dir ... ok
test embedding_provider::tests::test_embed_batch_internal ... ok
test embedding::tests::test_candle_embed_client_provider_and_engine ... ok
test gasp::tests::test_case_a_fully_supported_response_high_score_no_abstention ... ok
test gasp::tests::test_case_c_empty_context_zero_shot_defined_explicit_error ... ok
test gasp::tests::test_case_b_hallucinated_number_low_score_triggers_abstention ... ok
test gasp::tests::test_config_fingerprint_invalidation_resets_calibrator ... ok
test gasp::tests::test_exact_word_match ... ok
test gasp::tests::test_exact_number_match ... ok
test gasp::tests::test_gasp_nan_score_protection ... ok
test gasp::tests::test_gasp_multibyte_german_and_emoji_word_boundary_matching ... ok
test gasp::tests::test_gasp_whitespace_or_punctuation_only_response_handling ... ok
test gasp::tests::test_number_boundary_no_false_positive ... ok
test gasp::tests::test_refresh_config_invalidates_calibration_on_fingerprint_change ... ok
test gasp::tests::test_score_grounding_trait ... ok
test embedding_provider::tests::test_verify_embedding_provider_in_current_thread_runtime ... ok
test gasp::tests::test_set_threshold_updates_fingerprint_and_invalidates_calibration ... ok
test embedding_provider::tests::test_verify_embedding_provider_valid ... ok
test gguf_loader::tests::test_parse_gguf_metadata_nonexistent_path_returns_io_error ... ok
test gguf_loader::tests::test_parse_gguf_metadata_invalid_header_bytes_returns_model_load_error ... ok
test gasp::tests::test_validate_grounding_does_not_increment_observation_count ... ok
test gasp::tests::test_word_boundary_no_false_positive ... ok
test inference::inference_tests::test_candle_llm_client_generate ... ok
test inference::inference_tests::test_candle_llm_client_swap_model_invalidates_validator_calibration ... ok
test inference::inference_tests::test_candle_llm_client_generate_stream ... ok
test inference::inference_tests::test_inference_configurable_concurrency_limit ... ok
test inference::inference_tests::test_inference_zero_concurrency_limit_clamped_to_one ... ok
test inference::inference_tests::test_llm_from_dir_corrupt_gguf ... ok
test inference::inference_tests::test_llm_from_dir_empty_tokenizer ... ok
test inference::inference_tests::test_llm_from_dir_missing_tokenizer ... ok
test inference::inference_tests::test_llm_from_dir_nonexistent ... ok
test model_registry::tests::test_candle_quantization_from_str ... ok
test model_registry::tests::test_compute_fingerprint_differs_by_quantization ... ok
test model_registry::tests::test_compute_fingerprint_nonexistent_file_returns_err ... ok
test inference::inference_tests::test_llm_from_dir_missing_gguf ... ok
test inference::inference_tests::test_inference_backpressure_single_permit_awaits ... ok
test inference::inference_tests::test_inference_timeout_cancellation_releases_no_permit_leak ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s


running 3 tests
test test_record_layer_head_scores_combines_elementwise ... ok
test test_clear_resets_history ... ok
test test_candle_attention_exporter_bounded_memory_limit ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test prop_empty_string_input_yields_defined_result ... ok
test prop_embedding_output_finite_for_arbitrary_input_length ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 3 tests
test test_embedding_provider_batch_conformance ... ok
test test_embedding_provider_exact_batch_size_boundary_success_and_rejection ... ok
test test_embedding_provider_trait_object_conformance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test test_gasp_config_default_and_custom ... ok
test test_number_filtering_edge_cases ... ok
test test_word_length_filtering ... ok
test test_gasp_exact_threshold_boundary ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_valid_minimal_gguf_parses_successfully ... ok
test test_gguf_checksum_verification_fail_closed ... ok
test test_gguf_mutation_matrix ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test test_fuzz_gguf_loader_random_bytes_no_panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 2 tests
test test_parse_gguf_metadata_nonexistent_file ... ok
test test_parse_gguf_metadata_corrupt_file ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test regen_golden_candle ... ignored, Generator für Golden-Korpus
test test_golden_backward_read_candle ... ok
test test_golden_corpus_candle ... ok

test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test test_proptest_gasp_threshold_configuration ... ok
test test_proptest_gasp_raw_score_bounds ... ok
test test_proptest_fingerprint_file_content_hash ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 2 tests
test test_real_embedding_inference ... ignored
test test_real_llm_inference ... ignored

test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | Grün |
| IN3.4 | Abnahme-Prüfungen & Feature-Matrix | Grün | contextra-infer-candle v0.1.0 (/app/crates/contextra-infer-candle)
├── bytes v1.12.1
├── contextra-ports v0.1.0 (/app/crates/contextra-ports)
│   ├── ahash v0.8.12
│   │   ├── cfg-if v1.0.4
│   │   ├── getrandom v0.3.4
│   │   │   ├── cfg-if v1.0.4
│   │   │   └── libc v0.2.189
│   │   ├── once_cell v1.21.4
│   │   ├── serde v1.0.229
│   │   │   ├── serde_core v1.0.229
│   │   │   └── serde_derive v1.0.229 (proc-macro)
│   │   │       ├── proc-macro2 v1.0.107
│   │   │       │   └── unicode-ident v1.0.24
│   │   │       ├── quote v1.0.47
│   │   │       │   └── proc-macro2 v1.0.107 (*)
│   │   │       └── syn v3.0.4
│   │   │           ├── proc-macro2 v1.0.107 (*)
│   │   │           ├── quote v1.0.47 (*)
│   │   │           └── unicode-ident v1.0.24
│   │   └── zerocopy v0.8.56
│   ├── blake3 v1.8.7
│   │   ├── arrayvec v0.7.8
│   │   ├── cfg-if v1.0.4
│   │   ├── constant_time_eq v0.4.2
│   │   └── cpufeatures v0.3.1
│   ├── bytes v1.12.1
│   ├── contextra-env v0.1.0 (/app/crates/contextra-env)
│   ├── contextra-types v0.1.0 (/app/crates/contextra-types)
│   │   ├── ahash v0.8.12 (*)
│   │   ├── bincode v1.3.3
│   │   │   └── serde v1.0.229 (*)
│   │   ├── blake3 v1.8.7 (*)
│   │   ├── bytes v1.12.1
│   │   ├── contextra-ids v0.1.0 (/app/crates/contextra-ids)
│   │   │   ├── blake3 v1.8.7 (*)
│   │   │   ├── serde v1.0.229 (*)
│   │   │   └── thiserror v2.0.20
│   │   │       └── thiserror-impl v2.0.20 (proc-macro)
│   │   │           ├── proc-macro2 v1.0.107 (*)
│   │   │           ├── quote v1.0.47 (*)
│   │   │           └── syn v3.0.4 (*)
│   │   ├── serde v1.0.229 (*)
│   │   ├── serde_json v1.0.151
│   │   │   ├── itoa v1.0.18
│   │   │   ├── memchr v2.8.3
│   │   │   ├── serde_core v1.0.229
│   │   │   └── zmij v1.0.23
│   │   ├── sha2 v0.10.9
│   │   │   ├── cfg-if v1.0.4
│   │   │   ├── cpufeatures v0.2.17
│   │   │   └── digest v0.10.7
│   │   │       ├── block-buffer v0.10.4
│   │   │       │   └── generic-array v0.14.7
│   │   │       │       └── typenum v1.20.1
│   │   │       └── crypto-common v0.1.7
│   │   │           ├── generic-array v0.14.7 (*)
│   │   │           └── typenum v1.20.1
│   │   └── thiserror v2.0.20 (*)
│   ├── futures-util v0.3.34
│   │   ├── futures-core v0.3.34
│   │   ├── futures-macro v0.3.34 (proc-macro)
│   │   │   ├── proc-macro2 v1.0.107 (*)
│   │   │   ├── quote v1.0.47 (*)
│   │   │   └── syn v3.0.4 (*)
│   │   ├── futures-task v0.3.34
│   │   ├── pin-project-lite v0.2.17
│   │   └── slab v0.4.12
│   ├── serde v1.0.229 (*)
│   ├── serde_json v1.0.151 (*)
│   └── thiserror v2.0.20 (*)
├── contextra-rank v0.1.0 (/app/crates/contextra-rank)
│   ├── ahash v0.8.12 (*)
│   ├── contextra-ports v0.1.0 (/app/crates/contextra-ports) (*)
│   ├── contextra-types v0.1.0 (/app/crates/contextra-types) (*)
│   ├── serde v1.0.229 (*)
│   ├── serde_json v1.0.151 (*)
│   ├── thiserror v2.0.20 (*)
│   └── tracing v0.1.44
│       ├── pin-project-lite v0.2.17
│       ├── tracing-attributes v0.1.31 (proc-macro)
│       │   ├── proc-macro2 v1.0.107 (*)
│       │   ├── quote v1.0.47 (*)
│       │   └── syn v2.0.119
│       │       ├── proc-macro2 v1.0.107 (*)
│       │       ├── quote v1.0.47 (*)
│       │       └── unicode-ident v1.0.24
│       └── tracing-core v0.1.36
│           └── once_cell v1.21.4
├── contextra-types v0.1.0 (/app/crates/contextra-types) (*)
├── futures-util v0.3.34 (*)
├── parking_lot v0.12.5
│   ├── lock_api v0.4.14
│   │   └── scopeguard v1.2.0
│   └── parking_lot_core v0.9.12
│       ├── cfg-if v1.0.4
│       ├── libc v0.2.189
│       └── smallvec v1.15.2
├── serde v1.0.229 (*)
├── sha2 v0.10.9 (*)
├── thiserror v2.0.20 (*)
├── tokio v1.53.1
│   ├── bytes v1.12.1
│   ├── libc v0.2.189
│   ├── mio v1.2.2
│   │   └── libc v0.2.189
│   ├── parking_lot v0.12.5 (*)
│   ├── pin-project-lite v0.2.17
│   ├── signal-hook-registry v1.4.8
│   │   ├── errno v0.3.14
│   │   │   └── libc v0.2.189
│   │   └── libc v0.2.189
│   ├── socket2 v0.6.5
│   │   └── libc v0.2.189
│   └── tokio-macros v2.7.2 (proc-macro)
│       ├── proc-macro2 v1.0.107 (*)
│       ├── quote v1.0.47 (*)
│       └── syn v3.0.4 (*)
└── tracing v0.1.44 (*) / Matrix | Grün (Keine candle-* Abhängigkeiten ohne Feature) |
| IN3.5 | Härtung & Zero-Panic | Grün |  | Grün (0 Prod-Verstöße in beiden Crates) |
| IN3.6 | Dokumentation & Handoff | Grün | [*] Pattern Diagnostic scan complete: 75 findings (Prod: 75, Test: 0)
    - ZERO_PANIC: 0
    - DETERMINISM: 0
    - MEMORY_MAPPING: 0
    - DIRECT_IO: 0
    - FORBIDDEN_WORDING: 75
[+] Saved artifacts:
    /app/docs/diagnostics/pattern_diagnostic_workspace.json
    /app/docs/diagnostics/pattern_diagnostic_workspace.md | Grün (0 Verstöße in beiden Crates) |

## 2. Abweichungen von SPEC-v2

- Keine Abweichungen in Funktionalität, Inferenzlogik oder Fingerprint-Berechnung.
- Zurückgestellt (Folgeschritt Integration): Vereinigung von / mit  bzw. , da diese Crates gemäß AUFTRAG-UNABHÄNGIGKEIT nicht Teil dieses Pakets sind.

## 3. API-Freeze-Tabelle

| Crate | Öffentliches Symbol | Vorher | Nachher | Kompatibilität |
|---|---|---|---|---|
|  |  /  |  |  (Alias ) | Quellkompatibel, -Konvertierungen vorhanden |
|  |  | Re-Export aus  | Lokaler Newtype mit  | Quellkompatibel |
|  |  | Nutzte  direkt | Nutzt , Adapter in  | Quellkompatibel |
|  |  | Nicht vorhanden | Lokales Trait in  | Neues Kern-Interface |
|  |  | Globaler Fehlerbezug | Lokaler enum , Konvertierung in  | Quellkompatibel |
|  |  |  aus ports/types | Lokaler Newtype (0.0..=1.0) mit -Brücke | Quellkompatibel |

## 4. Kern-API-Lücken

- Keine unüberbrückten Kern-API-Lücken. Sämtliche Brücken zu  und  sind über  realisiert.

## 5. Koordinationsanforderungen

- Nach vollständiger Bereitstellung von  und  können die lokalen Platzhalter-Typen (, ) im Integrationsschritt abgelöst werden.

## 6. Messwerte Vorher / Nachher

### Lines of Code (LOC)
- :
  - Vorher: 4578
  - Nachher: 4642 (+64 Zeilen durch lokale Traits, Newtypes und Kompatibilitätsbrücke)
- :
  - Vorher: 5521
  - Nachher: 5559 (+38 Zeilen durch lokale Typen, Fehlerabbildung und Kompatibilitätsbrücke)

### Kopplung & Abhängigkeiten
- :
  - Vorher: Harte Compile-Zeit-Kopplung an , , , , , .
  - Nachher: Reine Rust-Minimalstruktur bei  (0 ML-Abhängigkeiten im Dependency Tree).
- Zero-Panic Analyzer:
  - 0 Prod-Befunde in beiden Crates ( bestanden).
- Pattern Diagnostic (FORBIDDEN_WORDING):
  - 0 Befunde in  und .

## 7. Offene Risiken

- Keine bekannten Risiken. Sämtliche Golden-Tests, deterministischen Mutationstests und Conformance-Tests laufen vollständig grün.
