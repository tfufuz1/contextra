# Modulinventar und Abgrenzung: contextra-engine (Ring 3)

> **Datum:** 2026-11-02
> **Normative Grundlage:** SPEC , SPEC §5 (Ringmatrix), SPEC D-84 (Rückbauliste), SPEC §14.1/14.3 (Lifecycle)
> **Status:** Analysiert & Entschieden

---

## 1. Übersicht

`contextra-engine` orchestriert die Teilsysteme von Contextra (LSM-Store, HNSW-Vektorindex, BM25-Textindex) und stellt die Collection-Abstraktion, Mandanten-Isolation, Transaktionskoordination (2PC mit Intent-Log) sowie Hintergrundworker bereit.

Laut **SPEC D-84** und **AR-6** gilt:
- Engine fügt Schichten zusammen, enthält keine eigene Speicher- oder Kryptologik.
- Graph-Reste werden entfernt, nicht repariert (D-84).
- Unnötiger Fremdumfang (Chunker, Auto-Extraction, Wasm-Reste, Export/Import, Decayed-Controller) gehört nicht in den transaktionalen Kern des eingebetteten Datenbank-Kerns und ist für die Bereinigung bzw. Verlagerung vorgemerkt.

---

## 2. Modulinventar & Kategorisierung

| Modulpfad | Kategorie | Zeilen (ca.) | Befund / Bezug zu SPEC | Entscheidung |
|---|---|---|---|---|
| `collection/crud/delete.rs` | **Kern** | 85 | DEF-01: Lock vorzeitig gedroppt, Graph-Reste | **Behalten & Härten**: Lock-Guard bis Commit-Ende halten; Graph-Calls entfernt. |
| `collection/crud/update.rs` | **Kern** | 120 | DEF-01: Lock vorzeitig gedroppt | **Behalten & Härten**: Lock-Guard bis Commit-Ende halten. |
| `collection/crud/kv.rs` | **Kern** | 90 | DEF-01: Lock vorzeitig gedroppt | **Behalten & Härten**: Lock-Guard bis Commit-Ende halten. |
| `collection/crud/insert.rs` | **Kern** | 380 | TOCTOU-Kollision, Extraktions-Hooks | **Behalten**: Entkoppeln von Extraktion. |
| `collection/crud/read.rs` | **Kern** | 160 | Lesepfade, Prefix-Scans | **Behalten**. |
| `collection/crud/links.rs` | **Altlast** | 110 | Graph-/Memory-Linking außerhalb 2PC | **Entfernen / Bereinigen**: Graph-Entkopplung gem. D-84. |
| `collection/crud/auto_extraction.rs` | **Fremdumfang** | 140 | Auto-Triple-Extraction | **Entfernen** gem. D-84 (`auto-extraction-opt-out`, `entity-extraction`). |
| `collection/kv_lock.rs` | **Kern** | 160 | Sharded KV-Locks für Mutationsserialisierung | **Behalten & Härten** (Zero-Panic, Deadlock-Freiheit). |
| `collection/maintenance.rs` | **Kern** | 740 | DEF-01 (L707), Repair, Forward-Recovery | **Behalten & Härten**: Lock bis Transaktionsende, Scan-Optimierung. |
| `collection/query_builder/*` | **Kern** | 450 | Abfrage-Builder für Hybrid-Suchen | **Behalten**. |
| `collection/search/*` | **Kern** | 520 | Hybridsuche mit RRF und Rehydration | **Behalten**. |
| `transaction/db_transaction.rs` | **Kern** | 220 | Transaktionszustand, Staging | **Behalten & Bereinigen**: Graph-Staging entfernt, Drop-Rollback gehärtet. |
| `transaction/db_transaction_commit.rs` | **Kern** | 390 | 2PC Phase 2 (Commit-Protokoll), Kompensationsledger | **Behalten & Bereinigen**: Graph-Commit entfernt, TenantId dynamisiert. |
| `transaction/db_transaction_rollback.rs` | **Kern** | 230 | Transaktions-Rollback | **Behalten & Bereinigen**: Graph-Rollback entfernt. |
| `transaction/compensating_actions.rs` | **Kern** | 410 | Kompensationsaktionen für LSM, HNSW, Text | **Behalten & Bereinigen**: `CompensateGraphAction` entfernt. |
| `transaction/intent.rs` | **Kern** | 130 | Commit-Intent WAL | **Behalten**. |
| `transaction/cleanup.rs` | **Kern** | 150 | Intent-Recovery nach Crash | **Behalten & Härten**. |
| `background_workers/hyperedge_worker.rs` | **Altlast** | 15 | Leerer Stub | **Vollständig entfernt** (D-84). |
| `background_workers/orphan_workers.rs` | **Kern** | 180 | Bereinigung von Transaktionsresten | **Behalten**: Clock-Injektion sicherstellen. |
| `background_workers/expiry_workers.rs` | **Kern** | 160 | TTL-Bereinigung im Hintergrund | **Behalten**. |
| `extraction/*` | **Fremdumfang** | 800 | Regelbasierte & OpenIE Extraktion | **Entfernen / Ausgliedern** (D-84). |
| `chunker/*` | **Fremdumfang** | 1200 | Text-Splitting, Windowing, Schätzung | **Ausgliedern / Entfernen** (D-84). |
| `decay_controller.rs` | **Altlast** | 190 | Ebbinghaus-Decay Berechnungen | **Entfernen / Ausgliedern** (gehört zu Cognition/Agent, nicht Engine-Kern). |
| `temporal_filter.rs` | **Kern** | 120 | Zeitbereichsfilterung für MVCC/Metadaten | **Behalten**. |
| `fusion.rs` | **Altlast** | 210 | Legacy Fusion-Wrapper | **Entfernen / Delegieren an `contextra-rank`** (WP-8.2). |
| `merge_operator_adapter.rs` | **Altlast** | 25 | Stub / Wasm-Reste | **Entfernen** gem. D-84 (`WasmMergeOperator`). |
| `import.rs` / `export.rs` | **Fremdumfang** | 320 | Dump/Restore-Hilfen | **Ausgliedern** (Ring 4 Adapter). |
| `contextra_impl/*` | **Kern** | 650 | Fassaden-Implementierung | **Behalten & Umbenennen nach `instance`** (WP-1.5 / D-89). |

---

## 3. Umgesetzte Maßnahmen in diesem Arbeitszyklus

1. **Lock-Invariante (DEF-01)**:
   Entfernung von `drop(_guard)` in `delete.rs`, `update.rs`, `kv.rs`, `maintenance.rs`. Der Lock-Guard wird in einer lokalen Variable gehalten, die bis zum Abschluss des Commits oder Rollbacks gültig bleibt.
2. **Graph-Rückbau (D-84)**:
   - Löschung von `hyperedge_worker.rs` und Deregistrierung im Modul `background_workers`.
   - Entfernung aller `staged_graph_*` und `staged_hyperedges` aus `DbTransaction`.
   - Bereinigung von `db_transaction_commit.rs`, `db_transaction_rollback.rs` und `compensating_actions.rs` von allen `graph_index`-Aufrufen.
   - Entfernung des Aufrufs `check_and_trigger_community_detection` aus `delete.rs`, `insert.rs` und `update.rs`.
3. **Multi-Tenant Härtung (DEF-03)**:
   Ersatz der hardcodierten `TenantId(1)` durch den Mandanten der aktuellen Transaktion/Collection (`self.collection.tenant_id()`).
4. **Drop-Sicherheit (DEF-02)**:
   Entfernung der unkontrollierten Graph-Rollback-Operationen im `Drop`-Trait von `DbTransaction`.
