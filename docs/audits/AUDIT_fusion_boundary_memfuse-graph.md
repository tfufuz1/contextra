# AUDIT: Fusion-Grenzflächen (`memfuse-graph` PPR/CSR vs. `memfuse-db` Fusion)

**Datum:** 2026-09-15
**Crate:** `memfuse-graph` (Layer 2) & `memfuse-db` (Layer 3)
**Status:** COMPLETE (Read-Only Audit)
**Auditierte Dateien:**
- `crates/memfuse-graph/src/ppr.rs`
- `crates/memfuse-graph/src/csr.rs`
- `crates/memfuse-db/src/fusion.rs`
- `crates/memfuse-db/src/collection/search.rs`

---

## 1. EXECUTIVE SUMMARY & SYSTEMKONTEXT

Das Modul `memfuse-graph` stellt über die Compressed Sparse Row (CSR) Graph-Implementierung (`CsrGraph`) Signal 3 der 4-Signal-Fusion im MemFuse Cognitive OS bereit.
In diesem Audit wurden die Schnittstellen zwischen `memfuse-graph` (`ppr.rs`, `csr.rs`) und der Fusion/Retrieval-Schicht in `memfuse-db` (`fusion.rs`, `collection/search.rs`) auf Blockierverhalten, Fehlerbehandlung, ID-Herkunft, Tombstone-Sichtbarkeit und Timeouts untersucht.

---

## 2. ABGLEICHS-AUFTRAG: FRAGEN & BELEGE

### Frage 1: Öffentliche Funktionssignaturen für `memfuse-db/src/fusion.rs`
**Frage:** Welche öffentlichen Funktionssignaturen aus `ppr.rs`/`csr.rs` werden von `memfuse-db/src/fusion.rs` konsumiert? Vollständige Signaturen.

- **Befund:** **KEINE (0 Funktionssignaturen)**.
- **Analyse:** `memfuse-db/src/fusion.rs` ist ein reine Reciprocal Rank Fusion (RRF) Berechnungseinheit. Sie konsumiert weder `ppr.rs` noch `csr.rs` direkt, sondern arbeitet ausschließlich auf universellen `SearchResult`-Strukturen (`Vec<SearchResult>` bzw. `Vec<(String, Vec<SearchResult>, f32)>`).
- **Grenzflächen-Fluss:** Die Interaktion mit `CsrGraph` erfolgt in `memfuse-db/src/collection/search.rs`. Dort ruft `Collection` Traversal-Methoden des `GraphIndex`-Traits (wie `multi_traverse_at`) auf, wandelt die Ergebnisse in `SearchResult`-Kandidaten um und übergibt diese an `fusion::weighted_reciprocal_rank_fusion_with_options`.
- **Exakte Fundstellen:**
  - `crates/memfuse-db/src/fusion.rs:453–459`:
    `pub fn weighted_reciprocal_rank_fusion_with_options(mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>) -> Vec<SearchResult>`
  - `crates/memfuse-db/src/collection/search.rs:750, 1101`: Aufruf von `self.graph_index.multi_traverse_at(...)`.
- **Status:** **BELEGT**

---

### Frage 2: Blockierverhalten bei laufendem `compact()`
**Frage:** Wenn `compute_personalized_page_rank_with_context` während eines laufenden `compact()` aufgerufen wird — blockiert der Aufruf synchron auf den `RwLock`, gibt es einen Timeout-Parameter, oder liefert die Funktion sofort einen "busy"-Fehler zurück? Exakte Fundstelle.

