# Dead Code & Unused Item Microprobe Diagnostic

> **Werkzeug:** `dead_code_microprobe.py`  
> **Erstellungszeit:** `2026-11-07T22:58:49.146511+00:00`  
> **Status/Befunde:** `38 Befunde erfasst`  

## Beschreibung
Identifies unreferenced private functions, types, constants, #[allow(dead_code)] suppressions, and deprecated items across crates.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `dead_code_candidates` | `32` |
| `allow_dead_code_attributes` | `0` |
| `deprecated_attributes` | `6` |
| `total_findings` | `38` |

## Zusammenfassung der technischen Schulden
| Kategorie | Anzahl | Beschreibung |
|---|---|---|
| Unused Private Items | 32 | Private Funktionen, Structs oder Consts ohne Aufrufer im Crate |
| `#[allow(dead_code)]` | 0 | Unterdrückte Dead-Code-Warnungen |
| `#[deprecated]` | 6 | Veraltete Relikte zur Entfernung |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `38` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `41` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `145` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `207` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `316` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `462` | Deprecated item marked for removal |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `511` | Private function 'contextra_error_display_no_panic' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `772` | Private function 'prop_fusion_weights_never_panics' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `406` | Private function 'default_tl_hfd_sigma' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `409` | Private function 'default_tl_hfd_delta' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `412` | Private function 'default_tl_hfd_gamma' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `415` | Private function 'default_tl_hfd_max_iterations' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `418` | Private function 'default_tl_hfd_max_top_k_expansion' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `421` | Private function 'default_tl_hfd_max_hyperedge_sort_size' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/src/types/domain/misc.rs` | `481` | Private function 'default_warn_on_non_convergence' appears unused in crate (freq=1) |
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
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `148` | Private function 'prop_config_fingerprint_bit_exact_floats' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `160` | Private function 'prop_model_fingerprint_serde_roundtrip' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `32` | Private function 'prop_doc_id_from_key_properties' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `54` | Private function 'prop_entity_id_from_key_properties' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `76` | Private function 'prop_tx_id_ordering_and_display' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `98` | Private function 'prop_collection_id_invariants' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `113` | Private function 'prop_importance_score_clamping' appears unused in crate (freq=1) |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `125` | Private function 'prop_token_budget_consumption' appears unused in crate (freq=1) |

---
*Automatisch generiert durch `dead_code_microprobe.py` für Agentic Engineering & Context Auditing.*