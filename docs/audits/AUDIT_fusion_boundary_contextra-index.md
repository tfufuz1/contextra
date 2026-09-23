# AUDIT: Fusion-Grenzflächen (HNSW / DiskANN) in `contextra-index`

* **Crate:** `crates/contextra-index` (Layer 3)
* **Audited Files:**
  * `crates/contextra-index/src/hnsw.rs`
  * `crates/contextra-index/src/diskann.rs`
  * `crates/contextra-db/src/fusion.rs` (Konsument-Vergleich)
* **Datum:** September 2026
* **Auditor:** Google-Jules (Principal Rust Systems Engineer)
* **Status:** AUDIT COMPLETE — READ-ONLY

---

## Executive Summary

Dieses Audit untersucht die Schnittstellen und Invarianten der Vektorindex-Implementierungen (`HnswIndex` und `DiskAnnIndex`) an der Fusion-Grenzfläche zu `contextra-db` (`fusion.rs`).

Sämtliche Befunde wurden anhand des Quellcodes verifiziert und mit **BELEGT** oder **WIDERLEGT** gekennzeichnet.

---

## 1. Öffentliche Funktionssignaturen an der Fusion-Schnittstelle

Die Indizes in `contextra-index` implementieren das `VectorIndex`-Trait aus `contextra-core::traits::vector_index` und stellen öffentliche Methoden zur Verfügung, die von `contextra-db` (u.a. `Collection`, `DbTransaction` und Orchestrierung) konsumiert werden.

### 1.1 `HnswIndex` (`crates/contextra-index/src/hnsw.rs`)
* `pub fn try_new(config: HnswConfig) -> Result<Self>` (`hnsw.rs:253`)
* `pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` (`hnsw.rs:1882`)
* `pub async fn search_filtered(&self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>) -> Result<Vec<ScoredDocument>>` (`hnsw.rs:1942`)
* `pub async fn search_at(&self, query: &[f32], k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>>` (`hnsw.rs:2065`)
* `pub async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()>` (`hnsw.rs:1853`)
* `pub async fn delete(&self, tx: TxId, id: DocId) -> Result<()>` (`hnsw.rs:1951`)
* `pub async fn commit(&self, tx: TxId) -> Result<()>` (`hnsw.rs:1968`)
* `pub async fn rollback(&self, tx: TxId) -> Result<()>` (`hnsw.rs:2073`)
* `pub async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()>` (`hnsw.rs:2083`)
* `pub async fn all_doc_ids(&self) -> Result<Vec<DocId>>` (`hnsw.rs:2144`)
* `pub async fn len(&self) -> usize` (`hnsw.rs:2172`)
* `pub fn is_rebuild_required(&self) -> bool` (`hnsw.rs:2186`)
* `pub fn trigger_rebuild_async(&self)` (`hnsw.rs:2190`)
* `pub async fn stats(&self) -> Result<VectorIndexStats>` (`hnsw.rs:2194`)

### 1.2 `DiskAnnIndex` (`crates/contextra-index/src/diskann.rs`)
* `pub fn try_new(config: DiskAnnConfig) -> Result<Self>` (`diskann.rs:254`)
* `pub async fn search_internal(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` (`diskann.rs:1099`)
* `pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` (`diskann.rs:1204`)
* `pub async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()>` (`diskann.rs:1181`)
* `pub async fn delete(&self, tx: TxId, id: DocId) -> Result<()>` (`diskann.rs:1208`)
* `pub async fn commit(&self, tx: TxId) -> Result<()>` (`diskann.rs:1238`)
* `pub async fn rollback(&self, tx: TxId) -> Result<()>` (`diskann.rs:1246`)
* `pub async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()>` (`diskann.rs:1253`)
* `pub async fn all_doc_ids(&self) -> Result<Vec<DocId>>` (`diskann.rs:1260`)
* `pub async fn len(&self) -> usize` (`diskann.rs:1282`)
* `pub async fn stats(&self) -> Result<VectorIndexStats>` (`diskann.rs:1290`)

**Status:** **BELEGT**

---

## 2. Score-Skala & Metrik-Annahmen an der Schnittstelle

Die Vektorindizes berechnen Distanzen und konvertieren diese vor der Rückgabe in ein `ScoredDocument` wie folgt:

* **HNSW (`hnsw.rs:503-511` & `1928-1936`)**:
  * `DistanceMetric::Cosine`: `score = 1.0 - final_dist` (Kosinus-Ahnlichkeit in `[-1.0, 1.0]`)
  * `DistanceMetric::Euclidean`: `score = 1.0 / (1.0 + final_dist)` (Monoton in `(0.0, 1.0]`)
  * `DistanceMetric::DotProduct`: `score = -final_dist` (Rohes Skalarprodukt)
