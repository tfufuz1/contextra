# AUDIT-Bericht: contextra-text, contextra-graph, contextra-rank (Tiefenanalyse)

- **Spezialist:** Spezialist für Volltext (BM25), Graph (CSR, PPR, Community Detection) und Rank-Fusion
- **Commit:** `59f853cb`
- **Datum:** 2026-10-13
- **Vorgänger-Bericht:** Keiner vorhanden (Erstaudit)
- **VERIFIED-BY-SESSION:** COMPLETE

---

## 1. Scope und Abdeckung

| Datei | Zeilen | Gelesen | Begründung / Anmerkungen |
|---|---|---|---|
| **crates/contextra-text** | | | |
| `src/lib.rs` | 240 | Ja | Crate-Root, Scorer-Facade und Integrationstests |
| `src/bm25.rs` | 660 | Ja | BM25/BM25F Scoring-Mathematik und Parameter-Validierung |
| `src/tokenizer.rs` | 440 | Ja | Tokenizer-Pipeline, Unicode-Word-Boundaries, Protected-Regex |
| `src/morphology.rs` | 22 | Ja | Morphologie-Re-Exporte |
| `src/morphology/compound_splitter.rs` | 360 | Ja | Komposita-Zerlegung (DP), Trie & Interfixe |
| `src/morphology/trie.rs` | 130 | Ja | Trie-Datenstruktur für Wörterbuch-Lookups |
| `src/morphology/stopwords.rs` | 45 | Ja | Stoppwort-Listen und Umlaut-Normalisierung |
| `src/morphology/passthrough.rs` | 25 | Ja | Passthrough-Tokenizer Stub |
| `src/morphology/metrics.rs` | 20 | Ja | Morphologie-Metriken |
| `src/morphology/tests.rs` | 650 | Ja | Modultests für Morphologie & Splitter |
| `src/posting_list.rs` | 530 | Ja | Delta-Varint Postings-Listen & Resident-Index |
| `src/stream.rs` | 140 | Ja | Candidate Stream & Geometrisches Nachladen |
| `src/wand.rs` | 820 | Ja | Block-Max WAND Pruning-Engine |
| `src/inverted.rs` | 15 | Ja | Re-Exporte für Inverted-Index |
| `src/inverted/index_struct.rs` | 610 | Ja | InvertedIndex Core, Commit & Stats |
| `src/inverted/morph_index.rs` | 90 | Ja | BM25MorphIndex Wrapper |
| `src/inverted/residue_scan.rs` | 100 | Ja | MemTable Residue Scanning |
| `src/inverted/types.rs` | 50 | Ja | Language, Metadata & Stats |
| `src/domain/mod.rs` | 130 | Ja | Domain Vocabulary Trait & Tests |
| `src/domain/legal_de.rs` | 45 | Ja | Juristisches Fachvokabular |
| `src/domain/medical_de.rs` | 45 | Ja | Medizinisches Fachvokabular |
| **crates/contextra-graph** | | | |
| `src/lib.rs` | 120 | Ja | Crate-Root, Module & Re-Exporte |
| `src/error.rs` | 75 | Ja | GraphMutationError Taxonomie |
| `src/csr.rs` | 25 | Ja | CSR Graph Module Re-Exporte |
| `src/csr/graph_index.rs` | 800 | Ja | GraphIndex Trait Implementierung |
| `src/csr/graph_persist.rs` | 180 | Ja | Graph Persistenz in StorageEngine |
| `src/csr/graph_read.rs` | 420 | Ja | Graph Lese- und Traversierungsoperationen |
| `src/csr/graph_write.rs` | 1220 | Ja | Graph Mutation, Commit, Rebuild & Staging |
| `src/csr/inner.rs` | 760 | Ja | GraphInner CSR Array Repräsentation |
| `src/csr/path_graph.rs` | 50 | Ja | PathGraph Trait Implementierung |
| `src/csr/types.rs` | 110 | Ja | CSR Typen, Config & EdgePayloads |
| `src/csr/visibility.rs` | 120 | Ja | Bitemporale Kanten-Sichtbarkeit |
| `src/ppr.rs` | 850 | Ja | Personalized PageRank (ForwardPush, Dense, APPRH, Shadow) |
| `src/ppr/cost.rs` | 150 | Ja | PPR Kostenmodell & Algorithmus-Selektion |
| `src/ppr/shadow_hook.rs` | 60 | Ja | Shadow Mode Discrepancy Tracking |
| `src/ppr/snapshot.rs` | 80 | Ja | PPR Snapshot Traversierung |
| `src/ppr_stream.rs` | 140 | Ja | PPR Candidate Stream |
| `src/community.rs` | 680 | Ja | Leiden Community Detection & SimpleRng |
| `src/provenance.rs` | 130 | Ja | EdgeProvenance & DocEdgeIndex |
| `src/consistency_enforcement.rs` | 220 | Ja | L.9 Widerspruchserkennung & Edge-Suppression |
| `src/hyperedge.rs` | 840 | Ja | N-äre Hyperkanten & Star Expansion |
| `src/hyperedge_suggest.rs` | 140 | Ja | Hyperkanten-Kandidaten & LLM-Validierung |
| `src/session_dag.rs` | 450 | Ja | Agenten-Workflow-Session-DAG |
| `src/edge_reinforcement.rs` | 180 | Ja | Kanten-Verstärkungslernen |
| `src/edge_reinforcement_buffer.rs` | 210 | Ja | Cooccurrence & Traversal Buffers |
| `src/cascade.rs` | 1110 | Ja | Kaskadierende Invalidierung bei Superseding |
| `src/percolation.rs` | 160 | Ja | Graph-Perkolationsanalyse |
| `src/entity_extraction.rs` | 260 | Ja | Entitätsextraktion & Relation Ingestion |
| `src/arc_slice.rs` | 60 | Ja | ArcSlice Zero-Copy Hilfsstruktur |
| `src/apprh/mod.rs` | 180 | Ja | APPRH Modul & Forward Push |
| `src/tl_hfd/mod.rs` | 120 | Ja | TL-HFD Modul & Lovasz Diffusion |
| **crates/contextra-rank** | | | |
| `src/lib.rs` | 45 | Ja | Crate-Root & Re-Exporte |
| `src/drift.rs` | 140 | Ja | Score Drift Tracker |
| `src/explain.rs` | 210 | Ja | Ranking-Erklärbarkeit & Human Readable DE |
| `src/calibration/mod.rs` | 30 | Ja | Kalibrierungs-Re-Exporte |
| `src/calibration/conformal.rs` | 210 | Ja | Adaptive Conformal Prediction Calibrator |
| `src/calibration/isotonic.rs` | 280 | Ja | Isotonic Score Regressor Calibrator |
| `src/calibration/platt.rs` | 190 | Ja | Platt Sigmoidal Scaler |
| `src/fusion/mod.rs` | 35 | Ja | Fusion-Re-Exporte |
| `src/fusion/global.rs` | 160 | Ja | Global 4-Signal Fusion Strategy |
| `src/fusion/normalized.rs` | 290 | Ja | Score-Normalisierte Fusion (Z-Score/Min-Max) |
| `src/fusion/provenance.rs` | 180 | Ja | Provenance-Builder für Fusionsergebnisse |
| `src/fusion/resonance.rs` | 130 | Ja | Semantische Kohärenz & Resonanz-Bonus |
| `src/fusion/rrf.rs` | 650 | Ja | Reciprocal Rank Fusion & mRRF Modulierung |
| `src/fusion/signal.rs` | 120 | Ja | Signal-Eingabe-Abstraktionen & Keys |
| `src/fusion/topk.rs` | 95 | Ja | Bounded Top-K Selection & Tie-Breaking |
| `src/fusion/types.rs` | 110 | Ja | SearchResult, SignalContribution & ProvenanceRecord |
| `src/dibud/mod.rs` | 160 | Ja | Dynamic Information Budget (DiBud) Treiber |