- **Befund:** **Synchrones Blockieren auf Mutex ohne Timeout / ohne Busy-Fehler.**
- **Analyse:**
  1. `compute_personalized_page_rank_with_context` (`ppr.rs:108`) nimmt direkt eine immutable Referenz `&GraphInner` entgegen und führt keine eigenen Lock-Operationen aus.
  2. Der Einstiegspunkt über `CsrGraph` ist `personalized_page_rank_with_context_async` (`csr.rs:885–897`).
  3. Bei einem Aufruf von `personalized_page_rank_with_context_async` wird zuerst `self.deleted_view().await` und danach `self.compact_async().await` aufgerufen (`csr.rs:886–888`).
  4. `compact_async()` verlegt den Rebuild in `tokio::task::spawn_blocking` und fordert dort `self.write_state.lock()` an (`csr.rs:736`). `write_state` ist ein `parking_lot::Mutex<GraphInner>` (`csr.rs:310`).
  5. Wenn bereits ein `compact()` läuft, **blockiert** der Mutex-Acquire synchron im `spawn_blocking`-Thread, bis das laufende `compact()` abgeschlossen ist.
  6. Es existiert **kein Timeout-Parameter** und **kein "busy"-Error-Return**.
  7. Nach Abschluss/Freigabe lädt `inner_read()` über `ArcSwap::load()` (`csr.rs:374–376`) den neuen `GraphInner`-Snapshot vollkommen lock-frei.
- **Exakte Fundstellen:**
  - `crates/memfuse-graph/src/csr.rs:310`: `write_state: Arc<Mutex<GraphInner>>`
  - `crates/memfuse-graph/src/csr.rs:374–376`: `pub(crate) fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>> { self.inner.load() }`
  - `crates/memfuse-graph/src/csr.rs:736`: `let mut inner_writer = write_state.lock();`
  - `crates/memfuse-graph/src/csr.rs:885–897`: `personalized_page_rank_with_context_async`
  - `crates/memfuse-graph/src/ppr.rs:108`: `pub(crate) fn compute_ppr_with_context(...)`
- **Status:** **BELEGT**

---

### Frage 3: Partial-Failure bei leerem Graph oder fehlendem Startknoten
**Frage:** Was liefert die PPR-Schnittstelle bei leerem Graph-Segment oder wenn der Startknoten nicht existiert — leerer `Vec`/`HashMap` oder `Err`? Exakte Fundstelle.

- **Befund:** **Leerer `Vec` (`Vec::new()`) bzw. `Ok(Vec::new())` (KEIN `Err`).**
- **Analyse:**
  1. In `ppr.rs:116–118` wird zu Beginn von `compute_ppr_with_context` geprüft, ob der Graph leer ist (`n == 0`) oder die `seed_nodes` leer sind:
     ```rust
     let n = inner.reverse_map.len();
     if n == 0 || seed_nodes.is_empty() {
         return Vec::new();
     }
     ```
  2. In Step 1 (`ppr.rs:122–135`) werden valide Seed-Knoten in `ctx.valid_seeds` gesammelt. Wenn der Startknoten nicht in `inner.id_map` existiert, nicht comittet oder gelöscht ist, bleibt `ctx.valid_seeds` leer.
  3. In `ppr.rs:137–139`:
     ```rust
     if ctx.valid_seeds.is_empty() {
         return Vec::new();
     }
     ```
  4. Auf CsrGraph-Ebene gibt `personalized_page_rank_with_context_async` direkt `Vec<(EntityId, f32)>` zurück (`csr.rs:885–897`). Die Trait-Methode `personalized_page_rank` verpackt das Ergebnis in `Ok(Vec::new())` (`csr.rs:980–998`).
- **Exakte Fundstellen:**
  - `crates/memfuse-graph/src/ppr.rs:116–118`
  - `crates/memfuse-graph/src/ppr.rs:137–139`
  - `crates/memfuse-graph/src/csr.rs:896`
  - `crates/memfuse-graph/src/csr.rs:988`
- **Status:** **BELEGT**

---

### Frage 4: DocId-/EntityId-Herkunft & Mapping
**Frage:** Wird `DocId`/`NodeId` beim Insert-Pfad in `csr.rs` selbst berechnet oder von außen übergeben? Ist es dieselbe `DocId`-Typinstanz wie in Text-/Vektor-Insert, oder eine separate Graph-interne ID mit eigener Mapping-Tabelle?

