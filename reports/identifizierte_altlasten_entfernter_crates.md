# Audit- und Identifikationsreport: Altlasten und Rest-Artefakte entfernter Crates

- **Status:** Abgeschlossen (Vollständige forensische Erfassung)
- **Geltende Norm:** `docs/SPEC.md` (v4.2.0), SPEC §15.4 (WP-1.1, WP-1.2, WP-1.3, WP-2.1), D-24, D-84
- **Analysierte Crates (20):** `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-db`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`, `contextra-privacy`, `contextra-py`, `contextra-router`, `contextra-sandbox`, `contextra-wire`

---

## 1. Executive Summary

Im Rahmen der Konsolidierung und Refaktorisierung von Contextra auf den durch `docs/SPEC.md` definierten Kern wurden 20 historische Peripherie- und Hilfs-Crates aus dem Workspace (`Cargo.toml`) und dem Dateisystem (`crates/`) entfernt bzw. archiviert.
Trotz physischer Entfernung der Crate-Ordner verbleiben im bestehenden Code-, Konfigurations- und Dokumentationsbestand **762 dokumentierte Referenzen**, davon:

- **57 aktive Code-Imports (`use contextra_...`)**: Führen zu harten Compilerfehlern, sobald die betroffenen Dateien in den Build einbezogen werden (z. B. `contextra-engine`, `benchmarks/`).
- **7 aktive Code-Verwendungen / Aufrufe**: Direkte Pfadaufrufe (z. B. `contextra_infer_onnx::...`, `contextra_graph::...`).
- **195 Build-, Konfigurations- und CI-Reste**: Einträge in `deny.toml`, `justfile`, `CODEOWNERS`, `Cargo.toml`, `.github/workflows/`.
- **503 Dokumentations-, Modul-Header- und Inline-Kommentar-Reste**: Veraltete Architekturbeschreibungen (`//!`), Entkopplungskommentare und Invarianten-Verweise.

### Gesamtübersicht nach Crates

| Crate | Code-Imports | Code-Usages | Config & CI | Docs & Kommentare | Gesamt |
|---|:---:|:---:|:---:|:---:|:---:|
| `contextra-adapt` | 0 | 0 | 8 | 21 | **29** |
| `contextra-agent` | 0 | 0 | 8 | 39 | **47** |
| `contextra-audit-export` | 0 | 0 | 6 | 9 | **15** |
| `contextra-avv-generator` | 0 | 0 | 5 | 9 | **14** |
| `contextra-checkpoint` | 1 | 0 | 8 | 32 | **41** |
| `contextra-cognition` | 0 | 0 | 8 | 25 | **33** |
| `contextra-core` | 0 | 0 | 26 | 9 | **35** |
| `contextra-db` | 7 | 1 | 7 | 53 | **68** |
| `contextra-graph` | 48 | 4 | 13 | 47 | **112** |
| `contextra-infer-candle` | 0 | 0 | 5 | 34 | **39** |
| `contextra-infer-ollama` | 0 | 0 | 9 | 25 | **34** |
| `contextra-infer-onnx` | 0 | 2 | 9 | 21 | **32** |
| `contextra-kvcache` | 0 | 0 | 8 | 23 | **31** |
| `contextra-license` | 0 | 0 | 6 | 7 | **13** |
| `contextra-mcp` | 0 | 0 | 19 | 38 | **57** |
| `contextra-privacy` | 0 | 0 | 6 | 31 | **37** |
| `contextra-py` | 0 | 0 | 16 | 16 | **32** |
| `contextra-router` | 0 | 0 | 9 | 27 | **36** |
| `contextra-sandbox` | 1 | 0 | 8 | 19 | **28** |
| `contextra-wire` | 0 | 0 | 11 | 18 | **29** |
| **SUMME** | **57** | **7** | **195** | **503** | **762** |

---

## 2. Kritische Fundstellen: Aktive Code-Imports und Code-Usages

Diese Fundstellen stellen Compiler-Blocker dar oder binden Altlasten direkt an aktiven Rust-Quelltext.

### `benchmarks/competitive_bench.rs` (2 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 4 | `IMPORT` | `contextra-db` | `use contextra_db::context_compaction::{CompactionStrategy, ContextCompactor};` |
| 5 | `IMPORT` | `contextra-db` | `use contextra_db::Contextra;` |

### `benchmarks/migration_benchmarks.rs` (2 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 5 | `IMPORT` | `contextra-checkpoint` | `use contextra_checkpoint::PersistentCheckpointStore;` |
| 6 | `IMPORT` | `contextra-db` | `use contextra_db::{Contextra, ContextraConfig};` |

### `benchmarks/rrf_scale_bench.rs` (2 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 1 | `IMPORT` | `contextra-db` | `use contextra_db::fusion::weighted_reciprocal_rank_fusion;` |
| 2 | `IMPORT` | `contextra-db` | `use contextra_db::SearchResult;` |

### `benchmarks/scale_bench.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 5 | `IMPORT` | `contextra-db` | `use contextra_db::Contextra;` |

### `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 13 | `IMPORT` | `contextra-graph` | `use contextra_graph::hyperedge::HyperEdgeId;` |

### `crates/contextra-engine/src/background_workers/tests.rs` (9 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::hyperedge::HyperEdgeId;` |
| 109 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 171 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 237 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 395 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 504 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 558 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 601 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 694 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/src/collection/mod.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 99 | `USAGE` | `contextra-db` | `#[deprecated(note = "Parsing LLM responses should happen outside contextra-db")]` |

### `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` (9 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 76 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 198 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 242 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 286 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 375 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 492 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 554 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 593 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/src/collection/tests/insert_tests.rs` (5 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 81 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 132 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 183 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 234 | `IMPORT` | `contextra-graph` | `use contextra_graph::csr::CsrGraph;` |

### `crates/contextra-engine/src/collection/tests/link_tests.rs` (5 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::csr::CsrGraph;` |
| 63 | `IMPORT` | `contextra-graph` | `use contextra_graph::csr::CsrGraph;` |
| 174 | `IMPORT` | `contextra-graph` | `use contextra_graph::csr::{CsrGraph, CsrGraphConfig};` |
| 314 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 411 | `USAGE` | `contextra-graph` | `let config = contextra_graph::percolation::PercolationConfig {` |