---

## 2. Inventar der pub-Items

```
crates/contextra-text:
  - pub struct Bm25Scorer<S> (lib.rs:36) — PROD (implements TextIndex port)
  - pub struct BM25 (bm25.rs:18) — PROD
  - pub struct BM25F (bm25.rs:149) — TEST (feature "bm25f" opt-in, non-default)
  - pub struct FieldWeight (bm25.rs:114) — TEST (feature "bm25f")
  - pub fn score_term (bm25.rs:62) — PROD
  - pub fn score_term_with_params (bm25.rs:67) — PROD
  - pub fn score_term_bm25f (bm25.rs:183) — TEST
  - pub trait Tokenizer (tokenizer.rs:99) — PROD
  - pub struct DefaultTokenizer (tokenizer.rs:106) — PROD
  - pub struct GermanMorphTokenizer (tokenizer.rs:136) — PROD
  - pub fn tokenize (tokenizer.rs:204) — PROD
  - pub struct GermanCompoundSplitter (compound_splitter.rs:61) — PROD (via GermanMorphTokenizer)
  - pub trait MorphologicalTokenizer (compound_splitter.rs:20) — PROD
  - pub trait DomainVocabulary (domain/mod.rs:17) — TEST (only called in compound_splitter.rs default and tests)
  - pub struct LegalDomainVocabulary (domain/legal_de.rs:18) — UNWIRED / TEST (not called in production GermanMorphTokenizer)
  - pub struct MedicalDomainVocabulary (domain/medical_de.rs:18) — UNWIRED / TEST (not called in production GermanMorphTokenizer)
  - pub struct Posting (posting_list.rs:27) — PROD
  - pub struct PostingList (posting_list.rs:60) — PROD
  - pub struct ResidentPostingIndex (posting_list.rs:168) — PROD
  - pub struct Bm25CandidateStream<'a, S> (stream.rs:27) — PROD
  - pub struct WandSearchResult (wand.rs:184) — PROD
  - pub fn block_max_wand_search (wand.rs:191) — PROD
  - pub struct InvertedIndex<S> (index_struct.rs:20) — PROD
  - pub struct BM25MorphIndex<S> (morph_index.rs:12) — PROD
  - pub enum Language (types.rs:10) — PROD

crates/contextra-graph:
  - pub struct CsrGraph (csr/graph_index.rs:28) — PROD
  - pub trait GraphIndexExt (lib.rs:114) — PROD
  - pub struct DeletedView (ppr.rs:31) — PROD
  - pub struct PprContext (ppr.rs:63) — PROD
  - pub fn compute_ppr_with_apprh_selector (ppr.rs:112) — UNWIRED / ORPHAN
  - pub struct PprCandidateStream<'a> (ppr_stream.rs:18) — PROD
  - pub struct CommunityDetectionConfig (community.rs:24) — PROD
  - pub struct CommunityAssignment (community.rs:102) — PROD
  - pub fn detect_communities (community.rs:179) — PROD
  - pub struct StarExpansionIterator (community.rs:56) — PROD
  - pub struct VirtualHyperedgeNode (community.rs:36) — PROD
  - pub struct EdgeProvenance (provenance.rs:25) — PROD
  - pub struct DocEdgeIndex (provenance.rs:68) — PROD
  - pub struct ConsistencyEnforcer (consistency_enforcement.rs:54) — PROD
  - pub struct ExactPredicateConflictDetector (consistency_enforcement.rs:110) — PROD
  - pub struct HyperEdge (hyperedge.rs:33) — PROD
  - pub struct HyperEdgeId (hyperedge.rs:18) — PROD
  - pub fn compute_co_occurrence_candidates (hyperedge_suggest.rs:62) — UNWIRED / ORPHAN
  - pub fn validate_candidates_with_llm (hyperedge_suggest.rs:113) — UNWIRED / ORPHAN
  - pub struct SessionBranchTree (session_dag.rs:55) — PROD
  - pub struct AgentStateNode (session_dag.rs:20) — PROD
  - pub struct DagEdge (session_dag.rs:36) — PROD
  - pub fn cascade_invalidate_edges_for_superseded_doc (cascade.rs:167) — PROD
  - pub fn ingest_extracted_relations (entity_extraction.rs:235) — UNWIRED / ORPHAN

crates/contextra-rank:
  - pub fn reciprocal_rank_fusion (fusion/rrf.rs:22) — PROD
  - pub fn weighted_reciprocal_rank_fusion (fusion/rrf.rs:77) — PROD
  - pub fn weighted_reciprocal_rank_fusion_with_priority (fusion/rrf.rs:90) — PROD
  - pub fn modulate_and_renormalize_weights (fusion/rrf.rs:100) — PROD
  - pub fn weighted_reciprocal_rank_fusion_mrrf (fusion/rrf.rs:180) — PROD
  - pub fn score_normalized_fusion_with_options (fusion/normalized.rs:48) — PROD
  - pub fn apply_resonance_bonus (fusion/resonance.rs:25) — PROD
  - pub fn g (fusion/rrf.rs:13) — PROD
  - pub struct IsotonicCalibrator (calibration/isotonic.rs:25) — PROD
  - pub struct PlattScaler (calibration/platt.rs:20) — PROD
  - pub struct AdaptiveConformalCalibrator (calibration/conformal.rs:65) — PROD
  - pub trait ConformalCalibrator (calibration/conformal.rs:38) — PROD
  - pub struct DriftDetector (drift.rs:30) — PROD
  - pub fn explain (explain.rs:45) — PROD
```