- **Befund:** **Von außen übergeben; nutzt `EntityId` (bzw. optional `DocId` als Provenance-Feld); verwendet Graph-interne Contiguous-Index Mapping-Tabelle.**
- **Analyse:**
  1. `EntityId` wird beim Einfügen (`add_entity`, `add_edge`, `insert_edge_direct`) vollständig von außen übergeben.
  2. In `memfuse-db/src/collection/search.rs` teilen sich Dokumente und Graphknoten dieselben Schlüssel: `EntityId::from_key(doc.id.as_str())`.
  3. In `csr.rs` verwaltet `GraphInner` zwei interne Mapping-Tabellen (`csr.rs:200–202`):
     - `id_map: HashMap<EntityId, InternalIndex>` (externes `EntityId` -> internes `usize`)
     - `reverse_map: Vec<EntityId>` (internes `usize` -> externes `EntityId`)
  4. Zusätzlich kann beim Einfügen von Kanten eine optionale `source_doc_id: Option<DocId>` zur Provenance-Rückverfolgung übergeben werden (`csr.rs:207`, `csr.rs:509`).
- **Exakte Fundstellen:**
  - `crates/memfuse-graph/src/csr.rs:200–202`: `id_map` und `reverse_map`
  - `crates/memfuse-graph/src/csr.rs:252–265`: `get_or_create_index(id: EntityId) -> InternalIndex`
  - `crates/memfuse-graph/src/csr.rs:468–481`: `add_edge(...)` Akzeptanz von `from`, `to` und `source_doc_id`
  - `crates/memfuse-db/src/collection/search.rs:739, 1083`: Mapping via `EntityId::from_key(r.id.as_str())`
- **Status:** **BELEGT**

---

### Frage 5: Tombstone-Sichtbarkeit & Race-Fenster
**Frage:** Filtert `ppr.rs` tombstonierte Knoten/Kanten VOR der PPR-Berechnung oder erst NACH der Berechnung beim Zusammenstellen der Ergebnisliste? Race-Fenster zwischen `ArcSwap`-Snapshot und Pending-Puffer relevant?

- **Befund:** **Filterung erfolgt VOR und WÄHREND der PPR-Berechnung (Knoten-Tombstones) — Kanten-Tombstones werden in `ppr.rs` jedoch NICHT direkt ausgewertet, sondern verlassen sich auf `compact_async()`. Es existiert ein Race-Fenster.**
- **Analyse:**
  1. **Knoten-Tombstones (`DeletedView`):**
     - `deleted_view` wird vor der PPR-Berechnung via `self.deleted_view().await` aus dem LSM-Storage geladen (`csr.rs:886`).
     - In `ppr.rs` werden gelöschte Knoten in Step 1 (Seed-Validierung, `ppr.rs:125`), Step 3 (Out-Weight-Sums, `ppr.rs:150, 165`), Step 4 (Power Iteration, `ppr.rs:178, 191, 207`), Step 5 (Re-Normalisierung, `ppr.rs:241, 248`) und Step 6 (Ergebnisaufbau, `ppr.rs:258`) **vor und während** der Berechnung gefiltert.
  2. **Kanten-Tombstones (`inner.tombstoned_edges`):**
     - `ppr.rs` wertet `inner.tombstoned_edges` während der Power-Iteration **nicht** aus. Es verlässt sich darauf, dass `compact_async()` vor `compute_ppr_with_context` aufgerufen wird, wodurch `tombstoned_edges` aus den CSR-Arrays und dem Pending-Puffer entfernt werden (`csr.rs:350–360`).
  3. **Race-Fenster:**
     - `deleted_view()` liest den Storage vor `compact_async()` und vor `inner_read()`. Wenn zwischen `deleted_view()` und `inner_read()` eine Entität gelöscht wird, ist der `DeletedView` veraltet.
     - Wenn `compact_async()` aufgrund des Speicherbudgets (`max_compaction_peak_memory_mb`) verzögert wird (`csr.rs:728–730`), verbleiben `tombstoned_edges` im `GraphInner`-Snapshot und werden von `ppr.rs` irrtümlich traversiert.
