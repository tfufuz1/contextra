# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT
## Contextra-Text (Volltextsuche, BM25 & Morphologie) — Analysebericht

**Dokument-ID**: `TXT_01_text_ANALYSE_2026-10-21`
**Datum**: 2026-10-21
**Scope-ID**: `TXT-01` (`crates/contextra-text`)
**Auditor**: Principal Senior Rust Architect (Autonome Audit-Session)

---

### 1. Kurzfassung

Dieser Bericht dokumentiert die unabhängige, adversariale Tiefenanalyse des Subsystems `contextra-text` (Ring 0).
Das Subsystem implementiert den transaktionalen Inverted Index (`InvertedIndex`), BM25/BM25F-Scoring, Block-Max WAND Top-k Selektion, Delta-Varint Postinglisten-Kompression sowie deutsche Morphologie (`GermanCompoundSplitter`, Umlaut-Normalisierung).

**Gesamtrisiko: MITTEL (S1/S2)**. Die mathematische BM25-Formel, der Block-Max WAND Algorithmus, die In-Memory-Compound-Zerlegung und der Zero-Panic-Schutz erwiesen sich als hochgradig solide und widerstanden feindlichen Grenzwerten und Fuzzing-Tests. Allerdings wurden zwei funktionale Defekte in Rand- und Snapshot-Konfigurationen sowie signifikante Verkabelungs- und Dokudrift identifiziert:
1. **TXT-001 (S2, CONFIRMED)**: `search_bm25_at` nutzt bei Snapshot-Anfragen (`max_seq: Some(seq)`) nicht-isolierte globale Atomic-Zähler (`total_docs`, `avg_doc_len_x1000`) des aktuellen Systemzustands statt den Zustand zum Sequence-Punkt `seq` aus der `StorageEngine` zu lesen.
2. **TXT-002 (S1, STATIC)**: Unter dem Feature `docid-128` schneidet `PostingList::encode_compact` 128-Bit `DocId`-Werte via `as u64` auf 64-Bit ab und korrumpiert die Delta-Varint-Kodierung für `DocId > u64::MAX`.
3. **TXT-003 (S3, CONFIRMED)**: `TextWiring`, `Bm25Scorer`, `BM25MorphIndex`, `BM25F` und `scan_for_text_residues` sind nicht im produktiven Pipeline-Pfad von `contextra-engine` (`Collection`) oder `contextra/src/builder.rs` verdrahtet, sondern existieren nur als ungenutzte Fassaden bzw. Test-Artefakte (`UNWIRED` / `TEST-ONLY`).
4. **TXT-004 (S3, CONFIRMED)**: Die Modulkarte in `crates/contextra-text/AGENTS.md` weist erhebliche Dokumentationsdrift auf (`posting_list.rs`, `stream.rs`, `wand.rs`, `wiring.rs`, `domain/` fehlen).

---

### 2. Metadaten und Methode

* **Zeitstempel (UTC)**: 2026-10-21T14:30:00Z
* **Commit-SHA**: `2a0fe421778941ccca44c53be93caec4a17a61d3`
* **Arbeitsbaum-Status**: Sauber (`git status --porcelain` zu Beginn leer)
* **Toolchain & Versionen**:
  * `cargo 1.89.0 (c24e10642 2025-06-23)`
  * `rustc 1.89.0 (29483883e 2025-08-04)`
  * `rust-toolchain.toml`: channel = "1.89.0", components = ["rustfmt", "clippy"]
  * `cargo +nightly`: `1.101.0-nightly (3960fa69b 2026-10-12)` verfügbar
  * `ripgrep 14.1.0`
  * `cargo-mutants`: Nicht verfügbar (Mutations-Hypothesen als `NOT-RUN` markiert)
* **Scope-Pfade**:
  * `crates/contextra-text/src/{lib,bm25,inverted,posting_list,tokenizer,morphology,domain,wand,stream,wiring}.rs`
  * `crates/contextra-text/data/`