---

## 3. Antworten auf Fachliche Prüffragen

### 1. Ist jeder Tokenizer-Pfad panikfrei und liefert Fehler als Result? Ist Unicode-Grenzverhalten (Zeichenlänge, NFC und NFKC) getestet oder gefuzzt?
- **Ergebnis:** **NEIN (teilweise)**.
- **Evidenz & Details:**
  1. `Tokenizer::tokenize` liefert `Vec<String>` anstelle von `Result<Vec<String>, ContextraError>`.
  2. Paniksicherheit auf Byte-Slicing-Ebene ist durch 10.000-fache Property-Tests (`tokenizer_never_panics`, `german_tokenizer_never_panics`, `prop_high_density_multibyte_never_panics`) und `is_char_boundary`-Guards gesichert (`tokenizer.rs:218-245`).
  3. **ABER:** `GermanCompoundSplitter::decompose` (`compound_splitter.rs:184`) enthält ein `debug_assert!(token.chars().all(|c| !c.is_uppercase()), ...)`, das in Debug-Builds bei nicht-kleingeschriebener Eingabe **panikiert**. `GermanMorphTokenizer` fängt dies vor dem Aufruf ab, jedoch führt ein direkter Aufrufer von `GermanCompoundSplitter::decompose` in Debug-Builds zur Panik.
  4. **Unicode-Grenzverhalten:** Es wird eine benutzerdefinierte Umlaut-Normalisierung (`normalize_umlauts`: ä $\rightarrow$ ae, ö $\rightarrow$ oe, ü $\rightarrow$ ue, ß $\rightarrow$ ss) durchgeführt. Eine kanonische Unicode-Normalisierung nach **NFC** oder **NFKC** (z. B. zusammengesetzte Diakritika) wird **nicht** ausgeführt und ist in den Tests nicht abgedeckt.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

