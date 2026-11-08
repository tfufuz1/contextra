# Error Topology & Conversion Architecture Report

> **Werkzeug:** `error_topology_mapper.py`  
> **Erstellungszeit:** `2026-11-07T23:01:33.302943+00:00`  
> **Status/Befunde:** `12 Befunde erfasst`  

## Beschreibung
Maps strongly-typed error definitions (thiserror), variants, error codes, and cross-boundary error conversion implementations (impl From<E>).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_error_enums` | `2` |
| `total_error_variants` | `73` |
| `total_conversions_from` | `10` |

## Erfasste Error-Klassen
| Crate | Error-Typ | Zeile | Varianten | Datei |
|---|---|---|---|---|
| `contextra-types` | `HnswDeletionError` | 38 | 4 | `crates/contextra-types/src/error.rs` |
| `contextra-types` | `ContextraError` | 129 | 69 | `crates/contextra-types/src/error.rs` |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-types` | `crates/contextra-types/src/error.rs` | `38` | Error enum `HnswDeletionError` with 4 variants |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `129` | Error enum `ContextraError` with 69 variants |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `81` | impl From<u64> for StepId |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `499` | impl From<std::array::TryFromSliceError> for ContextraError |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `75` | impl From<ContextraError> for ContextraErrorDto |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `81` | impl From<String> for ContextraErrorDto |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `62` | impl From<FusionStrategy> for SignalFusionStrategies |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `152` | impl From<u64> for CollectionId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `277` | impl From<u64> for DocId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `284` | impl From<u128> for DocId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `366` | impl From<u64> for EntityId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `401` | impl From<String> for EntityId |

---
*Automatisch generiert durch `error_topology_mapper.py` für Agentic Engineering & Context Auditing.*