# AGENTS.md — contextra-rank
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.22, §A2, §A3, §A4, §0, §4, §7, §17, §21, §D

1. Zweck
`contextra-rank` stellt Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen für das Contextra Cognitive OS bereit (RRF Fusion, Normalisierte Fusion, Isotonische Kalibrierung, Platt Scaler, Conformal Calibration).
Er ist strikt synchron (P26) und erzwingt `#![forbid(unsafe_code)]`.

2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte der Fusion-, Kalibrierungs- und Ranking-Schnittstellen. |
| `src/drift.rs` | Drift-Statistiken für Ranking-Scores (`ScoredDocumentDriftTracker`). |
| `src/explain.rs` | Score-Erklärbarkeit (`RankingExplanation`, `SignalScoreContribution`). |
| `src/calibration/mod.rs` | Kalibrierungs-Modul-Re-Exporte (`IsotonicCalibrator`, `PlattScaler`, `AdaptiveConformalCalibrator`). |
| `src/calibration/isotonic.rs` | `IsotonicCalibrator` zur nicht-parametrischen isotopen Score-Regression. |
| `src/calibration/platt.rs` | `PlattScaler` zur sigmoidalen Score-Transformation. |
| `src/calibration/conformal.rs` | `AdaptiveConformalCalibrator` und `ConformalCalibrator` Trait (`INV-CALIBRATION-CONFORMAL-1`). |
| `src/fusion/mod.rs` | Fusion-Subsystem-Re-Exporte (RRF, Normalized, Resonance). |
| `src/fusion/rrf.rs` | Reciprocal Rank Fusion (RRF) über mehrere Signal-Slices. |
| `src/fusion/normalized.rs` | Score-Normalisierte Fusion (Z-Score / Min-Max). |
| `src/fusion/resonance.rs` | Semantische Resonanz- und Kohärenz-Bonus-Scorer. |
| `src/fusion/global.rs` | Globale 4-Signal-Fusion (Vector, Text, Graph, Rerank/Metadata). |
| `src/fusion/provenance.rs` | Provenance-Tracking für Fusions-Ergebnisse. |
| `src/fusion/signal.rs` | Signal-Eingabe-Abstraktionen für Fusions-Pipelines. |
| `src/fusion/topk.rs` | Bounded Top-K Selection und Aggregation. |
| `src/fusion/types.rs` | Fusion-spezifische Typen und Gewichtungskonfigurationen. |
| `src/dibud/` | Dynamic Information Budget (DiBud) Treiber, Typen und Zustandsverwaltung (`feature = "dibud"`). |

3. Invarianten
- **INV-CALIBRATION-CONFORMAL-1:** Adaptive Conformal Calibration wird über `AdaptiveConformalCalibrator` im Code bereitgestellt.
- **NO-AD-HOC-SIGMOID:** Sigmoidale Kalibrierungs- und Transformationslogik darf NICHT ad-hoc außerhalb von `contextra-rank` (z. B. in `PlattScaler`) reimplementiert werden.
- **NO-DUPLICATE-RECALIBRATE:** `recalibrate_conformal` ist eine High-Level-Routing-Aktion und lebt primär in `contextra-router` (Spec Anhang C Nr. 2); `contextra-rank` liefert die reinen Kalibrierungs-Algorithmen.

4. Verboten / Anti-Patterns
- **VERBOTEN:** Async-Operationen, I/O oder `tokio`-Abhängigkeiten in `contextra-rank` einfügen.
- **VERBOTEN:** Unsafe Rust (`#![forbid(unsafe_code)]`).
- **VERBOTEN:** Ad-hoc-Ranking-Formeln verstreut in Storage- oder DB-Crates implementieren.

5. Nebenläufigkeit, Async- und Lock-Regeln
- Reiner synchroner CPU-gebundener Ring-0-Code (P26).
- Alle Datenstrukturen sind thread-safe, reentrant und lock-frei oder nutzen unveränderliche Referenzen.

6. Verifikation
- `cargo test -p contextra-rank`

7. Bekannte Lücken / SOLL
- DiBud-Integration ist hinter dem Feature `dibud` gesealt.