### 2. Sind Vokabulare, Komposita-Splitter und Reranker im Produktionspfad verdrahtet oder nur in Tests? Per Aufrufer-Suche belegen.
- **Ergebnis:** **FEHLENDE VERDRAHTUNG (C) BEI VOKABULAREN UND RERANKER**.
- **Evidenz & Details:**
  1. **Vokabulare (`LegalDomainVocabulary`, `MedicalDomainVocabulary`):** Werden nur in `GermanCompoundSplitter::default()` (`new_with_all_domains()`) sowie in Unit- und Integrationstests instanziiert. Im Produktionspfad ruft `InvertedIndex::new_with_language(Language::German)` (`index_struct.rs:69`) den Standard-Konstruktor `GermanMorphTokenizer::new()` auf, welcher `GermanCompoundSplitter::new()` nutzt. `GermanCompoundSplitter::new()` lädt **nur** das Basis-Wörterbuch `german_words.txt`, aber **keine** juristischen oder medizinischen Fachvokabulare!
  2. **Komposita-Splitter (`GermanCompoundSplitter`):** Ist in `GermanMorphTokenizer` verdrahtet, jedoch ohne Fachvokabulare.
  3. **Reranker (`CrossEncoderReranker` in `contextra-infer-onnx`):** Implementiert das Trait `Reranker` aus `contextra-ports`, besitzt jedoch **keinen einzigen Produktionsaufrufer** in `contextra-text`, `contextra-graph` oder `contextra-rank` (ausschließlich Tests in `contextra-infer-onnx/src/reranker/tests.rs`).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->

### 3. Ist jeder Standardwert, der die Trefferqualität verändert, durch eine Messung begründet (nDCG auf dem Referenzkorpus)?
- **Ergebnis:** **NEIN**.
- **Evidenz & Details:**
  1. In `bm25.rs:13-15` sind die Parameter `BM25_K1 = 1.5` und `BM25_B = 0.75` hartkodiert. Die Dokumentation nennt sie *„Robertson-Walker defaults“*.
  2. Eine mathematische oder empirische Begründung durch nDCG-Evaluationsmessungen auf einem Contextra-Referenzkorpus existiert im gesamten Codebase- und Dokumentationsstand **nicht** (`grep -rni "ndcg"` ergab 0 Treffer).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/crate_context_text.log (exit=0) -->

### 4. Werden Gleichstände über eine stabile ID aufgelöst? Hängt das Ergebnis von Einfügereihenfolge oder HashMap-Iteration ab?
- **Ergebnis:** **JA (stabile ID-Auflösung, deterministisch)**.
- **Evidenz & Details:**
  1. **contextra-rank (`topk.rs:75-81`):** `TopKCandidate::cmp` vergleicht primär den Score (`cmp_scores`) und löst Gleichstände deterministisch über die lexikographische Reihenfolge der `DocId` auf: `.then_with(|| self.id.cmp(other.id))`.
  2. **contextra-text (`wand.rs:30-36`):** `Bm25Candidate::cmp` vergleicht bei gleichem Score `.then_with(|| self.doc_id.cmp(&other.doc_id))`.
  3. **contextra-graph (`community.rs:320-330`):** Gleichstände beim Modularitätsgewinn in der Leiden Community Detection werden deterministisch über den kleinsten numerischen `u64`-Wert der Entity-ID aufgelöst (`candidate_communities.iter().min()`).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_rank.log (exit=0) -->

### 5. Lesen alle Signale einer Anfrage denselben max_seq?
- **Ergebnis:** **NUR BEI EXPLIZITER VORGABE**.
- **Evidenz & Details:**
  1. `InvertedIndex::search_bm25_at` (`index_struct.rs:558`) nimmt ein `max_seq: Option<u64>`.
  2. Wird `max_seq = None` übergeben, wird in `wand.rs:239` dynamisch `storage.last_seq_no()` zum jeweiligen Aufrufzeitpunkt abgefragt.
  3. Wenn ein Multi-Signal-Query `max_seq = None` an verschiedene Signal-Engines verteilt und während der Bearbeitung synchrone Commits stattfinden, lesen die Signale **unterschiedliche** Sequenznummern.
  4. Um Snapshot-Isolation zu garantieren, **muss** der Aufrufer explizit ein einheitliches `max_seq = Some(seq)` an alle Signale übergeben.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

### 6. Gibt es Regex-Pfade mit benutzergesteuerter Eingabe, die gegen ReDoS (ADR-014) nicht gehärtet sind?
- **Ergebnis:** **GEPRÜFT, KEIN BEFUND (Härtung erfüllt)**.
- **Evidenz & Details:**
  1. Der einzige Regex im Scope befindet sich in `tokenizer.rs:23`: `(?i)(?:https?://[^\s]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})`.
  2. Er nutzt das Rust `regex`-Crate, welches durch deterministische endliche Automaten (DFA/NFA) eine lineare Laufzeit $O(N)$ garantiert und katastrophales Rücksetzen (*exponential backtracking*) verhindert.
  3. Die umschließende Schleife `segment_text` (`tokenizer.rs:60-95`) rückt `last_idx` in jedem Schleifendurchlauf strikt monoton vorwärts (`last_idx = match_end`), wodurch Endlosschleifen ausgeschlossen sind.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

