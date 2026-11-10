# Spezifikation: `contextra-ports`

**Ring:** Ring 0 (Foundation)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §4, §6.3
**Klasse:** K (Kanonische Port-Verträge)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-ports` definiert die stabilen, synchronen und asynchronen Trait-Verträge und Schnittstellen für alle Entkopplungs-Bausteine der Contextra-Architektur. Es fungiert als der zentrale Inversion-of-Control (IoC)-Knotenpunkt:
- **Kern:** Definiert Trait-Verträge für Speichertriebwerke, Vektor-, Volltext- und Graph-Indizes, Dateisysteme (`Vfs`), Umgebungsabhängigkeiten (`Clock`, `Rng`, `IdGen`, `MetricsSink`), Inferenzanbindungen und neutrale Wartungsschnittstellen (`Purger`, `LayerScanner`).
- **Löschdomänen-Entkopplung:** Keine juristischen Begrifflichkeiten oder Saga-Orchestrierungen. Physische Bereinigungen und Messungen erfolgen ausschließlich über neutrale Schnittstellen.
- **Abhängigkeiten:** Ausschließlich `contextra-types` (bzw. auf Übergangsbasis zur Entkopplung).

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-ports`:*
- **LOC:** 4.746 LOC über 41 Rust-Dateien.
- **Typen-Inventar:** 46 Traits, 75 Structs, 20 Enums, 87 öffentliche Funktionen (`pub fn`).
- **Kopplung:** Direkte Bindung an `contextra-types` (42 Symbole).
- **Entkopplungsbefund:** 21 Verweise auf Löschketten/Purger-Begriffe in Legacy-Schnittstellen (`erasure_legacy.rs`, `contract_tests.rs`, `conformance.rs`), die gemäß SPEC §6.3 in Richtung `contextra-erasure` bzw. `testkit` migriert werden.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Verträge / Traits |
|---|---|---|
| `vfs` | Konsolidierte synchrone/asynchrone VFS-Abstraktion | `Vfs`, `VfsFile`, `VfsMetadata` |
| `storage` | Key-Value- und LSM-Speichertriebwerks-Port | `StorageEngine`, `KvPrefixStore` |
| `vector_index` | Vektorindex-Schnittstellen für HNSW und DiskANN | `VectorIndex`, `VectorSearchQuery` |
| `text_index` | BM25/BM25F-Volltext-Index-Schnittstellen | `TextIndex`, `TextMatch` |
| `graph_index` | Semantische Netze und CSR-Graph-Schnittstellen | `GraphIndex`, `EdgeSink` |
| `scanner` / `purge` | Neutrale Bereinigungs- und Scan-Primitive | `Purger`, `LayerScanner`, `ScanTarget` |
| `env` | Deterministische Umgebungs-Injektion | `Clock`, `Rng`, `IdGen`, `MetricsSink` |
| `inference` | LLM- und Embedding-Provider-Schnittstellen | `EmbeddingProvider`, `Reranker`, `QueryRewriter` |
| `lifecycle` | Komponentenlebenszyklus und Graceful Shutdown | `ComponentLifecycle`, `HealthCheck` |

---

## 4. Invarianten (INV-PRT-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-PRT-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. | `cargo check -p contextra-ports` |
| `INV-PRT-DETERMINISM-1` | Umgebungs-Traits (`Clock`, `Rng`, `IdGen`) erzwingen vollständigen Verzicht auf globale OS-Singletons im Pfad. | `src/contract_tests.rs` |
| `INV-PRT-PURGE-NEUTRAL` | `Purger`- und `LayerScanner`-Traits sind vollkommen neutral ohne juristische Bezüge oder Saga-Bindung definiert. | `tools/erasure_leak_prober.py` |
| `INV-PRT-FAIL-CLOSED` | Schnittstellen-Methoden erzwingen strukturierte Fehler-Rückgaben (`Result<T, PortError>`) ohne Panics. | `tools/zero_panic_analyzer.py` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. `clock`, `rng`, `id_gen`, `metrics` wandern schrittweise in `contextra-env`.
2. `conformance.rs` und `contract_tests.rs` werden mit `contextra-testkit` konsolidiert.
3. Legacy-Löschrelikte in `erasure_legacy.rs` verbleiben deprecated bis zur vollständigen Isolation in `contextra-erasure-spi`.
