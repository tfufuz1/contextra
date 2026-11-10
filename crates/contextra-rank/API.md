# API-Dokumentation: `contextra-rank`

Dieses Dokument beschreibt die öffentlichen Schnittstellen und Kernmodule von `contextra-rank` (Ranking & Kernlogik).

## 1. Modulübersicht
- **`fusion`**: Algorithmen zur Zusammenführung mehrdimensionaler Suchergebnislisten:
  - `weighted_reciprocal_rank_fusion`: **Produktivpfad** für gewichtete Multi-Signal-Rangfusion auf `SearchResult`.
  - `reciprocal_rank_fusion`: Convenience-Wrapper für ungewichtete Listen mit Einheitsgewichten ($w=1.0$) und Standardglättung $k=60$.
  - `rrf_fuse`: **Mathematisches Referenz-Orakel** auf `ScoredItem`-Listen mit frei konfigurierbarem $k$.
  - `score_normalized_fusion_with_options`: Linearkombination min-max-normalisierter Scores.
  - `fusion::mrrf`: Margin-modulierte Gewichtsadaption (`modulate_and_renormalize_weights`, `weighted_reciprocal_rank_fusion_mrrf`).
  - `resonance`: Resonanz-Bonus bei Koinzidenz über mehrere Retrieval-Kanäle (`apply_resonance_bonus`, `fuse_signals`).
  - `topk::BoundedTopK`: Speicher- und zeiteffizienter Min-Heap für Top-K-Selektion.
- **`calibration`** (Feature `experimental-rank`):
  - `IsotonicCalibrator`: Nicht-parametrische Wahrscheinlichkeitskalibrierung via PAVA (Pool-Adjacent Violators Algorithm).
  - `PlattScaler`: Parametrische logistische Regression.
  - `ConformalCalibrator`, `AdaptiveConformalCalibrator`: Finite-Sample-Abdeckungsgarantien.
  - `calibrate_score`: Universelle Auswertungsfunktion für `Calibrator`.
- **`drift`** (Feature `experimental-rank`):
  - `DriftDetector`, `DriftStatus`: Kontinuierliche Überwachung von Score-Verteilungen zur Laufzeit.
- **`explain`**:
  - `explain`, `ExplanationEntry`, `RetrievalExplanation`: Erzeugung strukturierter, deutschsprachiger Herkunftserklärungen aus `ProvenanceRecord`.
- **`metrics`**:
  - IR-Evaluierungsmetriken: `mean_reciprocal_rank`, `ndcg_at_k`, `precision_at_k`, `recall_at_k`, `reciprocal_rank`.
- **`dibud`** (Feature `dibud` / `experimental-rank`):
  - Streaming-Fusion unter strikten Zeit- und Kanalbudgets (`fuse_exact_prefix`, `DiBudFusionState`).

## 2. Mathematische Verträge und Invarianten
- **Gleichstandsauflösung (Tie-Break):** Bei identischen Scores erfolgt die Sortierung stets deterministisch nach Dokument-ID lexikografisch aufsteigend (`score DESC`, danach `id ASC`).
- **Permutationsinvarianz:** Bei identischen Gewichten führt jede beliebige Reihenfolge der Eingabekanäle zum identischen Ranking und identischen Scores.
- **Zero-Panic:** `#![forbid(unsafe_code)]`, fail-closed Validierung auf NaN und unendliche Werte.