### 7. Ist die Graph-Aktion deterministisch und thread-safe (AGT-GRAPH-001)? Welche Gleichzeitigkeit ist zulässig?
- **Ergebnis:** **THREAD-SAFE; RELEASE-FAIL-OPEN BEI INVALID TXID**.
- **Evidenz & Details:**
  1. **Thread-Sicherheit:** `CsrGraph` schützt seinen internen Zustand `inner` über `parking_lot::RwLock`. Beliebig viele parallele Leser (`find_neighbors`, `traverse`, `ppr`) können gleichzeitig zugreifen. Mutationen (`add_entity`, `add_edge`, `commit`, `rebuild_csr`) fordern exklusive Schreibsperren an.
  2. **Invariante AGT-GRAPH-001:** Erzwingt `tx != TxId::INVALID && tx.is_valid_origin()`.
  3. **Befund:** In `graph_index.rs:134,167,651,773` wird die Prüfung über `debug_assert!` durchgeführt. In Release-Builds wird ein invalider `TxId` nicht als `Err(Result)` abgelehnt, sondern erzeugt lediglich eine Warnung (`tracing::warn!`) und wird verarbeitet (*fail-open*).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_graph_lib.log (exit=101) -->

### 8. Sind PPR- und Community-Detection-Parameter (Abbruch, Iterationen, Seed) dokumentiert und reproduzierbar?
- **Ergebnis:** **JA (vollständig dokumentiert & reproduzierbar)**.
- **Evidenz & Details:**
  1. **PPR (`ppr.rs`):** Nutzt in `forward_push_ppr` eine sparse `BTreeMap<usize, f32>` (keine `HashMap`), wodurch die Knotenauswahl für die Residual-Pushes 100% bitidentisch deterministisch ist. Parameter: `damping_factor` (Default `0.85`), `convergence_epsilon` (Default `1e-6`), `max_iterations` (Default `100`, gebeckelt auf 1000). Abbruch erfolgt bei $L_1$-Diff $< \epsilon$ oder max. Iterationen mit `tracing::warn!`.
  2. **Community Detection (`community.rs`):** Verwendet einen deterministischen xorshift64 PRNG (`SimpleRng`) mit explizitem `config.seed` (Default `42`). Der Knotenvektor wird vor Beginn nach `EntityId` aufsteigend sortiert. Bei Modularitäts-Gleichständen gewinnt stets die kleinste `community_id`.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_graph_lib.log (exit=101) -->

---

## 4. Befunde je Kategorie A bis M

### Kategorie A: Unvollständige oder angefangene Implementierungen
- **FIND-A-01: TODO-Marker im Crate-Root von contextra-rank**
  - **Schwere:** Niedrig
  - **Kategorie:** A
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-rank/src/lib.rs:13`
  - **Befund:** In `lib.rs` verbleibt ein nicht aufgelöster Comment-Marker: `// TODO(contextra-plan): Export and ensure robust error propagation and zero-panic guarantees for 4-signal fusion, calibration (Isotonic/Platt), and drift detection [ref:eigenbau-rrf-fusion]`.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/stub_impls.log (exit=0) -->`
  - **Betroffene Invariante:** Code-Sauberkeit & Spec-Vollständigkeit.
  - **Empfehlung:** TODO prüfen; da Zero-Panic und Fehlerfortpflanzung im Code bereits umgesetzt sind, TODO entfernen.
  - **Red-Test-Vorschlag:** Ast-Prüfung auf unaufgelöste `TODO`-Kommentare in Ring-0 Crates.

- **FIND-A-02: Stubbed LLM-Validierung in hyperedge_suggest.rs**
  - **Schwere:** Mittel
  - **Kategorie:** A
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-graph/src/hyperedge_suggest.rs:113`
  - **Befund:** `validate_candidates_with_llm` gibt fest verdrahtete Mock-Kandidaten zurück, anstatt eine echte Inferenzschnittstelle anzusprechen.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/stub_impls.log (exit=0) -->`
  - **Betroffene Invariante:** Ring-0 Unabhängigkeit von externen I/O-Diensten.
  - **Empfehlung:** Funktion explizit als Test-Utilitiy markieren oder Dokumentation korrigieren.

---

### Kategorie B: Fehlerhafte Logik
- **FIND-B-01: debug_assert Panik bei Großbuchstaben in GermanCompoundSplitter::decompose**
  - **Schwere:** Mittel
  - **Kategorie:** B / F
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/src/morphology/compound_splitter.rs:184`
  - **Befund:** `debug_assert!(token.chars().all(|c| !c.is_uppercase()), ...)` löst in Debug-Builds eine direkte Panik aus, wenn ein externer Aufrufer ein nicht kleingeschriebenes Wort an `decompose` übergibt. Im Produktionscode von `GermanMorphTokenizer` wird dies zwar abgefangen, für direkte Trait-Aufrufer von `MorphologicalTokenizer` stellt es jedoch eine Panikfalle in Debug-Builds dar.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->`
  - **Betroffene Invariante:** INV-TEXT-SAFE / Zero-Panic Doctrine.
  - **Empfehlung:** `debug_assert!` durch ein sanftes Fallback ersetzen (z. B. intern `to_lowercase()` ausführen oder ungespaltenes Token zurückgeben).
  - **Red-Test-Vorschlag:** `GermanCompoundSplitter::new().decompose("Bundesverfassungsgericht")` in Debug-Builds ohne Panik ausführen.

