# ADR-N29: Dekomposition des God-Objects `Collection` in modulare Sub-Komponenten und Port-Traits

* **Status:** Proposed
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** `crates/contextra-engine` (`collection/mod.rs`, `crud/`, `search/`, `relate/`, `tx/`, `maintenance/`, `query_builder/`), `crates/contextra-ports`
* **Spezifikationsreferenzen:** Spezifikation §05 (Speicherschicht), §06 (Wissensgraph), §07 (Retrieval Pipeline), §20.2 (Phase 3b / 5), Prinzipien P1, P26, P27, P30

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine detaillierte Codebase-Analyse am HEAD zeigt, dass `Collection` in `crates/contextra-engine/src/collection/mod.rs` ein klassisches God-Object darstellt.

### 1.1 Metriken & Umfang
* **Hauptdatei:** `crates/contextra-engine/src/collection/mod.rs` (1277 LOC gesamt).
* **Struct-Definition:** `crates/contextra-engine/src/collection/mod.rs:603–638` (21 Felder inkl. `auto_extraction`).
* **Submodule / `impl`-Blöcke unter `collection/`:** 38 `impl`-Blöcke verteilt über 20 Quellcodedateien:
  - `crud/` (`insert.rs`, `read.rs`, `delete.rs`, `update.rs`, `links.rs`, `auto_extraction.rs`) — Document-CRUD & Auto-Extraction.
  - `search/` (`hybrid.rs`, `vector.rs`, `bm25.rs`, `checkpoint.rs`) — Multi-Signal-Retrieval Pipeline & RRF.
  - `relate/` — Graph-Relationen & Zettelkasten-Memory-Links.
  - `tx/` — MVCC Transaction isolation & snapshot reads.
  - `maintenance/` — Index-Rebuilds, Memory-Decay, Expiry & Community Detection.
  - `query_builder/` — `HybridQueryBuilder` Konstruktion & Signal-Gewichtung.
  - `kv_lock.rs` — Key-granulares Lock-Sharding (`KvKeyLocks`).

### 1.2 Vollständige Feldtabelle von `Collection` (21 Felder)

| Nr. | Feldname | Typ | Verwender-Dateien | Lebenszyklus / Zweck |
| :--- | :--- | :--- | :--- | :--- |
| 1 | `name` | `String` | `mod.rs`, `crud/`, `search/`, `tx.rs` | Instanz-Konstante; Identifikationsname des Namespaces. |
| 2 | `prefix` | `Vec<u8>` | `mod.rs`, `crud/`, `tx.rs` | Instanz-Konstante; Key-Präfix (`__col:{name}:\x00` bzw. leer für default). |
| 3 | `index` | `Arc<V>` (Default `HnswIndex`) | `mod.rs`, `crud/`, `search/`, `maintenance/` | Shared Arc; Vektor-Index-Instanz für semantische Ähnlichkeitssuche. |
| 4 | `text_index` | `InvertedIndex<S>` | `mod.rs`, `crud/`, `search/`, `maintenance/` | Inverted Text Index; In-Memory/Disk BM25 Posting-Listen. |
| 5 | `graph_index` | `Arc<CsrGraph>` | `mod.rs`, `relate/`, `search/`, `maintenance/` | Shared Arc; CSR Graph für Entitäts-Beziehungen & PPR-Walks. |
| 6 | `storage` | `Arc<S>` (Default `LsmStorage`) | `mod.rs`, `crud/`, `tx.rs`, `maintenance/` | Shared Arc; LSM Storage Engine für physikalische Persistenz. |
| 7 | `next_tx` | `Arc<AtomicU64>` | `mod.rs`, `crud/`, `tx.rs` | Shared Atomic; globaler/instanzweiter Monotonie-Zähler für Transaktions-IDs. |
| 8 | `dimension` | `usize` | `mod.rs`, `crud/insert.rs`, `search/` | Instanz-Konstante; Dimension der Einbettungs-Vektoren. |
| 9 | `embedder` | `parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>` | `mod.rs`, `crud/insert.rs`, `auto_extraction.rs` | Dynamische Injektion; Text-Embedder zur On-the-Fly-Vektorisierung. |
| 10 | `consolidation_guard` | `Arc<tokio::sync::Mutex<()>>` | `mod.rs`, `maintenance/` | Mutex-Lock; Serialisierung von Hintergrund-Konsolidierungsläufen. |
| 11 | `kv_locks` | `Arc<kv_lock::KvKeyLocks>` | `mod.rs`, `crud/`, `relate/` | Sharded Lock-Manager; Verhindert TOCTOU-Races bei parallelen Key-Updates. |
| 12 | `kv_store` | `Option<Arc<TenantIsolatedKvStore>>` | `mod.rs`, `crud/delete.rs`, `tx.rs` | Optional (`cfg(feature = "encryption-at-rest")`); KV-Cache Rollback Invalidierung. |
| 13 | `mutations_since_community_detection` | `Arc<AtomicU64>` | `mod.rs`, `relate/`, `maintenance/` | Atomic Counter; Zählt Graph-Mutationen für auto-getriggerte Community Detection. |
| 14 | `community_detection_trigger_threshold` | `Arc<AtomicU64>` | `mod.rs`, `maintenance/` | Atomic Threshold; Schwelle für automatische Community-Erkennung. |
| 15 | `consolidation_in_progress` | `Arc<AtomicBool>` | `mod.rs`, `maintenance/` | Atomic Flag; verhindert parallele Doppel-Trigger der Konsolidierung. |
| 16 | `auto_extraction` | `Arc<RwLock<Option<AutoExtractionBinding>>>` | `mod.rs`, `crud/auto_extraction.rs` | Shared RwLock; Bindung für automatische Triple- & Entity-Extraktion (P08). |
| 17 | `config` | `parking_lot::RwLock<CollectionConfig>` | `mod.rs`, `crud/insert.rs` | Konfiguration; Backpressure-Delay und Sammlungs-Parameter. |
| 18 | `pressure_rx` | `parking_lot::RwLock<Option<watch::Receiver<SystemPressure>>>` | `mod.rs`, `crud/insert.rs` | Watch-Channel; Empfang von System-Pressure-Signalen aus dem LSM-Store. |
| 19 | `clock` | `parking_lot::RwLock<Arc<dyn Clock>>` | `mod.rs`, `crud/`, `tx.rs` | Injizierter Port (P28); Deterministische Zeitquelle für Inserte/Deletes. |
| 20 | `metrics` | `parking_lot::RwLock<Arc<dyn MetricsSink>>` | `mod.rs`, `crud/`, `search/` | Injizierter Port; Erfassung operationaler Performanz-Metriken. |
| 21 | `last_load_report` | `parking_lot::Mutex<Option<IndexLoadReport>>` | `mod.rs` | Mutex; speichert den Ergebnis-Bericht des letzten `load_index`-Scans. |