### `crates/contextra-engine/src/collection/tests/misc_tests.rs` (7 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 34 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 140 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 352 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 404 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 455 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 497 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 545 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/src/collection/tests/read_tests.rs` (3 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::csr::CsrGraph;` |
| 155 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 289 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` (6 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 12 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 66 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 136 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 203 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 251 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |
| 299 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/src/contextra_impl/lifecycle.rs` (2 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 183 | `USAGE` | `contextra-infer-onnx` | `let model_dir = contextra_infer_onnx::ensure_onnx_model_download(` |
| 188 | `USAGE` | `contextra-infer-onnx` | `let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;` |

### `crates/contextra-engine/src/merge_operator_adapter.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 30 | `IMPORT` | `contextra-sandbox` | `use contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction};` |

### `crates/contextra-engine/src/transaction/db_transaction.rs` (2 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 30 | `USAGE` | `contextra-graph` | `pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,` |
| 150 | `USAGE` | `contextra-graph` | `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge) {` |

### `crates/contextra-engine/src/transaction/tests.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 11 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/tests/commit_crash_consistency.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 16 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/tests/engine_commit_crash_points.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 10 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/tests/engine_recovery_crash_windows.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 15 | `IMPORT` | `contextra-graph` | `use contextra_graph::CsrGraph;` |

### `crates/contextra-engine/tests/ssi_read_path_coverage.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 185 | `USAGE` | `contextra-graph` | `let graph = Arc::new(contextra_graph::CsrGraph::new());` |

### `crates/contextra/tests/llm_query_rewriter.rs` (1 kritische Referenzen)

| Zeile | Typ | Crate | Code-Auszug |
|---|---|---|---|
| 15 | `IMPORT` | `contextra-db` | `use contextra_db::QueryRewriter;` |

---

## 3. Detailmatrix je Crate (Vollständige Aufschlüsselung)

### Crate `contextra-adapt`

- **Gesamtanzahl Referenzen:** 29 (Imports: 0, Usages: 0, Config/CI: 8, Docs: 21)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `CODEOWNERS` | 30 | `/crates/contextra-adapt/               @contextra/intelligence-tier3` |
| `parse_all_remnants.py` | 6 | `"contextra-adapt",` |
| `scripts/contextra_full_audit.py` | 100 | `"contextra-agent", "contextra-checkpoint", "contextra-cognition", "contextra-adapt",` |
| `scripts/contextra_full_audit.py` | 117 | `"contextra-cognition", "contextra-adapt", "contextra-graph", "contextra-license"}` |
| `scripts/contextra_full_audit.py` | 1153 | `"contextra-avv-generator", "contextra-cognition", "contextra-adapt"]     # Final 2.2 + E-17` |
| `tools/generate_migration_edges.py` | 7 | `contextra-adapt, contextra-agent, contextra-cognition, contextra-core,` |
| `tools/generate_migration_edges.py` | 24 | `'contextra-adapt': 'contextra_adapt',` |
| `tools/generate_migration_edges.py` | 36 | `'contextra-adapt': '`AdaptiveController` durch feste Startwerte ersetzen: `efSearch = 64`, `exact_scan_max = 20_000`, Overfetch `3 / 6 / 12` (PLAN.md §7, WP 2.2 Schritt 3).',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-engine/src/decay_controller.rs` | 9 | `// ZWECK: Adaptive Decay Controller — WP 2.2: Typen aus contextra-adapt hierher kopiert (archiviert).` |
| `crates/contextra-engine/src/decay_controller.rs` | 10 | `// INVARIANTEN: Identische Logik wie contextra-adapt/src/decay_controller.rs (ADR-016).` |
| `crates/contextra-engine/src/decay_controller.rs` | 15 | `//! Kopiert aus contextra-adapt (archiviert in WP 2.1), um die Kante zu entfernen.` |
| `crates/contextra-types/src/lib.rs` | 22 | `//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`,` |
| `docs/CHANGELOG.md` | 107 | `\| `TS:2026-09-26T00:00:00Z` \| `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Konfigurations- und Validierungstests für SketchedProjection (§13.2). \|` |
| `docs/CHANGELOG.md` | 108 | `\| `TS:2026-09-26T00:00:00Z` \| `crates/contextra-adapt/tests/sketched_projection_determinism.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Determinismus-Tests für SketchedProjection (§13.2 / P28). \|` |
| `docs/CHANGELOG.md` | 109 | `\| `TS:2026-09-26T00:00:00Z` \| `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Differential-Test SketchedProjection vs. ShermanMorrison baseline (§13.2). \|` |
| `docs/CHANGELOG.md` | 127 | `\| `TS:2026-09-13T00:00:00Z (SESSION: pid-regler-impl)` \| `crates/contextra-adapt/src/pid_latency_controller.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| PID Latency Controller & Latency Budget Guard für Multi-Step Retrieval. \|` |
| `docs/CHANGELOG.md` | 135 | `\| `TS:2026-09-11T00:00:00Z` \| `crates/contextra-adapt/src/homeostat.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| P95 retrieval latency feedback-regulated PID controller and hard deadline management (Feature F-08 & P11 requirement). \|` |
| `docs/CHANGELOG.md` | 231 | `\| `2026-09-27T00:00:00Z` \| `crates/contextra-adapt/tests/pid_anti_windup_tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrationstests für conditional Back-Calculation Anti-Windup im PID-Regler. \|` |
| `docs/CHANGELOG.md` | 278 | `\| `2026-09-15T00:00:00Z` \| `crates/contextra-adapt/tests/pid_stability.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Stabilitätstest des PID-Reglers mit Bekannter Sprungantwort (Settling Time, Overshoot, Steady-State Error). \|` |
| `docs/CHANGELOG.md` | 301 | `\| `2026-09-11T12:00:00Z` \| `crates/contextra-adapt/src/lyapunov.rs` \| `AI-TAG` \| `AGT-ROUTER-00808347` \| `21a8d3e8` \| `RESOLVED` \| `0` \| // AI-TAG[RESOLVED][MINOR] Added per-bin contribution clipping (`MAX_BIN_KL_CONTRIBUTION = 10.0`) to prevent numerical instability during KL divergence calculations under extreme distribution shift. (ID: AGT-ROUTER-00808347) (TS: 2026-09-11T12:00:00Z) (SESSION: 21a8d3e8) \|` |
| `docs/CHANGELOG.md` | 304 | `\| `2026-09-11T00:00:00Z` \| `crates/contextra-adapt/src/pid.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Adaptive Reranking candidate pool-size regulation via PID latency control (F-08 & P11). \|` |
| `docs/CHANGELOG.md` | 325 | `\| `2026-09-09T15:49:44Z (SESSION: 5b65397f)` \| `crates/contextra-adapt/src/lyapunov.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Proaktiver Distributional-Drift-Wächter via Lyapunov-Exponenten über KL-Divergenzen. \|` |
| `docs/CHANGELOG.md` | 479 | `\| `` \| `crates/contextra-engine/src/decay_controller.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Re-Export des Adaptive Decay Controllers aus contextra-adapt (Ring 0/Adapt). \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 431 | `contextra-adapt⚠: bandit-routing, flow-corrected-thompson, egress-sherman-morrison, sketched-bandit, rie-greedy-personalization` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/cleanup/g0-baseline.txt` | 202 | `Checking contextra-adapt v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-adapt)` |

### Crate `contextra-agent`

- **Gesamtanzahl Referenzen:** 47 (Imports: 0, Usages: 0, Config/CI: 8, Docs: 39)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 48 | `"contextra-agent",` |
| `parse_all_remnants.py` | 7 | `"contextra-agent",` |
| `scripts/contextra_full_audit.py` | 100 | `"contextra-agent", "contextra-checkpoint", "contextra-cognition", "contextra-adapt",` |
| `scripts/contextra_full_audit.py` | 113 | `RUECKBAU_GEPLANT = {"contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest",` |
| `scripts/contextra_full_audit.py` | 1150 | `ENTFERNEN_V1 = ["contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest", "contextra-privacy",` |
| `tools/generate_migration_edges.py` | 7 | `contextra-adapt, contextra-agent, contextra-cognition, contextra-core,` |
| `tools/generate_migration_edges.py` | 25 | `'contextra-agent': 'contextra_agent',` |
| `tools/generate_migration_edges.py` | 37 | `'contextra-agent': 'Feature `agent-workflows` entfernen; optionale Kante aus `mcp/Cargo.toml` löschen.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `.agents/jules/SCHEDULED_AUDIT.md` | 15 | `- **Fokus-Crates.** Die Vorfassung nennt `contextra-graph`, `contextra-agent` und `contextra-db`. Diese Crates sind nach SPEC D-24 und D-45 entfernt. Die Rotation muss vorhandene Crates verwenden, etwa `contextra-store`, `contextra-crypto`, `contextra-vector` und `contextra-text`. Ab Gate G-D kommt `contextra-erasure` hinzu.` |
| `crates/contextra-store/src/lib.rs` | 26 | `//!   `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra-agent`,` |
| `crates/contextra-testkit/src/lib.rs` | 26 | `//! - **Wird eingebunden von:** `contextra`, `contextra-agent`, `contextra-checkpoint`,` |
| `crates/contextra-text/src/lib.rs` | 23 | `//! - **Wird genutzt von**: `contextra-engine`, `contextra-db`, `contextra-router`, `contextra-agent`` |
| `crates/contextra-types/src/lib.rs` | 22 | `//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 26 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthält noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldet FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. Dies ist kein Code-Bug, sondern eine nicht synchronisierte Konfigurationsdatei. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 28 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthielt noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldete FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. \| `capabilities.toml` synchronisiert (Entfernung von 21 Nicht-Mitgliedern, Aufnahme von `contextra-durable-fs` und `contextra-arch-tests`). \|` |
| `docs/CHANGELOG.md` | 96 | `\| `TS:2026-10-01T00:00:00Z (SESSION: 5a38054a)` \| `crates/contextra-agent/src/graph.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Declarative StateGraph definition routing workflow step transitions. \|` |
| `docs/CHANGELOG.md` | 97 | `\| `TS:2026-10-01T00:00:00Z (SESSION: 088b4a44)` \| `crates/contextra-agent/src/goal_condition.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Strukturierte, deterministische Auswertung von Teilziel-Bedingungen (GoalCondition) für Agentenschritte. \|` |
| `docs/CHANGELOG.md` | 98 | `\| `TS:2026-10-01T00:00:00Z` \| `crates/contextra-agent/src/clm_scratchpad.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Temporärer, aufgabengebundener Kontext-Scratchpad für sehr lange Agentenschleifen (Context-Language-Model). \|` |
| `docs/CHANGELOG.md` | 113 | `\| `TS:2026-09-17T19:00:00Z (SESSION: jules-agent-budget)` \| `crates/contextra-agent/src/budget.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Token budget RAII reservation abstraction for agent workflow steps. \|` |
| `docs/CHANGELOG.md` | 126 | `\| `TS:2026-09-13T01:36:52Z (SESSION: c1c85419)` \| `crates/contextra-agent/tests/proptest_workflow_tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Property-based testing for contextra-agent structural invariants (StateGraph, AgentContext, BackgroundEvent). \|` |
| `docs/CHANGELOG.md` | 134 | `\| `TS:2026-09-11T14:41:11Z (SESSION: b979ee5a)` \| `crates/contextra-agent/src/audit.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Append-only immutable audit trail logging for agent state transitions. \|` |
| `docs/CHANGELOG.md` | 161 | `\| `TS:2026-09-06T11:18:35Z (SESSION: 820afd9c)` \| `crates/contextra-agent/src/dlq.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Persistent dead-letter queue storage abstraction for failed agent steps. \|` |
| `docs/CHANGELOG.md` | 164 | `\| `TS:2026-09-02T23:19:10Z (SESSION: 088b4a44)` \| `crates/contextra-agent/src/step.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Step result structures and AgentTool trait definitions for orchestration. \|` |
| `docs/CHANGELOG.md` | 165 | `\| `TS:2026-09-01T23:11:04Z (SESSION: 5a38054a)` \| `crates/contextra-agent/src/context.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Workflow execution state context carrying task ID, budget, and memory. \|` |
| `docs/CHANGELOG.md` | 166 | `\| `TS:2026-09-01T23:11:04Z (SESSION: 5a38054a)` \| `crates/contextra-agent/src/engine.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Deterministic graph-walker orchestrator engine for autonomous agent workflows. \|` |
| `docs/CHANGELOG.md` | 167 | `\| `TS:2026-09-01T23:11:04Z (SESSION: 5a38054a)` \| `crates/contextra-agent/src/event_source.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Continuous event stream abstractions delivering telemetry/trigger events to agents. \|` |
| `docs/CHANGELOG.md` | 178 | `\| `TS:2026-08-31T00:00:00Z (SESSION: 8a7c2f1e)` \| `crates/contextra-agent/tests/budget_race_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Test token budget concurrency and RMW race condition in contextra-agent workflow steps. \|` |
| `docs/CHANGELOG.md` | 264 | `\| `2026-09-15T16:46:26Z` \| `crates/contextra-agent/src/lib.rs` \| `REVIEW-PASS` \| `AGT-AGENT-692d9982` \| `692d9982` \| `PASS` \| `0` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: AGT-AGENT-692d9982) (TS: 2026-09-15T16:46:26Z) (SESSION: 692d9982) (PRÜFER-KONTEXT: FRESH) \|` |
| `docs/CHANGELOG.md` | 300 | `\| `2026-09-11T14:30:00Z` \| `crates/contextra-agent/src/dlq.rs` \| `AI-TAG` \| `AGT-AGENT-49bfd02e` \| `-` \| `RESOLVED` \| `0` \| // AI-TAG[SMELL][MINOR] RESOLVED: AGT-AGENT-49bfd02e — Replaced non-atomic last_tx_id + 1 read with OnceCell initialized AtomicU64 fetch_add. (TS: 2026-09-11T14:30:00Z) \|` |
| `docs/CHANGELOG.md` | 339 | `\| `2026-09-09T12:36:33Z` \| `crates/contextra-agent/src/lib.rs` \| `REVIEW-PASS` \| `AGT-AGENT-266186df` \| `3db4fa31` \| `PASS` \| `1` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: AGT-AGENT-266186df) (TS: 2026-09-09T12:36:33Z) (SESSION: 3db4fa31) \|` |
| `docs/CHANGELOG.md` | 350 | `\| `2026-09-06T11:18:35Z` \| `crates/contextra-agent/src/dlq.rs` \| `AI-TAG` \| `AGT-AGENT-0399362d` \| `820afd9c` \| `RESOLVED` \| `0` \| // AI-TAG[INVENTORY-DRIFT][MINOR] RESOLVED: AGT-AGENT-0399362d — Documented dlq.rs inventory drift in audit report. (TS: 2026-09-06T11:18:35Z) (SESSION: 820afd9c) \|` |
| `docs/CHANGELOG.md` | 369 | `\| `2026-09-01T23:02:47Z` \| `crates/contextra-agent/src/lib.rs` \| `REVIEW-PASS` \| `AGT-AGENT-266186df` \| `49022c36` \| `PASS` \| `1` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: AGT-AGENT-266186df) (TS: 2026-09-01T23:02:47Z) (SESSION: 49022c36) \|` |
| `docs/CHANGELOG.md` | 403 | `\| `2026-08-30T15:00:19Z` \| `crates/contextra-agent/src/context.rs` \| `AI-TAG` \| `-` \| `283abf0f` \| `RESOLVED` \| `-` \| // AI-TAG[HARDENING][CRITICAL] RESOLVED: Validates non-empty input parameters for agent workflow context initialization. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) \|` |
| `docs/CHANGELOG.md` | 404 | `\| `2026-08-30T15:00:19Z` \| `crates/contextra-agent/src/event_source.rs` \| `AI-TAG` \| `-` \| `283abf0f` \| `RESOLVED` \| `-` \| // AI-TAG[HARDENING][CRITICAL] RESOLVED: Validates non-empty event source to prevent silent telemetry attribution loss. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) \|` |
| `docs/CHANGELOG.md` | 405 | `\| `2026-08-30T15:00:19Z` \| `crates/contextra-agent/src/event_source.rs` \| `AI-TAG` \| `-` \| `283abf0f` \| `RESOLVED` \| `-` \| // AI-TAG[HARDENING][CRITICAL] RESOLVED: Enforces bounded event queue capacity to guard against unbounded memory growth. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) \|` |
| `docs/CHANGELOG.md` | 406 | `\| `2026-08-30T15:00:19Z` \| `crates/contextra-agent/src/graph.rs` \| `AI-TAG` \| `-` \| `283abf0f` \| `RESOLVED` \| `-` \| // AI-TAG[HARDENING][CRITICAL] RESOLVED: Validates non-empty Node ID and description for graph nodes. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) \|` |
| `docs/CHANGELOG.md` | 407 | `\| `2026-08-30T15:00:19Z` \| `crates/contextra-agent/src/graph.rs` \| `AI-TAG` \| `-` \| `283abf0f` \| `RESOLVED` \| `-` \| // AI-TAG[HARDENING][CRITICAL] RESOLVED: Validates non-empty from/to endpoints for workflow edges. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) \|` |
| `docs/CHANGELOG.md` | 416 | `\| `2026-08-27T00:00:00Z` \| `crates/contextra-agent/src/lib.rs` \| `AI-TAG` \| `AGT-AGENT-001` \| `-` \| `RESOLVED` \| `0` \| // AI-TAG[DOC-DRIFT][MINOR] RESOLVED: AGT-AGENT-001 — Re-extracted workflow engine crate requires integration verification. (TS:2026-08-27T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 426 | `\| `` \| `crates/contextra-agent/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Crate entry point exposing audit, context, engine, event_source, graph, and step submodules. \|` |
| `docs/CHANGELOG.md` | 427 | `\| `` \| `crates/contextra-agent/tests/clm_goal_condition_edges.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Tests für alle GoalCondition-Varianten, Randfälle und Determinismus. \|` |
| `docs/CHANGELOG.md` | 428 | `\| `` \| `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Tests für CLM Scratchpad Reset-Flow, RAM-Zeroization, Audit-/Invalidierungscounter und PinnedRegion. \|` |
| `docs/CHANGELOG.md` | 429 | `\| `` \| `crates/contextra-agent/tests/workflow_edge_serde_compat.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Serde-Kompatibilitätstests für WorkflowEdge (Lesen alter JSONs ohne goal, Roundtrip neuer JSONs mit goal). \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 812 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-agent`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 830 | `H8 [Drei Audit-Ketten, SEC-05/K8]: AuditChain (contextra-crypto), ContextEditAuditRecord (contextra-privacy⚠, verify_audit_chain), AgentAudit (contextra-agent⚠): Dokumentiere Genesis-Hash, Schlüssel und Verifikationsweg jeder Kette; prüfe, ob irgendeine Querverbindung existiert. Liste Redundanzen als Text mit Datei:Zeile (reiner Architekturbefund S3).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-audit-export`

- **Gesamtanzahl Referenzen:** 15 (Imports: 0, Usages: 0, Config/CI: 6, Docs: 9)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 49 | `"contextra-audit-export", # TRANSITION E29, bis 2027-04-30, REQ-ASYNC-1` |
| `parse_all_remnants.py` | 8 | `"contextra-audit-export",` |
| `scripts/contextra_full_audit.py` | 104 | `"contextra-router", "contextra-sandbox", "contextra-wire", "contextra-audit-export",` |
| `scripts/contextra_full_audit.py` | 116 | `"contextra-infer-ollama", "contextra-audit-export", "contextra-avv-generator",` |
| `scripts/contextra_full_audit.py` | 1152 | `"contextra-sandbox", "contextra-infer-onnx", "contextra-infer-ollama", "contextra-audit-export",` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-types/src/lib.rs` | 22 | `//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 24 | `\| Z. 60–62 \| `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]` \| **Kardinaler Modellfehler im Skript:** Hier wird eine völlig veraltete Definition der "Produktmenge" zugrunde gelegt. Laut `docs/SPEC.md §5.1` umfasst der Kern 14 Crates (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`, `engine`, `contextra`). Crates wie `ingest`, `privacy`, `infer-ollama`, `audit-export`, `avv-generator` sind explizit Nicht-Ziele (SPEC §1.4). \| Schritt `produkt_huelle_ohne_kern` liefert FAIL (5 Crates mit Befund), weil diese Crates entweder gar nicht mehr existieren oder falsche Hüllen prüfen. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 840 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-audit-export`, `contextra-avv-generator`, `contextra-mcp`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 842 | `**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-privacy⚠/tests/campaign_egress_*.rs`, `crates/contextra-mcp⚠/tests/campaign_mcp_*.rs`, `crates/contextra-audit-export⚠/tests/campaign_*.rs`` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 847 | `AUFTRAG: Prüfe die Egress-Firewall (contextra-privacy⚠), den MCP-Server (contextra-mcp⚠) und die DSGVO-Generatoren (contextra-audit-export⚠, contextra-avv-generator⚠). Quellen: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md (SEC-04, K6, K7), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (4.7, 4.8, Hinweis zum offenen MAJOR-Befund normalize_text), CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md (Teil VI).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 862 | `H10 [Art. 30 DSGVO, processing_registry.rs, contextra-audit-export⚠]: generate_registry/render_markdown/render_register_json/render_register_markdown: Ausgabe ist deterministisch; JSON ist gültig (parse mit serde_json im Test); enthält die Pflichtangaben des Verarbeitungsverzeichnisses nach Art. 30 Abs. 1 lit. a–g (Verantwortlicher, Zwecke, Kategorien Betroffener und Daten, Empfängerkategorien, Drittlandübermittlung, Löschfristen, technische/organisatorische Maßnahmen) — prüfe als technische VOLLSTÄNDIGKEITSPRÜFUNG, nicht als Rechtsberatung. Canary-Test: ein eingeschleuster Geheimniswert (z. B. "CANARY-SECRET-1234") in Eingabedaten taucht in keiner Ausgabe auf. bsi_mapping_table/render_bsi_mapping_markdown: jede referenzierte Maßnahme verweist auf existierenden Code (Datei:Zeile) oder wird als unbelegt markiert.` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-avv-generator`

- **Gesamtanzahl Referenzen:** 14 (Imports: 0, Usages: 0, Config/CI: 5, Docs: 9)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `parse_all_remnants.py` | 9 | `"contextra-avv-generator",` |
| `scripts/contextra_full_audit.py` | 105 | `"contextra-avv-generator", "contextra-bench",` |
| `scripts/contextra_full_audit.py` | 116 | `"contextra-infer-ollama", "contextra-audit-export", "contextra-avv-generator",` |
| `scripts/contextra_full_audit.py` | 1153 | `"contextra-avv-generator", "contextra-cognition", "contextra-adapt"]     # Final 2.2 + E-17` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-types/src/lib.rs` | 22 | `//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 24 | `\| Z. 60–62 \| `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]` \| **Kardinaler Modellfehler im Skript:** Hier wird eine völlig veraltete Definition der "Produktmenge" zugrunde gelegt. Laut `docs/SPEC.md §5.1` umfasst der Kern 14 Crates (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`, `engine`, `contextra`). Crates wie `ingest`, `privacy`, `infer-ollama`, `audit-export`, `avv-generator` sind explizit Nicht-Ziele (SPEC §1.4). \| Schritt `produkt_huelle_ohne_kern` liefert FAIL (5 Crates mit Befund), weil diese Crates entweder gar nicht mehr existieren oder falsche Hüllen prüfen. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 840 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-audit-export`, `contextra-avv-generator`, `contextra-mcp`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 847 | `AUFTRAG: Prüfe die Egress-Firewall (contextra-privacy⚠), den MCP-Server (contextra-mcp⚠) und die DSGVO-Generatoren (contextra-audit-export⚠, contextra-avv-generator⚠). Quellen: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md (SEC-04, K6, K7), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (4.7, 4.8, Hinweis zum offenen MAJOR-Befund normalize_text), CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md (Teil VI).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 863 | `H11 [Art. 28 DSGVO, contextra-avv-generator⚠]: render_avv_markdown/render mit AvvContext/default_technical_measures: prüfe technische Vollständigkeit gegen die Pflichtinhalte nach Art. 28 Abs. 3 lit. a–h (Gegenstand/Dauer, Art/Zweck, Datenarten, Betroffenenkategorien, Weisungsgebundenheit, Vertraulichkeit, Maßnahmen nach Art. 32, Unterauftragsverarbeiter, Unterstützungspflichten, Rückgabe/Löschung, Kontrollrechte). Jede im generierten Text behauptete technische Maßnahme (z. B. "Ende-zu-Ende-Verschlüsselung", "Crypto-Shredding") gegen den Code belegen: nicht belegte Zusage = Befund S3 (Wahrheit der Compliance-Aussage). Keine rechtliche Bewertung.` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/cleanup/g0-baseline.txt` | 282 | `Checking contextra-avv-generator v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-avv-generator)` |

### Crate `contextra-checkpoint`

- **Gesamtanzahl Referenzen:** 41 (Imports: 1, Usages: 0, Config/CI: 8, Docs: 32)

#### Aktive Code-Stellen
| Datei | Zeile | Typ | Quelltext |
|---|---|---|---|
| `benchmarks/migration_benchmarks.rs` | 5 | `IMPORT` | `use contextra_checkpoint::PersistentCheckpointStore;` |

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `.cargo/mutants.toml` | 6 | `"crates/contextra-checkpoint/src/**/*.rs",` |
| `CODEOWNERS` | 19 | `/crates/contextra-checkpoint/          @contextra/storage-tier1` |
| `deny.toml` | 51 | `"contextra-checkpoint", # TRANSITION E29, bis 2027-04-30, REQ-ASYNC-1` |
| `parse_all_remnants.py` | 10 | `"contextra-checkpoint",` |
| `scripts/contextra_full_audit.py` | 100 | `"contextra-agent", "contextra-checkpoint", "contextra-cognition", "contextra-adapt",` |
| `scripts/contextra_full_audit.py` | 1143 | `"contextra-store": 2, "contextra-checkpoint": 2,` |
| `scripts/contextra_full_audit.py` | 1154 | `ENTKOPPELN_V1 = ["contextra-graph", "contextra-license", "contextra-checkpoint"]     # Entscheidungsgrundlage 6.2` |
| `tools/loc.py` | 24 | `"contextra-checkpoint",` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-mvcc/src/lib.rs` | 20 | `//! - `contextra-checkpoint`: Schützt Snapshot-Zustände bei Backup-Erstellung.` |
| `crates/contextra-store/src/lib.rs` | 26 | `//!   `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra-agent`,` |
| `crates/contextra-testkit/src/lib.rs` | 26 | `//! - **Wird eingebunden von:** `contextra`, `contextra-agent`, `contextra-checkpoint`,` |
| `crates/contextra-types/src/lib.rs` | 23 | `//!   `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 22 | `\| Z. 42–56 \| `BENCHES` listet Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). \| Schritt `befund_bench_fuzz_dateien` schlägt fehl (FAIL); Benchmarks für diese Crates können nicht existieren. \|` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 46 | `\| **A. Skript-Artefakte / Veraltetes Soll-Modell** \| ca. 22 Schritte (`befund_bench_fuzz_dateien`, `produkt_huelle_ohne_kern`, diverse `fuzz__*`, `bench__*` von entfernten Crates) \| Das Skript testet Crates, die nach SPEC §5.1 gar nicht mehr im Workspace existieren (`contextra-checkpoint`, `router`, `graph`, `candle`, `bench`, `ingest`, `mcp`, `wire`). \| **Skript korrigieren:** Bench- und Fuzz-Listen auf die 14 Kern-Crates synchronisieren. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 24 | `\| Z. 42–56 \| `BENCHES` enthielt Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench` sowie `deletion_proof_latency_bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). Die Benchmarks existieren nicht mehr. \| Schritt `befund_bench_fuzz_dateien` und Benchmarks scheiterten mit FAIL. \| `BENCHES` auf die existierenden Benchmarks der aktiven Crates (`mvcc`, `crypto`, `vector`, `store`, `simd`) bereinigt. \|` |
| `docs/CHANGELOG.md` | 214 | `\| `2026-09-28T00:00:00Z` \| `crates/contextra-checkpoint/src/hardlink_cloner.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Hardlink-Cloner für Agent-Forking und SSTable CoW-Klone. \|` |
| `docs/CHANGELOG.md` | 237 | `\| `2026-09-25T00:00:00Z` \| `crates/contextra-checkpoint/tests/checkpoint_systematic_crash.rs` \| `ANCHOR` \| `TEST:CHK-005` \| `-` \| `DONE` \| `0` \| // ANCHOR[TEST:CHK-005] STATUS:DONE (TS:2026-09-25T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 259 | `\| `2026-09-16T16:23:23Z` \| `crates/contextra-checkpoint/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `8d62c439` \| `-` \| `-` \| // REVIEW-PASS[2/2] (TS: 2026-09-16T16:23:23Z) (SESSION: 8d62c439) PRÜFER-KONTEXT: FRESH \|` |
| `docs/CHANGELOG.md` | 274 | `\| `2026-09-15T15:09:43Z (SESSION: 2e382e86)` \| `crates/contextra-checkpoint/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| RAII CheckpointGuard + persistente Snapshot-Verwaltung (Re-Exports) \|` |
| `docs/CHANGELOG.md` | 275 | `\| `2026-09-15T15:09:43Z` \| `crates/contextra-checkpoint/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `2e382e86` \| `-` \| `-` \| // REVIEW-PASS[1/2] (TS: 2026-09-15T15:09:43Z) (SESSION: 2e382e86) PRÜFER-KONTEXT: FRESH \|` |
| `docs/CHANGELOG.md` | 358 | `\| `2026-09-02T23:18:12Z` \| `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` \| `ANCHOR` \| `TEST:CKPT-001` \| `358e3b0a` \| `DONE` \| `2` \| // ANCHOR[TEST:CKPT-001] STATUS:DONE (TS:2026-09-02T23:18:12Z) (SESSION: 358e3b0a) \|` |
| `docs/CHANGELOG.md` | 359 | `\| `2026-09-02T23:18:12Z` \| `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` \| `REVIEW-PASS` \| `TEST:CKPT-001` \| `2155aaa2` \| `PASS` \| `1` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: TEST:CKPT-001) (TS:2026-09-02T23:18:12Z) (SESSION: 2155aaa2) PRÜFER-KONTEXT: FRESH - Confirmed concurrent stress test, cache hit/miss reloading, and unpinning lifecycle. \|` |
| `docs/CHANGELOG.md` | 368 | `\| `2026-09-01T23:09:00Z` \| `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` \| `REVIEW-PASS` \| `TEST:CKPT-001` \| `fdf7a62e` \| `PASS` \| `1` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: TEST:CKPT-001) (TS:2026-09-01T23:09:00Z) (SESSION: fdf7a62e) PRÜFER-KONTEXT: FRESH - Test covers concurrent pinning, GC safety, and cache behavior. \|` |
| `docs/CHANGELOG.md` | 412 | `\| `2026-08-29T08:06:29Z` \| `crates/contextra-checkpoint/src/manifest.rs` \| `AI-TAG` \| `AGT-CKPT-f3a1b2c4` \| `14348074` \| `RESOLVED` \| `0` \| /// AI-TAG\[PANIC-SAFETY\]\[CRITICAL\] RESOLVED: AGT-CKPT-f3a1b2c4 (TS:2026-08-29T08:06:29Z) (SESSION:14348074) \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 878 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-db`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 893 | `cargo bench -p contextra-checkpoint⚠ --bench checkpoint_bench -- (gleiche Zeitparameter)` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 972 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 987 | `e) Dateizugriff am VFS vorbei: std::fs::, File::open, OpenOptions außerhalb der VFS-/FaultVfs-Abstraktion in contextra-store und contextra-checkpoint⚠. Jeder direkte Zugriff ist ein Blindfleck der Fehlerinjektion (Testbarkeit) und ein Befund S3.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1026 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-kvcache`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1046 | `9) Geschwister-Schichten einzeln: HNSW (Vektor weg + keine Geisterzeiger: verify_no_ghost_pointers, INV-DELETION-2), contextra-text (Posting-Listen und Dokumentstatistik/IDF aktualisiert?), contextra-graph⚠ (Knoten/Kanten/Hyperkanten und abgeleitete Community-/Konsolidierungsergebnisse), contextra-cognition⚠ (konsolidierte Zusammenfassungen oder Embeddings, die den Inhalt abgeleitet enthalten — hier kann personenbezogene Information fortleben, obwohl das Original gelöscht ist: Befund S2, falls so), contextra-kvcache⚠ (gecachte Tensoren zum Prompt, kv-encryption), contextra-checkpoint⚠ (Blake3-Manifest enthält Hashes, keine Klartexte?).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1189 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1198 | `AUFTRAG: Beweise die Crash-Konsistenz von contextra-store (und contextra-checkpoint⚠) unter wiederholtem SIGKILL. Quelle der Hypothesen: PRODUKT-1-LSM-STORAGE-ANALYSE.md (Reifegrad R1: Datenverlust-/Replay-Bugs S1-01, S2-01). Harness als eigenes Cargo-Projekt unter /tmp/contextra-campaign⚠/L-10/harness mit path-Dependency auf contextra-store (Release-Profil). Nicht ins Repo committen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1216 | `Zusätzlich: Wiederhole die Kampagne für contextra-checkpoint⚠ (Checkpoint erstellen, während das Kind weiterschreibt, SIGKILL mitten im Checkpoint; danach muss der letzte VOLLSTÄNDIGE Checkpoint (Blake3-Manifest) verifizierbar sein und ein halber Checkpoint erkannt und verworfen werden).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1368 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1381 | `SCHRITT 5 — Fehlerpfad-Abdeckung speziell: Anteil der Err-Zweige (match-Arme mit Err, ?-Operator-Pfade) die je getroffen werden — nutze die Branch-/Region-Daten für contextra-store/src/wal/**, lsm/recovery.rs, manifest/**, contextra-checkpoint⚠/**.` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/ROOT_CAUSE_ANALYSIS.md` | 19 | `Dabei wurden die entsprechenden Crates (`contextra-graph`, `contextra-checkpoint` etc.) aus dem Workspace entfernt und das Feld `graph_index` aus `Collection` gelöscht.` |
| `docs/cleanup/g0-baseline.txt` | 207 | `Checking contextra-checkpoint v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-checkpoint)` |

### Crate `contextra-cognition`

- **Gesamtanzahl Referenzen:** 33 (Imports: 0, Usages: 0, Config/CI: 8, Docs: 25)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 52 | `"contextra-cognition",` |
| `parse_all_remnants.py` | 11 | `"contextra-cognition",` |
| `scripts/contextra_full_audit.py` | 100 | `"contextra-agent", "contextra-checkpoint", "contextra-cognition", "contextra-adapt",` |
| `scripts/contextra_full_audit.py` | 117 | `"contextra-cognition", "contextra-adapt", "contextra-graph", "contextra-license"}` |
| `scripts/contextra_full_audit.py` | 1153 | `"contextra-avv-generator", "contextra-cognition", "contextra-adapt"]     # Final 2.2 + E-17` |
| `tools/generate_migration_edges.py` | 7 | `contextra-adapt, contextra-agent, contextra-cognition, contextra-core,` |
| `tools/generate_migration_edges.py` | 26 | `'contextra-cognition': 'contextra_cognition',` |
| `tools/generate_migration_edges.py` | 38 | `'contextra-cognition': 'Kante aus `db/Cargo.toml` löschen; betroffene Module in `db/src/` entfernen oder stubben.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-store/src/lib.rs` | 26 | `//!   `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra-agent`,` |
| `crates/contextra-testkit/src/lib.rs` | 27 | `//!   `contextra-cognition`, `contextra-crypto`, `contextra-db`, `contextra-engine`,` |
| `crates/contextra-types/src/lib.rs` | 23 | `//!   `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`,` |
| `docs/CHANGELOG.md` | 104 | `\| `TS:2026-09-27T00:00:00Z` \| `crates/contextra-cognition/src/transitivity_veto.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Transitivity Veto Inspection & Orchestration for Near-Duplicate Memory Consolidation. \|` |
| `docs/CHANGELOG.md` | 105 | `\| `TS:2026-09-27T00:00:00Z` \| `crates/contextra-cognition/tests/transitivity_veto_tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Tests für Transitivity Veto Inspektion, Orchestrierung & Budget-Steuerung. \|` |
| `docs/CHANGELOG.md` | 128 | `\| `TS:2026-09-13T00:00:00Z` \| `crates/contextra-cognition/src/consolidation_locks.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Guard-Konstrukt fuer typsicheres Locking und geordnete Kaskaden-Invalidierung in der Sleep-Cycle-Konsolidierung. \|` |
| `docs/CHANGELOG.md` | 158 | `\| `TS:2026-09-07T08:30:00Z` \| `crates/contextra-cognition/src/consolidation_executor.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Verbindet Structural Consolidation Pass-Ergebnisse mit der Collection-Mutation-API. \|` |
| `docs/CHANGELOG.md` | 179 | `\| `TS:2026-08-31T00:00:00Z` \| `crates/contextra-cognition/src/maintenance_config.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Konfiguration des MaintenanceSchedulers für Background-Maintenance-Layer. \|` |
| `docs/CHANGELOG.md` | 195 | `\| `TS:2026-08-29T18:00:00Z` \| `crates/contextra-cognition/src/memory_consolidation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Structural Consolidation Pass & Generative Synthesis Pass Architecture — Memory Consolidation (Near-Duplicate-Detection & Segmentation). \|` |
| `docs/CHANGELOG.md` | 196 | `\| `TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)` \| `crates/contextra-cognition/src/context.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Kontextfenster-Verwaltung und Token-Budgetierung für RAG-Prompts. \|` |
| `docs/CHANGELOG.md` | 197 | `\| `TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)` \| `crates/contextra-cognition/src/context_compaction/mod.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Kontextkompaktierung und Zusammenfassung langer Gesprächs- und Dokumentverläufe. \|` |
| `docs/CHANGELOG.md` | 414 | `\| `2026-08-28T00:00:00Z` \| `crates/contextra-cognition/src/context_compaction/compactor.rs` \| `AI-TAG` \| `AGT-DB-004` \| `-` \| `RESOLVED` \| `0` \| // AI-TAG[SMELL][MINOR][RESOLVED] Async LLM-Summarization for context compaction (ID: AGT-DB-004) (TS:2026-08-28T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 422 | `\| `2026-06-01T00:00:00Z` \| `crates/contextra-cognition/src/context.rs` \| `ANCHOR` \| `INTEGRATION:WP-7.1-CHUNKER` \| `-` \| `DONE` \| `0` \| // ANCHOR[INTEGRATION:WP-7.1-CHUNKER] STATUS:DONE (TS:2026-06-01T00:00:00Z) — Wire MarkdownChunker to ContextManager \|` |
| `docs/CHANGELOG.md` | 430 | `\| `` \| `crates/contextra-cognition/src/leanrag_input.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Extraktion von LeanRAG-Eingabedaten (AggregationNode/AggregationEdge) aus einer Collection (Spec §21.4). \|` |
| `docs/CHANGELOG.md` | 431 | `\| `` \| `crates/contextra-cognition/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Contextra Cognition Engine (Layer 3 - Cognition). \|` |
| `docs/CHANGELOG.md` | 432 | `\| `` \| `crates/contextra-cognition/src/maintenance_scheduler.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Zentraler MaintenanceScheduler für koordiniertes, sequenzielles Ausführen aller Background-Maintenance-Prozesse (ADR-079). \|` |
| `docs/CHANGELOG.md` | 433 | `\| `` \| `crates/contextra-cognition/src/memory_consolidation.rs` \| `AI-TAG` \| `AGT-DB-660fbb5f` \| `-` \| `RESOLVED` \| `0` \| /// AI-TAG[SLEEP][MINOR] RESOLVED: AGT-DB-660fbb5f — Position im turns-Slice wird anstelle des \|` |
| `docs/CHANGELOG.md` | 434 | `\| `` \| `crates/contextra-cognition/src/semantic_aggregation_facade.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Öffentliche Facade-Schnittstelle für LeanRAG Stage-3-Hyperkantenaggregatierung (Spec §5.3, §12.4). \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 722 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-cognition`, `contextra-graph`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 732 | `Test (mit Feature docid-128): DocId A = 1, DocId B = (1<<64)+1 und C = (2<<64)+1 mit verschiedenen Vektoren einfügen. Beide/alle müssen einzeln auffindbar (search), einzeln löschbar (delete/remove_with_graph_repair) und mit korrekter all_doc_ids/len-Statistik sein. Löschen von B darf A und C nicht beeinflussen. Gleicher Test für DiskANN (sofern per Feature baubar), contextra-graph⚠ (CSR/Knoten-IDs), contextra-text (Dokument-IDs) und contextra-cognition⚠. Danach Mustersuche (R7): rg nach `as u64`, `as usize`, `as u32`, `.0 as`, `inner() as` in allen Crates und liste JEDE Stelle, an der eine DocId-/u128-Größe verengt wird, mit Bewertung (harmlos/Kollision/Datenverlust).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1026 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-kvcache`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1046 | `9) Geschwister-Schichten einzeln: HNSW (Vektor weg + keine Geisterzeiger: verify_no_ghost_pointers, INV-DELETION-2), contextra-text (Posting-Listen und Dokumentstatistik/IDF aktualisiert?), contextra-graph⚠ (Knoten/Kanten/Hyperkanten und abgeleitete Community-/Konsolidierungsergebnisse), contextra-cognition⚠ (konsolidierte Zusammenfassungen oder Embeddings, die den Inhalt abgeleitet enthalten — hier kann personenbezogene Information fortleben, obwohl das Original gelöscht ist: Befund S2, falls so), contextra-kvcache⚠ (gecachte Tensoren zum Prompt, kv-encryption), contextra-checkpoint⚠ (Blake3-Manifest enthält Hashes, keine Klartexte?).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-core`

- **Gesamtanzahl Referenzen:** 35 (Imports: 0, Usages: 0, Config/CI: 26, Docs: 9)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `.agents/jules/tasks/_TEMPLATE.toml` | 5 | `crate     = "contextra-core"` |
| `.agents/jules/tasks/_TEMPLATE.toml` | 10 | `scope     = ["crates/contextra-core/src/**"]` |
| `.agents/jules/tasks/_TEMPLATE.toml` | 14 | `"cargo test -p contextra-core",` |
| `deny.toml` | 53 | `"contextra-core", # TRANSITION E29, bis 2027-04-30, REQ-ASYNC-1` |
| `parse_all_remnants.py` | 12 | `"contextra-core",` |
| `scripts/contextra_full_audit.py` | 101 | `"contextra-core", "contextra-db", "contextra-graph", "contextra-infer-candle",` |
| `scripts/contextra_full_audit.py` | 114 | `"contextra-privacy", "contextra-mcp", "contextra-wire", "contextra-core",` |
| `scripts/contextra_full_audit.py` | 1151 | `"contextra-mcp", "contextra-wire", "contextra-core", "contextra-bench", "contextra-py",` |
| `tools/decouple_vector_core.py` | 73 | `lines.append(f"use contextra_core::{{{', '.join(unknown_imports)}}};")` |
| `tools/decouple_vector_core.py` | 83 | `pattern = re.compile(r"use\s+contextra_core::(\{[^}]+\}\|[a-zA-Z0-9_:]+);")` |
| `tools/decouple_vector_core.py` | 86 | `content = content.replace("contextra_core::ContextraError", "contextra_types::ContextraError")` |
| `tools/decouple_vector_core.py` | 87 | `content = content.replace("contextra_core::DocId", "contextra_types::DocId")` |
| `tools/decouple_vector_core.py` | 88 | `content = content.replace("contextra_core::TxId", "contextra_types::TxId")` |
| `tools/decouple_vector_core.py` | 89 | `content = content.replace("contextra_core::DistanceMetric", "contextra_types::DistanceMetric")` |
| `tools/decouple_vector_core.py` | 90 | `content = content.replace("contextra_core::Result", "contextra_types::Result")` |
| `tools/decouple_vector_core.py` | 91 | `content = content.replace("contextra_core::ScoredDocument", "contextra_types::ScoredDocument")` |
| `tools/decouple_vector_core.py` | 92 | `content = content.replace("contextra_core::MAX_SEARCH_K", "contextra_types::MAX_SEARCH_K")` |
| `tools/decouple_vector_core.py` | 93 | `content = content.replace("contextra_core::SequenceLog", "contextra_mvcc::SequenceLog")` |
| `tools/decouple_vector_core.py` | 94 | `content = content.replace("contextra_core::SeqLogChange", "contextra_mvcc::SeqLogChange")` |
| `tools/decouple_vector_core.py` | 95 | `content = content.replace("contextra_core::TxBuffer", "contextra_mvcc::TxBuffer")` |
| `tools/decouple_vector_core.py` | 96 | `content = content.replace("contextra_core::VectorIndex", "contextra_ports::VectorIndex")` |
| `tools/decouple_vector_core.py` | 97 | `content = content.replace("contextra_core::traits::VectorIndex", "contextra_ports::VectorIndex")` |
| `tools/generate_migration_edges.py` | 7 | `contextra-adapt, contextra-agent, contextra-cognition, contextra-core,` |
| `tools/generate_migration_edges.py` | 27 | `'contextra-core': 'contextra_core',` |
| `tools/generate_migration_edges.py` | 39 | `'contextra-core': 'Mechanische Ersetzung `use contextra_core::` → `use contextra_types::` (K-11: ~286 Dateien). Nach `cargo check` Crate und Eintrag in `Cargo.toml` löschen (D-05).',` |
| `tools/generate_migration_edges.py` | 55 | `return 'WP 2.3' if dep_name in ('contextra-py', 'contextra-core') else 'WP 2.4'` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-mvcc/src/lib.rs` | 21 | `//! - `contextra-core`: Exportiert Kern-MVCC-Fähigkeiten für übergeordnete Abfrage-Pipelines.` |
| `crates/contextra-store/src/lsm/mod.rs` | 11 | `// IMPLEMENTS: StorageEngine Trait (contextra-core/src/traits.rs)` |
| `crates/contextra-types/src/lib.rs` | 23 | `//!   `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`,` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1280 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-core`, `contextra-graph`, `contextra-kvcache`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1291 | `SCHRITT 3 — Miri (ergänzend, nicht im Projekt-justfile): cargo +nightly miri test -p <crate> für die reinen Safe-Rust-Crates mit überschaubarer Laufzeit (contextra-types, contextra-mvcc, contextra-ports, contextra-rank, Teile von contextra-core⚠/text/graph). Miri unterstützt Dateisystem, mmap und SIMD-Intrinsics nur eingeschränkt: Tests, die daran scheitern, als "Miri nicht unterstützt" listen — NICHT als Fehler und NICHT durch Code- oder Test-Änderungen umgehen. Setze MIRIFLAGS="-Zmiri-seed=<n>" und wiederhole mit 3 Seeds; für Nebenläufigkeit MIRIFLAGS="-Zmiri-preemption-rate=0.1".` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 933 | `D4 PEER-IMPORT SCAN: Ring-0-Crates dürfen keine anderen Ring-0-Crates importieren außer contextra-types/contextra-ports. Prüfe: grep -rn "^use contextra_" crates/contextra-store/src/ — store ist Ring 1, darf contextra-core⚠ importieren aber nicht contextra-vector (Ring-0-Peer).` |

### Crate `contextra-db`

- **Gesamtanzahl Referenzen:** 68 (Imports: 7, Usages: 1, Config/CI: 7, Docs: 53)

#### Aktive Code-Stellen
| Datei | Zeile | Typ | Quelltext |
|---|---|---|---|
| `benchmarks/competitive_bench.rs` | 4 | `IMPORT` | `use contextra_db::context_compaction::{CompactionStrategy, ContextCompactor};` |
| `benchmarks/competitive_bench.rs` | 5 | `IMPORT` | `use contextra_db::Contextra;` |
| `benchmarks/migration_benchmarks.rs` | 6 | `IMPORT` | `use contextra_db::{Contextra, ContextraConfig};` |
| `benchmarks/rrf_scale_bench.rs` | 1 | `IMPORT` | `use contextra_db::fusion::weighted_reciprocal_rank_fusion;` |
| `benchmarks/rrf_scale_bench.rs` | 2 | `IMPORT` | `use contextra_db::SearchResult;` |
| `benchmarks/scale_bench.rs` | 5 | `IMPORT` | `use contextra_db::Contextra;` |
| `crates/contextra-engine/src/collection/mod.rs` | 99 | `USAGE` | `#[deprecated(note = "Parsing LLM responses should happen outside contextra-db")]` |
| `crates/contextra/tests/llm_query_rewriter.rs` | 15 | `IMPORT` | `use contextra_db::QueryRewriter;` |

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 55 | `"contextra-db",` |
| `justfile` | 79 | `"contextra-db:rrf_fusion"` |
| `parse_all_remnants.py` | 13 | `"contextra-db",` |
| `scripts/contextra_full_audit.py` | 101 | `"contextra-core", "contextra-db", "contextra-graph", "contextra-infer-candle",` |
| `scripts/contextra_full_audit.py` | 1145 | `"contextra-db": 4, "contextra": 4,` |
| `scripts/contextra_full_audit.py` | 1149 | `"contextra-engine", "contextra-db", "contextra"]                          # Final 2.2 "Behalten"` |
| `tools/loc.py` | 26 | `"contextra-db",` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `.agents/jules/SCHEDULED_AUDIT.md` | 15 | `- **Fokus-Crates.** Die Vorfassung nennt `contextra-graph`, `contextra-agent` und `contextra-db`. Diese Crates sind nach SPEC D-24 und D-45 entfernt. Die Rotation muss vorhandene Crates verwenden, etwa `contextra-store`, `contextra-crypto`, `contextra-vector` und `contextra-text`. Ab Gate G-D kommt `contextra-erasure` hinzu.` |
| `crates/contextra-crypto/src/lib.rs` | 20 | `//! - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`,` |
| `crates/contextra-store/src/lib.rs` | 25 | `//! - **Eingehende Abhaengigkeiten:** Wird genutzt von `contextra-engine`, `contextra-db`,` |
| `crates/contextra-testkit/src/lib.rs` | 27 | `//!   `contextra-cognition`, `contextra-crypto`, `contextra-db`, `contextra-engine`,` |
| `crates/contextra-text/src/lib.rs` | 12 | `//! `fusion.rs` layer in `contextra-db`.` |
| `crates/contextra-text/src/lib.rs` | 23 | `//! - **Wird genutzt von**: `contextra-engine`, `contextra-db`, `contextra-router`, `contextra-agent`` |
| `crates/contextra-types/src/error.rs` | 25 | `// DOWNSTREAM: contextra-store, contextra-index, contextra-db konvertieren via `?` und `From`.` |
| `crates/contextra-types/src/lib.rs` | 23 | `//!   `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`,` |
| `crates/contextra-types/src/types/domain/ids.rs` | 219 | `/// Um stille Datenkorruption zu verhindern, führt die Orchestrierungsschicht (`contextra-db::Collection`)` |
| `crates/contextra-types/src/types/domain/misc.rs` | 80 | `/// Callers in contextra-mcp and contextra-db both enforce this limit before forwarding k to HNSW/BM25.` |
| `crates/contextra-types/src/types/domain/misc.rs` | 83 | `/// This cap is applied at the orchestration layer (contextra-db) before forwarding `k`` |
| `crates/contextra-types/src/types/filter.rs` | 17 | `// NICHT-OFFENSICHTLICH: Legacy MetadataFilter aus contextra-db konvertiert verlustfrei in FilterExpr.` |
| `crates/contextra-types/src/types/saos.rs` | 270 | `/// Verwendet für Budget-Management in `contextra-db/src/context.rs`.` |
| `crates/contextra/src/query_rewriter.rs` | 13 | `//!   erweitert den Suchfluss von `contextra-db` um semantische Abfrageerweiterung.` |
| `docs/CHANGELOG.md` | 130 | `\| `TS:2026-09-12T00:00:00Z` \| `crates/contextra-db/src/multistep.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Multi-Step Iterative Retrieval Engine für komplexe Agenten-Abfragen (o-series Pattern). \|` |
| `docs/CHANGELOG.md` | 131 | `\| `TS:2026-09-12T00:00:00Z` \| `crates/contextra-db/tests/consolidation_double_trigger_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrierter Test zur Verifizierung des Double-Trigger-Schutzes zwischen ConsolidationEngine und MaintenanceScheduler. \|` |
| `docs/CHANGELOG.md` | 162 | `\| `TS:2026-09-05T00:00:00Z (SESSION: 0dcb9f3b)` \| `crates/contextra-db/tests/no_blanket_allow_deprecated.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Automated regression guard ensuring no blanket crate-level `#![allow(deprecated)]` attribute is reintroduced. \|` |
| `docs/CHANGELOG.md` | 168 | `\| `TS:2026-08-31T23:10:00Z (SESSION: 0dcb9f3b)` \| `crates/contextra-db/tests/cross_signal_isolation_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prüft Cross-Signal Snapshot-Isolation in contextra-db während paralleler Writes/Updates. \|` |
| `docs/CHANGELOG.md` | 169 | `\| `TS:2026-08-31T22:30:00Z (SESSION: 0dcb9f3b)` \| `crates/contextra-db/tests/fault_injection_2pc.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prüft atomare 2PC-Transaktions-Kompensation und Crash-Recovery (repair_on_open) über alle 4 Sub-Engines. \|` |
| `docs/CHANGELOG.md` | 284 | `\| `2026-09-15` \| `crates/contextra-db/tests/proptest_search_invariants.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Property-based Tests für Retrieval- & Search-Invarianten in contextra-db (RRF, BM25, End-to-End). \|` |
| `docs/CHANGELOG.md` | 324 | `\| `2026-09-09T22:00:00Z` \| `crates/contextra-db/tests/split_brain_vector_orphan_test.rs` \| `ANCHOR` \| `TEST:SPLIT_BRAIN_CONSISTENCY` \| `504d02fc` \| `IN_PROGRESS` \| `0` \| // ANCHOR[TEST:SPLIT_BRAIN_CONSISTENCY] STATUS:IN_PROGRESS (TS:2026-09-09T22:00:00Z) (SESSION: 504d02fc) \|` |
| `docs/CHANGELOG.md` | 408 | `\| `2026-08-29T18:30:00Z` \| `crates/contextra-db/tests/fusion_edge_cases_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Standalone Randfall- & Mutation-Testsuite für Reciprocal Rank Fusion (fusion.rs). \|` |
| `docs/CHANGELOG.md` | 410 | `\| `2026-08-29T11:00:00Z` \| `crates/contextra-db/tests/semantic_recall.rs` \| `REVIEW-PASS` \| `PERF:EVAL-001` \| `c9f5e2b3` \| `PASS` \| `1` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: PERF:EVAL-001) (TS: 2026-08-29T11:00:00Z) (SESSION: c9f5e2b3) \|` |
| `docs/CHANGELOG.md` | 411 | `\| `2026-08-29T10:00:00Z` \| `crates/contextra-db/tests/semantic_recall.rs` \| `REVIEW-PASS` \| `PERF:EVAL-001` \| `b8e4f1a2` \| `PASS` \| `1` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: PERF:EVAL-001) (TS: 2026-08-29T10:00:00Z) (SESSION: b8e4f1a2) \|` |
| `docs/CHANGELOG.md` | 413 | `\| `2026-08-29T00:00:00Z` \| `crates/contextra-db/tests/semantic_recall.rs` \| `ANCHOR` \| `PERF:EVAL-001` \| `a3f29c1d` \| `DONE` \| `2` \| // ANCHOR[PERF:EVAL-001] STATUS:DONE (TS:2026-08-29T00:00:00Z) (SESSION: a3f29c1d) — Semantic Retrieval Evaluation Framework \|` |
| `docs/CHANGELOG.md` | 421 | `\| `2026-06-20T00:00:00Z` \| `crates/contextra-db/tests/checkpoint_stress.rs` \| `ANCHOR` \| `INTEGRATION:CHECKPOINT-STRESS` \| `-` \| `DONE` \| `0` \| // ANCHOR[INTEGRATION:CHECKPOINT-STRESS] STATUS:DONE (TS:2026-06-20T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 424 | `\| `2026-05-18T00:00:00Z` \| `crates/contextra-db/tests/concurrent_collection_stress.rs` \| `ANCHOR` \| `INTEGRATION:STRESS-001` \| `-` \| `DONE` \| `0` \| // ANCHOR[INTEGRATION:STRESS-001] STATUS:DONE (TS:2026-05-18T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 425 | `\| `2026-05-18T00:00:00Z` \| `crates/contextra-db/tests/full_stack_e2e.rs` \| `ANCHOR` \| `INTEGRATION:E2E-001` \| `-` \| `DONE` \| `0` \| // ANCHOR[INTEGRATION:E2E-001] STATUS:DONE (TS:2026-05-18T00:00:00Z) \|` |
| `docs/CHANGELOG.md` | 455 | `\| `` \| `crates/contextra-db/src/homeostat.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management. \|` |
| `docs/CHANGELOG.md` | 456 | `\| `` \| `crates/contextra-db/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Contextra Database Strangler Shell / Re-export Facade (Layer 3). \|` |
| `docs/CHANGELOG.md` | 457 | `\| `` \| `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \|  \|` |
| `docs/CHANGELOG.md` | 458 | `\| `` \| `crates/contextra-db/tests/diskann_collection_fallback.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Collection Integration Fallback Test (Pflichttest 3). \|` |
| `docs/CHANGELOG.md` | 459 | `\| `` \| `crates/contextra-db/tests/embedding_dimension_switch.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \|  \|` |
| `docs/CHANGELOG.md` | 460 | `\| `` \| `crates/contextra-db/tests/hnsw_delete_and_backfill.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Verifiziert, dass ein Fehler beim HNSW-Delete nicht verschluckt wird und Vektorsuchen bei Tombstones durch Backfill k valide Ergebnisse liefern. \|` |
| `docs/CHANGELOG.md` | 461 | `\| `` \| `crates/contextra-db/tests/multistep_query_rewriter_modalities.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Testet Multi-Modal Sub-Queries (Text, Vektor, Graph) im Multi-Step Retrieval mit Mock-Rewriter und Mock-Embedder. \|` |
| `docs/CHANGELOG.md` | 462 | `\| `` \| `crates/contextra-db/tests/pid_pool_production_caller.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prüft die PID-regulatorische Einbindung in den Multi-Step Candidate Pool und Grenzwerte [50, 200]. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 878 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-db`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 889 | `cargo bench -p contextra-db⚠ --bench scale_bench -- --warm-up-time 1 --measurement-time 3 --sample-size 20` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 890 | `cargo bench -p contextra-db⚠ --bench rrf_scale_bench -- (gleiche Zeitparameter)` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 891 | `cargo bench -p contextra-db⚠ --bench competitive_bench -- (gleiche Zeitparameter; benötigt ggf. Vergleichs-Engines als Dev-Dependencies — Netz/Build-Fehler als ENV melden)` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 892 | `cargo bench -p contextra-db⚠ --bench migration_benchmarks -- (gleiche Zeitparameter)` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1388 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-db`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1399 | `LAST: gemischter Workload über die Facade bzw. contextra-db⚠: Inserts mit Embedding (mock-Provider), hybride Suche (Vektor + Text + Graph), Updates, Löschungen (inkl. Löschbeweis im sovereign-Ring in separatem zweiten Lauf), contextra_consolidate in Intervallen, Checkpoints alle 10 Minuten, Hintergrundwartung (Feature background-maintenance). Datenmenge wächst bis zu einem definierten Plattenlimit und wird dann durch Löschungen im Gleichgewicht gehalten (steady state).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/SPEC.md` | 172 | `\|F-35\|**Matrix und Ringköpfe am HEAD `4e208c179`:** `vector→sys` und `vector→mvcc` (verboten, D-88); `engine` ohne `durable-fs` und `erasure`; `text` ohne `crypto`; `store` hat `vector` als Dev-Abhängigkeit (AR-2 dem Sinn nach); Selbst-Dev-Abhängigkeiten zum Aktivieren von Features (`store`, `crypto`, `contextra`). Die `//!`-Köpfe nennen falsche Ringe in 8 Crates (`crypto` 0 statt 1, `mvcc` 0 statt 2, `simd` 0 statt 1, `sys` 0 statt 1, `store` 1 statt 2, `text` 0 statt 2, `rank` 0 statt 2, `engine` 2 statt 3) und keine Angabe in `vector`; `text` und `rank` nennen Verbraucher, die nicht existieren (`contextra-db`, `-router`, `-agent`, `-bench`, `-infer-*`, `-mcp`, `-py`). `tests/arch` enthält die richtige Ringtabelle, aber keine Prüfung T-ARCH-5..8. **[V]**\|`Cargo.toml` aller Crates, `lib.rs`, `tests/arch/src/lib.rs`\|` |
| `docs/SPEC.md` | 228 | `\|D-24\|Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator, `contextra-db`, Wasm-Merge-Operator sowie die Ports `embedding`, `reranker`, `query_rewriter` entfallen.\|Nicht im Kern.\|T-ARCH-3\|` |
| `docs/SPEC.md` | 249 | `\|D-45\|`contextra-db` entfällt; `homeostat`, `multistep`, `volatile_vault` sind nicht im Kern.\|Features leiten an entfernte Crates.\|T-ARCH-3\|` |
| `docs/analysis/SPEZIFIKATION_LOESCHUNG_PRODUKTIONSREIFE_2026-11-01.md` | 124 | `- **Architektur-Test (Behoben):** `tests/arch/src/lib.rs` enthielt im Test `erlaubt_abwaerts_abhaengigkeit` noch einen Verweis auf das archivierte Crate `contextra-db`. Dieser wurde erfolgreich auf das Ring-4-Crate `contextra` korrigiert; alle Architekturtests bestehen nun zu 100%.` |
| `docs/specs/06-contextra-crypto.md` | 24 | `\|Kopf\|`lib.rs`: Ring 0 und Verbraucherliste mit nicht vorhandenen Crates (`contextra-db`, `-infer-candle`, `-kvcache`, `-mcp`, `-privacy`); Dateikopf-Marken mit Zeitstempel und Sitzungskennung (Q-09)\|` |
| `docs/specs/09-contextra-text.md` | 24 | `\|Manifest\|`bincode`, `serde_json`, `regex`, `futures`, `itoa`, `bytes`, `parking_lot`, `blake3`; **`crypto` fehlt** (F-35); `ring = "0"`; `lib.rs` nennt Ring 0 und Verbraucher `contextra-db`, `-router`, `-agent`, `-bench`\|` |
| `docs/specs/09-contextra-text.md` | 106 | `\|Manifest: `ring = "0"`, fehlendes `crypto`, tote Abhängigkeiten; `lib.rs`-Kopf (Ring 0, `contextra-db`, …)\|TX-14; Kopf nach Q-01\|WP-1.5\|T-ARCH-5\|` |

### Crate `contextra-graph`

- **Gesamtanzahl Referenzen:** 112 (Imports: 48, Usages: 4, Config/CI: 13, Docs: 47)

#### Aktive Code-Stellen
| Datei | Zeile | Typ | Quelltext |
|---|---|---|---|
| `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` | 13 | `IMPORT` | `use contextra_graph::hyperedge::HyperEdgeId;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 12 | `IMPORT` | `use contextra_graph::hyperedge::HyperEdgeId;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 109 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 171 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 237 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 395 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 504 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 558 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 601 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/background_workers/tests.rs` | 694 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 12 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 76 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 198 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 242 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 286 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 375 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 492 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 554 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | 593 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | 12 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | 81 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | 132 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | 183 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | 234 | `IMPORT` | `use contextra_graph::csr::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | 12 | `IMPORT` | `use contextra_graph::csr::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | 63 | `IMPORT` | `use contextra_graph::csr::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | 174 | `IMPORT` | `use contextra_graph::csr::{CsrGraph, CsrGraphConfig};` |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | 314 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | 411 | `USAGE` | `let config = contextra_graph::percolation::PercolationConfig {` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 34 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 140 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 352 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 404 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 455 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 497 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | 545 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/read_tests.rs` | 12 | `IMPORT` | `use contextra_graph::csr::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/read_tests.rs` | 155 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/read_tests.rs` | 289 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 12 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 66 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 136 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 203 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 251 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | 299 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/src/transaction/db_transaction.rs` | 30 | `USAGE` | `pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,` |
| `crates/contextra-engine/src/transaction/db_transaction.rs` | 150 | `USAGE` | `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge) {` |
| `crates/contextra-engine/src/transaction/tests.rs` | 11 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/tests/commit_crash_consistency.rs` | 16 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/tests/engine_commit_crash_points.rs` | 10 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/tests/engine_recovery_crash_windows.rs` | 15 | `IMPORT` | `use contextra_graph::CsrGraph;` |
| `crates/contextra-engine/tests/ssi_read_path_coverage.rs` | 185 | `USAGE` | `let graph = Arc::new(contextra_graph::CsrGraph::new());` |

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `CODEOWNERS` | 27 | `/crates/contextra-graph/               @contextra/intelligence-tier3` |
| `deny.toml` | 57 | `"contextra-graph", # TRANSITION E29, bis 2027-04-30, REQ-ASYNC-1` |
| `parse_all_remnants.py` | 14 | `"contextra-graph",` |
| `scripts/contextra_full_audit.py` | 101 | `"contextra-core", "contextra-db", "contextra-graph", "contextra-infer-candle",` |
| `scripts/contextra_full_audit.py` | 117 | `"contextra-cognition", "contextra-adapt", "contextra-graph", "contextra-license"}` |
| `scripts/contextra_full_audit.py` | 119 | `"contextra-graph": 3600, "contextra-engine": 3600}` |
| `scripts/contextra_full_audit.py` | 1154 | `ENTKOPPELN_V1 = ["contextra-graph", "contextra-license", "contextra-checkpoint"]     # Entscheidungsgrundlage 6.2` |
| `scripts/contextra_full_audit.py` | 1157 | `RING_EXTERN = {"contextra-graph", "contextra-license", "contextra-testkit"}           # nur ueber Ports / optional` |
| `scripts/contextra_full_audit.py` | 1533 | `"FEHLT" if deps.get("contextra-engine", {}).get("contextra-graph") is False else "PASS",` |
| `scripts/contextra_full_audit.py` | 1534 | `str(deps.get("contextra-engine", {}).get("contextra-graph", "nicht abhaengig")), "optional", "Umsetzungsplan 7 G2")` |
| `tools/generate_migration_edges.py` | 8 | `contextra-graph, contextra-infer-ollama, contextra-infer-onnx,` |
| `tools/generate_migration_edges.py` | 28 | `'contextra-graph': 'contextra_graph',` |
| `tools/generate_migration_edges.py` | 40 | `'contextra-graph': '`engine/src/collection/crud/links.rs` löschen; graph-bezogene Funktionen (`stage_graph_entity_delete`, `check_and_trigger_community_detection`, `CommunityDetectionConfig`, PPR-Pfade im Hybrid-Planner) entfernen; Modalität als `modalities_unavailable` markieren.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `.agents/jules/SCHEDULED_AUDIT.md` | 15 | `- **Fokus-Crates.** Die Vorfassung nennt `contextra-graph`, `contextra-agent` und `contextra-db`. Diese Crates sind nach SPEC D-24 und D-45 entfernt. Die Rotation muss vorhandene Crates verwenden, etwa `contextra-store`, `contextra-crypto`, `contextra-vector` und `contextra-text`. Ab Gate G-D kommt `contextra-erasure` hinzu.` |
| `crates/contextra-engine/src/transaction/mod.rs` | 18 | `//! BM25 inverted text index (`contextra-text`), and CSR graph index (`contextra-graph`).` |
| `crates/contextra-store/src/lib.rs` | 26 | `//!   `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra-agent`,` |
| `crates/contextra-types/src/lib.rs` | 24 | `//!   `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`,` |
| `crates/contextra-types/src/types/domain/misc.rs` | 180 | `/// Im Gegensatz dazu ist `contextra_graph::csr::types::Edge` (bzw. `CsrEdge`) ein speicherkompakter` |
| `crates/contextra-types/src/types/domain/misc.rs` | 374 | `/// Defined in `contextra-types` (Ring 0) to avoid dependency on `contextra-graph` (Ring 1).` |
| `crates/contextra-types/src/types/domain/misc.rs` | 375 | `/// Graph algorithms in `contextra-graph` map this configuration to `contextra_graph::TlHfdParams`.` |
| `crates/contextra-types/src/types/domain/misc.rs` | 381 | `/// Im Gegensatz dazu ist `contextra_graph::tl_hfd::params::TlHfdParams` die Graph-Engine-Spezifische` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 22 | `\| Z. 42–56 \| `BENCHES` listet Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). \| Schritt `befund_bench_fuzz_dateien` schlägt fehl (FAIL); Benchmarks für diese Crates können nicht existieren. \|` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 26 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthält noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldet FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. Dies ist kein Code-Bug, sondern eine nicht synchronisierte Konfigurationsdatei. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 24 | `\| Z. 42–56 \| `BENCHES` enthielt Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench` sowie `deletion_proof_latency_bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). Die Benchmarks existieren nicht mehr. \| Schritt `befund_bench_fuzz_dateien` und Benchmarks scheiterten mit FAIL. \| `BENCHES` auf die existierenden Benchmarks der aktiven Crates (`mvcc`, `crypto`, `vector`, `store`, `simd`) bereinigt. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 28 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthielt noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldete FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. \| `capabilities.toml` synchronisiert (Entfernung von 21 Nicht-Mitgliedern, Aufnahme von `contextra-durable-fs` und `contextra-arch-tests`). \|` |
| `docs/CHANGELOG.md` | 72 | `- Repository auf Kern v1 reduziert (Commit `ed402e21a`). Entfernt: `contextra-graph`, `-adapt`, `-agent`, `-cognition`, `-infer-*`, `-ingest`, `-mcp`, `-privacy`, `-wire`, `-checkpoint`, `-audit-export`, `-avv-generator`, `-kvcache`, `-router`, `-sandbox`, `-py`, `-license`, `-core`; `xtask/` (Commit `068261abd`).` |
| `docs/CHANGELOG.md` | 101 | `\| `TS:2026-09-28T00:00:00Z` \| `crates/contextra-graph/src/apprh/gate_monitor.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| ApprhGateMonitor zur Serienbewertung von APPRH-Shadow-Vergleichen fuer Produktionsfreigabe (P1.3) \|` |
| `docs/CHANGELOG.md` | 102 | `\| `TS:2026-09-28T00:00:00Z` \| `crates/contextra-graph/src/apprh/selector.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| ApprhSelector zur Steuerung von Shadow-Mode-Abtastung und expliziter Produktionsschaltung (P1.3) \|` |
| `docs/CHANGELOG.md` | 193 | `\| `TS:2026-08-30T19:00:00Z` \| `crates/contextra-graph/src/cascade.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Cascading-Invalidation (Supersedes-Chunk -> Graph-Kanten-Tombstone) \|` |
| `docs/CHANGELOG.md` | 218 | `\| `2026-09-28` \| `crates/contextra-graph/src/path_rag/snapshot.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Multi-Version Concurrency Control (MVCC) snapshot-isolated PathGraph for PathRAG. \|` |
| `docs/CHANGELOG.md` | 239 | `\| `2026-09-24` \| `crates/contextra-graph/src/ppr/snapshot.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Multi-Version Concurrency Control (MVCC) snapshot-isolated PPR computation. \|` |
| `docs/CHANGELOG.md` | 249 | `\| `2026-09-18T12:00:00Z` \| `crates/contextra-graph/src/csr/graph_read.rs` \| `AI-TAG` \| `-` \| `e095d708` \| `RESOLVED` \| `-` \| // AI-TAG[RESOLVED][IP-08-BUDGET-COUPLING](TS:2026-09-18T12:00:00Z)(SESSION:e095d708): Connected to global ResourceTracker when cross-crate tracker handle is configured. \|` |
| `docs/CHANGELOG.md` | 271 | `\| `2026-09-15T16:07:56Z (SESSION: 5d958ef0)` \| `crates/contextra-graph/src/edge_reinforcement_buffer.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Akkumuliert Co-Occurrence- und Traversal-Update-Signale für asynchronen Flush in CSR-Graph (F-03) \|` |
| `docs/CHANGELOG.md` | 315 | `\| `2026-09-10T19:23:40Z` \| `crates/contextra-graph/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `bd6ff800` \| `PASS` \| `-` \| // REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-10T19:23:40Z) (SESSION: bd6ff800) \|` |
| `docs/CHANGELOG.md` | 323 | `\| `2026-09-10T00:00:00Z (SESSION: 61b93a6f)` \| `crates/contextra-graph/src/provenance.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Herkunftsnachweis für Graph-Kanten (DocEdgeIndex & EdgeProvenance) \|` |
| `docs/CHANGELOG.md` | 382 | `\| `2026-08-30T19:30:00Z (SESSION: b1234567)` \| `crates/contextra-graph/src/community.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Community-Erkennung via Leiden-Algorithmus für GraphRAG \|` |
| `docs/CHANGELOG.md` | 394 | `\| `2026-08-30T18:53:58Z (SESSION: b1234567)` \| `crates/contextra-graph/src/ppr.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Personalized PageRank Power Iteration über CSR Graph \|` |
| `docs/CHANGELOG.md` | 395 | `\| `2026-08-30T18:53:58Z (SESSION: b1234567)` \| `crates/contextra-graph/src/session_dag.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Session-DAG für persistente, azyklische Agent-State-Verzweigung (Contextra Session-DAG Pattern) \|` |
| `docs/CHANGELOG.md` | 489 | `\| `` \| `crates/contextra-graph/src/entity_extraction.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Regelbasierte Entitäts- und Relations-Extraktion aus Freitext mit Graph-Ingestion-Anbindung. \|` |
| `docs/CHANGELOG.md` | 490 | `\| `` \| `crates/contextra-graph/tests/entity_extraction_tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrations- und Evaluationstests für die regelbasierte Entitäts- und Relations-Extraktion in contextra-graph. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 430 | `contextra-text: bm25f · contextra-graph⚠: ppr-forward-push, k-path-diffusion, graph-connectivity-health, edge-reinforcement-learning, docid-128` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 455 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-graph`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 473 | `contextra-graph⚠: CSR-Aufbau gegen Adjazenz-Map-Referenz (Nachbarn, Grade, Pfade/BFS-Distanzen); PageRank/PPR: Masse bleibt ~1, alle Werte >= 0, deterministisch; Selbstschleifen, Mehrfachkanten, isolierte Knoten, leerer Graph; TxId::is_valid_origin() wird auf allen Graph-Operationen erzwungen (AGT-GRAPH-001).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 722 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-cognition`, `contextra-graph`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 732 | `Test (mit Feature docid-128): DocId A = 1, DocId B = (1<<64)+1 und C = (2<<64)+1 mit verschiedenen Vektoren einfügen. Beide/alle müssen einzeln auffindbar (search), einzeln löschbar (delete/remove_with_graph_repair) und mit korrekter all_doc_ids/len-Statistik sein. Löschen von B darf A und C nicht beeinflussen. Gleicher Test für DiskANN (sofern per Feature baubar), contextra-graph⚠ (CSR/Knoten-IDs), contextra-text (Dokument-IDs) und contextra-cognition⚠. Danach Mustersuche (R7): rg nach `as u64`, `as usize`, `as u32`, `.0 as`, `inner() as` in allen Crates und liste JEDE Stelle, an der eine DocId-/u128-Größe verengt wird, mit Bewertung (harmlos/Kollision/Datenverlust).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 905 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-graph`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 917 | `c) contextra-graph⚠ CSR: Aufbauzeit, Nachbarschaftsabfrage, PPR bei 10k, 40k, 160k, 640k Kanten.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1026 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-kvcache`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1046 | `9) Geschwister-Schichten einzeln: HNSW (Vektor weg + keine Geisterzeiger: verify_no_ghost_pointers, INV-DELETION-2), contextra-text (Posting-Listen und Dokumentstatistik/IDF aktualisiert?), contextra-graph⚠ (Knoten/Kanten/Hyperkanten und abgeleitete Community-/Konsolidierungsergebnisse), contextra-cognition⚠ (konsolidierte Zusammenfassungen oder Embeddings, die den Inhalt abgeleitet enthalten — hier kann personenbezogene Information fortleben, obwohl das Original gelöscht ist: Befund S2, falls so), contextra-kvcache⚠ (gecachte Tensoren zum Prompt, kv-encryption), contextra-checkpoint⚠ (Blake3-Manifest enthält Hashes, keine Klartexte?).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1280 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-core`, `contextra-graph`, `contextra-kvcache`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1290 | `SCHRITT 2 — Gezielt für die kritischsten Crates einzeln mit höherem Parallelitätsgrad: contextra-store (Group-Commit, Memtable-Sharding, Observer), contextra-vector (HNSW Insert/Search parallel, ComputePool), contextra-mvcc, contextra-engine, contextra-kvcache⚠, contextra-text, contextra-graph⚠. Befehlsmuster: RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test -Zbuild-std --target <host-triple> -p <crate> -- --test-threads=<Anzahl Kerne> (Host-Triple per rustc -vV ermitteln). Wiederhole TSan-Läufe 3x (Datenrennen sind probabilistisch).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/ROOT_CAUSE_ANALYSIS.md` | 19 | `Dabei wurden die entsprechenden Crates (`contextra-graph`, `contextra-checkpoint` etc.) aus dem Workspace entfernt und das Feld `graph_index` aus `Collection` gelöscht.` |
| `docs/SPEC.md` | 142 | `\|F-05\|Rückbau halb fertig: Workspace listet 15 Mitglieder (ohne `contextra-erasure`). **100 Zeilen** verweisen auf `contextra_graph`/`contextra-graph` (33c3e78e4: 101). Der Commit `c2a864d` hat die Ports `graph`, `graph_index`, `checkpoint`, `license`, `attention`, `plugin`, `lifecycle` entfernt; `ports` führt weiter `embedding`, `reranker`, `query_rewriter` (D-24: zu entfernen). `AutoExtractionMode` in 10+ Dateien. **[V]**\|`grep -rIn "contextra_graph\\|contextra-graph" crates tests tools .github Cargo.toml Cargo.lock`\|` |
| `docs/SPEC.md` | 168 | `\|F-31\|**Umfang gegenüber D-24/D-26/D-84 (Zeilen `src`, inkl. Inline-Tests, Obergrenzen):** `vector`: `diskann` 4.122, `quantize` 1.116, `quantize_rabitq` 506, `partial_rebuild` 363, `acorn` 213; `rank` gesamt 3.440 (`fusion` 2.252, `dibud` 524, `explain` 469, `drift` 139, `lib` 56; die Angabe 5.553 in 4.1.0 nannte keine Zählmethode und stimmt mit den Teilsummen nicht überein) bei Soll „nur RRF“; `types/types/saos.rs` 940; `engine`: `chunker` 1.533, `extraction` 642, `export`+`import` 528, `decay_controller` 115, `temporal_filter` 359, `merge_operator_adapter` 201. Restmuster (`crates/*/src`, Dateien/Treffer): `DocId` 95/791, `contextra_graph` 18/78, `LicenseGate` 5/25, `AutoExtractionMode` 4/24, `TextEmbeddingEngine` 10/28, `Reranker` 9/76, `TenantPolicy` 3/10, `saos` 4/20, `allow_legacy_integrity_key_fallback` 28 Treffer, `Option<Arc<KeyManager>>`/`Option<EncryptionMode>` 15 Treffer. Undeklarierte `cfg(feature)`: `encryption-at-rest`, `adaptive-candidate-pool-sizing`, `partial-index-rebuild`, `onnx`, `graph-connectivity-health`. **[V]**\|`wc -l`, `grep`\|` |
| `docs/analysis/SPEZIFIKATION_LOESCHUNG_PRODUKTIONSREIFE_2026-11-01.md` | 125 | `- **Engine-Kompilierung (In Konsolidierung, WP-1.1):** 33 Kompilierungsfehler in `contextra-engine`, primär durch historische Restreferenzen auf das entfernte Crate `contextra-graph` (`graph_index`, `relate`, `get_community`). Diese Methoden werden gemäß SPEC §1.4 und §5.1 aus dem v1-Kern entfernt.` |
| `docs/cleanup/g0-baseline.txt` | 209 | `Checking contextra-graph v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-graph)` |

### Crate `contextra-infer-candle`

- **Gesamtanzahl Referenzen:** 39 (Imports: 0, Usages: 0, Config/CI: 5, Docs: 34)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 58 | `"contextra-infer-candle",` |
| `deny.toml` | 79 | `{ crate = "candle-core", wrappers = ["candle-nn", "candle-transformers", "contextra-infer-candle"] },` |
| `parse_all_remnants.py` | 15 | `"contextra-infer-candle",` |
| `scripts/contextra_full_audit.py` | 101 | `"contextra-core", "contextra-db", "contextra-graph", "contextra-infer-candle",` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-crypto/src/lib.rs` | 21 | `//!   `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,` |
| `crates/contextra-types/src/lib.rs` | 24 | `//!   `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 22 | `\| Z. 42–56 \| `BENCHES` listet Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). \| Schritt `befund_bench_fuzz_dateien` schlägt fehl (FAIL); Benchmarks für diese Crates können nicht existieren. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 24 | `\| Z. 42–56 \| `BENCHES` enthielt Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench` sowie `deletion_proof_latency_bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). Die Benchmarks existieren nicht mehr. \| Schritt `befund_bench_fuzz_dateien` und Benchmarks scheiterten mit FAIL. \| `BENCHES` auf die existierenden Benchmarks der aktiven Crates (`mvcc`, `crypto`, `vector`, `store`, `simd`) bereinigt. \|` |
| `docs/CHANGELOG.md` | 235 | `\| `2026-09-26T00:00:00Z (SESSION: KV-BRIDGE-ZERO-COPY-IMPL)` \| `crates/contextra-infer-candle/src/kv_bridge.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| KvBridgeAdapter verbindet Retrieval-Chunks mit mandantenisoliertem KV-Cache-Store (RAM Tier 1 + LSM Tier 2 Spill via KvBridgeStorage Port). \|` |
| `docs/CHANGELOG.md` | 240 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/src/inference/prefix_reuse.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prefix reuse helper structs, context, and key/seed builders for KV cache stage B (Spec §9.2). \|` |
| `docs/CHANGELOG.md` | 241 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/src/kv_state.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| KvState implementation for native Candle Llama model KV cache management (Spec §9.2 Stufe B). \|` |
| `docs/CHANGELOG.md` | 242 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/src/model/mod.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Forked Llama model module with direct KvState access (Spec §9.2 Stufe B). \|` |
| `docs/CHANGELOG.md` | 243 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/src/model/quantized_llama.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Forked Candle Quantized Llama model with explicit KvState management (Spec §9.2 Stufe B). \|` |
| `docs/CHANGELOG.md` | 244 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/tests/kv_prefix_reuse_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integration test suite for KV cache stage B prefix reuse, counters, and tenant isolation (Spec §9.2). \|` |
| `docs/CHANGELOG.md` | 245 | `\| `2026-09-22T00:00:00Z` \| `crates/contextra-infer-candle/tests/kv_stage_b_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Unit and integration tests for Stage B KvState and forked Llama KV cache integration. \|` |
| `docs/CHANGELOG.md` | 261 | `\| `2026-09-16T16:13:00Z (SESSION: afafdd44)` \| `crates/contextra-infer-candle/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Crate root for contextra-infer-candle native GGUF inference backend. \|` |
| `docs/CHANGELOG.md` | 262 | `\| `2026-09-16T16:13:00Z` \| `crates/contextra-infer-candle/src/lib.rs` \| `REVIEW-PASS` \| `AGT-CANDLE-d495a019` \| `afafdd44` \| `-` \| `0` \| // REVIEW-PASS[1/2] (ID: AGT-CANDLE-d495a019) (TS: 2026-09-16T16:13:00Z) (SESSION: afafdd44) PRÜFER-KONTEXT: FRESH \|` |
| `docs/CHANGELOG.md` | 287 | `\| `2026-09-13T01:42:00Z (SESSION: 50c8c755)` \| `crates/contextra-infer-candle/tests/kv_bridge_and_stress_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integration tests for KV-Bridge fail-open behavior, fingerprint mismatches, GASP grounding, and backpressure saturation. \|` |
| `docs/CHANGELOG.md` | 292 | `\| `2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)` \| `crates/contextra-infer-candle/src/embedding.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Native Candle ML vector embedding client implementation. \|` |
| `docs/CHANGELOG.md` | 293 | `\| `2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)` \| `crates/contextra-infer-candle/src/inference.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Candle LLM text generator client implementing LlmTextGenerator. \|` |
| `docs/CHANGELOG.md` | 294 | `\| `2026-09-12T00:00:00Z (SESSION: BACKPRESSURE-CONTRACT-D1)` \| `crates/contextra-infer-candle/src/inference/core.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Candle LLM text generator client implementing LlmTextGenerator. \|` |
| `docs/CHANGELOG.md` | 297 | `\| `2026-09-11T16:00:00Z` \| `crates/contextra-infer-candle/tests/embedding_normalization_proptest.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Property-based tests for numerical normalization, finite outputs, and empty/whitespace input edge cases. \|` |
| `docs/CHANGELOG.md` | 298 | `\| `2026-09-11T16:00:00Z` \| `crates/contextra-infer-candle/tests/gguf_corruption_and_batch_boundary_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Error path and boundary tests for GGUF loading, quantization mismatch, batch boundaries, and concurrent hot-swap. \|` |
| `docs/CHANGELOG.md` | 305 | `\| `2026-09-11` \| `crates/contextra-infer-candle/tests/real_inference_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integration tests for Candle real forward-pass GGUF LLM and Bert embedding inference. \|` |
| `docs/CHANGELOG.md` | 326 | `\| `2026-09-09T15:45:22Z (SESSION: 6cae458a)` \| `crates/contextra-infer-candle/src/embedding_provider.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Trait-based EmbeddingProvider implementation for CandleEmbedClient. \|` |
| `docs/CHANGELOG.md` | 327 | `\| `2026-09-09T15:45:22Z (SESSION: 6cae458a)` \| `crates/contextra-infer-candle/src/gasp.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| GASP Grounding-Aware Sensitivity by Perturbation post-hoc hallucination validator. \|` |
| `docs/CHANGELOG.md` | 328 | `\| `2026-09-09T15:45:22Z (SESSION: 6cae458a)` \| `crates/contextra-infer-candle/src/gguf_loader.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| GGUF model container header and metadata parser for Candle models. \|` |
| `docs/CHANGELOG.md` | 329 | `\| `2026-09-09T15:45:22Z (SESSION: 6cae458a)` \| `crates/contextra-infer-candle/src/model_registry.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Model fingerprinting and CandleQuantization definitions. \|` |
| `docs/CHANGELOG.md` | 334 | `\| `2026-09-09T12:44:49Z (SESSION: c74a1828)` \| `crates/contextra-infer-candle/tests/gasp_mutant_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Additional unit tests targeting mutants in GaspValidator and GaspConfig. \|` |
| `docs/CHANGELOG.md` | 335 | `\| `2026-09-09T12:44:49Z (SESSION: c74a1828)` \| `crates/contextra-infer-candle/tests/gguf_loader_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Unit tests for GGUF metadata parsing error paths. \|` |
| `docs/CHANGELOG.md` | 336 | `\| `2026-09-09T12:44:49Z (SESSION: c74a1828)` \| `crates/contextra-infer-candle/tests/proptest_candle.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Property-based tests for contextra-infer-candle components (fingerprinting, GaspValidator, and clients). \|` |
| `docs/CHANGELOG.md` | 340 | `\| `2026-09-09T00:00:00Z (SESSION: CANDLE-EMBEDDING-PROVIDER)` \| `crates/contextra-infer-candle/tests/embedding_provider_conformance.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Conformance integration tests for CandleEmbedClient as EmbeddingProvider. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 434 | `contextra-infer-candle⚠: kv-bridge, kv-stage-b, cuda · contextra-crypto: cloud-egress-guard · contextra-mcp⚠: agent-workflows, kv-bridge, test-utils` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1097 | `\| CUDA-Toolkit (optional) \| Feature `cuda` von `contextra-infer-candle⚠` \| L-19 \| nur wenn GPU vorhanden \|` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-infer-ollama`

- **Gesamtanzahl Referenzen:** 34 (Imports: 0, Usages: 0, Config/CI: 9, Docs: 25)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 59 | `"contextra-infer-ollama",` |
| `deny.toml` | 81 | `{ crate = "reqwest", wrappers = ["contextra-infer-ollama"] },` |
| `parse_all_remnants.py` | 16 | `"contextra-infer-ollama",` |
| `scripts/contextra_full_audit.py` | 102 | `"contextra-infer-ollama", "contextra-infer-onnx", "contextra-ingest", "contextra-kvcache",` |
| `scripts/contextra_full_audit.py` | 116 | `"contextra-infer-ollama", "contextra-audit-export", "contextra-avv-generator",` |
| `scripts/contextra_full_audit.py` | 1152 | `"contextra-sandbox", "contextra-infer-onnx", "contextra-infer-ollama", "contextra-audit-export",` |
| `tools/generate_migration_edges.py` | 8 | `contextra-graph, contextra-infer-ollama, contextra-infer-onnx,` |
| `tools/generate_migration_edges.py` | 29 | `'contextra-infer-ollama': 'contextra_infer_ollama',` |
| `tools/generate_migration_edges.py` | 41 | `'contextra-infer-ollama': 'Feature `ollama` bleibt in Fassade, aber Kante nur noch über Feature-Gate.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-types/src/lib.rs` | 24 | `//!   `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`,` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 24 | `\| Z. 60–62 \| `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]` \| **Kardinaler Modellfehler im Skript:** Hier wird eine völlig veraltete Definition der "Produktmenge" zugrunde gelegt. Laut `docs/SPEC.md §5.1` umfasst der Kern 14 Crates (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`, `engine`, `contextra`). Crates wie `ingest`, `privacy`, `infer-ollama`, `audit-export`, `avv-generator` sind explizit Nicht-Ziele (SPEC §1.4). \| Schritt `produkt_huelle_ohne_kern` liefert FAIL (5 Crates mit Befund), weil diese Crates entweder gar nicht mehr existieren oder falsche Hüllen prüfen. \|` |
| `docs/CHANGELOG.md` | 257 | `\| `2026-09-16T16:24:19Z` \| `crates/contextra-infer-ollama/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `4b816c9a` \| `PASS` \| `-` \| // REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-16T16:24:19Z) (SESSION: 4b816c9a) PRÜFER-KONTEXT: FRESH - Independent diff review verified: prompt injection XML-escaping, retry & backoff error classification, forbid(unsafe_code) invariant, and 91 tests green. \|` |
| `docs/CHANGELOG.md` | 276 | `\| `2026-09-15T14:48:58Z` \| `crates/contextra-infer-ollama/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `1d6e3667` \| `PASS` \| `-` \| // REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-15T14:48:58Z) (SESSION: 1d6e3667) PRÜFER-KONTEXT: FRESH - Tier 3 deep audit verified: zero unsafe, proptests green, 10x 8-thread concurrency stress test green, prompt injection evasion matrix passed. \|` |
| `docs/CHANGELOG.md` | 286 | `\| `2026-09-13T01:45:00Z` \| `crates/contextra-infer-ollama/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `6e371f4e` \| `PASS` \| `-` \| // REVIEW-PASS[3/3] STATUS:PASS (TS: 2026-09-13T01:45:00Z) (SESSION: 6e371f4e) PRÜFER-KONTEXT: FRESH - Tier 3 deep audit complete: verified prompt injection structural isolation, proptest evasion matrix, concurrency stress tests, and ML calibration provenance. \|` |
| `docs/CHANGELOG.md` | 296 | `\| `2026-09-11T19:01:58Z` \| `crates/contextra-infer-ollama/src/lib.rs` \| `REVIEW-PASS` \| `AGT-OLLAMA-4d606464` \| `089dd3c0` \| `PASS` \| `0` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: AGT-OLLAMA-4d606464) (TS: 2026-09-11T19:01:58Z) (SESSION: 089dd3c0) PRÜFER-KONTEXT: FRESH - Clarified historical "zero unsafe" status observation vs compile-time enforced invariant via forbid(unsafe_code). \|` |
| `docs/CHANGELOG.md` | 299 | `\| `2026-09-11T14:45:00Z` \| `crates/contextra-infer-ollama/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `089dd3c0` \| `PASS` \| `-` \| // REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-11T14:45:00Z) (SESSION: 089dd3c0) PRÜFER-KONTEXT: FRESH - Verified prompt injection safety (xml_escape, build_rag_prompt), HTTP retry policy, zero unsafe, and test suite green. \|` |
| `docs/CHANGELOG.md` | 343 | `\| `2026-09-07T06:00:00Z` \| `crates/contextra-infer-ollama/src/importance.rs` \| `AI-TAG` \| `AGT-OLLAMA-14c0c140` \| `jules` \| `RESOLVED` \| `0` \| // AI-TAG[ML-SCORING][MAJOR] RESOLVED: Score importance enriches output with model_id provenance and optional calibrated_confidence via IsotonicCalibrator. Post-hoc outcome feedback interface record_importance_outcome added (ID: AGT-OLLAMA-14c0c140) (TS: 2026-09-07T06:00:00Z) (SESSION: jules) \|` |
| `docs/CHANGELOG.md` | 347 | `\| `2026-09-06T12:00:00Z` \| `crates/contextra-infer-ollama/src/context_prefixer.rs` \| `AI-TAG` \| `AGT-OLLAMA-47e6619b` \| `e4f906ee` \| `RESOLVED` \| `0` \| // AI-TAG[TESTING][MINOR] RESOLVED: AGT-OLLAMA-47e6619b — compute_max_prefix_chars extrahiert & Unit/Prompt-Boundary-Tests ergänzt (TS: 2026-09-06T12:00:00Z) (SESSION: e4f906ee) \|` |
| `docs/CHANGELOG.md` | 384 | `\| `2026-08-30T19:05:00Z` \| `crates/contextra-infer-ollama/src/client/tests.rs` \| `REVIEW-PASS` \| `TEST:OLL-001` \| `c9f5e2b3` \| `PASS` \| `1` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: TEST:OLL-001) (TS: 2026-08-30T19:05:00Z) (SESSION: c9f5e2b3) \|` |
| `docs/CHANGELOG.md` | 386 | `\| `2026-08-30T19:00:00Z` \| `crates/contextra-infer-ollama/src/client/tests.rs` \| `REVIEW-PASS` \| `TEST:OLL-001` \| `b8e4f1a2` \| `PASS` \| `1` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: TEST:OLL-001) (TS: 2026-08-30T19:00:00Z) (SESSION: b8e4f1a2) \|` |
| `docs/CHANGELOG.md` | 387 | `\| `2026-08-30T19:00:00Z` \| `crates/contextra-infer-ollama/src/query_rewriter.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| LLM-gestütztes Multi-Signal Query-Rewriting für Ollama (Text, Semantik, Graph-Anker) \|` |
| `docs/CHANGELOG.md` | 388 | `\| `2026-08-30T18:54:39Z (SESSION: ed7b7b38)` \| `crates/contextra-infer-ollama/src/client.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| HTTP-Client für lokale Ollama LLM/Embedding API mit Retry/Timeout/Streaming-Semantik \|` |
| `docs/CHANGELOG.md` | 389 | `\| `2026-08-30T18:54:39Z (SESSION: ed7b7b38)` \| `crates/contextra-infer-ollama/src/context_prefixer.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Contextra Contextual Chunk Prefixing — LLM-basierte Präfix-Generierung für Chunks \|` |
| `docs/CHANGELOG.md` | 390 | `\| `2026-08-30T18:54:39Z (SESSION: ed7b7b38)` \| `crates/contextra-infer-ollama/src/embedding.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| `TextEmbeddingEngine`-Implementierung für Ollama Embedding API \|` |
| `docs/CHANGELOG.md` | 391 | `\| `2026-08-30T18:54:39Z (SESSION: ed7b7b38)` \| `crates/contextra-infer-ollama/src/importance.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| LLM-basierte Wichtigkeits-Bewertung (ImportanceScore 0.0-1.0) für Memory Chunks \|` |
| `docs/CHANGELOG.md` | 392 | `\| `2026-08-30T18:54:39Z (SESSION: ed7b7b38)` \| `crates/contextra-infer-ollama/src/model_info.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Modell-Inspektion (/api/show) und Statisches Mapping bekannter Embedding-Dimensionen \|` |
| `docs/CHANGELOG.md` | 393 | `\| `2026-08-30T18:54:39Z` \| `crates/contextra-infer-ollama/src/client/tests.rs` \| `ANCHOR` \| `TEST:OLL-001` \| `ed7b7b38` \| `DONE` \| `2` \| // ANCHOR[TEST:OLL-001] STATUS:DONE (TS:2026-08-30T18:54:39Z) (SESSION:ed7b7b38) \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 236 | `\| Ollama \| `contextra-infer-ollama⚠` real \| **nicht in Jules** \| — \| Mock/Stub (`CONTEXTRA_EMBEDDING_PROVIDER=mock`) \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1095 | `\| Ollama + ein lokales Embedding-Modell \| `contextra-infer-ollama⚠` real \| L-19 \| `ollama list` muss ein Embedding-Modell zeigen \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1158 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-py`, `contextra-sandbox`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1170 | `3) cargo build -p contextra-sandbox⚠ -p contextra-py⚠ -p contextra-infer-ollama⚠ -p contextra-infer-onnx⚠ (Non-Default-Member; Fehler als ENV/CODE klassifizieren)` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-infer-onnx`

- **Gesamtanzahl Referenzen:** 32 (Imports: 0, Usages: 2, Config/CI: 9, Docs: 21)

#### Aktive Code-Stellen
| Datei | Zeile | Typ | Quelltext |
|---|---|---|---|
| `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | 183 | `USAGE` | `let model_dir = contextra_infer_onnx::ensure_onnx_model_download(` |
| `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | 188 | `USAGE` | `let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;` |

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `Cargo.toml` | 122 | `# Embedding & ML (activate with contextra-infer-onnx feature)` |
| `deny.toml` | 60 | `"contextra-infer-onnx",` |
| `parse_all_remnants.py` | 17 | `"contextra-infer-onnx",` |
| `scripts/contextra_full_audit.py` | 102 | `"contextra-infer-ollama", "contextra-infer-onnx", "contextra-ingest", "contextra-kvcache",` |
| `scripts/contextra_full_audit.py` | 115 | `"contextra-bench", "contextra-py", "contextra-sandbox", "contextra-infer-onnx",` |
| `scripts/contextra_full_audit.py` | 1152 | `"contextra-sandbox", "contextra-infer-onnx", "contextra-infer-ollama", "contextra-audit-export",` |
| `tools/generate_migration_edges.py` | 8 | `contextra-graph, contextra-infer-ollama, contextra-infer-onnx,` |
| `tools/generate_migration_edges.py` | 30 | `'contextra-infer-onnx': 'contextra_infer_onnx',` |
| `tools/generate_migration_edges.py` | 42 | `'contextra-infer-onnx': 'Feature `onnx` bleibt in Fassade, aber Kante nur noch über Feature-Gate.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-types/src/lib.rs` | 25 | `//!   `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`,` |
| `docs/CHANGELOG.md` | 258 | `\| `2026-09-16T16:24:18Z` \| `crates/contextra-infer-onnx/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `d3856baa` \| `PASS` \| `-` \| // REVIEW-PASS[3/3] STATUS:PASS (TS: 2026-09-16T16:24:18Z) (SESSION: d3856baa) PRÜFER-KONTEXT: FRESH - Independent review verified feature gate isolation, zero unsafe in production, spawn_blocking non-starvation model, and domain scoring invariants (APM-22, APM-23, APM-24). \|` |
| `docs/CHANGELOG.md` | 265 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| In-process ONNX Embedding Engine (Layer 3 im 5-Schichten-DAG). \|` |
| `docs/CHANGELOG.md` | 266 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/reranker/config.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Cross-Encoder Reranking Konfiguration und Ergebnisse. \|` |
| `docs/CHANGELOG.md` | 267 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Haupt-Reranker-API (`CrossEncoderReranker`) und Backend-Abstraktion. \|` |
| `docs/CHANGELOG.md` | 268 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/reranker/mod.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Cross-Encoder Reranking für Post-RRF Präzisionsverbesserung. \|` |
| `docs/CHANGELOG.md` | 269 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/reranker/onnx.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Real ONNX Runtime Cross-Encoder Inferenz-Backend. \|` |
| `docs/CHANGELOG.md` | 270 | `\| `2026-09-15T16:10:40Z (SESSION: ec33599e)` \| `crates/contextra-infer-onnx/src/reranker/tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Unit-Tests für Cross-Encoder Reranker. \|` |
| `docs/CHANGELOG.md` | 348 | `\| `2026-09-06T12:00:00Z` \| `crates/contextra-infer-onnx/src/reranker/onnx.rs` \| `AI-TAG` \| `AGT-EMBED-62093e61` \| `-` \| `RESOLVED` \| `0` \| // AI-TAG[CALIBRATION][MINOR] AGT-EMBED-62093e61 [RESOLVED] (TS: 2026-09-06T12:00:00Z) \|` |
| `docs/CHANGELOG.md` | 349 | `\| `2026-09-06T11:19:00Z` \| `crates/contextra-infer-onnx/src/reranker/mod.rs` \| `REVIEW-PASS` \| `-` \| `8efa6210` \| `PASS` \| `-` \| // REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-06T11:19:00Z) (SESSION: 8efa6210) PRÜFER-KONTEXT: FRESH - Verified ML domain APM alignment (APM-22, APM-23, APM-24), zero-unsafe in production, and hermetic feature isolation. \|` |
| `docs/CHANGELOG.md` | 353 | `\| `2026-09-04T11:42:28Z` \| `crates/contextra-infer-onnx/src/reranker/mod.rs` \| `REVIEW-PASS` \| `-` \| `3e5150c8` \| `PASS` \| `-` \| // REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-04T11:42:28Z) (SESSION: 3e5150c8) PRÜFER-KONTEXT: FRESH - Verified CrossEncoderReranker passthrough fallback, candidate limit bounds, and zero-unsafe invariants. \|` |
| `docs/CHANGELOG.md` | 354 | `\| `2026-09-03T19:40:00Z` \| `crates/contextra-infer-onnx/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `6da6a1c8` \| `PASS` \| `-` \| // REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-03T19:40:00Z) (SESSION: 6da6a1c8) PRÜFER-KONTEXT: FRESH - Verified Chaos Engineering fault tolerance, hermetic feature gate, and zero-unsafe invariants. \|` |
| `docs/CHANGELOG.md` | 364 | `\| `2026-09-02T08:17:15Z` \| `crates/contextra-infer-onnx/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `f260cbf2` \| `PASS` \| `-` \| // REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-02T08:17:15Z) (SESSION: f260cbf2) PRÜFER-KONTEXT: FRESH - Verified feature gate isolation, zero unsafe in production, and execution non-starvation model. \|` |
| `docs/CHANGELOG.md` | 376 | `\| `2026-08-31` \| `crates/contextra-infer-onnx/tests/reranker_adversarial_test.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Quantitativer Test- & Benchmark-Suite für Cross-Encoder Reranker Adversarial Attacks. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 234 | `\| ONNX Runtime (nativ) \| `contextra-infer-onnx⚠` \| J-19 \| abhängig von der `ort`-Konfiguration im `Cargo.toml` (Download beim Build oder `ORT_DYLIB_PATH`) — **prüfen** \| ENV-Befund \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1096 | `\| ONNX Runtime (nativ) + Testmodell \| `contextra-infer-onnx⚠` \| L-19 \| je nach `ort`-Konfiguration \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1158 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-py`, `contextra-sandbox`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1170 | `3) cargo build -p contextra-sandbox⚠ -p contextra-py⚠ -p contextra-infer-ollama⚠ -p contextra-infer-onnx⚠ (Non-Default-Member; Fehler als ENV/CODE klassifizieren)` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-kvcache`