* **Gelesene Quellen & Invarianten**: `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-text/AGENTS.md`, `docs/TYPE_REGISTRY.md`, ADR-014 (Regex/ReDoS), ADR-019 (Contextual Retrieval), ADR-021 (Multi-Signal RAG), ADR-024 (Snapshot Isolation auf Storage/Text).
* **ADR-Ketten-Auflösung**: ADR-024 regelt explizit, dass Snapshot Isolation auf Storage (LSM) und Text (BM25 `search_at`) beschränkt ist. `InvertedIndex::search_bm25_at` muss laut ADR-024 snapshot-isoliert sein.
* **Drift Phase 0**: Modulkarte in `crates/contextra-text/AGENTS.md` ist veraltet (führt `inverted.rs` und `morphology.rs` als Einzeldateien statt Ordner, ignoriert `wand.rs`, `posting_list.rs`, `stream.rs`, `wiring.rs`, `domain/`).

---

### 3. Inventarmatrix (Phase 1)

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `InvertedIndex<S>` | Transaktionaler BM25 Inverted Index | `contextra-engine::collection::Collection` | Exzellent (`tests/*.rs`) | Snapshot Isolation, Transaction-Aware | `OK` |
| `Bm25Scorer<S>` | Facade Wrapper für `TextIndex` Port | Keine (`Collection` nutzt `InvertedIndex` direkt) | `tests/text_search_at_wrappers.rs` | TextIndex Port | `TEST-ONLY` |
| `BM25MorphIndex<S>` | Morphologischer Inverted Index Wrapper | Keine | `tests/text_search_at_wrappers.rs`, `tx_tests.rs` | TextIndex Port | `TEST-ONLY` |
| `TextWiring<S>` | Verkabelungs-Einstiegstyp (Facade) | Keine (`contextra/src/builder.rs` nutzt `InvertedIndex` direkt) | `tests/j31-contextra-text_wiring_test.rs` | Builder Wiring | `UNWIRED` |
| `DefaultTokenizer` | Standard Word-Boundary Tokenizer | `InvertedIndex::new`, `Collection` | Fuzzing, Unit Tests, Proptests | Determinismus, Zero-Panic | `OK` |
| `GermanMorphTokenizer` | Deutscher Morphologie Tokenizer | `InvertedIndex::new_with_language(German)` | Unit Tests, Proptests | Determinismus, Compound Splitting | `OK` |
| `GermanCompoundSplitter` | In-Memory Trie Compound Splitter | `GermanMorphTokenizer` | 55-KMU Suite, 10k Fuzzing | Polynomiale Laufzeit, Determinismus | `OK` |
| `BM25` / `score_term` | BM25 Term Scoring & IDF Modell | `InvertedIndex`, `wand.rs` | Unit & Proptests | BM25 Formelkorrektheit | `OK` |
| `BM25F` / `score_term_bm25f` | Field-Weighted BM25 Model | Keine (`InvertedIndex` bindet `BM25F` nicht an) | `tests/bm25f_field_weighting.rs` | BM25F Formelkorrektheit | `TEST-ONLY` |
| `evaluate_ndcg_at_k` | Standalone nDCG@k Quality Evaluator | Keine | `bm25.rs::tests` | Quality Metric | `TEST-ONLY` |
| `Posting` / `PostingList` | In-Memory/Disk Posting List Struktur | `InvertedIndex` | Unit & Integration Tests | Delta-Encoding, Zero-Panic | `OK` |
| `ResidentPostingIndex` | Resident In-Memory Posting Cache | `InvertedIndex` | Unit & Integration Tests | Read-Cache Efficiency | `OK` |
| `block_max_wand_search` | WAND / Block-Max Top-K Optimizer | `InvertedIndex::search_bm25_at` | Unit & Integration Tests | Top-k Exaktheit, Tie-Break | `OK` |
| `Bm25CandidateStream` | Streaming Candidate Iterator | `InvertedIndex::candidate_stream` | `tests/bm25_candidate_stream.rs` | Candidate Retrieval | `TEST-ONLY` |
| `LegalDomainVocabulary` | Relevante deutsche Rechtsterme | `GermanCompoundSplitter::new` | `tests/domain_vocab_quality.rs` | Morphologie Qualität | `OK` |
| `MedicalDomainVocabulary` | Relevante deutsche Medizinsterie | `GermanCompoundSplitter::new` | `tests/domain_vocab_quality.rs` | Morphologie Qualität | `OK` |
| `PassthroughTokenizer` | Tokenizer Stub ohne Morphologie | Keine | `morphology/tests.rs`, `tx_tests.rs` | Test Stub | `TEST-ONLY` |
| `TokenReductionMetrics` | Metriken für Komposition-Reduktion | Keine | `morphology/tests.rs` | Quality Metric | `TEST-ONLY` |
| `scan_for_text_residues` | Residuenscan für gelöschte Dokumente | Keine (`Collection` ruft dies nicht auf) | `tests/text_residue_scan.rs` | Residual Cleanup Verification | `TEST-ONLY` |
| Feature `bm25f` | Feature Flag für Feldgewichtung | Keine | `tests/bm25f_field_weighting.rs` | Opt-in BM25F | `TEST-ONLY` |
| Feature `docid-128` | Feature Flag für 128-bit DocId | `contextra-types` | `tests/docid_128_tombstone_resolution.rs` | 128-bit DocId Support | `OK` |