---

## 2. Soll-Zustand & Dekomposition in Sub-Komponenten

Um Prinzip P30 (Crate-/Modul-Schnittkriterien) und P26 (Sync-Kern, async-Schale) einzuhalten, wird die `Collection` in klar abgegrenzte Sub-Komponenten aufgeteilt, die über Port-Traits miteinander interagieren.

### 2.1 Gruppierung nach Verantwortung

1. **`IndexRegistry` / Index-Manager:**
   - **Aufgabe:** Verwalter der Drei-Index-Architektur (Vektor `V`, Text `InvertedIndex`, Graph `CsrGraph`).
   - **Felder:** `index` (#3), `text_index` (#4), `graph_index` (#5), `dimension` (#8).
   - **Entkopplung:** Exponiert synchrones/asynchrones Trait `CollectionIndexSet`.

2. **`DocumentStore` / CRUD-Engine:**
   - **Aufgabe:** Schreiben und Lesen von Dokumenten-Metadaten, Payloads und Namespacing.
   - **Felder:** `name` (#1), `prefix` (#2), `storage` (#6), `next_tx` (#7), `kv_locks` (#11), `kv_store` (#12).

3. **`AutoExtractionController`:**
   - **Aufgabe:** Automatische Triple-Extraktion (P08) und LLM-Einbindung.
   - **Felder:** `embedder` (#9), `auto_extraction` (#16). Registrierung verbleibt getrennt vom Kern im Inferenz-Port.

4. **`MaintenanceCoordinator`:**
   - **Aufgabe:** Hintergrund-Tasks, Memory-Decay, Community Detection & Task-Drosselung.
   - **Felder:** `consolidation_guard` (#10), `mutations_since_community_detection` (#13), `community_detection_trigger_threshold` (#14), `consolidation_in_progress` (#15), `last_load_report` (#21).

5. **`OperationalContext` / Environment Ports:**
   - **Aufgabe:** Injizierte Ports für Zeit, Metriken und Backpressure.
   - **Felder:** `config` (#17), `pressure_rx` (#18), `clock` (#19), `metrics` (#20).

---

## 3. Schnittstellen zu Nachbar-Aufgaben (P15 und P18)

* **Abhängigkeit zu P15 (KV-Store-Entscheidung):**
  Feld #12 (`kv_store`) verweist derzeit direkt auf `TenantIsolatedKvStore`. Bei der Refaktorisierung wird der KV-Store hinter das allgemeine Port-Trait `ErasureBackend` / `TenantKvCache` abstrahiert. Dies verhindert direkte Kompilierabhängigkeiten zwischen `Collection` und der physischen KV-Store-Implementierung.
* **Abhängigkeit zu P18 (System-Härtung / Phase-Exit Gates):**
  Die Dekomposition von `Collection` wird in Phase 5 (§20.2) abgeschlossen ("God-Files zerlegen"). Testabdeckungen über `collection_contract.rs` und `concurrent_collection_stress.rs` sichern ab, dass keine Regressionsfehler im Locking oder in der Snapshot-Isolationslogik auftreten.

---

## 4. Migrationsreihenfolge nach Risiko & Absichernde Tests

Die Migration erfolgt schrittweise in vier strikt isolierten PRs, um kaskadierende CI-Brüche zu vermeiden:

```
[PR-1: Ports & Sub-Structs] ──> [PR-2: Search & CRUD Trait-Splitting] ──> [PR-3: Extraction & Maintenance] ──> [PR-4: Collection Wrapper Struct]
```

1. **PR 1 (Geringes Risiko): Extrahieren der Environment-Ports (`OperationalContext`)**
   - Zusammenfassung von `clock`, `metrics`, `config`, `pressure_rx` in eine eigene Sub-Struct `OperationalContext`.
   - *Absichernder Test:* `crates/contextra-engine/tests/collection_contract.rs`
   - *Prüfbefehl:* `cargo test -p contextra-engine --test collection_contract`

2. **PR 2 (Mittleres Risiko): Kapselung des `IndexRegistry` (Vektor-, Text-, Graph-Indizes)**
   - Extraktion der Indizes in `IndexRegistry` und Auslagerung der Multi-Signal-Such-Logik (`search/`).
   - *Absichernde Tests:* `crates/contextra-db/tests/proptest_search_invariants.rs`, `crates/contextra-db/tests/graph_hybrid_search.rs`
   - *Prüfbefehl:* `cargo test -p contextra-db --test proptest_search_invariants`

3. **PR 3 (Mittleres Risiko): Maintenance & Auto-Extraction Entkopplung**
   - Auslagerung der Community Detection und Konsolidierungs-Guards aus dem Haupt-Header.
   - *Absichernder Test:* `crates/contextra-db/tests/auto_community_detection_test.rs`
   - *Prüfbefehl:* `cargo test -p contextra-db --test auto_community_detection_test`

4. **PR 4 (Hohes Risiko): Reduktion der Struct `Collection` auf Sub-Komponenten-Aggregat**
   - `Collection` besteht nur noch aus den 5 Sub-Komponenten.
   - *Absichernde Tests:* `crates/contextra-db/tests/concurrent_collection_stress.rs`, `crates/contextra-db/tests/full_stack_e2e.rs`
   - *Prüfbefehl:* `cargo test --workspace --locked`

---

## 5. Risiken & Mitigationen

| Risiko | Schwere | Mitigation |
| :--- | :--- | :--- |
| **Deadlocks durch veränderte Lock-Acquisition-Reihenfolge** | Hoch | Lock-Hierarchie (`kv_locks` -> `embedder`) wird explizit in den Sub-Komponenten beibehalten. Absicherung über `loom_relate_n_ary.rs`. |
| **Performance-Regressions im heißen Such-Pfad** | Hoch | Indirektionen über Trait-Objekte im Suchpfad vermeiden; Monomorphisierung via Generics (`S: StorageEngine, V: VectorIndex`) beibehalten. |
| **Rollback-Fehler im KV-Cache bei Tx-Abbruch** | Mittel | Unit-Tests in `tx.rs` sichern die synchrone Invalidierung über `kv_store` ab. |

---

## 6. Offene Fragen

1. Soll `InvertedIndex<S>` künftig als trait-objektfähiger Port `dyn TextIndex` deklariert werden (gemäß P27), um alternative BM25-Backends ohne Re-Kompilierung zu ermöglichen?
2. Bietet es sich an, `KvKeyLocks` aus `collection/kv_lock.rs` in `contextra-ports` oder ein eigenes Hilfs-Crate zu verschieben, da auch andere Subsysteme sharded Key-Locks benötigen?
