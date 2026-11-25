# ADR-108: Status und Architekturanalyse des Adaptive Query Routing, Drift & Latency Control Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-adapt` (`pid_latency_controller.rs`, `drift.rs`, `rie_greedy.rs`, `bandit.rs`, `flow_thompson.rs`), `contextra-router` (`router/lifecycle.rs`, `profile.rs`, `arm_registry.rs`, `dispatch.rs`, `fc_ts_dispatch.rs`, `router/outcomes.rs`), `contextra-rank` (`calibration/conformal.rs`, `calibration/isotonic.rs`, `calibration/platt.rs`, `dibud/state.rs`, `dibud/types.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§6.4, §8), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Codebase-Auditing der öffentlichen API-Fläche wurden 31 Funktionen im Umfeld von adaptivem Query-Routing, Drift-Erkennung, Latency-Control und Score-Kalibrierung identifiziert, die aktuell keine direkten Aufrufer im Produktsystem aufweisen.

### Kernbefunde:
1. **Adaptive Controller (`contextra-adapt`)**:
   In `contextra-adapt` existieren Steuerungsprimitive wie `compute_adjustment` (`PidLatencyController`), `observe_and_decide`/`observe_and_react` (`DriftPolicyBridge`), `trace_precision_inv` (`RieGreedyProfile`), `derive_seed` (`LinUCB`) und `update_with_flow` (`FlowCorrectedThompsonBandit`). Sie bilden mathematische Regler- und Bandit-Primitivstrukturen ab (§8).
2. **Router LifeCycle & Profile Management (`contextra-router`)**:
   In `contextra-router` sind Hooks wie `with_initial_decision_id`, `with_routing_strategy`, `try_update_profiles`, `with_resource_cost_estimate`, `empirical_error_rate`, `reset_window`, `average_confidence`, `strategy_for`, `arm_for`, `dispatch_to_slm`, `select_profile_fc_ts`, `deterministic_fc_ts_rng` sowie Outcome-Recording-Funktionen (`bandit_decision_propensity`, `calibration_stats`, `reset_calibration`, `set_lyapunov_baseline`, `pending_decision_count`, `reset_all_calibration`) definiert.
3. **Score Calibration & DiBuD (`contextra-rank`)**:
   In `contextra-rank` finden sich `with_bounds` (`ConformalCalibrator`), `force_rebuild`, `expected_calibration_error` (`IsotonicCalibrator`), `is_fitted` (`PlattScaler`) sowie DiBuD-Funktionen (`provenance_of`, `from_index`).

---

## 2. Architekturanalyse & Kontext

Dieser Scaffolding-Code stellt die Infrastruktur für lernende und adaptive Query-Verteilung (Bandit Routing, Conformal Prediction, PID-basierte Pool-Anpassung) dar. In Standard-Deployments läuft das Routing nach festen Heurismen (`DeploymentTier`). Die Funktionalitäten dienen der Erfüllung der Spezifikationsanforderungen in Teil 8 (Adaptive Steuerung & Routing).

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Anbindung & Vollausbau der adaptiven Routing-Schleife
* **Beschreibung**:
  Verdrahtung des `PidLatencyController` und der Bandit-Outcomes in den Hot-Path der `HybridQueryBuilder`-Ausführung in `contextra-engine`, inkl. automatischer Rückkoppelung von Query-Latenzen an den Lyapunov-Watcher.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. Integration in Engine Hot-Path, Multi-Arm-Bandit State Persistence, Feedback-Loop-Kalibrierung und Latenz-Benchmarks)*.
* **Pro**:
  - Dynamische Latenz- und Kostenoptimierung für heterogene Multi-SLM Deployments.
  - Automatische Drift-Anpassung bei schwankenden Workloads.
* **Contra**:
  - Signifikante Komplexität im Query-Hot-Path.
  - Risiko von Latenz-Instabilitäten bei ungeglätteten PID-Signalen.

### Option B: Rückbau der unintegrierten Regler- & Outcome-Methoden
* **Beschreibung**:
  Entfernen der nicht genutzten Hilfs- und Outcome-Methoden aus `contextra-adapt`, `contextra-router` und `contextra-rank`.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Testbereinigung, Entfernung aus TYPE_REGISTRY.md und API-Anpassung)*.
* **Pro**:
  - Beseitigung von ca. 1.200 LOC Scaffolding.
  - Verringerung der Maintenance-Oberfläche.
* **Contra**:
  - Verlust von vorbereiteter Forschungsinfrastruktur für adaptive Bandit-Algorithmen.

### Option C: Beibehaltung des Ist-Zustands als Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Funktionen in den jeweiligen Crates als isoliertes, 100% unit-getestetes Scaffolding für adaptive Routing- und Kalibrierungsexperimente.
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Null Laufzeitkosten bei Nicht-Nutzung.
  - Keine API-Breakages für künftige Experimente.
* **Contra**:
  - Vorhandensein von ungenutzten öffentlichen Methoden im Crate.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Beibehaltung des Scaffolding-Codes in `contextra-adapt`, `contextra-router` und `contextra-rank`. Die Module erzeugen keine Allokationskosten, solange sie nicht im `HybridQueryBuilder` aktiviert werden.

2. **Langfristig**:
   Anbindung von `PidLatencyController` und Conformal Calibration (Option A) sobald Multi-SLM Cloud-Hybrid-Deployments im Produkt-Fokus stehen.
