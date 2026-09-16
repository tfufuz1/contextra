# AUDIT MEMFUSE-TEXT — Fusion-Grenzflächen (WAND/BM25/Inverted)

**Stand:** September 2026
**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**Target Crate:** `crates/memfuse-text` (Layer 2)
**Scope:** `crates/memfuse-text/src/wand.rs`, `crates/memfuse-text/src/bm25.rs`, `crates/memfuse-text/src/inverted.rs`

---

## 1. Übersicht & Zielsetzung

Dieser Audit analysiert die Fusion-Grenzflächen des Text-Subsystems (`crates/memfuse-text`) in Bezug auf die WAND-Suche (`wand.rs`), die BM25-Score-Berechnung (`bm25.rs`) und den invertierten LSM-Index (`inverted.rs`). Ziel ist die systematische Untersuchung der Schnittstellen-Typen, der Score-Skalierung, des Partial-Failure-Verhaltens, der `DocId`-Herkunft sowie der Synchronität der Tombstone-Konsultation.

---

## 2. Abgleichs-Befunde (Fragen 1 bis 6)

### 1. Öffentliche Funktionssignaturen an der Grenzfläche zu `memfuse-db`

**BELEGT** (`crates/memfuse-text/src/inverted.rs`, `crates/memfuse-db/src/collection/mod.rs`, `crates/memfuse-db/src/collection/search.rs`)

* **Cross-Crate Imports in `memfuse-db`:**
  In `crates/memfuse-db/src/collection/mod.rs:27-28` wird importiert:
  ```rust
  use memfuse_text::inverted::InvertedIndex;
  use memfuse_text::Language;
  ```
  In `crates/memfuse-db/src/fusion.rs` existiert **kein** direkter Import von `memfuse_text` — `fusion.rs` empfängt bereits abstrahierte `SearchResult`-Kandidatenlisten von `collection/search.rs`.

* **Öffentliche Signaturen in `inverted.rs` (aufgerufen von `memfuse-db`):**
  - `pub fn InvertedIndex::new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self` (`crates/memfuse-text/src/inverted.rs:121`)
  - `pub fn InvertedIndex::new(storage: Arc<S>, namespace: &str) -> Self` (`crates/memfuse-text/src/inverted.rs:146`)
  - `pub async fn InvertedIndex::load_stats(&self) -> Result<()>` (`crates/memfuse-text/src/inverted.rs:157`)
  - `pub async fn InvertedIndex::upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>` (`crates/memfuse-text/src/inverted.rs:242`)
  - `pub async fn InvertedIndex::resolve_tombstones(&self, tx: TxId) -> Result<u64>` (`crates/memfuse-text/src/inverted.rs:310`)
  - `pub async fn InvertedIndex::delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()>` (`crates/memfuse-text/src/inverted.rs:361`)
  - `pub async fn InvertedIndex::search_bm25(&self, query: &str, k: usize, max_seq: Option<u64>) -> Result<Vec<(DocId, f32)>>` (`crates/memfuse-text/src/inverted.rs:427`)
  - `pub async fn InvertedIndex::search_bm25_at(&self, query: &str, k: usize, max_seq: Option<u64>) -> Result<Vec<(DocId, f32)>>` (`crates/memfuse-text/src/inverted.rs:438`)
  - Implementation von `TextIndex` Trait for `InvertedIndex<S>` (`crates/memfuse-text/src/inverted.rs:486-541`):
    - `async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>>`
    - `async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>>`
    - `async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()>`
    - `async fn delete(&self, tx: TxId, id: DocId) -> Result<()>`
    - `async fn commit(&self, tx: TxId) -> Result<()>`
    - `async fn rollback(&self, tx: TxId) -> Result<()>`
    - `async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()>`
    - `async fn last_tx_id(&self) -> Result<TxId>`
    - `async fn len(&self) -> usize`
    - `async fn stats(&self) -> Result<TextIndexStats>`

* **Öffentliche Signaturen in `bm25.rs` (Exportiert für Trait-/Helfer-Nutzung):**
  - `pub fn BM25::new(k1: f32, b: f32) -> Result<Self>` (`crates/memfuse-text/src/bm25.rs:32`)
  - `pub fn BM25::score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` (`crates/memfuse-text/src/bm25.rs:43`)
  - `pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` (`crates/memfuse-text/src/bm25.rs:58`)
  - `pub fn score_term_with_params(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32, k1: f32, b: f32) -> f32` (`crates/memfuse-text/src/bm25.rs:63`)

* **Öffentliche Signaturen in `wand.rs`:**
  - `pub fn BoundedTopK::new(capacity: usize) -> Self` (`crates/memfuse-text/src/wand.rs:43`)
  - `pub fn BoundedTopK::push(&mut self, doc_id: DocId, score: f32)` (`crates/memfuse-text/src/wand.rs:51`)
  - `pub fn BoundedTopK::min_threshold(&self) -> f32` (`crates/memfuse-text/src/wand.rs:65`)
  - `pub fn BoundedTopK::into_sorted_vec(self) -> Vec<(DocId, f32)>` (`crates/memfuse-text/src/wand.rs:73`)
  - `pub async fn block_max_wand_search<S: StorageEngine>(...) -> Result<WandSearchResult>` (`crates/memfuse-text/src/wand.rs:179`)

