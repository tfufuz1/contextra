# Übergabebericht MISSION `RANK-1`

## 1. Soll/Ist-Delta
- **Module:** `fusion`, `calibration`, `drift`, `explain`, `dibud`, `metrics` vollständig vorhanden und verifiziert.
- **API Standardisierung:**
  - `ScoredItem` als universeller Crate-lokaler Typ eingeführt (`id`, `score`, `provenance`), kompatibel zu `SearchResult`.
  - `rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>` implementiert und mathematisch gehärtet ($k \ge 1$, finite Scores, Duplikat-Deduplizierung je Einzelliste, exakte Kommutativität durch per-Doc Rang-Sortierung, deterministischer Tie-Break).
  - `Calibrator` Container mit `format_version` und `calibrate_score(raw: f32, &Calibrator) -> Result<f32>` bereitgestellt.
- **Mathematische Verträge & Invarianten:**
  - `INV-RANK-DETERMINISM-1`: Durch deterministischen Tie-Break und geordnete Aggregation garantiert und getestet.
  - `INV-RANK-MONOTONIC-1`: Isotonische Kalibrierung (PAVA) getestet und per Proptest bewiesen.
  - Kommutativität ungewichteter RRF per Unit- und Property-Test bewiesen.
  - Conformal Prediction: Empirische Überdeckung $\ge 1 - \alpha$ auf synthetischen Verteilungen verifiziert.
  - Streaming-Fusion (`dibud`): Exakte Äquivalenz zur Batch-Fusion verifiziert.
- **Hygiene & Zero-Panic:**
  - 0 Panics im Produktionscode (`tools/zero_panic_analyzer.py` bestätigt 0 Treffer).
  - Veraltete Marker (`// ANCHOR`, `// STAND:`, `// SESSION:`, `// FILE-CONTEXT`) entfernt (D-91).
  - SystemClock mit `// ALLOW-DETERMINISM: Wall clock baseline in standalone SystemClock (P28)` annotiert.

## 2. Invarianten & Tests
- `crates/contextra-rank/tests/rank_invariants.rs`:
  - `test_inv_rank_determinism_1`
  - `test_inv_rank_monotonic_1` & `prop_inv_rank_monotonic_1`
  - `test_rrf_commutativity` & `prop_rrf_commutativity`
  - `test_conformal_empirical_coverage_guarantee`
  - `test_dibud_streaming_vs_batch_equivalence`

## 3. Benchmarks
Criterion-Benchmarks in `benches/rank_bench.rs` implementiert:
- 2, 4, 8 Listen $\times$ 100, 1.000, 10.000 Kandidaten.
- Platt-, Isotonische- und Conformal-Vorhersagen.
- DiBud Streaming Push & Finish.

## 4. Offene Punkte / Externe Abhängigkeiten
- Keine Änderungen an geschützten Dateien oder fremden Crates.
- `Cargo.lock` unberührt bzw. wiederhergestellt.
