# AGENTS.md — `contextra-rank`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-rank` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** 4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection for Contextra Cognitive OS
- **src:** 31 Dateien, 4440 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (2):** `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 5):** `ahash`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (5):** `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-infer-candle`, `contextra-mcp`
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