- **Gesamtanzahl Referenzen:** 31 (Imports: 0, Usages: 0, Config/CI: 8, Docs: 23)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 61 | `"contextra-kvcache", # TRANSITION E29, bis 2027-04-30, REQ-ASYNC-1` |
| `parse_all_remnants.py` | 18 | `"contextra-kvcache",` |
| `scripts/contextra_full_audit.py` | 102 | `"contextra-infer-ollama", "contextra-infer-onnx", "contextra-ingest", "contextra-kvcache",` |
| `scripts/contextra_full_audit.py` | 113 | `RUECKBAU_GEPLANT = {"contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest",` |
| `scripts/contextra_full_audit.py` | 1150 | `ENTFERNEN_V1 = ["contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest", "contextra-privacy",` |
| `tools/generate_migration_edges.py` | 9 | `contextra-kvcache, contextra-router.` |
| `tools/generate_migration_edges.py` | 31 | `'contextra-kvcache': 'contextra_kvcache',` |
| `tools/generate_migration_edges.py` | 43 | `'contextra-kvcache': '`engine/src/kv_cache_integration/` löschen; `Collection::with_kv_store`, `set_kv_store`, `kv_store`, `start_kv_cache_eviction`, `TenantIsolatedKvStore`-Reste entfernen; Layer `KvCache` → `FeatureDisabled`.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-crypto/src/lib.rs` | 21 | `//!   `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,` |
| `crates/contextra-types/src/lib.rs` | 25 | `//!   `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`,` |
| `docs/CHANGELOG.md` | 99 | `\| `TS:2026-10-01T00:00:00Z` \| `crates/contextra-kvcache/src/prefix_store.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Mandantenisolierter KV-Prefix-Store mit Byte-Budget & LRU-Eviction (Spec §9.2 / INV-CLM-SCRATCHPAD-1). \|` |
| `docs/CHANGELOG.md` | 100 | `\| `TS:2026-10-01T00:00:00Z` \| `crates/contextra-kvcache/src/scratchpad_invalidation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Targeted Invalidation von Agent-Scratchpad-Prefixes im Radix-Baum (Spec §9.2 / INV-CLM-SCRATCHPAD-1). \|` |
| `docs/CHANGELOG.md` | 116 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/src/eviction_worker.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Eviction-Worker (nicht-blockierender Hot-Path LRU / Attention) und emergency_wipe (synchroner Notfall). \|` |
| `docs/CHANGELOG.md` | 117 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/src/radix.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prefix-Radix-Baum über Token-Sequenzen, RAII-Guards & KvReusePolicy (§9.2). \|` |
| `docs/CHANGELOG.md` | 118 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/src/segment.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| KvSegment mit Zeroize-Garantie, Tier-2 AEAD-Verschlüsselung & Crypto-Shredding. \|` |
| `docs/CHANGELOG.md` | 119 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/src/store.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Tenant-isolierter KV-Segment-Store (INV-TENANT Isolation) mit Prefix-Radix & Guard-Schutz. \|` |
| `docs/CHANGELOG.md` | 120 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/tests/cancellation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Cancellation- & RAII-Guard-Tests für den KV-Cache (Task KV-06). \|` |
| `docs/CHANGELOG.md` | 121 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/tests/golden_greedy_decode.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Golden-Test für identische Greedy-Tokenfolgen mit/ohne Prefix-Reuse (Spec §9.2 / Task 4-04). \|` |
| `docs/CHANGELOG.md` | 122 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/tests/prefix_store_port.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Port- und Isolationstests für TenantPrefixKvStore (§9.2). \|` |
| `docs/CHANGELOG.md` | 123 | `\| `TS:2026-09-15T00:00:00Z` \| `crates/contextra-kvcache/tests/tenant_isolation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Tenant-Isolations-Tests für den KV-Cache (Task KV-05 / INV-TENANT). \|` |
| `docs/CHANGELOG.md` | 491 | `\| `` \| `crates/contextra-kvcache/tests/clm_pinned_base_preserved.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrationstest für INV-CLM-SCRATCHPAD-1 (Gepinnter Basiskontext bleibt bei Scratchpad-Reset unberührt). \|` |
| `docs/CHANGELOG.md` | 492 | `\| `` \| `crates/contextra-kvcache/tests/scratchpad_invalidation_isolation.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrationstest für Mandantenisolation bei der Invalidation von Scratchpad-Cache-Scopes. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 433 | `contextra-kvcache⚠: content-addressed-kv-cache, kv-encryption, kvcache-attention-eviction, kvcache-kivi-quant, kivi-quantization` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1026 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-kvcache`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1046 | `9) Geschwister-Schichten einzeln: HNSW (Vektor weg + keine Geisterzeiger: verify_no_ghost_pointers, INV-DELETION-2), contextra-text (Posting-Listen und Dokumentstatistik/IDF aktualisiert?), contextra-graph⚠ (Knoten/Kanten/Hyperkanten und abgeleitete Community-/Konsolidierungsergebnisse), contextra-cognition⚠ (konsolidierte Zusammenfassungen oder Embeddings, die den Inhalt abgeleitet enthalten — hier kann personenbezogene Information fortleben, obwohl das Original gelöscht ist: Befund S2, falls so), contextra-kvcache⚠ (gecachte Tensoren zum Prompt, kv-encryption), contextra-checkpoint⚠ (Blake3-Manifest enthält Hashes, keine Klartexte?).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1280 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-core`, `contextra-graph`, `contextra-kvcache`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1290 | `SCHRITT 2 — Gezielt für die kritischsten Crates einzeln mit höherem Parallelitätsgrad: contextra-store (Group-Commit, Memtable-Sharding, Observer), contextra-vector (HNSW Insert/Search parallel, ComputePool), contextra-mvcc, contextra-engine, contextra-kvcache⚠, contextra-text, contextra-graph⚠. Befehlsmuster: RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test -Zbuild-std --target <host-triple> -p <crate> -- --test-threads=<Anzahl Kerne> (Host-Triple per rustc -vV ermitteln). Wiederhole TSan-Läufe 3x (Datenrennen sind probabilistisch).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-license`

