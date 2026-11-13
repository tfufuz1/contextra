# AGENTS.md — `contextra-adapt`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-adapt` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Adaptive controllers, bandits, and PID regulators for Contextra
- **src:** 20 Dateien, 3864 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 6):** `ahash`, `blake3`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra-db`, `contextra-router`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (11):** `bandit-routing`, `default`, `exp-bandit-routing`, `exp-flow-thompson`, `exp-rie-greedy`, `flow-corrected-thompson`, `ports-compat`, `rie-greedy-personalization`, `std`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (34): `AdaptiveDecayController`, `AlignedF32Vec`, `BanditProfileState`, `BanditShadowReport`, `CatoniDriftDetector`, `DecayControllerConfig`, `DecaySignalInputs`, `DefaultBanditFlipGate`, `DiagonalApproximationBandit`, `DriftPolicyBridge`, `DriftReason`, `EnsembleDriftWatcher`, `FcTsArmSet`, `FcTsConfig`, `FcTsSamplingDistribution` … (+19)
- enum (9): `AdaptError`, `BanditError`, `BanditImplementation`, `DriftSignal`, `FcTsError`, `LyapunovResult`, `OffPolicyError`, `RetrievalStrategy`, `RieGreedyError`
- trait (12): `AdaptiveController`, `AntiWindupController`, `BanditArmSelector`, `BanditDefaultFlipGate`, `BanditPolicy`, `Clock`, `DriftDetector`, `FcTsRng`, `OffPolicyCompatibility`, `Rng`, `ShadowSink`, `StateSnapshot`
- fn (74): `_assert_contract_conformance`, `analyze`, `apply_drift_if_detected`, `baseline_score`, `budget_ms`, `cold_start`, `compute_adjustment`, `compute_propensities`, `config`, `cumulative_ips`, `current_pool_size`, `deadline_exceeded`, `deadline_exceeded_nanos`, `derive_seed`, `derive_seed_u64` … (+59)
- type (1): `Result`
- const (9): `DEFAULT_TARGET_LATENCY_MS`, `MAX_CUMULATIVE_REGRET_THRESHOLD`, `MAX_INTEGRAL`, `MAX_SCALING_FACTOR`, `MIN_BANDIT_SHADOW_SAMPLES`, `MIN_SCALING_FACTOR`, `PID_MAX_POOL_SIZE_DEFAULT`, `PID_MIN_POOL_SIZE_DEFAULT`, `SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`
- mod (22): `bandit`, `bandit_algorithms`, `clock`, `compat`, `contract`, `control`, `decay`, `decay_controller`, `drift`, `drift_detection`, `error`, `flow_thompson`, `homeostat`, `lyapunov`, `offpolicy` … (+7)
- pub use: `bandit::*`, `clock::PortsClockAdapter`, `clock::{Clock, SystemClock}`, `crate::bandit::*`, `crate::clock::Clock`, `crate::decay_controller::*`, `crate::drift::*`, `crate::flow_thompson::*`, `crate::homeostat::*`, `crate::lyapunov::*` … (+19)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