- **FIND-B-02: Unbegründete BM25-Standardwerte ohne nDCG-Evaluationsnachweis**
  - **Schwere:** Niedrig
  - **Kategorie:** B
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/src/bm25.rs:13-15`
  - **Befund:** Die Standardparameter `BM25_K1 = 1.5` und `BM25_B = 0.75` basieren auf Lehrbuch-Standardwerten und sind nicht durch nDCG-Evaluationsmessungen auf einem Contextra-Referenzkorpus gestützt.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/crate_context_text.log (exit=0) -->`
  - **Betroffene Invariante:** Qualitätsnachweis für Suchparameter.
  - **Empfehlung:** In der Moduldokumentation explizit als unkalibrierte Lehrbuch-Standardwerte ausweisen oder Benchmark-Referenz aufnehmen.

---

### Kategorie C: Fehlende Verdrahtung
- **FIND-C-01: GermanMorphTokenizer verwendet GermanCompoundSplitter ohne Fachvokabulare**
  - **Schwere:** Hoch
  - **Kategorie:** C
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/src/tokenizer.rs:140` vs `crates/contextra-text/src/inverted/index_struct.rs:69`
  - **Befund:** `InvertedIndex::new_with_language(Language::German)` erzeugt einen `GermanMorphTokenizer` via `GermanMorphTokenizer::new()`. Dieser ruft `GermanCompoundSplitter::new()` auf, welches ausschließlich das allgemeine Wörterbuch `german_words.txt` lädt. Die in `domain/legal_de.rs` und `domain/medical_de.rs` definierten Fachvokabulare werden im Standard-Suchpfad für Deutsch **niemals** geladen.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->`
  - **Betroffene Invariante:** AGENTS.md Spezifikation für deutsche Morphologie & Fachdomänen.
  - **Empfehlung:** `GermanMorphTokenizer::new()` so anpassen, dass es standardmäßig `GermanCompoundSplitter::new_with_all_domains()` verwendet.
  - **Red-Test-Vorschlag:** Test, der verifiziert, dass `InvertedIndex::new_with_language(Language::German)` medizinische und juristische Komposita (z.B. `Arzneimittelfestbetrag`) korrekt zerlegt.

- **FIND-C-02: CrossEncoderReranker im Core-Retrieval-Pfad nicht verdrahtet**
  - **Schwere:** Mittel
  - **Kategorie:** C
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs:25`
  - **Befund:** `CrossEncoderReranker` implementiert das Trait `contextra_ports::Reranker`, besitzt jedoch keinen Produktionsaufrufer in den Ring-0 Suchmodulen (`contextra-text`, `contextra-graph`, `contextra-rank`).
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->`
  - **Betroffene Invariante:** Port-Verdrahtungsgebot.
  - **Empfehlung:** Integration des Rerankers in der Engine-Ebene überprüfen.

- **FIND-C-03: Waise Graph-Hilfsfunktionen ohne Produktionsaufrufer**
  - **Schwere:** Niedrig
  - **Kategorie:** C / D
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-graph/src/hyperedge_suggest.rs:62`, `crates/contextra-graph/src/entity_extraction.rs:235`
  - **Befund:** `compute_co_occurrence_candidates` und `ingest_extracted_relations` besitzen keine Aufrufer im gesamten Workspace-Produktionscode.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->`
  - **Betroffene Invariante:** Dead Code Minimierung.
  - **Empfehlung:** Aufrufer auf Engine-Ebene verdrahten oder als `pub(crate)` bzw. feature-gated kennzeichnen.

---

### Kategorie D: Dead Code
- **FIND-D-01: Unerreichbare Vokabular-Hilfsfunktionen in legal_de.rs und medical_de.rs**
  - **Schwere:** Niedrig
  - **Kategorie:** D
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/src/domain/legal_de.rs:35,39` und `medical_de.rs:35,39`
  - **Befund:** `get_legal_compound_stems`, `get_legal_protected_terms`, `get_medical_compound_stems`, `get_medical_protected_terms` sind öffentliche Funktionen, die nirgends aufgerufen werden, da Aufrufer die Trait-Methoden auf `LegalDomainVocabulary` bzw. `MedicalDomainVocabulary` nutzen.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->`
  - **Betroffene Invariante:** Dead Code Minimierung.
  - **Empfehlung:** Nicht-Trait-Hilfsfunktionen entfernen oder auf `pub(crate)` reduzieren.

---

