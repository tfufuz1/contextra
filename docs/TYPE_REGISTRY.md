# Contextra – Typ-Registry & Kanonische Typen

<!-- doc-stamp: 2026-10-31 | Normativ: docs/SPEC.md Fassung 2.4 §16 (Regel 11) | PLAN.md §4.3 -->

> **Regel 11 (Eine Typfamilie je Konzept):** Jeder Domänenbegriff, Port und Typ existiert genau einmal an seinem kanonischen Ort im DAG. Doppelte Typdefinitionen über Crate-Grenzen hinweg sind Architektur-Schulden und werden konsolidiert.

---

## 1. Kanonische Typ-Verteilung nach Ringen

| Konzept / Typ | Kanonischer Ort | Erlaubte Re-Exports | Status |
|---|---|---|---|
| `RequestContext`, `TenantId`, `UserId` | `contextra-types::types::domain` | `contextra`, `contextra-db` | ✅ Eindeutig |
| `ContextraError`, `ErrorCode` | `contextra-types::error` | Workspace-weit | ✅ Eindeutig |
| `DocDek`, `GroupKek`, `KeyRegistry` | `contextra-crypto::kv_shredding` | `contextra-store` | ✅ Eindeutig |
| `StorageEngine`, `Clock`, `Rng`, `IdGen`, `Vfs` | `contextra-ports` | Workspace-weit | ✅ Eindeutig |
| `FusionWeights` | `contextra-types` | `contextra-rank`, `contextra-py` | 🟡 In Bereinigung (B-07) |
| `MetadataFilter` | `contextra-db::filter` | `contextra-engine`, `contextra` | 🟡 In Bereinigung (B-12) |
| `InMemoryStorageEngine` | `contextra-store::engine::memory_only` | `contextra-testkit` | 🟡 Re-Export in Testkit vereinheitlichen |
| `BoundedTopK` | `contextra-rank::fusion::topk` | `contextra-engine` | 🟡 Text-Variante konsolidieren |
| `HybridQueryBuilder` | `contextra-engine::collection::query_builder` | `contextra` | 🟡 SAOS-Rest in `types` entfernen |
| `Signal` | `contextra-rank::fusion::types` | – | 🟡 SAOS-Rest in `types` entfernen |

---

## 2. Bewusste, legitime Namensgleichheiten

Folgende Typnamen sind bewusst in getrennten Modulen definiert, da sie unterschiedliche Domänenkonzepte repräsentieren und durch Module sauber getrennt sind:

| Name | Fundstellen | Fachliche Begründung |
|---|---|---|
| `Result<T>` | `contextra_types::Result`, `contextra_crypto::Result`, `contextra_ports::Result` | Typaliase für systemspezifische Fehlerdomänen. Vermeidet syntaktisches Rauschen. |
| `DocId` vs. `PostingDocId` | `contextra_types::DocId` (UUID/String) vs. `contextra_text::DocIdRaw` (`u32`) | Die Domänen-Dokument-ID ist ein mandantenspezifischer Identifier; die Volltext-Posting-ID ist ein kompakter 32-Bit-Slot im invertierten Index. |

---

## 3. Bereinigungsplan für Implementierer

1. **B-07 (`contextra-py`):** In `crates/contextra-py/src/crud_macros.rs` den Import von `contextra_rank::FusionWeights` auf `contextra_types::FusionWeights` umstellen.
2. **B-12 (`contextra-db`):** In `crates/contextra-db/tests/filter_tests.rs:6` den Importpfad von `contextra_db::MetadataFilter` auf `contextra_db::filter::MetadataFilter` korrigieren.
3. **SAOS-Relikte:** `crates/contextra-types/src/types/saos.rs` entfernen oder deprecaten, um Doppeldefinitionen von `Signal` und `HybridQueryBuilder` vollständig aufzulösen.
