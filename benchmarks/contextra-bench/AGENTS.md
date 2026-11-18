# AGENTS.md — contextra-bench — Benchmark-Harness & Regressions-Gates

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-bench.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-bench.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §12 (Testkatalog), §13 (INV-CLAIMS-1)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E3 (Endpunkte / Benchmarks)  
**Kern-Closure:** Darf alle Ebenen E0-E2 nutzen  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Reproduzierbare Benchmark-Harness: Recall@K, MRR, NDCG, Latenz-Profiling (hdrhistogram) und CI-Regressions-Gates.

---

## 2. Kern-Richtlinien für Agenten

- **Keine Rückwirkung auf den Kern:** Benchmarks dienen ausschließlich der externen Messung; kein Kern-Crate darf von `contextra-bench` abhängen.
- **Kontext-Pflicht für Ergebnisse (INV-CLAIMS-1):** Jede Zahl nennt Commit-Hash, CPU/Hardware, OS-Kernel, Korpus-Hash und Konfigurationsprofil.
- **Ratchet-Prinzip:** Baselines (`baseline_metrics.json`) dürfen nur nachweislich verbessert oder bei dokumentierten Upgrades angepasst werden.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `benchmarks/contextra-bench` · **Ring:** tooling · **Version:** 0.1.0
- **Beschreibung:** Contextra — Reproducible Benchmark Harness for Retrieval Accuracy
- **src:** 12 Dateien, 3551 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 6):** `contextra-db`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`
- **Interne Dev/Build-Abhängigkeiten (1):** `contextra-sys`
- **Externe Abhängigkeiten (normal, 9):** `anyhow`, `hdrhistogram`, `rand`, `serde`, `serde_json`, `tempfile`, `tokio`, `tracing`, `tracing-subscriber`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-db`, `contextra-types`; eingehend –
- **Features (3):** `datasets`, `default`, `external-benchmarks`

### Öffentliche API (grob, aus src/)
- struct (29): `AggregateMetrics`, `AnnBenchResult`, `BeirDocument`, `BeirMetrics`, `BeirQuery`, `BenchmarkDocument`, `BenchmarkMetrics`, `BenchmarkReport`, `CombinedMetrics`, `ComparisonMatrixReport`, `EngineHit`, `GroundTruthQuery`, `IntervalResult`, `LanceDbAdapterSketch`, `LocomoMetricsSummary` … (+14)
- enum (1): `EvaluatedStrategy`
- trait (2): `FullTextEngineAdapter`, `VectorEngineAdapter`
- fn (28): `calculate_latency_percentiles`, `calculate_mrr_at_k`, `calculate_ndcg_at_k`, `calculate_recall_at_k`, `compare_metrics`, `compare_metrics_files`, `compute_ground_truth`, `from_rank_items`, `fuse_via_rank_rrf`, `fuse_via_rank_score_normalized`, `fuse_via_rank_weighted_rrf`, `generate_synthetic_vectors`, `load_beir_corpus`, `load_beir_queries`, `map_at_k` … (+13)
- const (1): `DEFAULT_TOLERANCE_THRESHOLD`
- mod (8): `adapters`, `ann_benchmarks`, `beir_eval`, `compare`, `matrix`, `metrics`, `path_rag_sweep`, `regression_gate`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