* **DiskANN (`diskann.rs:1167-1172`)**:
  * `DistanceMetric::Cosine`: `score = 1.0 - c.distance`
  * `DistanceMetric::Euclidean`: `score = 1.0 / (1.0 + c.distance)`
  * `DistanceMetric::DotProduct`: `score = -c.distance`

### Auswirkung auf `fusion.rs` (Reciprocal Rank Fusion):
* In `fusion.rs` (`contextra-db/src/fusion.rs:1-12`) wird **Reciprocal Rank Fusion (RRF)** eingesetzt:
  $$\text{RRF Score} = \sum \frac{w}{k + \text{rank}}$$
* RRF wandelt rohe Eingabe-Scores innerhalb jedes Ergebnissatzes in **Ränge (Ranks 1..N)** um (`fusion.rs:495-502`).
* Da RRF ausschließlich die **Rangordnung** (Ordering via Score) auswertet, ist keine skalenübergreifende Score-Normalisierung erforderlich (ADR-003). Ein Metrikwechsel (z. B. von Kosinus zu Euklidisch) bricht die Fusion in `fusion.rs` **nicht**, solange die Score-Transformation innerhalb der Metrik streng monoton fallend bezüglich der Distanz ist.

**Status:** **BELEGT**

---

## 3. Partial-Failure-Verhalten (Leere Collection / Snapshot-Fehler)

### 3.1 Leere Collection
* **HNSW (`hnsw.rs:467` & `1911`)**:
  * Sind keine Einstiegsknoten vorhanden (`ep.is_empty()`), liefert die Suche sofort `Ok(Vec::new())` zurück. Es wird **kein `Err`** erzeugt.
* **DiskANN (`diskann.rs:1127`)**:
  * Ist die Indexdatei leer oder wurden keine Knoten geladen (`header.node_count == 0`), liefert `search_internal` sofort `Ok(Vec::new())` zurück. Es wird **kein `Err`** erzeugt.

### 3.2 Fehlgeschlagenes / ungültiges `pin_snapshot`
* **HNSW (`hnsw.rs:204-219` & `2065-2070`)**:
  * `SnapshotPinGuard` ist ein RAII-Guard, der beim Erzeugen `core.cold.seq_log.write().pin_snapshot(seq_no)` aufruft.
  * Das Anpinnen einer Sequenznummer in `SequenceLog` scheitert strukturell nie (es fügt die Sequenznummer in ein `AHashSet` ein).
  * Wenn für die Sequenznummer `seq_no` keine Dokumente sichtbar sind, filtert `is_visible` alle Kandidaten heraus und `search_at` liefert `Ok(Vec::new())` zurück.
* **DiskANN (`diskann.rs`)**:
  * `DiskAnnIndex` implementiert derzeit weder `search_at` noch `SnapshotPinGuard`.

**Status:** **BELEGT**

---

## 4. DocId-Herkunft & Schlüsselrepräsentation

* **DocId-Berechnung vs. Übergabe**:
  * In beiden Indizes wird `DocId` **nicht** intern berechnet, sondern beim Aufruf von `insert(tx, id, embedding)` als Argument `id: DocId` von außen übergeben (`hnsw.rs:1853`, `diskann.rs:1181`).
* **Schlüsselrepräsentation**:
  * Es wird exakt derselbe `DocId`-Typ (`contextra_core::DocId(u64)`) verwendet wie im Text-Inverted-Index (`contextra-text`) und Graph-CSR-Index (`contextra-graph`).
  * Es gibt keine Zwischenrepräsentation (wie temporäre Token oder interne Hash-Keys) an der öffentlichen Trait-Schnittstelle.

**Status:** **BELEGT**

---

## 5. Tombstone-Verhalten & DiskANN-Löschung

### 5.1 Sichtbarkeit an der Fusion-Grenze
* Tombstones sind an der Fusion-Schnittstelle **vollständig intern verborgen**.
* `HnswIndex::search()` prüft `deleted.contains(c.index as u64)` (`hnsw.rs:1918`) und schließt gelöschte Knoten aus den Ergebnissen aus.
* `DiskAnnIndex::search_blocking()` prüft `tombstones.contains(node.doc_id.inner())` (`diskann.rs:1162`) und schließt gelöschte Knoten aus den Ergebnissen aus.
* `fusion.rs` erhält als Ergebnis ausschließlich valide `ScoredDocument`-Instanzen ohne gelöschte Dokumente und muss sich nicht um Tombstones kümmern.

