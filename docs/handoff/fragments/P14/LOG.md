# Protokoll P14: Evaluations-Harness

## Start-Check (vor der ersten Änderung)
- [V] Überprüfung der Crate-Baufähigkeit vor Beginn der Arbeit:
  - `contextra-crypto`: Fällt aus wegen `unexpected cfg condition value: serde` in `src/secret.rs`.
  - `contextra-graph`: Fällt aus wegen fehlender Importe von `contextra_ports` und `contextra_types` (>150 Fehler).
  - `contextra-vector`: Fällt aus wegen doppelter Definitionen (`remove`, `rebuild_sync`) und fehlender Trait-Implementierung `zeroize`.
  - `contextra-infer-candle`: Fällt aus wegen `thiserror`.
  - `contextra-db` & `contextra`: Fallen aus wegen vorgenannter Abhängigkeiten.
  - Baufähig vorab: `contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-text`, `contextra-mvcc`, `contextra-sys`, `contextra-durable-fs`, `contextra-testkit`, `contextra-erasure`, `contextra-ingest`, `contextra-router`.
- Fremdfehler in `OPEN_ISSUES.md` dokumentiert.

## WP-110: Bench reaktivieren und historische Läufe markieren
- [V] Veraltete Ergebnisdateien als `historical` gekennzeichnet:
  - `benchmarks/results/results.json`
  - `benchmarks/results/ann_results.json`
  - `benchmarks/results/scale_100k_2026-09-30.json`
  - `benchmarks/results/criterion-baseline.json`
  - `benchmarks/results/summary.md`
  - `benchmarks/contextra-bench/baseline_recall.json`
- [V] Analyse Fassade `contextra`:
  - `crates/contextra/src/lib.rs` re-exportiert `Contextra`, `ContextraConfig`, `SearchResult` etc., aber noch nicht `SearchStrategy`, `SignalWeights`, `HybridQueryBuilder`.
  - Eintrag in `PENDING_API_CHANGES.md` erstellt.
- [V] Status der Kern-Crates in `OPEN_ISSUES.md` dokumentiert.

## WP-111: Datensätze bereitstellen
- [V] Download-Skript `benchmarks/data/fetch_beir.sh` für öffentliche BEIR-Datensätze (SciFact, NFCorpus) mit Lizenzvermerken (CC-BY-4.0 / CC-BY-SA-4.0) erstellt und als ausführbar markiert. Keine Fremddaten ins Git-Repo eingecheckt.
- [V] Eigener deutscher Mini-Korpus unter `benchmarks/data/de_mini/` erstellt:
  - `corpus.jsonl`: 200 Dokumente aus 5 Fachthemenbereichen (Datenbank, Vektor, Text, Fusion, Graph).
  - `queries.jsonl`: 50 Testabfragen.
  - `qrels.tsv`: 400 Relevanzurteile (Score 1 und 2).
  - `README.md`: Lizenz CC0 / MIT dokumentiert.
- Prüfbefehl: `wc -l benchmarks/data/de_mini/*` → grün (200 Docs, 50 Queries, 401 Zeilen qrels inkl. Header).

## WP-112: Metriken implementieren und testen
- [V] Eigene, kleine Metrik-Implementierung in `benchmarks/contextra-bench/src/metrics.rs` erstellt:
  - `calculate_recall_at_k`: Exakte Relevanzquote mit Schwellwertprüfung und Duplikatelimination.
  - `calculate_mrr_at_k`: 1-basierter Kehrwert des ersten relevanten Rangs in Top-k.
  - `calculate_ndcg_at_k`: Abgestufte Relevanzgewinne mit idealem DCG-Vergleich.
  - `calculate_latency_percentiles`: p50, p95, p99 Perzentile sortierter Latenzzeitreihen.
  - `read_process_rss_bytes`: VmRSS Speicherverbrauchs-Messung via `/proc/self/status`.
- [V] Unit-Tests gegen unabhängig handgerechnete Zahlenwerte implementiert.
- Prüfbefehl: `test_latency_percentiles_hand_calculated`, `test_mrr_at_k_hand_calculated`, `test_recall_at_k_hand_calculated`, `test_ndcg_at_k_hand_calculated` → grün.

## WP-113: Vergleichsmatrix implementieren
- [V] Matrix-Engine in `benchmarks/contextra-bench/src/matrix.rs` erstellt:
  - 6 Retrieval-Strategien definiert: Vektor allein, BM25 allein, RRF (V+T), RRF (V+T+G), Gewichtetes RRF, Normalisierte Linearkombination.
  - Anbindung an die offizielle `contextra-rank` API (`rrf_fuse`, `weighted_reciprocal_rank_fusion`, `score_normalized_fusion_with_options`), kein lokaler Nachbau.
  - Unit-Tests zur Verifikation der API-Anbindung erfolgreich ausgeführt.
- [V] Status in `benchmarks/results/2026-11-09.json` und `benchmarks/results/2026-11-09.md` geschrieben (ehrlich als 'nicht ausgeführt' ausgewiesen wegen Fremdblocker in Kern-Crates).

## WP-114: Skalierungslauf 100k Dokumente
- [V] Analyse des 100k-Skalierungslaufs (`bin/scale_100k.rs`):
  - Vorhandenes Messergebnis: `benchmarks/results/scale_100k_2026-09-30.json` bricht bei 0 verarbeiteten Intervallen ab: `Transaction error: Storage error: Memory budget exceeded (95%)`.
  - Ursachenanalyse:
    - In `crates/contextra-types/src/types/budget.rs` prüft `has_memory_capacity()` strikt `self.memory_used() < threshold` mit `threshold = 95% * memory_limit`.
    - In `crates/contextra-store/src/lsm/config.rs` und `crates/contextra-engine/src/lib.rs` ist `max_ram_mb` standardmäßig auf 2048 MB konfiguriert.
    - Während des fortlaufenden Einfügens von 100k Dokumenten mit 128d-Embedding und JSON-Metadaten wächst der `memory_used` Tracker der unkompaktierten Memtables und Staging-Puffer schneller an, als Background-Flushes und Compaction den Speicher freigeben können. Sobald 95% (1945 MB) erreicht werden, lehnt `storage.apply_backpressure()` bzw. `put()` alle weiteren Writes hart ab.
  - Vorgabe eingehalten: Das Budget wird nicht stillschweigend erhöht. Die Ursache ist transparent dokumentiert.

## WP-115: Dokumentation (docs/EVALUATION.md) und Adapter-Skizze
- [V] Adapter-Schnittstellenskizzen in `benchmarks/contextra-bench/src/adapters.rs` implementiert:
  - Traits `FullTextEngineAdapter` (Tantivy) und `VectorEngineAdapter` (LanceDB).
  - Skizzen-Implementierungen `TantivyAdapterSketch` und `LanceDbAdapterSketch` ohne neue Fremdabhängigkeiten.
  - Unit-Test `test_adapter_sketches_basic_contract` erfolgreich ausgeführt.
- [V] Dokument `docs/EVALUATION.md` erstellt:
  - Systemumgebung: Intel Xeon 2.30 GHz, 8 GB RAM, Linux x86_64, Rust 1.89.0.
  - Ehrliche Vergleichsmatrix: Als "nicht ausgeführt" markiert wegen Compilerfehlern in den Kern-Crates außerhalb unseres Besitzes.
  - Fehlerursache des 100k-Skalierungslaufs dokumentiert.
  - Reproduktionsbefehle aufgeführt.
  - Abschnitt "Was dieser Bericht NICHT zeigt" ohne Marketing oder Superlative integriert.