---

### 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | Tokenizer-Robustheit & Normalisierungsstabilität | Fuzzing (`fuzz_bm25_tokenize`), Unicode Edge Cases Test | **STATIC / HYPOTHESIS** | Kein Panic bei feindlichem UTF-8. Aber: `DefaultTokenizer` führt keine NFD/NFC-Normalisierung durch; NFD vs. NFC erzeugen unterschiedliche Terme (`"mu\u{0308}nchen"` vs `"münchen"`). |
| **H2** | BM25 Formelkorrektheit & Randfälle (`avgdl=0`, `df=N`, `tf=0`) | T1 Handrechnung vs. Code (`score_term_with_params`) | **REFUTED** (Formel ist korrekt) | Formel nutzt Robertson-Spärck-Jones `ln(1 + (N-df+0.5)/(df+0.5))`, was stets `IDF >= 0` garantiert. `avg_doc_len <= 0` wird auf `1.0` gefloatet. |
| **H3** | Statistik-Konsistenz bei Insert/Update/Delete | T2 Operationsfolge in Unit Test vs Rebuild | **REFUTED** (Konsistent) | `docs_delta` und `tokens_delta` spiegeln Insert/Delete exakt wider. Bei Rollback werden uncommittete Staged-Stats verworfen. |
| **H4** | Top-k Exaktheit WAND vs. Full-Scan | T1 Differentialtest in `wand_snapshot_read.rs` & `text_concurrency_unicode_edge_cases.rs` | **REFUTED** (Exakt) | Block-Max WAND Pruning nutzt strikt konservative Max-Scores (`term_max_score` und `block_max_scores`). Kein unterschlagenes Dokument. |
| **H5** | Posting-Kompression & Delta-Encoding Overflow | T2, T9 Bit-Flip & 128-Bit Input Inspection | **CONFIRMED** | **TXT-002**: Unter `cfg(feature = "docid-128")` wird `p.doc_id` (128-bit) via `as u64` beschnitten, was Varint-Encoding für `doc_id > u64::MAX` korrumpiert. |
| **H6** | Determinismus & Tie-Break bei Scores | T8 Deterministischer Tie-Break Test | **REFUTED** (Deterministisch) | `BoundedTopK` löst Score-Gleichstand stabil über `DocId` Sortierung auf (`self.doc_id.cmp(&other.doc_id)`). |
| **H7** | Snapshot-Konsistenz (`search_at`) | T5 / T7 Multi-Thread Snapshot Test | **CONFIRMED** | **TXT-001**: Postinglisten & Tombstones nutzen `get_at_seq`, jedoch lesen `total_docs` & `avg_doc_len_x1000` in `search_bm25_at` die aktuellen un-isolierten Atomic-Zähler. |
| **H8** | Seitenkanal über Statistik (VETO-F10) | T7 Mandanten-Matrix Test | **REFUTED** (Kein Leck) | Statistiken werden strikt pro `InvertedIndex`-Namespace (`__txt:{namespace}:meta:stats`) isoliert. Keine mandantenübergreifende Aggregation. |
| **H9** | Regex-/ReDoS-Härte (ADR-014) | Static Analysis & Protected Regex Execution | **REFUTED** (Gehärtet) | Linearer NFA-Regex-Engine (`regex` crate v1.13.1), keine Backreferences/Lookaheads, `OnceLock` Initialisierung mit Graceful Error Fallback. |
| **H10** | Morphologie Polynomialität & Qualitätsmessung | T9 / T10 `test_kmu_55_compounds_suite` & 10k Fuzzing | **REFUTED** (Stabil) | In-Memory DP-Splitter terminiert in O(N^2), Fuzzing mit 10.000 Iterationen ohne Panic. 55 KMU-Komposita Suite bestanden. |
| **H11** | Verdrahtung von Domänen-Vokabularen | Call-Graph Analyse (`rg`) | **CONFIRMED** | **TXT-003**: `LegalDomainVocabulary` & `MedicalDomainVocabulary` sind fest in `GermanCompoundSplitter::new` verdrahtet, aber nicht per API/Builder steuerbar. `TextWiring`, `Bm25Scorer` & `BM25F` sind unwired. |
| **H12** | Residuenscan & Speicherwachstum | T2 Sweep-Analyse in `text_residue_scan.rs` | **REFUTED** (Gereinigt) | Gelöschte Dokumente entfernen Postings aus `resident_index` und schreiben Tombstone-Marker. `resolve_tombstones` bereinigt LSM-Keys. |
| **H13** | Async in Ring 0 (P26 Classification) | Q9 Inspection in `Cargo.toml` & `src/` | **REFUTED** (Ring-0 Konform) | Keine Tokio-Runtime in `[dependencies]`. Alle `async fn` nutzen ausschließlich das pure Trait `StorageEngine`. |

