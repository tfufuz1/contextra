# Vollständiger Audit-Bericht: `crates/contextra-db`

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect (Jules)
**Ziel-Crate:** `crates/contextra-db` (Strangler Shell / High-Level API Layer 3)
**Session:** `762ec1e8`

---

## 1. AGT-DB-001-Nachweis (TxId-Erzeugung)

**Invariante:** `TxId` MUSS IMMER über `collection.allocate_tx()` bezogen werden (deterministischer Zähler via atomaren `AtomicU64`-Inkrement, keine direkte Wall-Clock `SystemTime`).

### Prüf-Befehl
```bash
grep -rn "SystemTime::now()\|Instant::now()\|UNIX_EPOCH" crates/contextra-db/src/ | grep -v test
```

### Befund & Analyse
- **Ergebnis des Scans:** Der Scan liefert ein einziges Vorkommen in `crates/contextra-db/src/volatile_vault.rs:243`:
  ```rust
  PurgeReceipt {
      chunks_purged,
      bytes_zeroed,
      purged_at: Instant::now(),
  }
  ```
- **Bewertung:** `purged_at: Instant::now()` dient ausschließlich als lokale Quittungs-Zeitstempel-Information für In-Memory-`PurgeReceipt`-Messungen nach einer Speicherbereinigung. Es hat keinerlei Einfluss auf Kausalität, WAL, MVCC oder Transaktions-IDs.
- **TxId-Allokationspfad:** Sämtliche Transaktions-IDs (`TxId`) in `contextra-db` werden über `Collection::allocate_tx(&self)` in `contextra-engine` erzeugt. Diese Methode nutzt `self.next_tx.fetch_add(1, Ordering::SeqCst)` zur streng monotonen, atomaren Vergabe von `TxId`s. Es gibt in `crates/contextra-db/src/` keine Erzeugung von `TxId` über `SystemTime` oder `UNIX_EPOCH`.

`EVIDENCE[P1:AGT-DB-001]` — Konformität bestätigt.

---

## 2. Parallelitäts-Analyse der 4-Signal-Fusion

**Fragestellung:** Werden alle 4 Signal-Engines (Vector, Text, Graph, Storage/LSM) für eine `search()`-Anfrage parallel oder sequentiell abgefragt?

### Quelltext-Analyse (`crates/contextra-engine/src/collection/search/hybrid/query.rs`)
Die Abfrageverarbeitung in `Collection::hybrid_search_with_query_and_report_at` führt die Signalbeschaffung wie folgt aus:

1. **Vector Signal:**
   ```rust
   let vector_res = self.search_filtered_at(vector, candidate_k, None, seq).await;
   ```
2. **Text Signal (BM25):**
   ```rust
   let text_res = self.text_index.search_at(text, candidate_k, seq).await;
   ```
3. **Graph Signal (CSR / PPR / PathRAG):**
   ```rust
   let graph_res = self.graph_index.multi_traverse_at(anchors, max_hops, seq).await;
   ```
4. **Storage Hydration:**
   ```rust
   let hydrated = self.hydrate_from_tuples_at(doc_tuples, seq).await;
   ```

### Befund
- **Sequentielle Ausführung:** Die Aufrufe der 3 Abfragesignale (Vector, Text, Graph) erfolgen nacheinander durch sequentielle `.await`-Anweisungen innerhalb der gemeinsamen `with_pinned_checkpoint`-Anweisungsfolge.
- **Performance-Befund:** Da Vector-, Text- und Graph-Index voneinander unabhängige Lesezugriffe auf denselben Snapshot-Sequenzstand (`seq`) ausführen, führt die sequentielle Abarbeitung zu einer additiven Latenz ($T_{\text{ges}} = T_{\text{vec}} + T_{\text{text}} + T_{\text{graph}}$).
- **Empfehlung:** Umstellung auf parallele Signalausführung via `tokio::join!` oder `FuturesUnordered`, da alle 3 Lesezugriffe immun gegen Race Conditions am gemeinsamen Snapshot-Read-Guard sind.