### 5.2 Abgleich der Ausgangshypothese zu `DiskAnnIndex::delete()`
* **Soll-Hypothese:** `DiskAnnIndex::delete()` liefert unbedingt `ContextraError::InvalidInput("DiskAnn is a read-only out-of-core index.")` zurück.
* **Ist-Befund im Code:** **WIDERLEGT**.
  * In `diskann.rs:1208-1236` ist die native Tombstone-Löschung für DiskANN **vollständig implementiert**:
    ```rust
    pub async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        ...
        self.inner.tombstones.write().insert(doc_id_u64);
        let tombstone_wal = self.inner.config.index_path.with_extension("tombstone.wal");
        Self::append_to_tombstone_wal(&tombstone_wal, id).await?;
        Ok(())
    }
    ```
  * Wenn das Dokument nicht im Index vorhanden ist, wird `ContextraError::NotFound` zurückgegeben (`diskann.rs:1218`).

**Status:** **WIDERLEGT (Soll-Hypothese zu DiskANN read-only ist veraltet; DiskANN Tombstone-Löschung ist aktiv)**

---

## 6. Reifegrad-Befund zu `SnapshotPinGuard` / `pin_snapshot` / `unpin_snapshot`

* **Soll-Hypothese:** Reifegrad des `pin_snapshot`/`unpin_snapshot`-Pfads ist als `[REIFEGRAD UNGEPRÜFT]` markiert.
* **Ist-Befund im Code:** **BELEGT / STABIL VERIFIZIERT**.
  1. `SnapshotPinGuard` ist in `hnsw.rs:204-219` als RAII-Muster implementiert:
     * `new()` führt `core.cold.seq_log.write().pin_snapshot(seq_no)` aus.
     * `Drop` führt `core.cold.seq_log.write().unpin_snapshot(seq_no)` aus.
  2. In `search_at()` (`hnsw.rs:2066`) ist der Guard direkt eingebunden:
     `let _pin_guard = SnapshotPinGuard::new(&self.inner, seq_no);`
  3. Proptests und Unit-Tests verifizieren die Korrektheit:
     * `test_hnsw_search_at_snapshot_isolation` (`hnsw.rs:2285`)
     * `prop_hnsw_search_at_consistency` (`hnsw.rs:2325`)

**Status:** **BELEGT (Reifegrad geprüft und durch automatisierte Tests abgesichert)**

---

## 7. Performance & Allokations-Invariante in `compute_distance_with_mmap`

* **Soll-Hypothese:** `compute_distance_with_mmap` (F32-Zweig) alloziert pro Distanzberechnung einen `Vec<f32>` (768 Einzel-Dekodierungen).
* **Ist-Befund im Code**:
  * **In `hnsw.rs:793-801`**: **WIDERLEGT**.
    Für unquantisierte F32-Vektoren wird `compute_distance_f32_bytes_trusted` direkt auf dem Mmap-Byte-Slice (`&[u8]`) aufgerufen, ohne einen `Vec<f32>` auf dem Heap zu allozieren:
    ```rust
    // Safe unaligned SIMD F32 read directly from mmap byte slice (zero allocation)
    crate::distance::compute_distance_f32_bytes_trusted(
        query_exact,
        vector_bytes,
        self.cold.config.distance_metric,
    )
    ```
  * **In `diskann.rs:1015-1033` (`load_node`)**: **BELEGT**.
    DiskANN liest beim ersten Laden eines Knotens den F32-Vektor in einen `Vec<f32>` im `CachedNode`. Werden Knoten aus dem Cache gelesen, entfällt die Re-Allokation.

---

## 8. Zero-Panic Doctrine

Eine Überprüfung auf `.unwrap()` und `.expect()` in den Produktionsfunktionen von `hnsw.rs` und `diskann.rs` ergab:
* **0 Vorkommen** in Produktionscode.
* Sämtliche Unwraps/Expects sind auf `#[cfg(test)] mod tests` beschränkt.
* Alle Grenzflächen-Funktionen nutzen die `Result<T, ContextraError>`-Fehlerbehandlung.

**Status:** **BELEGT (Zero-Panic-Doktrin eingehalten)**

---

## Fazit & Audit-Zusammenfassung

Die Fusion-Grenzfläche zwischen `contextra-index` und `contextra-db/src/fusion.rs` ist sauber strukturiert:
1. `fusion.rs` nutzt RRF über Ränge, wodurch die unterschiedlichen Score-Skalen der Distanzmetriken harmonisch vereint werden.
2. `DiskAnnIndex` unterstützt entgegen älteren Spezifikationen native Tombstones via `tombstone.wal`.
3. `HnswIndex::search_at` nutzt RAII-basiertes Snapshot-Pinning über `SnapshotPinGuard` mit nachgewiesener Isolationstestsicherheit.
4. Hot-Path-Distanzberechnungen über Mmap in HNSW erfolgen allokationsfrei direkt auf `&[u8]`.