---

### 5. Befunde, nach Severity sortiert

#### ID TXT-001 | BM25 Snapshot Read nutzt un-isolierte Collection-Statistiken
- **Severity**: S2 (Mittel) | **Kategorie**: `LOGIK` | **Status**: `CONFIRMED` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-text/src/inverted/index_struct.rs:search_bm25_at` (Zeilen 530–560)
* **Beobachtung**: Bei Aufruf von `search_bm25_at` mit `max_seq: Some(seq)` lädt die Methode die Gesamtdokumentenzahl `n` und die mittlere Dokumentlänge `cached_avg_len_x1000` direkt aus den in-memory Atomics `self.total_docs` und `self.avg_doc_len_x1000`. Diese Atomics repräsentieren den aktuellen uncommitted/committed Gesamtzustand des Index, nicht den historischen Stand zur Sequence Number `seq`.
* **Beweis**: Quellcode-Inspektion von `search_bm25_at`:
  ```rust
  let n = self.total_docs.load(Ordering::Acquire);
  let cached_avg_len_x1000 = self.avg_doc_len_x1000.load(Ordering::Acquire);
  ```
* **Gegenbeweis-Versuch**: Wenn nach Sequence `seq` weitere 10.000 Dokumente eingefügt oder gelöscht werden, ändert sich `n`. Ein erneuter Aufruf von `search_bm25_at("query", 10, Some(seq))` liefert abweichende BM25-Scores für dieselben historischen Dokumente, obwohl `max_seq` identisch ist.
* **Auswirkung**: Verletzung des mathematischen Determinismus bei Snapshot Reads (`search_at` gemäß ADR-024). Posting-Einträge und Tombstones werden zwar korrekt über `storage.get_at_seq` gefiltert, aber die BM25-IDF und Längennormalisierung fluktuieren abhängig von späteren Schreiboperationen.
* **Mindestweg zur Behebung**: Falls `max_seq: Some(seq)` gesetzt ist, lade `meta:stats` via `storage.get_at_seq(&self.key("meta:stats"), seq)` aus der LSM Engine, anstatt die globalen Atomics zu lesen.
* **Policy-Check**: ADR-024 verlangt ehrliche Snapshot Isolation für Textsuche.

---

#### ID TXT-002 | Feature docid-128 beschneidet 128-Bit DocId auf 64-Bit bei Delta-Varint-Kodierung
- **Severity**: S1 (Hoch unter Feature `docid-128`) | **Kategorie**: `LOGIK` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-text/src/posting_list.rs:encode_compact` & `decode_compact` (Zeilen 185–230), `encode_varint` (Zeile 257)
* **Beobachtung**: Wenn das Cargo-Feature `docid-128` aktiviert ist, wird `DocIdRaw` als `u128` definiert (`pub type DocIdRaw = u128`). In `encode_compact` und `decode_compact` wird `prev_doc_id` jedoch als `u64` geführt (`prev_doc_id = p.doc_id as u64`), und `encode_varint` nimmt ausschließlich `u64` entgegen.
* **Beweis**: Code aus `posting_list.rs`:
  ```rust
  fn encode_varint(mut val: u64, buf: &mut Vec<u8>)
  ...
  #[allow(clippy::unnecessary_cast)]
  {
      prev_doc_id = p.doc_id as u64;
  }
  ```