`EVIDENCE[P2:FUSION-PARALLELISM]` — Performance-Befund: Sequentielle Abfrage identifiziert.

---

## 3. Markdown-Chunker-Pflicht & Dokumenten-Ingestion

**Fragestellung:** Gibt es einen Code-Pfad, bei dem ein Agenten-Dokument ohne `MarkdownChunker` als einzelner großer String verarbeitet/eingesetzt wird?

### Befund & Analyse
- **MCP Server & Client Layer:** In `crates/contextra-mcp/src/server_tools.rs` und `sandbox.rs` ist der `store_document`-Pfad explizit mit `MarkdownChunker` verdrahtet:
  ```rust
  let chunker = MarkdownChunker::new(ChunkerConfig::default());
  let chunks = chunker.chunk(doc_id, &content);
  ```
  Dokumente werden vor Generierung der Embeddings und vor Ingestion in semantische Abschnitte (~512 Tokens) zerlegt.
- **ContextManager Integration:** In `crates/contextra-cognition/src/context.rs` nutzt der `ContextManager` `MarkdownChunker` für die dynamische Dokumentzerlegung und Token-Budgetierung.
- **High-Level API (`contextra-db` / `contextra-engine`):** Die Low-Level `Collection::insert(id, vector, metadata)` Schnittstelle nimmt rohe Vektoren und Metadaten entgegen. Für Flüsse der Agenten-Ebene ist `MarkdownChunker` vorgeschrieben und in den relevanten MCP/Agent-Einfallstoren durchgesetzt.

`EVIDENCE[P3:MARKDOWN-CHUNKER]` — Konformität bestätigt.

---

## 4. Bug-Proof-Test-Ergebnisse & Invarianten (P4 & P5)

### P4: Bug-Proof `search_result_bound`
**Befehl:**
```bash
cargo test -p contextra-db --test search_result_bound -- --nocapture
```
**Ergebnisse (`logs/audits/db-bug3.log`):**
- `proof_search_never_exceeds_k`: **PASSED** (Anfrage mit $k=5$ gibt genau $\le 5$ Ergebnisse zurück).
- `proof_search_returns_nonzero_results_for_matching_query`: **PASSED**.
- `proof_search_with_k_zero_returns_empty`: **PASSED** ($k=0$ liefert leeres Ergebnis).
- `proof_usize_max_removed_from_search_path`: **PASSED**.

### P5: Proptest Retrieval-Invarianten `proptest_search_invariants`
**Befehl:**
```bash
PROPTEST_CASES=20 cargo test -p contextra-db --test proptest_search_invariants -- --nocapture
```
**Ergebnisse (`logs/audits/db-prop.log`):**
- `prop_rrf_bounded_by_k`: **PASSED** (RRF-Ausgabe immer $\le k$).
- `prop_e2e_search_bounded_by_k`: **PASSED** (E2E Vector- & Hybrid-Suche strikt $\le k$).
- `prop_bm25_score_monotonic_in_tf`: **PASSED** (BM25-Score monoton steigend mit Termfrequenz).
- `prop_rrf_scores_sorted_descending`: **PASSED** (RRF-Ergebnisse monoton fallend sortiert).
- `prop_rrf_empty_inputs_never_panic`: **PASSED** (Leere Vektoren / $k=0$ sind panic-frei).

### Unit- und Integrationstests (`cargo test -p contextra-db`)
- **Unit Tests (`lib.rs`):** 35 passed, 0 failed.
- **Integrationstest `checkpoint_layer_bounds.rs`:** 1 failure (`test_layer_001_fork_diverge_merge` schlägt aufgrund eines Isolationskonflikts bei `snapshot_seq 34 < pruned_through u64::MAX` nach Checkpoint-Neuöffnung fehl).