- **Gesamtanzahl Referenzen:** 13 (Imports: 0, Usages: 0, Config/CI: 6, Docs: 7)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `parse_all_remnants.py` | 19 | `"contextra-license",` |
| `scripts/contextra_full_audit.py` | 103 | `"contextra-license", "contextra-mcp", "contextra-privacy", "contextra-py",` |
| `scripts/contextra_full_audit.py` | 117 | `"contextra-cognition", "contextra-adapt", "contextra-graph", "contextra-license"}` |
| `scripts/contextra_full_audit.py` | 1154 | `ENTKOPPELN_V1 = ["contextra-graph", "contextra-license", "contextra-checkpoint"]     # Entscheidungsgrundlage 6.2` |
| `scripts/contextra_full_audit.py` | 1157 | `RING_EXTERN = {"contextra-graph", "contextra-license", "contextra-testkit"}           # nur ueber Ports / optional` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-types/src/lib.rs` | 25 | `//!   `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`,` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 559 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-license`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 569 | `SCHRITT 2 — Lückenliste: Dateien mit 0 % und die 30 größten ungedeckten Funktionen. Markiere die folgenden kritischen Kandidaten und prüfe jeweils die Abdeckung explizit: Wal::recover_from_poison, wal/replay.rs (replay_mmap vs replay), Wal::truncate/truncate_uncommitted_tail, rewrite_as_v3 und migrate_legacy_wal, manifest rollover, compaction retain_key_versions, validate_tombstone_safety, SequenceLogSsiValidator::coarsen_*, diagnose_pruning_blocker, TxBuffer::reap_orphans_bounded, HNSW rebuild_phase2_merge_and_swap, DiskANN init_hnsw_fallback/recover_pending_delta, Sq8Bias::calibrate, RaBitQ generate_orthogonal_matrix, RevocationLog::verify_integrity, DeletionProof::verify_external, AuditChain::verify_chain/verify_head_signature, EgressGuard fail-closed-Zweige, BulkExfiltrationDetector Zeit-Regression, contextra-license⚠ Prüfpfade, contextra-engine (Testlücke).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/cleanup/g0-baseline.txt` | 318 | `Checking contextra-license v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-license)` |

