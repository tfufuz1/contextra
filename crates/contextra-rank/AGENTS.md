# AGENTS.md — contextra-rank
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.22, §A2, §A3, §A4, §0, §4, §7, §17, §21, §D

## 1. Zweck
`contextra-rank` stellt Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen für das Contextra Cognitive OS bereit (RRF Fusion, Normalisierte Fusion, Isotonische Kalibrierung, Platt Scaler).
Er ist strikt synchron (P26) und erzwingt `#![forbid(unsafe_code)]`.

## 2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte der Fusion-, Kalibrierungs- und Ranking-Schnittstellen. |
| `src/drift.rs` | Drift-Statistiken für Ranking-Scores (`ScoredDocumentDriftTracker`). |
| `src/explain.rs` | Score-Erklärbarkeit (`RankingExplanation`, `SignalScoreContribution`). |
| `src/calibration/mod.rs` | Kalibrierungs-Modul-Re-Exporte (`IsotonicCalibrator`, `PlattScaler`). |
| `src/calibration/isotonic.rs` | `IsotonicCalibrator` zur nicht-parametrischen isotopen Score-Regression. |
| `src/calibration/platt.rs` | `PlattScaler` zur sigmoidalen Score-Transformation. |
| `src/fusion/mod.rs` | Fusion-Subsystem-Re-Exporte (RRF, Normalized, Resonance). |
| `src/fusion/rrf.rs` | Reciprocal Rank Fusion (RRF) über mehrere Signal-Slices. |
| `src/fusion/normalized.rs` | Score-Normalisierte Fusion (Z-Score / Min-Max). |
| `src/fusion/resonance.rs` | Semantische Resonanz- und Kohärenz-Bonus-Scorer. |
| `src/fusion/global.rs` | Globale 4-Signal-Fusion (Vector, Text, Graph, Rerank/Metadata). |
| `src/fusion/provenance.rs` | Provenance-Tracking für Fusions-Ergebnisse. |
| `src/fusion/signal.rs` | Signal-Eingabe-Abstraktionen für Fusions-Pipelines. |
| `src/fusion/topk.rs` | Bounded Top-K Selection und Aggregation. |
| `src/fusion/types.rs` | Fusion-spezifische Typen und Gewichtungskonfigurationen. |

## 3. Invarianten
- **NO-AD-HOC-SIGMOID:** Sigmoidale Kalibrierungs- und Transformationslogik darf NICHT ad-hoc außerhalb von `contextra-rank` (z. B. in `PlattScaler`) reimplementiert werden.

## 4. Verboten / Anti-Patterns
- **VERBOTEN:** Async-Operationen, I/O oder `tokio`-Abhängigkeiten in `contextra-rank` einfügen.
- **VERBOTEN:** Unsafe Rust (`#![forbid(unsafe_code)]`).
- **VERBOTEN:** Ad-hoc-Ranking-Formeln verstreut in Storage- oder DB-Crates implementieren.

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Reiner synchroner CPU-gebundener Ring-0-Code (P26).
- Alle Datenstrukturen sind thread-safe, reentrant und lock-frei oder nutzen unveränderliche Referenzen.

## 6. Verifikation

```bash
cargo xtask crate-context contextra-rank
cargo xtask check-nan-hot-loop
cargo xtask determinism-check
cargo xtask doctrine-scan --crate contextra-rank
cargo xtask check-ring0-async-purity
cargo xtask check-unsafe-islands
cargo test -p contextra-rank --locked
```

## 7. Bekannte Lücken / SOLL (EP3 & Phase 3 Roadmap)

- **Deterministische Floats**: Durchgängige Verwendung von `f64::total_cmp` zur Vermeidung von NaN-Panic und Nicht-Determinismus.
