# AGENTS.md — `contextra-router`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-router` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** –
- **src:** 30 Dateien, 6111 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-adapt`, `contextra-ports`, `contextra-types`, `contextra-wire`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 8):** `arc-swap`, `parking_lot`, `serde`, `serde_json`, `shlex`, `thiserror`, `tokio`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra-agent`, `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (8):** `bandit-routing`, `cloud-egress-guard`, `default`, `egress-sherman-morrison`, `exp-fc-ts`, `exp-routing-strategy`, `flow-corrected-thompson`, `subprocess-dispatch`

### Öffentliche API (grob, aus src/)
- struct (22): `ArmRegistry`, `CollectionAdapter`, `ConfidenceMetrics`, `Contextra`, `ContextraConfig`, `DecisionId`, `DecisionIdGenerator`, `MockDoc`, `NoCalibrationSource`, `ProfileCalibrationState`, `ProfileCalibrator`, `RouterEngine`, `RouterEngineBuilder`, `RouterState`, `RoutingDecision` … (+7)
- enum (6): `ArmRegistryError`, `FcTsDispatchError`, `QuantizationLevel`, `RoutingOutcome`, `RoutingStrategy`, `Transport`
- trait (1): `CalibrationSource`
- fn (73): `add_profile`, `advance`, `allocate_tx`, `arm_for`, `average_confidence`, `bandit_decision_propensity`, `bandit_rng`, `build`, `calibration_file`, `calibration_source`, `calibration_stats`, `calibration_store_path`, `calibration_vfs`, `check_and_invalidate_fingerprint`, `clock` … (+58)
- type (2): `ConformalCalibrator`, `DefaultRouterEngine`
- mod (13): `arm_registry`, `builder`, `dispatch`, `fc_ts_dispatch`, `outcome`, `ports_local`, `profile`, `router`, `routing_strategy`, `seam`, `serde_helpers`, `sorted_u64_set`, `transport`
- pub use: `arm_registry::{ArmRegistry, ArmRegistryError}`, `builder::RouterEngineBuilder`, `contextra_ports::{ CommunityResolver, ContextChunk, ContextPreparer, ContextWindow, DriftStatusProvider, HybridSearchProvider, TokenBudget, }`, `dispatch::dispatch_to_slm`, `fc_ts_dispatch::{deterministic_fc_ts_rng, select_profile_fc_ts, FcTsDispatchError}`, `outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome}`, `profile::{ConformalCalibrator, ProfileCalibrator, SlmProfile}`, `router::{DefaultRouterEngine, RouterEngine, RoutingDecision}`, `routing_strategy::RoutingStrategy`, `seam::{CalibrationSource, NoCalibrationSource, StdFsCalibrationSource, VfsCalibrationSource}` … (+1)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