### Crate `contextra-mcp`

- **Gesamtanzahl Referenzen:** 57 (Imports: 0, Usages: 0, Config/CI: 19, Docs: 38)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `.agents/governance/determinism-allow.toml` | 20 | `path = "crates/contextra-mcp/src/sandbox.rs"` |
| `.agents/jules/setup/environment_script.sh` | 143 | `grep -q axum crates/contextra-mcp/Cargo.toml 2>/dev/null && warn "ADR-010 verletzt: axum in contextra-mcp" \|\| ok "ADR-010: kein axum in contextra-mcp"` |
| `.agents/mcp_config.json` | 8 | `"contextra-mcp",` |
| `.agents/mcp_config.json` | 10 | `"contextra-mcp-server"` |
| `.gemini/config/mcp_config.json` | 8 | `"contextra-mcp",` |
| `.gemini/config/mcp_config.json` | 10 | `"contextra-mcp-server"` |
| `.gemini/mcp_config.json` | 8 | `"contextra-mcp",` |
| `.gemini/mcp_config.json` | 10 | `"contextra-mcp-server"` |
| `.gemini/settings.json` | 28 | `"contextra-mcp",` |
| `.gemini/settings.json` | 30 | `"contextra-mcp-server"` |
| `deny.toml` | 33 | `{ name = "contextra-mcp", expression = "MIT OR Apache-2.0", license-files = [] },` |
| `deny.toml` | 62 | `"contextra-mcp",` |
| `justfile` | 74 | `"contextra-mcp:fuzz_jsonrpc_parsing"` |
| `justfile` | 80 | `"contextra-mcp:fuzz_prompt_injection_guard"` |
| `parse_all_remnants.py` | 20 | `"contextra-mcp",` |
| `scripts/contextra_full_audit.py` | 103 | `"contextra-license", "contextra-mcp", "contextra-privacy", "contextra-py",` |
| `scripts/contextra_full_audit.py` | 114 | `"contextra-privacy", "contextra-mcp", "contextra-wire", "contextra-core",` |
| `scripts/contextra_full_audit.py` | 1151 | `"contextra-mcp", "contextra-wire", "contextra-core", "contextra-bench", "contextra-py",` |
| `tools/loc.py` | 28 | `"contextra-mcp",` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-crypto/src/lib.rs` | 21 | `//!   `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,` |
| `crates/contextra-types/src/error.rs` | 117 | `/// (such as `contextra-py` and `contextra-mcp`).` |
| `crates/contextra-types/src/lib.rs` | 25 | `//!   `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`,` |
| `crates/contextra-types/src/types/domain/ids.rs` | 189 | `/// by the MCP layer (`contextra-mcp/src/lib.rs` line 214 uses it as a` |
| `crates/contextra-types/src/types/domain/misc.rs` | 80 | `/// Callers in contextra-mcp and contextra-db both enforce this limit before forwarding k to HNSW/BM25.` |
| `crates/contextra-types/src/types/domain/misc.rs` | 84 | `/// to HNSW and BM25 sub-searches. All upstream layers (contextra-mcp, contextra-py,` |
| `crates/contextra/src/lib.rs` | 12 | `//! Systeme (wie z. B. `contextra-mcp` oder externe SDKs/Applikationen). Sie verbirgt die` |
| `docs/CHANGELOG.md` | 208 | `\| `2026-10-01` \| `crates/contextra-mcp/src/proof_key_env.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Einziger Resolver für DeletionProof-Schlüssel (CONTEXTRA_DELETION_PROOF_KEY / CONTEXTRA_PROOF_KEY) \|` |
| `docs/CHANGELOG.md` | 210 | `\| `2026-09-30` \| `crates/contextra-mcp/src/proof_key.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Lösen und Validieren des DeletionProof Schlüssel-Materials aus Umgebungsvariablen. \|` |
| `docs/CHANGELOG.md` | 285 | `\| `2026-09-13T14:29:07Z` \| `crates/contextra-mcp/src/io.rs` \| `AI-TAG` \| `AGT-MCP-782aa62e` \| `23626761` \| `RESOLVED` \| `0` \| // AI-TAG[SMELL][RESOLVED] Missing inactivity timeout on stdio read_line_bounded (ID: AGT-MCP-782aa62e) (TS: 2026-09-13T14:29:07Z) (SESSION: 23626761) \|` |
| `docs/CHANGELOG.md` | 309 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/bin/contextra-mcp-server.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Binary Entry Point for Contextra MCP Stdio JSON-RPC Server \|` |
| `docs/CHANGELOG.md` | 310 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/config.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Dynamic Provider Construction for Embedding & LLM backends (Ollama, ONNX, Candle, Mock) \|` |
| `docs/CHANGELOG.md` | 311 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| stdio JSON-RPC 2.0 MCP-Server (kein HTTP! ADR-010) \|` |
| `docs/CHANGELOG.md` | 312 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/prompt_injection.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Prompt-Injection-Erkennung & Quarantäne-System für MCP-Server \|` |
| `docs/CHANGELOG.md` | 313 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/protocol.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| MCP JSON-RPC 2.0 Protokoll-Typen & DTO-Abbildung für Contextra \|` |
| `docs/CHANGELOG.md` | 314 | `\| `2026-09-10T19:25:24Z (SESSION: ae8c2fb9)` \| `crates/contextra-mcp/src/sandbox.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| MCP Sandbox & Zero-Trust Tool Isolation Layer \|` |
| `docs/CHANGELOG.md` | 357 | `\| `2026-09-02T23:25:00Z` \| `crates/contextra-mcp/tests/mcp_test.rs` \| `REVIEW-PASS` \| `TEST:MCP-002` \| `4e4bb530` \| `PASS` \| `1` \| // REVIEW-PASS[2/2] STATUS:PASS (ID: TEST:MCP-002) (TS:2026-09-02T23:25:00Z) (SESSION: 4e4bb530) PRÜFER-KONTEXT: FRESH — Independent review pass confirming complete JSON-RPC 2.0 error path coverage across test_malformed_request_returns_error, test_unknown_tool_returns_error, test_search_missing_collection_returns_error, test_insert_empty_text_returns_error, test_jsonrpc_null_id_preserved_in_error_response, test_tools_call_without_name_field_returns_error, test_insert_missing_collection_field_returns_error, and unit tests in tests.rs. \|` |
| `docs/CHANGELOG.md` | 362 | `\| `2026-09-02T08:19:33Z` \| `crates/contextra-mcp/tests/mcp_test.rs` \| `REVIEW-PASS` \| `TEST:MCP-002` \| `e2c39779` \| `PASS` \| `1` \| // REVIEW-PASS[1/2] STATUS:PASS (ID: TEST:MCP-002) (TS:2026-09-02T08:19:33Z) (SESSION: e2c39779) PRÜFER-KONTEXT: FRESH — Error-Path Coverage verified in test_malformed_request_returns_error and unit tests. \|` |
| `docs/CHANGELOG.md` | 375 | `\| `2026-08-31T21:12:53Z` \| `crates/contextra-mcp/tests/mcp_test.rs` \| `ANCHOR` \| `TEST:MCP-002` \| `2c814094` \| `DONE` \| `2` \| // ANCHOR[TEST:MCP-002] STATUS:DONE (ID: TEST:MCP-002) (TS:2026-08-31T21:12:53Z) (SESSION: 2c814094) — Error-Path Coverage \|` |
| `docs/CHANGELOG.md` | 493 | `\| `` \| `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Layer-4 Bulk-Exfiltration-Detektor (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 494 | `\| `` \| `crates/contextra-mcp/src/egress_gateway.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Egress Security Gateway & Classifier Enforcement for Cloud MCP Queries (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 495 | `\| `` \| `crates/contextra-mcp/src/egress_guard.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Layer-4 EgressGuard (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 496 | `\| `` \| `crates/contextra-mcp/src/server_dispatch.rs` \| `AI-TAG` \| `-` \| `-` \| `RESOLVED` \| `-` \| // AI-TAG[SMELL][RESOLVED] audit-APM-38-mcp: Replay-Schutz für stdio-JSON-RPC \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 434 | `contextra-infer-candle⚠: kv-bridge, kv-stage-b, cuda · contextra-crypto: cloud-egress-guard · contextra-mcp⚠: agent-workflows, kv-bridge, test-utils` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 840 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-audit-export`, `contextra-avv-generator`, `contextra-mcp`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 842 | `**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-privacy⚠/tests/campaign_egress_*.rs`, `crates/contextra-mcp⚠/tests/campaign_mcp_*.rs`, `crates/contextra-audit-export⚠/tests/campaign_*.rs`` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 847 | `AUFTRAG: Prüfe die Egress-Firewall (contextra-privacy⚠), den MCP-Server (contextra-mcp⚠) und die DSGVO-Generatoren (contextra-audit-export⚠, contextra-avv-generator⚠). Quellen: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md (SEC-04, K6, K7), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (4.7, 4.8, Hinweis zum offenen MAJOR-Befund normalize_text), CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md (Teil VI).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 852 | `H3 [Prompt-Injection, contextra-mcp⚠::prompt_injection, normalize_text — offener MAJOR-Befund laut Analysebericht]: Homoglyphen-Bypass.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 861 | `H9 [MCP-Werkzeuge, contextra-mcp⚠, Feature test-utils]: 11 aktive Werkzeuge (contextra_search, _insert, _get, _forget, _collections, _consolidate, _cloud_query, _relate, _relate_n_ary, _plugin_status, _explain) und 4 "Phantom"-Namen (contextra_upsert, _delete, _create_collection, _drop_collection; im Katalog 🔴 Phantom). Test über den In-Prozess-Dispatch: jedes aktive Werkzeug mit gültiger und ungültiger Eingabe; die 4 Phantom-Namen müssen einen sauberen JSON-RPC-Fehler liefern, nie Panic; prüfe, ob die Sandbox-Klassifikation Namen kennt, die der Dispatch nicht kennt (Katalog-Befund P0-4 bestätigen). Grenzen: relate_n_ary mit 63/64/65 Teilnehmern; k/limit = 0, 1, u32::MAX; überlange Strings; ungültiges UTF-8 (falls über die API erreichbar); tief verschachteltes JSON (Rekursionsgrenze, Stack-Überlauf?); contextra_forget liefert bei aktivem Sovereign-Ring einen Löschbeweis; Mandantentrennung bei contextra_search/_get.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1056 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-mcp`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1069 | `Produkt 4 Krypto/DSGVO: SEC-01 Audit-Kette ohne externen Head-Anker (audit_chain.rs ~252) · SEC-02 Klartext im RAM nach Crypto-Shredding (kv_shredding.rs ~188; RAM-Teil lokal L-23) · SEC-03 Split-Brain KeyRegistry/RevocationLog (kv_shredding.rs ~188) · SEC-04 Rehydrator-Token-Injektion (egress_gateway.rs ~43) · SEC-05 drei getrennte Audit-Ketten · MAJOR normalize_text-Homoglyphen-Bypass (contextra-mcp⚠::prompt_injection).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 1057 | `F5 JSON-RPC-PARSING (contextra-mcp⚠): Malformierte JSON-RPC-Pakete, extrem große Payloads — Ziel: kein Panic, definierter McpError.` |
| `docs/SPEC.md` | 154 | `\|F-17\|CI: 19 Workflows, **15× `continue-on-error`**. `deny.toml`: 6 ignorierte Advisories **ohne Ablaufdatum**; `clarify` und `wrappers` verweisen auf entfernte Crates (`contextra-mcp`, `-db`, `-graph`, …). **[V]**\|`.github/workflows`, `deny.toml`\|` |
| `docs/SPEC.md` | 171 | `\|F-34\|**Fassade, Ist-Abhängigkeiten:** `contextra` hängt an `types`, `ports`, `engine`, `store`, `crypto`, `rank` (Soll `engine`, `durable-fs`, `types`); `durable-fs` fehlt, daher hat `RealVfs` **keine eingehende Kante** (kein Crate nutzt `durable-fs`; `store` nutzt `std::fs` mit 171, `vector` mit 54 Treffern, Muster inkl. `memmap2`). `lib.rs` behauptet „zero direct dependencies on Ring 0/1“ und nennt `contextra-mcp` (existiert nicht). `builder.rs` 551, `collection_profile.rs` 198, `performance_profile.rs` 267, `query_rewriter.rs` 262, `rng.rs` 75 Zeilen. **[V]**\|`Cargo.toml`, `wc -l`, `grep`\|` |

### Crate `contextra-privacy`

- **Gesamtanzahl Referenzen:** 37 (Imports: 0, Usages: 0, Config/CI: 6, Docs: 31)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `deny.toml` | 65 | `"contextra-privacy",` |
| `parse_all_remnants.py` | 21 | `"contextra-privacy",` |
| `scripts/contextra_full_audit.py` | 103 | `"contextra-license", "contextra-mcp", "contextra-privacy", "contextra-py",` |
| `scripts/contextra_full_audit.py` | 114 | `"contextra-privacy", "contextra-mcp", "contextra-wire", "contextra-core",` |
| `scripts/contextra_full_audit.py` | 1150 | `ENTFERNEN_V1 = ["contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest", "contextra-privacy",` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-crypto/src/lib.rs` | 21 | `//!   `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,` |
| `crates/contextra-testkit/src/lib.rs` | 28 | `//!   `contextra-mvcc`, `contextra-privacy`, `contextra-store`.` |
| `crates/contextra-types/src/lib.rs` | 26 | `//!   `contextra-mvcc`, `contextra-ports`, `contextra-privacy`, `contextra-py`, `contextra-rank`,` |
| `crates/contextra-types/src/types/domain/ids.rs` | 27 | `/// Im Gegensatz dazu ist `contextra_privacy::processing_registry::TenantId` ein lokaler `String`-Alias` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 24 | `\| Z. 60–62 \| `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]` \| **Kardinaler Modellfehler im Skript:** Hier wird eine völlig veraltete Definition der "Produktmenge" zugrunde gelegt. Laut `docs/SPEC.md §5.1` umfasst der Kern 14 Crates (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`, `engine`, `contextra`). Crates wie `ingest`, `privacy`, `infer-ollama`, `audit-export`, `avv-generator` sind explizit Nicht-Ziele (SPEC §1.4). \| Schritt `produkt_huelle_ohne_kern` liefert FAIL (5 Crates mit Befund), weil diese Crates entweder gar nicht mehr existieren oder falsche Hüllen prüfen. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 26 | `\| Z. 60–62 \| `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", ...]` \| Kardinaler Modellfehler: Veraltete Produktdefinition. Crates wie `ingest`, `privacy`, `infer-ollama` etc. sind Nicht-Ziele (SPEC §1.4). Der Kern besteht laut SPEC §5.1 aus 14 Crates. \| Schritt `produkt_huelle_ohne_kern` lieferte FAIL (5 Crates mit Befund). \| `PRODUKT` auf die 14 Kern-Crates gemäß SPEC §5.1 umgestellt. \|` |
| `docs/CHANGELOG.md` | 493 | `\| `` \| `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Layer-4 Bulk-Exfiltration-Detektor (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 494 | `\| `` \| `crates/contextra-mcp/src/egress_gateway.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Egress Security Gateway & Classifier Enforcement for Cloud MCP Queries (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 495 | `\| `` \| `crates/contextra-mcp/src/egress_guard.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Layer-4 EgressGuard (Re-export from contextra-privacy) \|` |
| `docs/CHANGELOG.md` | 502 | `\| `` \| `crates/contextra-privacy/src/audit_trace.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Deterministische Audit-Trace-Generierung für Egress-Klassifikationsentscheidungen (INV-EGRESS-AUDIT-1 / P28). \|` |
| `docs/CHANGELOG.md` | 503 | `\| `` \| `crates/contextra-privacy/src/bulk_exfiltration_detector.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Layer-4 Bulk-Exfiltration-Detektor (Volumen/Zeitfenster pro Session) \|` |
| `docs/CHANGELOG.md` | 504 | `\| `` \| `crates/contextra-privacy/src/context_edit_audit.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Audit-Protokollierung von Änderungen am temporären Agenten-Kontext (CLM-Scratchpad). \|` |
| `docs/CHANGELOG.md` | 505 | `\| `` \| `crates/contextra-privacy/src/egress_gateway.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Egress Security Gateway & Classifier Enforcement for Cloud Queries \|` |
| `docs/CHANGELOG.md` | 506 | `\| `` \| `crates/contextra-privacy/src/egress_vault.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Cloud-Egress-Vault mit Layer-1-Regex-Klassifikation und Fail-Closed-Semantik. \|` |
| `docs/CHANGELOG.md` | 507 | `\| `` \| `crates/contextra-privacy/tests/campaign_cov_privacy_egress.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Campaign coverage test for CloudResponseRehydrator and BulkExfiltrationDetector fail-closed policy evaluation. \|` |
| `docs/CHANGELOG.md` | 508 | `\| `` \| `crates/contextra-privacy/tests/context_edit_audit_chain.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrationstest für ContextEditAuditRecord-Verkettung, Hash-Integrität, Clock-Port-Injektion und Manipulationserkennung. \|` |
| `docs/CHANGELOG.md` | 509 | `\| `` \| `crates/contextra-privacy/tests/pii_vault_forces_crypto_shred.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Test PII vault forces CryptoShred under DeploymentTier::EdgeMinimal (INV-COLLECTION-PROFILE-3) \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 578 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 583 | `PARAMETER: ZIEL = contextra-crypto     SHARD = 1/4     (weitere Ziele: contextra-mvcc, contextra-store --file 'src/wal/**', contextra-privacy⚠)` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 812 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-agent`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 830 | `H8 [Drei Audit-Ketten, SEC-05/K8]: AuditChain (contextra-crypto), ContextEditAuditRecord (contextra-privacy⚠, verify_audit_chain), AgentAudit (contextra-agent⚠): Dokumentiere Genesis-Hash, Schlüssel und Verifikationsweg jeder Kette; prüfe, ob irgendeine Querverbindung existiert. Liste Redundanzen als Text mit Datei:Zeile (reiner Architekturbefund S3).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 840 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-audit-export`, `contextra-avv-generator`, `contextra-mcp`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 842 | `**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-privacy⚠/tests/campaign_egress_*.rs`, `crates/contextra-mcp⚠/tests/campaign_mcp_*.rs`, `crates/contextra-audit-export⚠/tests/campaign_*.rs`` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 847 | `AUFTRAG: Prüfe die Egress-Firewall (contextra-privacy⚠), den MCP-Server (contextra-mcp⚠) und die DSGVO-Generatoren (contextra-audit-export⚠, contextra-avv-generator⚠). Quellen: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md (SEC-04, K6, K7), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (4.7, 4.8, Hinweis zum offenen MAJOR-Befund normalize_text), CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md (Teil VI).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1342 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1347 | `PARAMETER: ZIELE = contextra-crypto, contextra-mvcc, contextra-privacy⚠, contextra-store (--file 'src/wal/**' und 'src/compaction/**'), contextra-vector (--file 'src/hnsw/deletion.rs' und 'src/quantize/**')` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 5 | `> - II.7: Paketname contextra-privacy → contextra-crypto korrigiert (Prompt-Fehler im Original)` |
| `docs/cleanup/g0-baseline.txt` | 286 | `Checking contextra-privacy v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-privacy)` |

### Crate `contextra-py`

- **Gesamtanzahl Referenzen:** 32 (Imports: 0, Usages: 0, Config/CI: 16, Docs: 16)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `.agents/jules/setup/environment_script.sh` | 107 | `# ── 4. Python-Umgebung (contextra-py / maturin) ──────────────────────────────` |
| `.cargo/audit.toml` | 7 | `# ISOLATION PROOF: `pyo3` is used exclusively in `contextra-py` for FFI Python bindings.` |
| `.cargo/audit.toml` | 8 | `# `contextra-py` does not invoke `PyList` or `PyTuple` `.nth()` / `.nth_back()` iterator methods.` |
| `.cargo/audit.toml` | 14 | `# ISOLATION PROOF: `pyo3` is used exclusively in `contextra-py` for FFI Python bindings.` |
| `.cargo/audit.toml` | 15 | `# `contextra-py` does not use `PyCFunction::new_closure`.` |
| `deny.toml` | 66 | `"contextra-py",` |
| `deny.toml` | 80 | `{ crate = "pyo3", wrappers = ["contextra-py", "numpy", "pythonize"] },` |
| `justfile` | 56 | `cargo llvm-cov --workspace --exclude contextra-py --json --output-path coverage.json` |
| `parse_all_remnants.py` | 22 | `"contextra-py",` |
| `scripts/contextra_full_audit.py` | 103 | `"contextra-license", "contextra-mcp", "contextra-privacy", "contextra-py",` |
| `scripts/contextra_full_audit.py` | 115 | `"contextra-bench", "contextra-py", "contextra-sandbox", "contextra-infer-onnx",` |
| `scripts/contextra_full_audit.py` | 791 | `py_dir = ws.members.get("contextra-py")` |
| `scripts/contextra_full_audit.py` | 1151 | `"contextra-mcp", "contextra-wire", "contextra-core", "contextra-bench", "contextra-py",` |
| `tools/generate_migration_edges.py` | 44 | `'contextra-router': 'Feature `router` in Fassade entfernen; `contextra-py` anpassen.',` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |
| `tools/generate_migration_edges.py` | 55 | `return 'WP 2.3' if dep_name in ('contextra-py', 'contextra-core') else 'WP 2.4'` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-store/src/lib.rs` | 27 | `//!   `contextra-py` und der uebergeordneten Facade `contextra`.` |
| `crates/contextra-types/src/error.rs` | 117 | `/// (such as `contextra-py` and `contextra-mcp`).` |
| `crates/contextra-types/src/error_dto.rs` | 5 | `//! Bei der Übertragung von Fehlern über FFI-Grenzen (z. B. zu Python via `contextra-py` oder` |
| `crates/contextra-types/src/lib.rs` | 26 | `//!   `contextra-mvcc`, `contextra-ports`, `contextra-privacy`, `contextra-py`, `contextra-rank`,` |
| `crates/contextra-types/src/types/domain/misc.rs` | 84 | `/// to HNSW and BM25 sub-searches. All upstream layers (contextra-mcp, contextra-py,` |
| `crates/contextra-types/src/types/domain/misc.rs` | 85 | `/// contextra-py) MUST reference this constant — never duplicate the literal `1000`.` |
| `docs/CHANGELOG.md` | 256 | `\| `2026-09-16T16:47:30Z` \| `crates/contextra-py/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `29edb6a4` \| `-` \| `-` \| // REVIEW-PASS[1/2] Systematischer Review der contextra-py PyO3 FFI Bindings (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-16T16:47:30Z) (SESSION: 29edb6a4) \|` |
| `docs/CHANGELOG.md` | 316 | `\| `2026-09-10T19:22:55Z (SESSION: 0b2ff57d)` \| `crates/contextra-py/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| PyO3 FFI bindings bridging Contextra embedded vector DB functionality to Python. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 232 | `\| `python3`, `venv`, `pip`, `maturin`, `numpy`, `pytest` \| `contextra-py⚠` (`just py-test`) \| J-19 \| `apt` + `pip` im venv \| `py-test` NICHT GEPRÜFT \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 291 | `# 4) Python-Umgebung für contextra-py⚠` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1098 | `\| Python 3 + `maturin`, `numpy`, `pytest`, `matplotlib` \| `contextra-py⚠`, Diagramme \| L-20, L-18 \| `nix develop` (PyO3-Toolchain) oder venv \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1158 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-py`, `contextra-sandbox`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1170 | `3) cargo build -p contextra-sandbox⚠ -p contextra-py⚠ -p contextra-infer-ollama⚠ -p contextra-infer-onnx⚠ (Non-Default-Member; Fehler als ENV/CODE klassifizieren)` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |

### Crate `contextra-router`

- **Gesamtanzahl Referenzen:** 36 (Imports: 0, Usages: 0, Config/CI: 9, Docs: 27)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `CODEOWNERS` | 28 | `/crates/contextra-router/              @contextra/intelligence-tier3` |
| `deny.toml` | 68 | `"contextra-router",` |
| `parse_all_remnants.py` | 23 | `"contextra-router",` |
| `scripts/contextra_full_audit.py` | 104 | `"contextra-router", "contextra-sandbox", "contextra-wire", "contextra-audit-export",` |
| `scripts/contextra_full_audit.py` | 113 | `RUECKBAU_GEPLANT = {"contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest",` |
| `scripts/contextra_full_audit.py` | 1150 | `ENTFERNEN_V1 = ["contextra-kvcache", "contextra-agent", "contextra-router", "contextra-ingest", "contextra-privacy",` |
| `tools/generate_migration_edges.py` | 9 | `contextra-kvcache, contextra-router.` |
| `tools/generate_migration_edges.py` | 32 | `'contextra-router': 'contextra_router',` |
| `tools/generate_migration_edges.py` | 44 | `'contextra-router': 'Feature `router` in Fassade entfernen; `contextra-py` anpassen.',` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-ports/src/vector_index.rs` | 159 | `// AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md)` |
| `crates/contextra-text/src/lib.rs` | 23 | `//! - **Wird genutzt von**: `contextra-engine`, `contextra-db`, `contextra-router`, `contextra-agent`` |
| `crates/contextra-types/src/lib.rs` | 27 | `//!   `contextra-router`, `contextra-simd`, `contextra-store`, `contextra-testkit`, `contextra-text`,` |
| `crates/contextra-types/src/retrieval_strategy.rs` | 9 | `//! ACHTUNG: NICHT zu verwechseln mit `contextra_router::RoutingStrategy`` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 22 | `\| Z. 42–56 \| `BENCHES` listet Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). \| Schritt `befund_bench_fuzz_dateien` schlägt fehl (FAIL); Benchmarks für diese Crates können nicht existieren. \|` |
| `docs/AUDIT_SCRIPT_VALIDATION.md` | 26 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthält noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldet FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. Dies ist kein Code-Bug, sondern eine nicht synchronisierte Konfigurationsdatei. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 24 | `\| Z. 42–56 \| `BENCHES` enthielt Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench` sowie `deletion_proof_latency_bench`. \| Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). Die Benchmarks existieren nicht mehr. \| Schritt `befund_bench_fuzz_dateien` und Benchmarks scheiterten mit FAIL. \| `BENCHES` auf die existierenden Benchmarks der aktiven Crates (`mvcc`, `crypto`, `vector`, `store`, `simd`) bereinigt. \|` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 28 | `\| Z. 320–335 \| `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern \| `capabilities.toml` enthielt noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). \| `befund_maturity_claims` meldete FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. \| `capabilities.toml` synchronisiert (Entfernung von 21 Nicht-Mitgliedern, Aufnahme von `contextra-durable-fs` und `contextra-arch-tests`). \|` |
| `docs/CHANGELOG.md` | 260 | `\| `2026-09-16T16:17:21Z` \| `crates/contextra-router/src/lib.rs` \| `REVIEW-PASS` \| `-` \| `cec8b8e9` \| `-` \| `-` \| // REVIEW-PASS[1/2] (TS: 2026-09-16T16:17:21Z) (SESSION: cec8b8e9) (PRÜFER-KONTEXT: FRESH) \|` |
| `docs/CHANGELOG.md` | 318 | `\| `2026-09-10T19:16:25Z (SESSION: 3f3e4637)` \| `crates/contextra-router/src/dispatch.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Client-seitiger MCP-Dispatch-Mechanismus über Stdio JSON-RPC 2.0 (ADR-010). \|` |
| `docs/CHANGELOG.md` | 319 | `\| `2026-09-10T19:16:25Z (SESSION: 3f3e4637)` \| `crates/contextra-router/src/outcome.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Outcome-Typen und DecisionId-Identifier für konformale Router-Kalibrierung. \|` |
| `docs/CHANGELOG.md` | 320 | `\| `2026-09-10T19:16:25Z (SESSION: 3f3e4637)` \| `crates/contextra-router/src/profile.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| SLM-Profildefinitionen und konformale Quantil-Kalibrierung (Gibbs & Candès). \|` |
| `docs/CHANGELOG.md` | 321 | `\| `2026-09-10T19:16:25Z (SESSION: 3f3e4637)` \| `crates/contextra-router/src/router/mod.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Haupt-Routing-Engine für Hybrid-Search-Kontext auf SLM-Profile. \|` |
| `docs/CHANGELOG.md` | 322 | `\| `2026-09-10T19:16:25Z (SESSION: 3f3e4637)` \| `crates/contextra-router/src/serde_helpers.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Serde-Hilfsfunktionen für deterministische JSON-Serialisierung. \|` |
| `docs/CHANGELOG.md` | 499 | `\| `` \| `crates/contextra-ports/src/graph_index.rs` \| `AI-TAG` \| `-` \| `-` \| `RESOLVED` \| `-` \| // AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md) \|` |
| `docs/CHANGELOG.md` | 500 | `\| `` \| `crates/contextra-ports/src/lifecycle.rs` \| `AI-TAG` \| `-` \| `-` \| `RESOLVED` \| `-` \| // AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md) \|` |
| `docs/CHANGELOG.md` | 501 | `\| `` \| `crates/contextra-ports/src/vector_index.rs` \| `AI-TAG` \| `-` \| `-` \| `RESOLVED` \| `-` \| // AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md) \|` |
| `docs/CHANGELOG.md` | 510 | `\| `` \| `crates/contextra-router/src/router/dispatch_core.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Kern-Routing-Dispatch und Kaskaden-/Bandit-Profilauswahl für RouterEngine. \|` |
| `docs/CHANGELOG.md` | 511 | `\| `` \| `crates/contextra-router/src/router/lifecycle.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Konstruktions- und Profilverwaltungs-Methoden (Hot-Reload) für RouterEngine. \|` |
| `docs/CHANGELOG.md` | 512 | `\| `` \| `crates/contextra-router/src/router/outcomes.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Outcome-Recording, Kalibrierungs- und Drift-Statistiken für RouterEngine. \|` |
| `docs/CHANGELOG.md` | 513 | `\| `` \| `crates/contextra-router/src/router/tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Unit-Tests für RouterEngine. \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 417 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 432 | `contextra-router⚠: bandit-routing, egress-sherman-morrison, flow-corrected-thompson, cloud-egress-guard · contextra-rank: dibud` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/cleanup/g0-baseline.txt` | 315 | `Checking contextra-router v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-router)` |

### Crate `contextra-sandbox`

- **Gesamtanzahl Referenzen:** 28 (Imports: 1, Usages: 0, Config/CI: 8, Docs: 19)

#### Aktive Code-Stellen
| Datei | Zeile | Typ | Quelltext |
|---|---|---|---|
| `crates/contextra-engine/src/merge_operator_adapter.rs` | 30 | `IMPORT` | `use contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction};` |

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `Cargo.toml` | 114 | `# contextra-cluster, contextra-sandbox, contextra-saos-agent wurden in Phase 0 ausgelagert. <!-- crate-ref-ignore -->` |
| `deny.toml` | 69 | `"contextra-sandbox",` |
| `deny.toml` | 82 | `{ crate = "wasmtime", wrappers = ["contextra-sandbox"] },` |
| `parse_all_remnants.py` | 24 | `"contextra-sandbox",` |
| `scripts/contextra_full_audit.py` | 104 | `"contextra-router", "contextra-sandbox", "contextra-wire", "contextra-audit-export",` |
| `scripts/contextra_full_audit.py` | 115 | `"contextra-bench", "contextra-py", "contextra-sandbox", "contextra-infer-onnx",` |
| `scripts/contextra_full_audit.py` | 1152 | `"contextra-sandbox", "contextra-infer-onnx", "contextra-infer-ollama", "contextra-audit-export",` |
| `tools/generate_migration_edges.py` | 54 | `elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `crates/contextra-engine/src/merge_operator_adapter.rs` | 11 | `// SIEHE AUCH: AGENTS.md §4.18, Spec §4.12, contextra-sandbox WasmMergeFunction` |
| `crates/contextra-engine/src/merge_operator_adapter.rs` | 20 | `//! [`contextra_sandbox::WasmMergeFunction`] via a dedicated worker thread hosting an isolated` |
| `crates/contextra-store/src/compaction/merge_operator.rs` | 11 | `/// Implementations running untrusted guest code (e.g. WASM via `contextra-sandbox`) MUST` |
| `crates/contextra-store/tests/compaction_merge_operator.rs` | 550 | `// V-RUECKBAU: contextra-sandbox wurde rückgebaut (Phase 0).` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 38 | `6. **Kompilierungsfehler in `engine` und Testsuites:** Verbliebene Aufrufe von gelöschten Typen/Methoden (`graph_index`, `relate`, `get_community`, `contextra_sandbox`), wodurch Tests und Builds scheitern.` |
| `docs/AUDIT_VALIDIERUNG_UND_BEFUND_AUSWERTUNG.md` | 63 | `- `crates/contextra-store`: Bereinigung verwaister `contextra_sandbox`-Imports in Tests.` |
| `docs/CHANGELOG.md` | 205 | `\| `2026-10-01T00:00:00Z` \| `crates/contextra-sandbox/src/merge.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Pure, deterministic WASM merge function executor (§4.18) \|` |
| `docs/CHANGELOG.md` | 229 | `\| `2026-09-27T22:42:00Z (SESSION: 7c14554a)` \| `crates/contextra-sandbox/src/wasi.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| WASI preview1 host function implementations (fd_read, fd_write, proc_exit, clock_time_get, random_get) \|` |
| `docs/CHANGELOG.md` | 238 | `\| `2026-09-25T00:00:00Z` \| `crates/contextra-sandbox/src/approval.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Human Approval Data Model & Pure State Machine for WASM Guest Execution (§4.18) \|` |
| `docs/CHANGELOG.md` | 272 | `\| `2026-09-15T16:00:00Z (SESSION: acf8fe72)` \| `crates/contextra-sandbox/src/capabilities.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Capability Whitelist & Configuration for WASM Execution Boundary \|` |
| `docs/CHANGELOG.md` | 273 | `\| `2026-09-15T16:00:00Z (SESSION: acf8fe72)` \| `crates/contextra-sandbox/src/executor.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| WASM execution engine with WASI and fuel enforcement \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 233 | `\| WASM-Target (`wasm32-wasip1`, ggf. `wasm32-unknown-unknown`) \| `contextra-sandbox⚠` \| J-19 \| `rustup target add …` \| ENV-Befund \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1099 | `\| WASM-Targets \| `contextra-sandbox⚠` \| L-21 \| `rustup target add …` \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1158 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-py`, `contextra-sandbox`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1170 | `3) cargo build -p contextra-sandbox⚠ -p contextra-py⚠ -p contextra-infer-ollama⚠ -p contextra-infer-onnx⚠ (Non-Default-Member; Fehler als ENV/CODE klassifizieren)` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/analysis/SPEZIFIKATION_LOESCHUNG_PRODUKTIONSREIFE_2026-11-01.md` | 126 | `- **Store & Test-Targets (WP-1.3 / WP-2.3):** In `contextra-store` Tests existierten Referenzen auf nicht deklarierte Features (`encryption-at-rest` anstelle standardmäßiger Pflichtverschlüsselung) und verwaiste Imports (`contextra_sandbox`).` |