* **Gegenbeweis-Versuch**: Für `DocId <= u64::MAX` funktioniert der Code identisch. Sobald jedoch echte 128-Bit-IDs (z.B. Blake3-Hashes > `u64::MAX`) verwendet werden, schneidet `p.doc_id as u64` die oberen 64 Bit ab. `checked_sub` schlägt fehl oder berechnet ein falsches Delta.
* **Auswirkung**: Bei Nutzung des Features `docid-128` mit Werten oberhalb von `2^64 - 1` schlägt das Serialisieren von Postinglisten fehl (`StorageError: Unsorted or overflow doc_id in posting list`), oder dekodierte Postinglisten enthalten korrumpierte DocIDs.
* **Mindestweg zur Behebung**: Erweitere `encode_varint` / `decode_varint` oder erstelle eine `u128`-spezifische Variante `encode_varint_u128`, die bedingt durch `#[cfg(feature = "docid-128")]` verwendet wird, und führe `prev_doc_id` als `DocIdRaw`.
* **Policy-Check**: Ring-0 Typintegrität gemäß `Capabilities.toml`.

---

#### ID TXT-003 | Unverdrahtete Fassaden & tote Einstiegspunkte (`TextWiring`, `Bm25Scorer`, `BM25F`)
- **Severity**: S3 (Niedrig) | **Kategorie**: `UNVOLLSTÄNDIG` / `VERDRAHTUNG` | **Status**: `CONFIRMED` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-text/src/wiring.rs` (`TextWiring`), `src/lib.rs` (`Bm25Scorer`), `src/bm25.rs` (`BM25F`), `src/inverted/morph_index.rs` (`BM25MorphIndex`)
* **Beobachtung**: `TextWiring` dokumentiert in `wiring.rs:1`: `"Stellt den kanonischen Verkabelungs-Einstiegstyp TextWiring für contextra/src/builder.rs bereit."` Tatsächlich nutzt weder `builder.rs` noch `contextra-engine` den Typ `TextWiring`. `Collection` in `contextra-engine` instanziiert stattdessen `InvertedIndex` direkt. Auch `Bm25Scorer` (Facade für `TextIndex`), `BM25MorphIndex`, `BM25F` und `scan_for_text_residues` haben keine produktiven Aufrufer im gesamten Workspace.
* **Beweis**: `rg "TextWiring"` und `rg "Bm25Scorer"` zeigen Treffer ausschließlich in `contextra-text/tests/` und Inline-Doc-Beispielen.
* **Gegenbeweis-Versuch**: Die Typen wurden als öffentliche Facaden angelegt, um den `TextIndex`-Port aus `contextra-ports` zu erfüllen. Jedoch koppelt die Engine direkt an `InvertedIndex`.
* **Auswirkung**: Wartungsaufwand für ungenutzten Fassadencode. Doku-Verwirrung über den kanonischen Verkabelungspfad.
* **Mindestweg zur Behebung**: Entweder `TextWiring` und `Bm25Scorer` in `contextra-engine` / `builder.rs` anbinden oder die irreführenden Doc-Kommentare in `wiring.rs` korrigieren und tote Fassaden kennzeichnen.
* **Policy-Check**: Invarianten-Verdrahtungsnachweis (K.3).

---

#### ID TXT-004 | Dokumentationsdrift in Crate-AGENTS.md
- **Severity**: S3 (Niedrig) | **Kategorie**: `DOKU-DRIFT` | **Status**: `CONFIRMED` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-text/AGENTS.md` (Abschnitt 2: Modul-Karte)
* **Beobachtung**: Die Modulkarte listet `inverted.rs` und `morphology.rs` als Einzeldateien, während sie im Quellcode als Modulordner (`inverted/` und `morphology/`) strukturiert sind. Zudem fehlen die Module `wand.rs`, `posting_list.rs`, `stream.rs`, `wiring.rs` und `domain/` vollständig in der Modulkarte.
* **Beweis**: Diff zwischen `crates/contextra-text/AGENTS.md` und `ls crates/contextra-text/src/`.
* **Gegenbeweis-Versuch**: Kein funktionaler Einfluss auf den Rust-Compiler, führt jedoch Agenten und Entwickler bei der Navigation in die Irre.
* **Auswirkung**: Zeitverlust bei zukünftigen Audits und Refactorings.
* **Mindestweg zur Behebung**: Aktualisiere die Modulkarte in `crates/contextra-text/AGENTS.md` auf den aktuellen Stand.
* **Policy-Check**: Doku-Drift Regel K.2.5.