---

### 2. Score-Skala an der Grenzfläche

**BELEGT** (`crates/memfuse-text/src/inverted.rs:427-483`, `crates/memfuse-text/src/bm25.rs:63-95`, `crates/memfuse-db/src/collection/search.rs:1021`)

* **Score vs. Rang:** Die Text-Suchfunktionen übergeben an den Aufrufer **nicht nur Rang-Information**, sondern explizit den **rohen BM25-Score** zusammen mit der `DocId` als `(DocId, f32)` oder als `ScoredDocument { doc_id, score: f32 }`.
* **Werttyp:** `f32`.
* **Wertebereich:** Unbegrenzt positiv ($[0.0, +\infty)$). Durch die Robertson-Spärck-Jones BM25+ Glättungsformel:
  $$\text{IDF} = \ln\left(1 + \frac{N - df + 0.5}{df + 0.5}\right)$$
  ist garantiert, dass $\text{IDF} \ge \ln(1) = 0.0$ für alle $df \in [0, N]$. Der minimale Term-Score ist $0.0$ (wenn $tf=0$ oder $df=0$).
* **Nutzung in Layer 3 (`memfuse-db`):** In `crates/memfuse-db/src/collection/search.rs` fließt der rohe BM25-Score in das RRF (Reciprocal Rank Fusion) Subsystem ein, wo entweder der Rang oder der unnormierte Score verarbeitet werden.

---

### 3. Partial-Failure-Verhalten bei leerem Index / fehlenden Query-Termen

**BELEGT** (`crates/memfuse-text/src/inverted.rs:443-455`, `crates/memfuse-text/src/wand.rs:188-193`, `crates/memfuse-text/src/wand.rs:248-253`)

* **Rückgabetyp:** `Result<Vec<(DocId, f32)>>` (bzw. `Result<Vec<ScoredDocument>>` für die Trait-Methode `search_at`).
* **Codepfad & Verhalten:**
  1. Ist die Query leer (`query.is_empty()`) oder `k == 0`, liefert `InvertedIndex::search_bm25_at` sofort `Ok(Vec::new())` zurück (`crates/memfuse-text/src/inverted.rs:443-445`).
  2. Nach der Tokenisierung: Erzeugt die Tokenizer-Pipelines 0 Token (`tokens.is_empty()`), liefert `InvertedIndex::search_bm25_at` ebenfalls sofort `Ok(Vec::new())` zurück (`crates/memfuse-text/src/inverted.rs:455`).
  3. In `block_max_wand_search`: Ist `terms.is_empty()` oder `k == 0`, wird ein `Ok(WandSearchResult { results: Vec::new(), evaluated_docs_count: 0 })` zurückgegeben (`crates/memfuse-text/src/wand.rs:188-193`).
  4. Befinden sich keine Postings für die Terme im Index oder ist die effektive $df = 0$ für alle Terme (`cursors.is_empty()`), gibt `block_max_wand_search` ebenfalls ein leeres `Ok(WandSearchResult)` zurück (`crates/memfuse-text/src/wand.rs:248-253`).
* **Ergebnis:** Es wird **kein `Err(...)`** ausgelöst, sondern immer ein leeres `Ok(Vec::new())` bzw. `Ok(WandSearchResult)`.

---

### 4. DocId-Herkunft beim Insert-Pfad

**BELEGT** (`crates/memfuse-text/src/inverted.rs:242`, `crates/memfuse-text/src/inverted.rs:429`)

* **Berechnung vs. Übergabe:** Die `DocId` wird beim Insert-Pfad **nicht** in `inverted.rs` selbst berechnet. Sie wird als bereits berechneter Wert (`doc_id: DocId` bzw. `id: DocId`) von höher gelegenen Layern (`memfuse-db` / `Collection`) übergeben.
* **Exakte Fundstellen:**
  - `crates/memfuse-text/src/inverted.rs:242`:
    ```rust
    pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>
    ```
  - `crates/memfuse-text/src/inverted.rs:503`:
    ```rust
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.upsert_document(tx, id, text).await
    }
    ```
* **Schlussfolgerung:** In `inverted.rs` existiert kein interner Aufruf von `DocId::from_key`.

---

### 5. Tombstone-Konsultation im WAND-Cursor (Synchronität & Race-Fenster)

**BELEGT** (`crates/memfuse-text/src/wand.rs:232-244`, `crates/memfuse-text/src/wand.rs:356-368`, `crates/memfuse-text/src/inverted.rs:310-358`)

