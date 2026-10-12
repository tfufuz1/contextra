# Audit-Bericht: Algorithmitischer & Determinismus-Tiefenaudit BM25-System (`contextra-text`)

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect (Contextra)
**Session Hash:** `98012b09` | **TS:** `2026-10-12T17:09:46Z`
**Crate:** `contextra-text` (Ring 0)
**Gegenstand:** `crates/contextra-text/src/` (`bm25.rs`, `posting_list.rs`, `wand.rs`, `morphology.rs`, `tokenizer.rs`, `inverted/`)

---

## 1. SESSION-BOOTSTRAP & PRÜFSEQUENZ

### (a) Session-Identität
- **SESSION_HASH:** `98012b09`
- **TS:** `2026-10-12T17:09:46Z`

### (b) Blocker- & Critical-Scan
- **Anzahl Funde:** 0 (`grep -rn "AI-TAG\[.*\]\[BLOCKER\]\|AI-TAG\[.*\]\[CRITICAL\]" crates/ --include="*.rs" | grep -v RESOLVED`)

### (c) Aktive VETOs
- **VETO F-02:** Review-Frist `2026-10-07` abgelaufen (Conditional Review Due). [BELEGT: VETOES.md:3-10]
- **VETO OP-03:** Review-Frist `2026-10-07` abgelaufen (Conditional Review Due). [BELEGT: VETOES.md:19-26]
- **Überfällig?:** JA (Review-Fristen vom 2026-10-07 abgelaufen). Da dieser Audit-PR keine VETO-geschützten HNSW/Memory-Funktionen modifiziert, ist die Session governance-konform.

### (d) Build-Status
- `cargo check --workspace --exclude contextra-py`: **OK** (Build erfolgreich in 47.53s, 0 Fehler, 1 gewohnte Deprecation-Warnung in `contextra-mcp`).

