# Anti-Pattern & Invariant Violation Diagnostic Report

> **Werkzeug:** `pattern_diagnostic.py`
> **Erstellungszeit:** `2026-11-07T22:58:41.513098+00:00`
> **Status/Befunde:** `225 Befunde erfasst`

## Beschreibung
Comprehensive audit of Zero-Panic (unwrap/expect), Determinism (Clock/Rng/Hash), Memory Mapping (mmap), Direct I/O, and §6.4 Forbidden Wording invariants.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_findings` | `225` |
| `production_violations` | `49` |
| `test_violations` | `176` |
| `category_zero_panic` | `223` |
| `category_determinism` | `0` |
| `category_memory_mapping` | `0` |
| `category_direct_io` | `0` |
| `category_forbidden_wording` | `2` |

## Kategorien-Übersicht
| Kategorie | Funde Gesamt | Beschreibung |
|---|---|---|
| `ZERO_PANIC` | 223 | `unwrap()`, `expect()`, `panic!()` in Code |
| `DETERMINISM` | 0 | Zeit/Zufall/Hasher ohne Ports (P28) |
| `MEMORY_MAPPING` | 0 | Verbotene mmap-Nutzung (D-94) |
| `DIRECT_IO` | 0 | Direkte std::fs Umgehung von Vfs |
| `FORBIDDEN_WORDING` | 2 | Garantieversprechen / DSGVO-Labels (§6.4) |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-types` | `crates/contextra-types/src/error.rs` | `579` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `595` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `610` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1008` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1059` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1076` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1089` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1121` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `1140` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `559` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `563` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `565` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/model_fingerprint.rs` | `74` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/model_fingerprint.rs` | `76` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/schema.rs` | `113` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/schema.rs` | `117` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/tombstone.rs` | `93` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `128` | Forbidden wording (§6.4): 'guaranteed' |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `467` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `480` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `484` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `491` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `500` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `503` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `519` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `525` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `548` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `562` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `636` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `651` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `682` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/budget.rs` | `683` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `218` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/filter.rs` | `219` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `642` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `654` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `682` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `696` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `698` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `699` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `710` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `714` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `721` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `766` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `807` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `808` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `814` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `870` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `922` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `48` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `51` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `55` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `64` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `67` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `74` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `78` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `98` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `107` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `116` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `126` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `135` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `160` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `161` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `163` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `164` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `216` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `222` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `228` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `236` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `238` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/doc_tests.rs` | `240` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `19` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `20` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `22` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `23` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `34` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `35` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `53` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `54` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `58` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `81` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `82` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `89` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `94` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `110` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `113` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `143` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `264` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `269` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `274` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `305` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `326` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `327` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `331` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `332` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `370` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `374` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `381` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `403` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `419` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `442` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/ids_tests.rs` | `468` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `21` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `25` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `40` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `41` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `42` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `46` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `47` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `52` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `53` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `58` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `59` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `64` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `65` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `85` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `86` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `103` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `104` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `112` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `120` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `132` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `138` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `160` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `165` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `170` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `179` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `180` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `186` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `187` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `204` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `205` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `212` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/src/types/domain/tests/misc_tests.rs` | `213` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `26` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `30` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `34` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `44` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `48` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `59` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `104` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `105` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/distance_metrics_and_embeddings_suite.rs` | `110` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `21` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `28` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `29` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `41` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `42` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `50` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `51` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `60` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `61` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `73` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `84` | Invocation of panic!() macro |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `95` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `112` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `123` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_and_witness_suite.rs` | `124` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/error_dto_and_schema_suite.rs` | `35` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `16` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `24` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `40` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `48` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `51` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `57` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `58` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/golden_bytes_suite.rs` | `62` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `33` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `34` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `107` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `111` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `135` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `139` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `162` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `163` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `172` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/model_fingerprint_and_schema_suite.rs` | `173` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `37` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `47` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `59` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `61` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `104` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `153` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `167` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/newtype_invariants_suite.rs` | `271` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `21` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `22` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `24` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `26` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `30` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `31` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `34` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `39` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `41` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `44` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `47` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `52` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `54` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `57` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `59` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `83` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `84` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `87` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `88` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `92` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `98` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `104` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `130` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `132` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `133` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `157` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `158` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `164` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `167` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `182` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `185` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `204` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/serde_roundtrips_suite.rs` | `205` | Call to .unwrap() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `2` | Forbidden wording (§6.4): 'garantiert' |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `11` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `20` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `26` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `32` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `33` | Call to .expect() in code |
| `contextra-types` | `crates/contextra-types/tests/tenant_scope_isolation.rs` | `59` | Call to .expect() in code |

---
*Automatisch generiert durch `pattern_diagnostic.py` für Agentic Engineering & Context Auditing.*