- **Exakte Fundstellen:**
  - `crates/memfuse-graph/src/ppr.rs:125, 150, 165, 178, 191, 207, 241, 258`
  - `crates/memfuse-graph/src/csr.rs:350–360` (Kanten-Tombstone-Filterung in `compact()`)
  - `crates/memfuse-graph/src/csr.rs:728–730` (`compact_async` Budget-Deferral)
  - `crates/memfuse-graph/src/csr.rs:886–895` (`personalized_page_rank_with_context_async` Reihenfolge)
- **Status:** **BELEGT**

---

### Frage 6: Async-Pfad & Timeout-Verhalten
**Frage:** Hat der `personalized_page_rank_with_context_async`-Aufruf tatsächlich einen wall-clock-messbaren Timeout, den `fusion.rs` setzen könnte, oder läuft er unbegrenzt?

- **Befund:** **Unbegrenzt bezüglich Wall-Clock-Zeit (KEIN Timeout-Parameter in `personalized_page_rank_with_context_async` oder `fusion.rs`).**
- **Analyse:**
  1. `personalized_page_rank_with_context_async` (`csr.rs:885–897`) besitzt keinen Timeout-Parameter und verwendet intern kein `tokio::time::timeout`.
  2. Die CPU-Schleife der Power Iteration in `ppr.rs:200` ist rein durch `config.max_iterations` begrenzt (hart gecappt auf maximal 1000 Iterationen, `ppr.rs:154`).
  3. In `memfuse-db/src/collection/search.rs` wird PPR unter MVCC-Snapshot-Isolation derzeit explizit abgelehnt (`MemFuseError::SnapshotUnsupportedForSignal`, `search.rs:756–760, 1106–1110`).
  4. `fusion.rs` hat keinerlei Kontrolle oder Parameter bezüglich Wall-Clock-Timeouts für Graph-Traversierungen.
- **Exakte Fundstellen:**
  - `crates/memfuse-graph/src/csr.rs:885–897`
  - `crates/memfuse-graph/src/ppr.rs:154`: `let max_iters = config.max_iterations.min(1000);`
  - `crates/memfuse-db/src/collection/search.rs:756–760, 1106–1110`
- **Status:** **BELEGT**

---

## 3. ARCHITEKTUR- & LOCKING-ANALYSE

### 3.1 Lock-Hierarchie in `csr.rs`
- **RCU Snapshot Isolation (Read Path):** Readers rufen `inner_read()` auf, welches `self.inner.load()` ausführt (`csr.rs:374–376`). Dies liefert ein `arc_swap::Guard<Arc<GraphInner>>`. Der Read-Pfad ist zu 100% lock-frei und blockiert niemals Writer oder Compaction.
- **Write-Pfad & Compaction:** Schreiboperationen (`add_entity`, `add_edge`, `commit`) sowie `compact()` fordern den `write_state: Arc<Mutex<GraphInner>>` Lock an (`csr.rs:378–383`, `csr.rs:736`).
- **Invariante:** Readers sehen immer konsistente Point-In-Time Snapshots. Es existiert kein Risiko von Torn Reads während concurrent compactions.

### 3.2 Koppelung an MVCC Snapshot Isolation in `memfuse-db`
- Traversal-Methoden wie `multi_traverse_at` berücksichtigen MVCC-Sequenznummern (`seq_no`) über bi-temporale Metadaten auf Kanten (`tx_valid_from`, `tx_valid_to`).
- `PersonalizedPageRank` operiert auf dem un-versionierten Live-Graph-Zustand. Daher wird PPR in `search.rs` bei aktiver Snapshot-Isolation mit `MemFuseError::SnapshotUnsupportedForSignal` abgewiesen.

---

## 4. VERIFIKATIONS-VERDIKT

Alle 6 Kernfragen zur Fusion-Grenzfläche zwischen `memfuse-graph` und `memfuse-db` wurden anhand des aktuellen Quellcodes vollständig und präzise beantwortet und belegt. Es wurden keine Modifikationen am Produktionscode vorgenommen.