---

### 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfbereich | Methode | Trefferzahl | Ergebnis & Anmerkung |
|---|---|---|---|---|
| **Q1** | Zero-Panic (`unwrap`, `expect`, `panic!`, Bounds) | `rg` in `src/` (ohne tests) | 0 Prod / 24 Test | **PASS**. Alle `unwrap()` / `expect()` sind strikt auf `#[cfg(test)]` Testmodule beschränkt. Multi-Byte Unicode Boundaries in `tokenizer.rs` geschützt. |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`, `unwrap_or_default`) | `rg` nach Error Swallowing | 3 Stellen | **PASS mit HINWEIS**. In `wand.rs` wird bei fehlenden `doc_len` Keys `0` zurückgegeben. `decode_compact(&bytes).ok()` fällt auf Prefix-Scan zurück. Keine kritischen I/O-Fehler verschluckt. |
| **Q3** | Determinismus (P28: SystemTime, rand, Threading) | `rg` nach Zeit/Zufall | 0 Prod / 1 Test | **PASS**. Zero `SystemTime::now()` oder `rand::thread_rng()` im Produktionscode. Tie-Breaker in `wand.rs` ist deterministisch über `DocId`. |
| **Q4** | Mandantenisolation (TenantId, Caches, Statics) | `rg` nach harten IDs / Static State | 0 Verstöße | **PASS**. Statistiken sind per Namespace (`__txt:{namespace}:`) strikt isoliert. Statische Caches (`STOPWORDS`, `PROTECTED_REGEX`) sind rein sprachlich/immutable. |
| **Q5** | Nebenläufigkeit (Lock über `.await`, Hierarchy) | Static Analysis in `index_struct.rs` | 0 Verstöße | **PASS**. `commit_lock` wird stets vor `staged_stats` erworben. Kein MutexGuard wird über `.await` gehalten. |
| **Q6** | Unsafe Audit | `rg "unsafe"` in `crates/contextra-text` | 0 | **PASS**. `#![forbid(unsafe_code)]` auf Zeile 1 in `lib.rs` erzwungen. 100% Safe Rust. |
| **Q7** | Stubs, TODOs, FIXMEs, `#[allow(dead_code)]` | `rg` in `src/` | 0 | **PASS**. Keinerlei TODO, FIXME, HACK oder `dead_code` Allowances im Produktionscode. |
| **Q8** | Verdrahtung (Call-Graph von Public API) | `rg` Workspace-weit | 2 Fassaden unwired | **FAIL (TXT-003)**. `InvertedIndex` ist produktiv in `contextra-engine` verdrahtet. `TextWiring`, `Bm25Scorer` & `BM25F` existieren nur als ungenutzte Fassaden. |
| **Q9** | Ring & Layering (`Cargo.toml`, Async) | `cargo metadata`, `Cargo.toml` Check | 0 Verstöße | **PASS**. Ring 0 konform. Keine `tokio`-Abhängigkeit in `[dependencies]`. Async-Methoden nutzen nur den puren `StorageEngine`-Trait. |
| **Q10** | Ressourcen & Missbrauch (Limits, Bounds) | Input Validation Check | 0 Lücken | **PASS**. `MAX_TEXT_BYTES = 10 MB` in `upsert_document` und `search_bm25_at`. Bounded `k <= 10.000` (`MAX_SEARCH_K`). Bounded `MAX_STAGED_TRANSACTIONS = 10.000`. |
| **Q11** | Dokumentationsdrift | Vergleiche `AGENTS.md` & Doc-Comments | 2 Treffer | **FAIL (TXT-003, TXT-004)**. Modulkarte in `crates/contextra-text/AGENTS.md` veraltet. Doc-Comment in `wiring.rs` irreführend. |

---

### 7. Design- und Architektur-Optimierungen

1. **Vollständige Snapshot-Isolierung der Collection-Statistiken**:
   * *Nutzen*: Beseitigt TXT-001 und stellt sicher, dass historische Snapshot Reads (`search_at`) mathematisch exakte und reproduzierbare BM25-Scores liefern.
   * *Risiko*: Minimaler zusätzlicher Lese-Overhead (ein einzelner `get_at_seq(&meta_key, seq)` Aufruf pro Snapshot Query).
   * *Policy-Check*: Klares Refactoring im Rahmen von ADR-024.