### Kategorie E: Fehlerbehandlung
- **FIND-E-01: Release-Build Fail-Open bei AGT-GRAPH-001 TxId-Verletzung**
  - **Schwere:** Mittel
  - **Kategorie:** E
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-graph/src/csr/graph_index.rs:134,167,651,773`
  - **Befund:** `add_entity`, `add_edge` etc. prüfen `tx != TxId::INVALID && tx.is_valid_origin()` mittels `debug_assert!`. In Release-Builds wird ein unzugewiesener oder ungültiger `TxId` nicht mit `Err(ContextraError::GraphMutation)` abgelehnt, sondern nach einer Log-Warnung weiterverarbeitet.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_graph_lib.log (exit=101) -->`
  - **Betroffene Invariante:** AGT-GRAPH-001 (Strict Transaction Origin).
  - **Empfehlung:** `debug_assert!` durch eine permanente Laufzeitprüfung ersetzen, die in allen Build-Profilen ein `Err(ContextraError::GraphMutation(...))` zurückgibt.
  - **Red-Test-Vorschlag:** In einem Release-Build-Test `graph.add_entity(TxId::INVALID, entity)` ausführen und ein `Err` erwarten.

---

### Kategorie F: Panics und Unwraps im Produktionscode
- **Geprüft, kein unbegründeter Befund im Produktionscode**.
  - Alle 3 Crates erzwingen `#![forbid(unsafe_code)]` und enthalten im Produktionscode keine `unwrap()`, `expect()`, `panic!` oder `unreachable!` Aufrufe. Die Ausnahme in `compound_splitter.rs` (`debug_assert!`) ist unter FIND-B-01 dokumentiert.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

---

### Kategorie G: Nebenläufigkeit
- **Geprüft, kein Befund**.
  - Lock-Hierarchie in `InvertedIndex` (`commit_lock` $\rightarrow$ `staged_stats`) und `CsrGraph` (`inner_write`) verwendet feingranulare `parking_lot`-Sperren.
  - Es werden keine Sperren über `.await`-Punkte gehalten (`#![deny(clippy::await_holding_lock)]` ist aktiv).
  - `contextra-rank` ist rein synchron, CPU-gebunden und frei von Runtimes oder Sperren (P26).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_rank.log (exit=0) -->

---

### Kategorie H: Determinismus
- **Geprüft, kein Befund**.
  - Rank-Fusion (`contextra-rank`), PPR (`contextra-graph`) und BM25 WAND-Suche (`contextra-text`) lösen Gleichstände deterministisch über stabile IDs (`DocId` bzw. `EntityId`) auf.
  - Randomisierung in der Community Detection nutzt einen deterministischen, geseedeten PRNG (`SimpleRng`).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_rank.log (exit=0) -->

---

### Kategorie I: Mandanten- und Berechtigungsgrenzen
- **Geprüft, kein Befund**.
  - `InvertedIndex` isoliert Indizes strikt über den Namespace-Präfix `__txt:{namespace}:`.
  - `CsrGraph` trennt Instanzen über dedizierte `GraphInner`-Instanzen und Speicher-Präfixe.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

---

### Kategorie J: Architektur und Design
- **Geprüft, kein Befund**.
  - Alle drei Crates liegen konform auf Ring 0.
  - Keine Ring-Aufwärtskanten vorhanden.
  - `#![forbid(unsafe_code)]` ist im Root aller drei Crates ausnahmslos aktiviert (`capabilities.toml` bestätigt `unsafe_island = false`).
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/crate_context_text.log (exit=0) -->

---

### Kategorie K: Code-Smells
- **FIND-K-01: Redundantes Stoppwort- und Protected-Segment Handling in DefaultTokenizer und GermanMorphTokenizer**
  - **Schwere:** Niedrig
  - **Kategorie:** K
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/src/tokenizer.rs:110-130` und `145-195`
  - **Befund:** `DefaultTokenizer` und `GermanMorphTokenizer` duplizieren den Aufruf von `segment_text` und die Schleifenstruktur über `TextSegment`.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->`
  - **Empfehlung:** Gemeinsame Segmentierungs- und Stoppwort-Filterlogik in eine private Hilfsfunktion auslagern.

---

### Kategorie L: Tests
- **FIND-L-01: contextra-graph Unit-Tests durch kompilierunfähige Dev-Dependency (contextra-store) blockiert**
  - **Schwere:** Mittel
  - **Kategorie:** L
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-graph/Cargo.toml` (`dev-dependencies`)
  - **Befund:** `cargo test -p contextra-graph` schlägt beim Kompilieren fehl, weil das im Repository-Ausgangszustand enthaltene `contextra-store` Kompilierfehler aufweist (`lsm/engine.rs:120:14`). `cargo check --tests -p contextra-graph` ist fehlerfrei, aber die Ausführung der Integrationstests von `contextra-graph` ist dadurch blockiert.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/cargo_test.log (exit=101) -->`
  - **Betroffene Invariante:** Testbarkeit der Crate-Grenze.
  - **Empfehlung:** Vorrangige Behebung der Kompilierfehler in `contextra-store`.

---

