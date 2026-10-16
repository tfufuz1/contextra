# AGENTS.md — contextra-types
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.30, §0, §3, §4, §20, §D

1. Zweck
`contextra-types` stellt die zentralen, kanonischen Domänentypen, Identifikatoren (`DocId`, `TxId`, `TenantId`), Filter-ASTs, Budget- und Fehlerstrukturen für das gesamte Contextra-Workspace bereit.
Die Crate ist strikt synchron (P26), frei von I/O- oder Async-Runtimes und erzwingt `#![forbid(unsafe_code)]`.

2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte der zentralen Typen und Modul-Deklarationen. |
| `src/error.rs` | `ContextraError` als zentrales Fehler-Enum für alle Workspace-Crates. |
| `src/error_dto.rs` | `ContextraErrorDto` für serialisierbare Fehlerübertragung an FFI/IPC-Grenzen. |
| `src/model_fingerprint.rs` | `ModelFingerprint` zur Eindeutigkeit von Einbettungs- und LLM-Modellgewichten. |
| `src/retrieval_strategy.rs` | `RetrievalStrategy` zur Kennzeichnung der Retrieval-Modi (Vector, Text, Graph, Hybrid, Global). |
| `src/schema.rs` | `ManifestSchemaVersion` und `DocIdWidth` für Schema-Kompatibilität (64-bit vs. 128-bit). |
| `src/tenant_scope.rs` | `TenantScoped` Wrapper-Typ und `TenantScopeViolation` für Mandantenisolierung. |
| `src/tombstone.rs` | `TombstoneSemanticsCheck` Trait und `SeqBitTombstone` zur Marker-Prüfung. |
| `src/types/budget.rs` | `TokenBudget`, `Reservation`, `ResourceBudget` und `ResourceTracker`. |
| `src/types/domain/ids.rs` | Kanonische Identifikatoren: `TenantId`, `CollectionId`, `DocId`, `EntityId`, `TxId`. |
| `src/types/domain/document.rs` | `DistanceMetric`, `Embedding`, `ScoredDocument` und `RerankResult`. |
| `src/types/domain/misc.rs` | Graphentypen (`Entity`, `Edge`), Zettelkasten-Links, Bitemporalität, TTL-Metadaten. |
| `src/types/filter.rs` | Metadaten-Filter-AST (`FilterExpr`) und Evaluierungs-Engine. |
| `src/types/importance.rs` | `ImportanceScore`, `DecayFunction` und `MemoryImportance`. |
| `src/types/math.rs` | Mathematische Hilfsfunktionen (`cosine_similarity`). |
| `src/types/saos.rs` | SAOS-Typen: `HybridQuery`, `HybridQueryBuilder`, `ContextChunk`, `ContextWindow`, `FusionWeights`. |
| `src/types/saturating.rs` | `SaturatingU8` Saturated Math Wrapper. |

3. Invarianten
- **INV-TENANT-1:** `TenantId::try_new(0)` muss immer ein `Err(ContextraError::InvalidInput)` liefern. Test: `cargo test -p contextra-types --lib types::domain::tests::ids_tests::test_tenant_id_zero_rejected`.
- **INV-TYPE-REGISTRY:** Neue öffentliche Typen müssen zwingend in `docs/TYPE_REGISTRY.md` eingetragen sein. Prüfung: `cargo xtask check-type-registry <TypName>`.
- **INV-DOCID-IDENTITY:** `DocId` kapselt standardmäßig `u64`. Unter Feature `docid-128` kapselt `DocId` `u128` (ADR-082 / Schema V2).

4. Verboten / Anti-Patterns
- **VERBOTEN:** Verwendung von `TenantId::new(0)` in Produktionspfaden (`TenantId::new(0)` ist deprecated, stets `TenantId::try_new` nutzen).
- **VERBOTEN:** Ad-hoc-Fehlertypen außerhalb von `ContextraError` an öffentlichen Crate-Grenzen definieren.
- **VERBOTEN:** Async-Blockaden, I/O oder Abhängigkeiten zu `tokio` einfügen.

5. Nebenläufigkeit, Async- und Lock-Regeln
- Ring 0 Purity (P26): Keine Async-Runtimes (`tokio`), keine Mutexes/RwLocks im Datenpfad von `contextra-types`.
- Alle Typen sind reine Datenstrukturen (`Send + Sync`, überwiegend `Clone` / `Copy` wo anwendbar).

6. Verifikation
- `cargo test -p contextra-types`
- `cargo xtask check-type-registry TenantId`

7. Bekannte Lücken / SOLL
- `TenantId::new()` existiert noch als Unchecked-Initializer für Legacy-Kompatibilität, ist jedoch als deprecated markiert.