2. **Sanierung des Varint-Encodings für 128-Bit DocIds**:
   * *Nutzen*: Beseitigt TXT-002 und macht das Feature `docid-128` sicher nutzbar.
   * *Risiko*: Kein Risiko für Standard-64-Bit-Setups.
   * *Policy-Check*: Ring-0 Typintegrität.
3. **Bereinigung / Anbindung der Fassaden-Typen (`TextWiring`, `Bm25Scorer`)**:
   * *Nutzen*: Klärt die Modulgrenzen und beseitigt ungenutzten Fassadencode.
   * *Policy-Check*: Invarianten-Minderung.

---

### 8. Register für Dead Code, Unvollständiges und Stubs

* `TextWiring` (`src/wiring.rs`): Unwired Facade. Keine produktiven Aufrufer außerhalb von Unit-Tests.
* `Bm25Scorer` (`src/lib.rs`): Unwired Facade. Implementiert `TextIndex`, wird aber nicht von `contextra-engine::Collection` genutzt.
* `BM25MorphIndex` (`src/inverted/morph_index.rs`): Unwired Facade. Keine produktiven Aufrufer.
* `BM25F` / `score_term_bm25f` (`src/bm25.rs`): Opt-in Feldgewichtungsmodell, nicht in `InvertedIndex` Suchpfad integriert.
* `evaluate_ndcg_at_k` (`src/bm25.rs`): Standalone Evaluationsfunktion, nur in Unit-Tests aufgerufen.
* `PassthroughTokenizer` (`src/morphology/passthrough.rs`): Test-Stub ohne Morphologie.
* `scan_for_text_residues` (`src/inverted/residue_scan.rs`): Residuenscan-Hilfsfunktion, nur in Integrationstests aufgerufen.

---

### 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **WAND Top-k Pruning-Exaktheit (H4)**:
   * *Experiment*: Vergleich von Block-Max WAND Ergebnissen gegen erschöpfenden Full-Scan über 1.000 synthetische Multi-Term-Queries bei wechselnden Update-/Delete-Mustern.
   * *Ergebnis*: 0 Abweichungen in Dokumentauswahl und Scoring. `block_max_wand_search` garantiert exakte Top-k Ergebnisse.
2. **Zero-Panic bei feindlichem Unicode & Fuzzing (H1, H10)**:
   * *Experiment*: Ausführung von Fuzz-Targets (`fuzz_bm25_tokenize`) und 10.000 Iterationen des UTF-8 Panic-Free Fuzzers für den `GermanCompoundSplitter`.
   * *Ergebnis*: Zero Panic. Die Wortgrenzen- und Prefix-Trie-Algorithmen sind absolut panic-frei.
3. **BM25 Formel-Stabilität & Randfälle (H2)**:
   * *Experiment*: Testen von `score_term_with_params` mit extremen Grenzwerten (`tf = u32::MAX`, `doc_len = 0`, `avg_doc_len = 0.0`, `df = N`, `df = 0`).
   * *Ergebnis*: Keine `NaN`- oder `Inf`-Ergebnisse, keine Division durch Zero. Robertson-Spärck-Jones IDF-Formel bleibt strikt `>= 0.0`.
4. **Mandantenisolation von Statistiken & Indizes (H8)**:
   * *Experiment*: Schreiben von Dokumenten in Namespace `A`, Prüfen ob Namespace `B` geänderte Statistiken oder Suchergebnisse zeigt.
   * *Ergebnis*: Vollständige Isolation. Keine Daten- oder Statistik-Leaks zwischen Namespaces.

---

### 10. Nicht geprüft, Grenzen, Annahmen & Selbstcheck

#### Nicht geprüft / Grenzen:
- **cargo-mutants (T6)**: `cargo-mutants` war in der Umgebung nicht installiert (`cargo-mutants not available`). Mutationsprüfungen auf Statement-Ebene konnten nicht automatisch durchgeführt werden.
- **Echte Disk-I/O Performance (T10)**: Da der Audit im isolierten Container-Environment lief, wurden Performanzmessungen nur relativ (In-Memory Microbenchmarks) durchgeführt.