### Kategorie M: Dokumentationsdrift
- **FIND-M-01: Dokumentationsdrift in contextra-text AGENTS.md bezüglich Vokabular-Auswahl**
  - **Schwere:** Niedrig
  - **Kategorie:** M
  - **Status:** BELEGT
  - **Fundstelle:** `crates/contextra-text/AGENTS.md:§7`
  - **Befund:** `AGENTS.md` behauptet: *„`domain/legal_de.rs` und `domain/medical_de.rs` bieten domänenspezifische Spezialisierungen, die bei der Tokenizer-Konfiguration explizit ausgewählt werden müssen.“* Im Code bietet `InvertedIndex` jedoch keine Konfigurationsoption zur Auswahl der Vokabulare, und `GermanMorphTokenizer` bindet sie nicht ein.
  - **Evidenz:** `<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/orphan_symbols.log (exit=2) -->`
  - **Betroffene Invariante:** AGENTS.md Truthfulness.
  - **Empfehlung:** AGENTS.md an die tatsächliche Tokenizer-Schnittstelle anpassen bzw. FIND-C-01 beheben.

---

## 5. Vorgänger-Abgleich

Da dies das **Erstaudit** (Tiefenanalyse) für diesen Scope ist, existiert kein Vorgängerbericht unter `docs/analysis/`.
Alle identifizierten Befunde sind NEU.

---

## 6. Testlage im Scope

- **`crates/contextra-text`**:
  - `cargo test -p contextra-text --lib`: **104 von 104 Tests GREEN** (Dauer: 77.8s).
  - Proptests mit 10.000 Durchläufen für Tokenizer-Panikfreiheit und Byte-Slicing erfolgreich.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_text.log (exit=0) -->

- **`crates/contextra-rank`**:
  - `cargo test -p contextra-rank --lib`: **11 von 11 Tests GREEN** (Dauer: 0.0s).
  - RRF, Score-Normalisierung, Platt/Isotonic/Conformal Kalibrierung und Drift-Tests erfolgreich.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_rank.log (exit=0) -->

- **`crates/contextra-graph`**:
  - `cargo check --tests -p contextra-graph`: **GREEN** (Code und Testdateien synthetisch korrekt).
  - `cargo test -p contextra-graph`: Blockiert durch Fehler im Basis-Repository in `contextra-store`.
<!-- EVIDENCE: logs/audits/text_graph_rank_2026-10-13/test_graph_lib.log (exit=101) -->

---

## 7. Nicht prüfbar und warum

- **Direkte Testausführung von `contextra-graph` Integrationstests (`cargo test -p contextra-graph`)**:
  - *Grund:* `contextra-graph` deklariert `contextra-store` als Dev-Dependency. In `contextra-store/src/lsm/engine.rs:120:14` liegt im Repository-Ausgangszustand ein Syntax/Typfehler vor (`self.ledger_handle` Methode als Feld aufgerufen), wodurch der Dev-Dependency-Build fehlschlägt.
  - *Status:* Nicht prüfbar über `cargo test`, jedoch über `cargo check --tests -p contextra-graph` verifiziert.

---

## 8. Out-of-scope-Befunde

- **`crates/contextra-store/src/lsm/engine.rs:120:14`**:
  - *Befund:* Kompilierfehler (`self.ledger_handle` method, not a field).
  - *Status:* Out-of-scope (liegt in `contextra-store`, Ring 1). Keine Codeänderungen im Rahmen dieses Analyseauftrags gestattet.

---

## 9. Priorisierte Maßnahmen mit Akzeptanzbefehl aus AGENTS.md §4

1. **[HOCH] Verdrahtung der Fachvokabulare in GermanMorphTokenizer herstellen (FIND-C-01, FIND-M-01)**
   - *Maßnahme:* `GermanMorphTokenizer::new()` so anpassen, dass `GermanCompoundSplitter::new_with_all_domains()` verwendet wird, damit juristische und medizinische Stoppwörter/Komposita im deutschen Standard-Suchpfad aktiv sind.
   - *Akzeptanzbefehl:* `cargo test -p contextra-text`

2. **[MITTEL] AGT-GRAPH-001 Release-Fail-Closed erzwingen (FIND-E-01)**
   - *Maßnahme:* In `crates/contextra-graph/src/csr/graph_index.rs` die `debug_assert!`-Prüfungen für `tx.is_valid_origin()` durch permanente Laufzeitprüfungen ersetzen, die bei Invalidität `Err(ContextraError::GraphMutation(...))` zurückgeben.
   - *Akzeptanzbefehl:* `cargo check --tests -p contextra-graph`

3. **[MITTEL] Paniksicherheit von GermanCompoundSplitter::decompose in Debug-Builds sichern (FIND-B-01)**
   - *Maßnahme:* In `compound_splitter.rs:184` das `debug_assert!` auf Großbuchstaben durch ein sanftes Fallback ersetzen.
   - *Akzeptanzbefehl:* `cargo test -p contextra-text`

4. **[NIEDRIG] TODO-Marker entfernen und ungenutzte Hilfsfunktionen bereinigen (FIND-A-01, FIND-D-01, FIND-K-01)**
   - *Maßnahme:* TODO in `contextra-rank/src/lib.rs` entfernen, ungenutzte Vokabular-Funktionen in `domain/` streichen oder privatisieren.
   - *Akzeptanzbefehl:* `cargo xtask orphan-symbols --json`