### Crate `contextra-wire`

- **Gesamtanzahl Referenzen:** 29 (Imports: 0, Usages: 0, Config/CI: 11, Docs: 18)

#### Build- & CI-Konfigurationen
| Datei | Zeile | Kontext |
|---|---|---|
| `.agents/governance/ratchet.toml` | 12 | `contextra-wire = 27` |
| `.agents/governance/unsafe-safety-baseline.toml` | 8 | `"crates/contextra-wire/src/contextra_generated.rs" = 0` |
| `.agents/jules/setup/environment_script.sh` | 48 | `# flatc-SHIM: crates/contextra-wire/build.rs ruft bei vorhandenem `flatc` IMMER neu generieren auf und` |
| `.agents/jules/setup/refresh-context.sh` | 108 | `echo "- Unsafe nur in: contextra-simd, contextra-sys, contextra-wire"` |
| `.agents/skills/contextra-invariants-gate/scripts/check_invariants.sh` | 14 | `grep -v "crates/contextra-wire" \| \` |
| `.gemini/skills/contextra-invariants-gate/scripts/check_invariants.sh` | 14 | `grep -v "crates/contextra-wire" \| \` |
| `justfile` | 73 | `"contextra-wire:fuzz_flatbuffers_ipc"` |
| `parse_all_remnants.py` | 25 | `"contextra-wire",` |
| `scripts/contextra_full_audit.py` | 104 | `"contextra-router", "contextra-sandbox", "contextra-wire", "contextra-audit-export",` |
| `scripts/contextra_full_audit.py` | 114 | `"contextra-privacy", "contextra-mcp", "contextra-wire", "contextra-core",` |
| `scripts/contextra_full_audit.py` | 1151 | `"contextra-mcp", "contextra-wire", "contextra-core", "contextra-bench", "contextra-py",` |

#### Dokumentation, Modul-Header & Kommentare
| Datei | Zeile | Kontext / Auszug |
|---|---|---|
| `.agents/jules/context/SESSION_START.md` | 10 | `- Unsafe nur in: contextra-simd, contextra-sys, contextra-wire` |
| `docs/CHANGELOG.md` | 224 | `\| `2026-09-27T22:49:25Z (SESSION: 72f4c80d)` \| `crates/contextra-wire/src/adapter.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Zero-copy byte buffer slice wrapper (WireBuffer) for FlatBuffers IPC messages. \|` |
| `docs/CHANGELOG.md` | 225 | `\| `2026-09-27T22:49:25Z (SESSION: 72f4c80d)` \| `crates/contextra-wire/src/jsonrpc.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Generic JSON-RPC 2.0 protocol request/response structures for Contextra IPC. \|` |
| `docs/CHANGELOG.md` | 226 | `\| `2026-09-27T22:49:25Z (SESSION: 72f4c80d)` \| `crates/contextra-wire/src/lib.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters. \|` |
| `docs/CHANGELOG.md` | 263 | `\| `2026-09-16T00:00:00Z` \| `crates/contextra-wire/tests/ipc_tests.rs` \| `FILE-CONTEXT` \| `-` \| `-` \| `-` \| `-` \| Integrationstests, Fault-Injection und Nebenläufigkeitstests für FlatBuffers IPC (contextra-wire) \|` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 455 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-graph`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 470 | `contextra-wire⚠: FlatBuffers Encode→Decode-Roundtrip für zufällige Nachrichten; Decode beliebiger Bytes darf nie paniken (nur Err); Stabilität: gleiche Eingabe → gleiche Bytes (Determinismus).` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 952 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 963 | `SCHRITT 3 — unsafe-Inventur: rg -n "unsafe" crates -g '*.rs' (Zählung je Crate und je Datei). Gegen die erlaubten Inseln abgleichen: capabilities.toml nennt contextra-simd, contextra-sys, contextra-wire; Dev-Guide/AGENTS.md nennen zusätzlich contextra-vector/src/distance.rs und sprechen von #![forbid(unsafe_code)] bzw. unsafe_code = "deny". Ermittle, welche Variante der Code tatsächlich lebt (Crate-Attribute #![forbid]/#![deny], Workspace-Lints in Cargo.toml) und liste JEDEN Widerspruch als Befund (S3). ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) und lint-unsafe-slices zitieren. Je unsafe-Block: hat er einen // SAFETY-Kommentar, der die Invariante nennt? Blöcke ohne: Liste.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1280 | `> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-core`, `contextra-graph`, `contextra-kvcache`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.` |
| `docs/Jules-Prompts/CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT_angepasst.md` | 1292 | `SCHRITT 4 — Unsafe-Inseln besonders: contextra-simd, contextra-sys (Memory-Mapping), contextra-wire⚠ (FlatBuffers), contextra-vector/src/distance.rs. Je unsafe-Block: Welche Invariante muss gelten (Ausrichtung, Länge, Lebensdauer, Aliasing)? Teste sie mit ASan + gezielten Grenzfall-Eingaben (Länge 0, 1, nicht Vielfaches der SIMD-Breite, Slice am Seitenende, mmap-Datei abgeschnitten → SIGBUS-Risiko: dokumentiere, ob Truncation einer gemappten Datei durch einen fremden Prozess das Programm abstürzen lässt (contextra-sys MemoryMap, DiskANN load_node/MmapIndex)).` |
| `docs/Jules-Prompts/Contextra___Kommentier-Prompts_angepasst.md` | 2 | `> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 3 | `> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 4 | `> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 56 | `12. UNSAFE-INSELN: ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 \| tail -20 — erwartet: ONLY contextra-simd, contextra-sys, contextra-wire, contextra-crypto als unsafe_island=true.` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 972 | `grep -rn "unsafe" crates/ --include="*.rs" \| grep -v "contextra-crypto\\|contextra-simd\\|contextra-sys\\|contextra-wire⚠" \| grep -v "forbid\\|allow\\|deny\\|//.*unsafe" \| grep -v "/tests/" \| grep -v "_test.rs" \| tee /tmp/audit-zeropanic-unsafe.log \| wc -l` |
| `docs/Jules-Prompts/Wiederverwendbare_Audit-Prompts_angepasst.md` | 1055 | `F4 FLATBUFFERS-IPC (contextra-wire⚠): Korrumpierte FlatBuffer-Bytes — Ziel: Fehler statt OOB-Read.` |
| `docs/cleanup/g0-baseline.txt` | 289 | `Checking contextra-wire v0.1.0 (/home/freddy/.local/share/Trash/files/contextra-222/crates/contextra-wire)` |

---

## 4. Architektonische Bewertung & Handlungsempfehlungen für die SPEC-Umsetzung

Basierend auf `docs/SPEC.md` §15.4 (Arbeitspakete) und den Architektur-Vetos D-24, D-45, D-84 ergeben sich folgende zwingende Maßnahmen:

### 4.1 Entkopplung von `contextra-graph` (WP-1.1, SPEC F-05)
- **Befund:** `contextra-graph` ist mit 112 Referenzen die größte Altlast. Insbesondere `contextra-engine` enthält in `collection/tests/`, `background_workers/`, `transaction/` und Integrationstests 48 aktive Imports auf `contextra_graph::CsrGraph` und `HyperEdge`.
- **Maßnahme:** Bereinigung der Engine von Graph-Abhängigkeiten gemäß SPEC §1.4 und §5.1. CSR-Graph und Hyperedges entfallen aus dem v1-Kern. Transaktionstests sind auf KV-, Vektor- und Volltextoperationen umzustellen.

### 4.2 Entfernung von `contextra-db` Resten (WP-1.1, SPEC D-24, D-45)
- **Befund:** In `benchmarks/` existieren noch Verweise auf `contextra_db::Contextra` und Fusion-Funktionen. In `crates/contextra/tests/llm_query_rewriter.rs` wird `use contextra_db::QueryRewriter;` importiert.
- **Maßnahme:** `contextra-db` vollständig entfernen. Benchmarks auf `contextra::Contextra` oder `contextra-engine` umstellen. Veraltete QueryRewriter-Ports eliminieren (D-24).

### 4.3 Bereinigung von WASM-Merge und `contextra-sandbox` (WP-1.2, SPEC D-84)
- **Befund:** `crates/contextra-engine/src/merge_operator_adapter.rs` importiert noch `contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction}`.
- **Maßnahme:** Gemäß SPEC D-84 entfällt `WasmMergeOperator`. Die Datei `merge_operator_adapter.rs` sowie `tests/wasm_merge_operator.rs` sind zurückzubauen.

### 4.4 Bereinigung von Inferenz-Altlasten (`contextra-infer-*`)
- **Befund:** `contextra-engine/src/contextra_impl/lifecycle.rs` enthält noch Aufrufe von `contextra_infer_onnx::ensure_onnx_model_download` unter `#[cfg(feature = "onnx")]`.
- **Maßnahme:** Gemäß SPEC D-24 entfällt der Inference-Port aus dem Kern. Embeddings werden der Engine extern übergeben.

### 4.5 Bereinigung von Konfigurations- und CI-Dateien (WP-1.3, SPEC F-17)
- **Befund:** In `deny.toml` existieren Ausnahmeregeln und Wrapper-Definitionen für `contextra-mcp`, `contextra-py`, `contextra-db`, `contextra-graph`, `contextra-sandbox` etc. `justfile` führt Targets für Fuzzing von `contextra-wire` und `contextra-mcp`.
- **Maßnahme:** `deny.toml` bereinigen. Veraltete Fuzzing-Targets und ungenutzte Workflows bereinigen.

### 4.6 Bereinigung veralteter Modul-Header (WP-1.5, SPEC D-91, F-35)
- **Befund:** In `crates/contextra-types/src/lib.rs`, `contextra-store/src/lib.rs`, `contextra-crypto/src/lib.rs` und `contextra-text/src/lib.rs` listen die `//!`-Header nicht mehr existierende Konsumenten wie `contextra-agent`, `contextra-cognition`, `contextra-router`, `contextra-mcp`, `contextra-py`.
- **Maßnahme:** Angleichung der Modul-Header an die kanonische Workspace-Matrix aus SPEC §5.3.