`EVIDENCE[P4:SEARCH-BOUND]` — Bug-Proof bestanden.
`EVIDENCE[P5:PROPTEST]` — Proptest Invarianten bestanden.

---

## 5. Volatile-Vault-Sicherheit & Multi-Step Engine (P6 & P7)

### P6: `VolatileContextVault` RAM-Sicherheit (`volatile_vault.rs`)
- **Zeroize-on-Drop (P6):** `VaultChunk` verwendet `#[derive(ZeroizeOnDrop)]`. Sowohl `purge()` als auch die `Drop`-Implementierung von `VolatileContextVault` rufen explizit `chunk.content.zeroize()` auf. Sensitiver Inhalt wird im Heap überschrieben.
- **Memory-Locking (`mlock`):** Chunk-Inhalte werden via `contextra_sys::LockedRegions` (`mlock`) im physischen RAM fixiert, um OS-Swap auf SSD/Disk zu verhindern. Falls `mlock` verweigert wird, schlägt der Vault nicht fehl, sondern gibt eine Warnung via `tracing::warn!` aus (`is_memory_locked() == false`).
- **Data-Leak Schutz:** `ingest()` führt keine Disk-I/O aus (INV-VAULT-2). `preview_metadata()` liefert nur Metadaten (`VaultChunkMetadata`), schließt Plaintext aus und verhindert versehentliches Logging.

`EVIDENCE[P6:VOLATILE-VAULT]` — Zeroize und RAM-Mlock Invarianten bestätigt.

### P7: Multi-Step LLM Loop-Limit (`multistep.rs`)
- **Iteration-Cap:** `MultiStepConfig::max_rounds` deckelt die maximalen Rewrite-Iterationen fest (Standard: `max_rounds = 3`). Loop läuft über `for _round in 2..=self.config.max_rounds`.
- **Latency Budget Guard:** `LatencyBudgetGuard` bricht iterative Suchrunden frühzeitig ab, wenn `latency_budget_ms` (Standard: 100.0 ms) überschritten wird.
- **Timeout-Schutz:** Einzelne `QueryRewriter.rewrite_structured()` Aufrufe sind mit `tokio::time::timeout(remaining_dur, ...)` abgesichert. Unendliche LLM-Loops sind ausgeschlossen.

`EVIDENCE[P7:LOOP-LIMIT]` — Iterations- und Timeout-Limits bestätigt.

---

## 6. VERDICT-Block

```
=== AUDIT VERDICT ===
CRATE: crates/contextra-db
VERSION/REVISION: 0.1.0 (commit 15b27624)
STATUS: CONDITIONAL PASS WITH FINDINGS

EVIDENCE SUMMARY:
- EVIDENCE[P1:AGT-DB-001]: PASS (TxId ausschließlich über AtomicU64 next_tx, zero SystemTime für TxId)
- EVIDENCE[P2:FUSION-PARALLELISM]: FINDING (Sequentielle Abfrage der 3 Signale Vector, Text, Graph)
- EVIDENCE[P3:MARKDOWN-CHUNKER]: PASS (MarkdownChunker Pflicht in MCP/Agent-Ingestion)
- EVIDENCE[P4:SEARCH-BOUND]: PASS (Result Count k-Bound Bug-Proof verifiziert)
- EVIDENCE[P5:PROPTEST]: PASS (Proptest Retrieval-Invarianten verifiziert)
- EVIDENCE[P6:VOLATILE-VAULT]: PASS (Zeroize-on-drop, RAM-Mlock Best-Effort, Null-Plaintext-Leak)
- EVIDENCE[P7:LOOP-LIMIT]: PASS (max_rounds Cap & LatencyBudgetGuard gestützt)

RECOMENDATIONS:
1. Parallelisiere Signal retrieval in `hybrid_search_with_query_and_report_at` mittels `tokio::join!`.
2. Behebe den Test-Konflikt in `checkpoint_layer_bounds.rs` (Isolations-Pruning-Interaktion).
=====================
```