### (e) Git-Kontext (Letzte 3 geänderte Module)
1. `crates/contextra-store/` (PR #4571 / Commit `50a3d899`): Tiefenaudit LSM-Tree-Kern.
2. `crates/contextra-store/` (PR #4569 / Commit `eb0ec8c0`): Audit WAL-Subsystem.
3. `crates/contextra-testkit/` (PR #4566 / Commit `297fed8f`): Quality Audit Report.

### (f) Ring-Layering
- `cargo xtask check-ring-layering-full`: **3 dokumentierte Allowlist-Ausnahmen**, **6 undokumentierte Ring-Verstöße** (Warning-Modus aktiv, Exit 0).

### (g) Unsafe-Inseln
- `cargo xtask check-unsafe-islands`: **Konform** (0 Fehler). In `contextra-text/src/` (Produktionscode) gibt es **0 `unsafe`-Blöcke**.

### (h) GO / STOP-Empfehlung für produktive Arbeit
- **GO** für reine Audit-Dokumentation / IST-Zustandserfassung.
- **STOP / VORBEHALT** vor funktionalen Code-Änderungen: Keine Code-Änderungen in dieser Sitzung vorgesehen.

---

## 2. DETAIL-PRÜFUNG DERS PRÜFPUNKTE (B1 - B8)

### B1: BM25-FORMEL-KORREKTHEIT (`bm25.rs`)

* **Mathematische Definition**:
  $$\text{BM25}(t, d) = \text{IDF}(t) \cdot \frac{\text{TF}(t, d) \cdot (k_1 + 1)}{\text{TF}(t, d) + k_1 \cdot \left(1 - b + b \cdot \frac{|d|}{\text{avgdl}}\right)}$$

* **Standardparameter**:
  - `k1 = 1.5` (`BM25_K1`, `bm25.rs:10`)
  - `b = 0.75` (`BM25_B`, `bm25.rs:12`)

* **Konfigurierbarkeit**:
  - `BM25::new(k1, b)` (`bm25.rs:28-42`) validiert `k1 >= 0.0` (non-NaN, finite) sowie `0.0 <= b <= 1.0` (non-NaN, finite).
  - `score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, b)` (`bm25.rs:78-112`) erlaubt dynamische Parameterübergabe per Query.

* **Codezeilen-Nachweis**:
  [EVIDENCE: BM25-FORMULA-1] `crates/contextra-text/src/bm25.rs:101-111`
  ```rust
  let avg_doc = avg_doc_len.max(1.0);
  let norm_doc_len = doc_len / avg_doc;

  let tf_numerator = tf * (k1 + 1.0);
  let tf_denominator = tf + k1 * (1.0 - b + b * norm_doc_len);

  idf * (tf_numerator / tf_denominator)
  ```

---

### B2: IDF-BERECHNUNG (`bm25.rs`)

* **Formel-Variante**:
  Contextra nutzt die geglättete Robertson-Spärck-Jones BM25+ IDF Formel:
  $$\text{IDF}(t) = \ln\left(1 + \frac{N - \text{df}(t) + 0.5}{\text{df}(t) + 0.5}\right)$$
  - **Vorteil**: Diese Formulierung garantiert mathematisch $\text{IDF}(t) \ge \ln(1) = 0$ für alle $\text{df}(t) \in [0, N]$. Es treten keine negativen IDF-Werte bei hochfrequenten Termen ($\text{df}(t) > N/2$) auf, und künstliche Floor-Klammern entfallen.

* **Document Frequency Validation**:
  - `df(t)` repräsentiert exakt die Anzahl der Dokumente, die den Term $t$ enthalten (Document Frequency), nicht die Term-Frequency.
  - Bei $\text{df} = 0$ oder $N = 0$ oder $\text{tf} = 0$ liefert die Funktion strikt `0.0` zurück (`bm25.rs:87-89`).
  - Sollte durch Datenkorruption $\text{df} > N$ übergeben werden, wird `df` per `df.min(n)` auf $N$ geclampt (`bm25.rs:93`).

* **Codezeilen-Nachweis**:
  [EVIDENCE: BM25-IDF-1] `crates/contextra-text/src/bm25.rs:92-100`
  ```rust
  let df = df.min(n) as f32;
  let n = n as f32;

  let idf = {
      let arg = 1.0 + (n - df + 0.5) / (df + 0.5);
      arg.ln()
  };
  ```

---

### B3: AVGDL-KONSISTENZ (`inverted/index_struct.rs` & `inverted/tests/tx_tests.rs`)

* **Inkrementelle Aktualisierung**:
  - Der Index hält `total_docs` (`AtomicU64`) und `total_tokens` (`AtomicU64`).
  - Bei `upsert_document` / `remove_document` werden in `commit_stats` die Gesamtdokumentanzahl und Gesamttokennanzahl transaktional aktualisiert.
  - `avg_doc_len_x1000` speichert den gerundeten Wert als Fixed-Point (`AtomicU64`):
    `avg_len = (total_tokens * 1000) / total_docs` (`index_struct.rs:500-505`).

* **Test-Nachweis**:
  - `test_avgdl_accuracy_incremental_updates` in `crates/contextra-text/src/inverted/tests/tx_tests.rs:99-186` prüft die `avgdl`-Genauigkeit nach wiederholtem Einfügen und Löschen von Dokumenten.
  - **Ergebnis**: Exakte Übereinstimmung der `avgdl`-Metrik vor und nach Löschungen (`actual_avgdl == expected_avgdl`).

* **Codezeilen-Nachweis**:
  [EVIDENCE: AVGDL-CONSISTENCY-1] `crates/contextra-text/src/inverted/index_struct.rs:500-505`
  ```rust
  let avg_len = if docs > 0 {
      (tokens * 1000) / docs
  } else {
      0
  };
  self.avg_doc_len_x1000.store(avg_len, Ordering::SeqCst);
  ```

---

### B4: POSTING-LIST SORTIERGARANTIE (`posting_list.rs`)

* **Sortierung & Deduplizierung**:
  - `PostingList::new(mut postings)` sortiert Posting-Einträge instabil nach `doc_id` (`postings.sort_unstable_by_key(|p| p.doc_id)`) und dedupliziert doppelte Eintrags-IDs (`posting_list.rs:92-97`).
  - `PostingList::upsert` führt `binary_search_by_key` aus und fügt neue Dokumente an der exakten Einfügeposition ein (`posting_list.rs:133-145`), wodurch die Sortierfolge $O(\log N)$ gewahrt bleibt.
  - `PostingList::remove` nutzt ebenfalls `binary_search_by_key` zur $O(\log N)$-Löschung (`posting_list.rs:148-157`).

* **Block-Max WAND Kompatibilität**:
  - Nach jeder In-Memory-Modifikation berechnet `compute_blocks(&new_postings)` (`posting_list.rs:58-86`) die 64-Element-Block-Metadaten (`max_doc_id`, `max_tf`, `min_doc_len`) neu.

* **Codezeilen-Nachweis**:
  [EVIDENCE: POSTING-SORT-1] `crates/contextra-text/src/posting_list.rs:92-97, 133-145`

---

### B5: WAND-KORREKTHEIT (`wand.rs`)

* **Pruning-Korrektheit**:
  - `block_max_wand_search` (`wand.rs:188-466`) implementiert den Block-Max WAND (Weak-AND) Algorithmus.
  - **Obere Schranke**: `block_max_score_at_doc_id` errechnet konservative Obergrenzen für BM25-Scores pro Block (`wand.rs:136-168`).
  - **Sicherheits-Garantie**: Es werden ausschließlich Dokumente übersprungen, deren obere Schranke strikt $\le$ `threshold` (dem $k$-besten bisherigen Score) liegt.

* **Test-Nachweis**:
  - `test_block_max_wand_equivalence_and_pruning_performance` in `crates/contextra-text/src/wand.rs:659-735` indexiert 500 Dokumente und vergleicht Top-k WAND-Ergebnisse gegen Brute-Force-Scores.
  - **Ergebnis**: 100% Identische Top-$k$ Dokument-IDs und BM25-Scores bei signifikant reduzierter Evaluierungsanzahl (`evaluated_docs_count < 500`).

* **Codezeilen-Nachweis**:
  [EVIDENCE: WAND-CORRECTNESS-1] `crates/contextra-text/src/wand.rs:360-375`

---

### B6: MORPHOLOGIE-KORREKTHEIT (`morphology.rs` & `tokenizer.rs`)

* **GermanCompoundSplitter**:
  - Zerlegt deutsche Komposita (z. B. "Bundesverfassungsgericht" $\rightarrow$ ["bundes", "verfassungs", "gericht"]) unter Berücksichtigung von Fugenelementen (`-s-`, `-en-`, `-e-`, `-er-`, `-n-`, `-es-`).

* **Idempotenz & Proper Nouns**:
  - **Idempotenz**: Erneuter Aufruf von `decompose` mit bereits zerlegten Token ("bundes", "gericht") liefert unzerlegt das jeweilige Morphem zurück (`path.len() < 2`), da die Mindestkomponentenlänge `min_component_len = 3` greift.
  - **Eigen- und Sonderschutz**: Unbekannte Stems oder Eigennamen (z. B. Firmennamen), die nicht im eingebetteten KMU-Wörterbuch enthalten sind, fallen auf die ungeöffnete Rückgabe des Original-Tokens zurück (*Safe Fallback*).
  - **Case Contract**: `decompose` erfordert klein-geschriebene Eingaben (`debug_assert!(!has_uppercase)` in Debug-Builds; automatische Umlaut-Normalisierung in `GermanMorphTokenizer`).

* **Komposita-Testergebnisse (KMU Suite)**:
  | Testfall / Wort | Erwartete Zerlegung | Tatsächliches Ergebnis | Status |
  |---|---|---|---|
  | `bundesverfassungsgericht` | `["bundes", "verfassungs", "gericht"]` | `["bundes", "verfassungs", "gericht"]` | 🟢 PASSED |
  | `datenschutzrichtlinie` | `["daten", "schutz", "richtlinie"]` | `["daten", "schutz", "richtlinie"]` | 🟢 PASSED |
  | `softwareentwicklungskontext` | `["software", "entwicklung", "kontext"]` | `["software", "entwicklung", "kontext"]` | 🟢 PASSED |
  | `haftpflichtversicherung` | `["haft", "pflicht", "versicherung"]` | `["haft", "pflicht", "versicherung"]` | 🟢 PASSED |

* **Codezeilen-Nachweis**:
  [EVIDENCE: MORPHOLOGY-1] `crates/contextra-text/src/morphology/compound_splitter.rs:182-273`

---

### B7: STOPWORD-FILTERUNG (`tokenizer.rs` & `morphology/stopwords.rs`)

* **Symmetrie**:
  - Sowohl der Indexierungspfad (`InvertedIndex::upsert_document`) als auch der Abfragepfad (`InvertedIndex::search_bm25_at`) nutzen `self.tokenizer.tokenize(text)`.
  - Stoppwörter werden in `DefaultTokenizer` und `GermanMorphTokenizer` herausgefiltert.

* **Verhalten bei reiner Stoppwort-Suche**:
  - Wird eine Query übergeben, die ausschließlich aus Stoppwörtern besteht (z. B. `"der die das"`), erzeugt `tokenize` einen leeren Token-Vektor `[]`.
  - `search_bm25_at` prüft `if tokens.is_empty() { return Ok(Vec::new()); }` (`index_struct.rs:568-570`) und gibt sofort 0 Ergebnisse zurück.
  - Es treten keine Fehltreffer oder Division-by-Zero Fehler auf.

* **Codezeilen-Nachweis**:
  [EVIDENCE: STOPWORD-SYMMETRY-1] `crates/contextra-text/src/tokenizer.rs:125-144, 163-207`

---

### B8: INVERTED-INDEX MVCC (`inverted/index_struct.rs` & `wand.rs`)

* **Isolation & Storage Engine MVCC**:
  - `InvertedIndex::search_bm25_at` akzeptiert `max_seq: Option<u64>`.
  - Werden Suchen mit `max_seq < T1` ausgeführt, während Parallel-Transaktion $T_1$ schreibt, werden Posting-Listen und Tombstones über `storage.get_at_seq(key, seq)` bzw. `storage.scan_prefix_at(prefix, seq)` mit der historischen Sequenznummer `seq` gelesen.
  - Uncommitted staged Data in `staged_terms` und `staged_stats` bleibt für Lesezugriffe isoliert, bis die Transaktion committed ist.

* **Test-Nachweis**:
  - Integrationstests in `crates/contextra-text/tests/tombstone_read_your_writes.rs` (`proof_tombstone_snapshot_isolation`) und `tests/proptest_bm25_invariants.rs` (`prop_5_tombstone_isolation_snapshot_semantics`) bestätigen 100% korrekte Snapshot-Isolierung.

* **Codezeilen-Nachweis**:
  [EVIDENCE: MVCC-ISOLATION-1] `crates/contextra-text/src/inverted/index_struct.rs:581-593`, `crates/contextra-text/src/wand.rs:218-225`

---

## 3. VERDICT BLOCK & BEWEISFÜHRUNG

```
=== CONTEXTRA AUDIT VERDICT BLOCK ===
Crate: contextra-text
Ring: Ring 0
Audit Date: 2026-10-12
Auditor: Principal Senior Rust Architect
Verdict: PASSED (STABLE)

Evidence Summary:
1. BM25-FORMULA-1: Verified Robertson BM25 formula with k1=1.5, b=0.75 in bm25.rs:101-111.
2. BM25-IDF-1: Verified Robertson-Spärck-Jones non-negative IDF formula in bm25.rs:92-100.
3. AVGDL-CONSISTENCY-1: Verified incremental avgdl fixed-point updates in index_struct.rs:500-505 and tx_tests.rs.
4. POSTING-SORT-1: Verified DocId binary-search sorting invariants in posting_list.rs:92-145.
5. WAND-CORRECTNESS-1: Verified Block-Max WAND top-k equivalence against brute force in wand.rs:659-735.
6. MORPHOLOGY-1: Verified German compound splitting, idempotency, and protected fallbacks in compound_splitter.rs:182-273.
7. STOPWORD-SYMMETRY-1: Verified symmetric stopword elimination across index/query paths in tokenizer.rs:125-207.
8. MVCC-ISOLATION-1: Verified storage snapshot isolation for as_of searches in index_struct.rs:581-593.
=====================================
```

---
*Ende des Audit-Berichts.*
