# AGENTS.md — contextra-rank — Fusion & Reranking

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-rank.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-rank.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §5 (Löschkette), §6 (ReadGate), §13 (INV-5 Determinismus, Z-E8)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E1 (Autonomer Domänenbaustein)  
**Kern-Closure:** ids, env (2 Mitglieder)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Mathematische Score-Fusion (RRF, Min-Max, Sigmoid), Relevanz-Reranking und Tie-Breaking für hybride Suchen.

---

## 2. Kern-Richtlinien für Agenten

- **Reines Rechen-Crate:** Keine eigene Persistenz, keine Dateihandles, keine Speicherung von Payloads.
- **Deterministisches Tie-Breaking (INV-5):** Bei identischen Scores erfolgt der Tie-Break zwingend nach `DocId` aufsteigend.
- **Datenschutz-Invarianz (Z-E8):** Gelöschte Dokumente dürfen die relative Relevanz und Rangfolge aktiver Dokumente niemals verzerren. Daher verarbeitet `contextra-rank` nur Kandidaten, die das `ReadGate` bereits erfolgreich passiert haben.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-rank` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** 4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection for Contextra Cognitive OS
- **src:** 31 Dateien, 4440 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (2):** `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 5):** `ahash`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (3):** `contextra-bench`, `contextra-db`, `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (10):** `default`, `dibud`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank`, `ports-compat`, `std`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (28): `AdaptiveConformalCalibrator`, `BoundedTopK`, `ConfigFingerprint`, `DiBudFusionState`, `DiBudOutcome`, `DocId`, `DriftDetector`, `ExplanationEntry`, `FusionBudget`, `FusionWeights`, `GlobalFusionConfig`, `GlobalFusionStrategy`, `IsotonicCalibrator`, `PlattScaler`, `PortsClockAdapter` … (+13)
- enum (11): `BudgetedChannel`, `Calibrator`, `ConformalError`, `DiBudStep`, `DriftStatus`, `FusionStrategy`, `MetadataMergePriority`, `RankError`, `Signal`, `SignalFailurePolicy`, `SignalKind`
- trait (4): `Clock`, `ConformalCalibrator`, `IsotonicPortsClockExt`, `ResonanceModifier`
- fn (103): `accesses`, `alpha`, `analyze`, `apply`, `apply_resonance_bonus`, `apply_temporal_validity_filter`, `apply_temporal_validity_filter_at`, `apply_temporal_validity_filter_with_clock`, `apply_temporal_validity_filter_with_ports_clock`, `as_bytes`, `as_str`, `as_u64`, `bm25`, `build`, `cached_ece` … (+88)
- type (3): `FusedScore`, `FusionResult`, `Result`
- const (6): `DEFAULT_INITIAL_THRESHOLD`, `DEFAULT_LEARNING_RATE`, `DEFAULT_MAX_THRESHOLD`, `DEFAULT_MIN_THRESHOLD`, `MAX_SEARCH_K`, `ZERO`
- mod (21): `calibration`, `clock`, `compat`, `conformal`, `contract`, `dibud`, `drift`, `driver`, `error`, `explain`, `ext`, `fusion`, `isotonic`, `local_types`, `metrics` … (+6)
- pub use: `calibration::{ calibrate_score, AdaptiveConformalCalibrator, Calibrator, Clock, ConformalCalibrator, ConformalError, IsotonicCalibrator, PlattScaler, SystemClock, }`, `clock::{IsotonicPortsClockExt, PortsClockAdapter}`, `conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError}`, `crate::compat::types::signal_contribution_synthesized_from_types`, `crate::compat::types::weights_to_signal_factors_types_compat`, `crate::fusion::SignalKind`, `crate::local_types::FusionStrategy`, `crate::local_types::SignalFailurePolicy`, `crate::local_types::{FusionStrategy, SignalFusionStrategies}`, `dibud::{ fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome, DiBudStep, FusionBudget, }` … (+26)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
