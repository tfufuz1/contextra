# AGENTS.md — contextra-adapt
> Ring 0 · experimental · Quelle: capabilities.toml · Spec: K.1, §A2, §A4, §0, §3, §4, §17, §21

1. Zweck
`contextra-adapt` stellt adaptive Regelungs- und Steuerungsalgorithmen bereit (LinUCB Contextual Bandit, Lyapunov Drift Watcher, PID Latency Controller, Off-Policy Inverse Propensity Scoring).
Er besitzt die Reife `experimental`, ist intern im Workspace integriert, ist strikt synchron (P26), frei von `tokio` und erzwingt `#![forbid(unsafe_code)]`.

2. Modul-Karte
| Datei / Verzeichnis | Responsibility |
| :--- | :--- |
| `src/lib.rs` | Modul-Deklarationen und Re-Exporte der adaptiven Controller. |
| `src/bandit.rs` | LinUCB Contextual Bandit für adaptives Multi-Kandidaten-Routing. |
| `src/decay_controller.rs` | Zerfalls-Regler für historische Feedback-Gewichte. |
| `src/drift.rs` | `DriftDetector`, `CatoniDriftDetector`, `EnsembleDriftWatcher`, `DriftPolicyBridge`. |
| `src/flow_thompson.rs` | Flow-corrected Thompson Sampling Bandit (`feature = "flow-corrected-thompson"`). |
| `src/homeostat.rs` | Homeostatischer Systemregler für Stabilität. |
| `src/lyapunov.rs` | Lyapunov-Exponenten-Regler (`LyapunovDriftWatcher`) zur proaktiven Drift-Vermeidung. |
| `src/off_policy.rs` / `src/offpolicy.rs` | Inverse Propensity Scoring (IPS) zur kontrafaktischen Evaluation. |
| `src/pid.rs` | `AntiWindupController` Trait und PID-Kernregler. |
| `src/pid_latency_controller.rs` | PID-Latenzregler für Dynamische Poolgrößensteuerung. |
| `src/rie_greedy.rs` | RIE Greedy Personalization Regler (`feature = "rie-greedy-personalization"`). |
| `src/shadow_mode.rs` | Shadow-Mode Evaluator für risikofreie Regler-Tests. |

3. Invarianten
- **INV-PID-ANTIWINDUP-1:** `PidLatencyController` erzwingt Integrator-Clamping auf `[-MAX_INTEGRAL, MAX_INTEGRAL]` mit `MAX_INTEGRAL = 10.0` (`pid_latency_controller.rs`) und setzt die Integrator-Akkumulation bei Sättigung vollständig aus (`update_with_anti_windup`).
- **INV-RING0-SYNC-PURITY:** Kein `tokio` oder Async-Runtime-Import in `contextra-adapt`.

4. Verboten / Anti-Patterns
- **VERBOTEN:** Async-Blockaden oder Netzzugriffe in Regler-Hot-Loops.
- **VERBOTEN:** Unsafe Rust (`#![forbid(unsafe_code)]`).
- **VERBOTEN:** Ungeklammerte Integrator-Summierung in PID-Schleifen.

5. Nebenläufigkeit, Async- und Lock-Regeln
- Strikt synchroner Code (P26).
- Regler-Zustände nutzen interne Locks (`parking_lot`) oder immutables State-Passing.

6. Verifikation
- `cargo test -p contextra-adapt`

7. Bekannte Lücken / SOLL
- Feature-Gating: `bandit-routing`, `flow-corrected-thompson`, `sketched-bandit`, `rie-greedy-personalization` sind steuerbar über Cargo-Features.