* **Synchrone Prüfung im WAND-Iterationspfad:**
  Die Tombstone-Prüfung erfolgt **vollständig synchron und deterministisch** während der WAND-Traversierung in `block_max_wand_search`:
  1. Vor Eintritt in die Pivot-Schleife werden aktive Tombstones für Terme am Snapshot-Punkt `seq` ermittelt (`crates/memfuse-text/src/wand.rs:232-244`).
  2. Während der Auswertung des Kandidaten `pivot_doc_id` prüft der Cursor direkt und synchron im Storage bzw. über den thread-lokalen `tbs_cache`:
     ```rust
     let tbs_key = tombstone_key_helper(doc_id, &c.term);
     let is_tombstone = match tbs_cache.get(&tbs_key) {
         Some(&ex) => ex,
         None => {
             let ex = storage.get_at_seq(&tbs_key, seq).await?.is_some();
             tbs_cache.insert(tbs_key.clone(), ex);
             ex
         }
     };
     ```
     (`crates/memfuse-text/src/wand.rs:356-368`)
* **Rolle des async Hintergrundlaufs (`resolve_tombstones`):**
  Der async Hintergrundlauf `resolve_tombstones` (`crates/memfuse-text/src/inverted.rs:310-358`) dient ausschließlich der **physischen Bereinigung** veralteter Posting-Einträge aus dem LSM-Storage.
* **Race-Fenster:** Ein Race-Fenster ist ausgeschlossen, da die logische Sichtbarkeit von Löschungen/Updates über synchrone LSM-Tombstone-Keys (`tbs:`) mit MVCC-Snapshot-Isolation (`get_at_seq`) abgesichert ist.

---

### 6. Überprüfung des §11.3-Befundes "kein Block-Max-WAND im Repo"

**BELEGT (WIDERLEGT / REFUTED)** (`crates/memfuse-text/src/wand.rs:1-473`)

* **Befund aus §11.3:** "Kein `BlockMaxWand`/`WandCursor`/`block_max_score` im Repo vorhanden (Stand v9)."
* **Ist-Zustand (aktueller Codebase-Stand):** **WIDERLEGT.**
  Block-Max WAND ist in `crates/memfuse-text/src/wand.rs` vollständig und produktiv implementiert:
  - `TermCursor` berechnet und speichert `term_max_score` sowie `block_max_scores: Vec<f32>` pro Posting-Block (`BLOCK_SIZE = 128` in `posting_list.rs`) (`crates/memfuse-text/src/wand.rs:96-128`).
  - `block_max_wand_search` nutzt zweistufiges Pruning: (1) Pivot-Selektion über die globale Term-Obergrenze `term_max_score` und (2) Pruning auf Block-Ebene via `block_max_score_at_doc_id(pivot_doc_id)` (`crates/memfuse-text/src/wand.rs:290-325`).
  - Die Implementierung ist durch Äquivalenz- und Performance-Tests in `crates/memfuse-text/src/wand.rs:412-473` belegt (`test_block_max_wand_equivalence_and_pruning_performance`).

---

## 3. Zero-Panic Audit der Grenzflächen-Funktionen

Eine Überprüfung auf `.unwrap()` / `.expect()` in den Grenzflächen-Modulen ergab:

1. `crates/memfuse-text/src/wand.rs`:
   - Produktion: In `Bm25Candidate::cmp` (`wand.rs:31`) wird `.unwrap_or(std::cmp::Ordering::Equal)` verwendet. Keine unbehandelten `.unwrap()`/`.expect()` Aufrufe.
   - Tests: `.expect()` / `.unwrap()` befinden sich ausschließlich innerhalb von `#[cfg(test)]` (`wand.rs:412-473`).
2. `crates/memfuse-text/src/bm25.rs`:
   - Produktion: Zero `.unwrap()` / `.expect()`.
   - Tests: `#[cfg(test)]` nutzt `.unwrap()` / `.expect()` für Zusicherungen.
3. `crates/memfuse-text/src/inverted.rs`:
   - Produktion: Zero `.unwrap()` / `.expect()`. Fehler werden konsequent via `MemFuseError` und `?` propagiert.
   - Tests: `#[cfg(test)]` nutzt `.unwrap()` / `.expect()` für Test-Assserts.

---

## 4. Zusammenfassung

| Prüfpunkt | Status | Befund / Zusammenfassung |
| :--- | :---: | :--- |
| **Cross-Crate Imports** | **BELEGT** | `memfuse-db` nutzt `InvertedIndex` & `Language` aus `memfuse-text`. |
| **Score-Skala** | **BELEGT** | Roher BM25-Score (`f32`, $[0.0, +\infty)$) wird an Aufrufer übergeben. |
| **Partial-Failure** | **BELEGT** | Bei leerer Query oder fehlenden Postings wird `Ok(Vec::new())` zurückgegeben. |
| **DocId-Herkunft** | **BELEGT** | `DocId` wird extern bereitgestellt; keine interne Generierung im Insert-Pfad. |
| **Tombstone-Konsultation** | **BELEGT** | Synchrone MVCC-Tombstone-Prüfung im WAND-Iterationspfad, frei von Race-Conditions. |
| **§11.3 Block-Max WAND** | **WIDERLEGT** | Block-Max WAND ist vollständig in `wand.rs` implementiert und getestet. |