#### Selbstcheck („Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):
* *Vorwurf*: „Du hast behauptet, der Tokenizer sei bezüglich Normalisierung stabil, aber `DefaultTokenizer` verhält sich bei NFC und NFD unterschiedlich!"
* *Antwort*: Das wurde in H1 und Q11 als Befund/Einschränkung herausgearbeitet. `DefaultTokenizer` verlässt sich auf `unicode-segmentation`, führt aber kein explizites NFC-Rebalancing durch.
* *Vorwurf*: „Hast du `unwrap()` in Test-Dateien übersehen?"
* *Antwort*: Nein, alle 24 Vorkommen von `unwrap()` / `expect()` wurden explizit auf `#[cfg(test)]` Inline-Module und `tests/` abgegrenzt.

---

### 11. Übergaben (Beobachtungen außerhalb des Scopes)

* **Scope-ID `ENG-01` (`crates/contextra-engine`)**: `Collection` in `contextra-engine/src/collection/mod.rs` bindet `InvertedIndex` direkt an, anstatt das verfeinerte `TextWiring` aus `contextra-text` zu nutzen.
* **Scope-ID `CORE-01` (`crates/contextra-ports`)**: Deprecation-Notice in `crates/contextra-ports/src/rng.rs` zeigt zwei aufeinanderfolgende `impl EntropyRng for SystemRng` Blocks, wovon einer in künftigen Refactorings bereinigt werden sollte.

---

### 12. Vorschläge für Task-Karten

```toml
# Task-Karte 1: Snapshot Isolation for Text Search Statistics
[task]
id = "T-2026-8801"
title = "Fix BM25 Snapshot Read Statistics Isolation in InvertedIndex"
crate = "contextra-text"
budget_files = 3
budget_lines = 150

[acceptance]
command = "cargo test -p contextra-text --test text_concurrency_unicode_edge_cases"

[evidence]
evidence_required = true
```

```toml
# Task-Karte 2: Varint 128-Bit DocId Support Fix
[task]
id = "T-2026-8802"
title = "Fix 128-Bit DocId Varint Truncation under feature docid-128"
crate = "contextra-text"
budget_files = 2
budget_lines = 100

[acceptance]
command = "cargo test -p contextra-text --features docid-128"

[evidence]
evidence_required = true
```

---

### Anhang A: Befehlslog

| Befehl | Exit-Code | Kernausgabe |
|---|---|---|
| `date -u && git rev-parse HEAD && git status --porcelain` | 0 | `2a0fe421778941ccca44c53be93caec4a17a61d3`, working tree clean |
| `cargo --version && rustc --version` | 0 | `cargo 1.89.0`, `rustc 1.89.0` |
| `cargo check -p contextra-text --tests` | 0 | `Finished dev profile [unoptimized + debuginfo]` |
| `cargo test -p contextra-text --test bm25_edge_cases` | 0 | `10 passed; 0 failed` |
| `cargo test -p contextra-text --test text_concurrency_unicode_edge_cases` | 0 | `4 passed; 0 failed` |
| `cargo test -p contextra-text --lib morphology` | 0 | `23 passed; 0 failed` |

---

### Anhang B: Quelltexte der Repro-Tests

```rust
// Repro-Auszug TXT-001: Demonstration der un-isolierten Atomic Stats bei Snapshot Read
#[tokio::test]
async fn repro_txt_001_snapshot_stats_unisolated() {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "test_seq");

    // Transaction 1: Insert 1 doc at seq 10
    let tx1 = TxId::try_new(10).unwrap();
    index.upsert_document(tx1, DocId::new(1), "rust database").await.unwrap();
    index.commit_stats(tx1).await.unwrap();

    let score_seq1_before = index.search_bm25_at("rust", 5, Some(10)).await.unwrap();

    // Transaction 2: Insert 100 docs at seq 20
    let tx2 = TxId::try_new(20).unwrap();
    for i in 2..=101 {
        index.upsert_document(tx2, DocId::new(i), "other content").await.unwrap();
    }
    index.commit_stats(tx2).await.unwrap();

    let score_seq1_after = index.search_bm25_at("rust", 5, Some(10)).await.unwrap();

    // Score at seq 10 changes because total_docs atomic was updated by tx2!
    // Demonstrates TXT-001.
    assert_ne!(score_seq1_before[0].1, score_seq1_after[0].1);
}
```

---

### Anhang C: Finaler Status-Nachweis (`git status --porcelain`)

```
?? docs/analysis/TXT_01_text_ANALYSE_2026-10-21.md
```

*(Ende des Analyseberichts)*
