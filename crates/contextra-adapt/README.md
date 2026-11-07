# contextra-adapt

Autarker Ring-0-Baustein für Regelungstechnik, Online-Lernen und Drift-Erkennung in eingebetteten Systemen und RAG-Speicherkernen.

## Rolle & Architektur
`contextra-adapt` regelt Laufzeitparameter (wie Latenzbudgets, Candidate-Pool-Sizing und Routing-Gewichte) deterministisch ohne eigene Hintergrund-Threads oder I/O.

- **Ring:** Ring 0 (Regelung & Adaption)
- **Normativität:** Rand `[R]`
- **Sicherheit & Unsafe:** `#![forbid(unsafe_code)]` strikt durchgesetzt.
- **Zero-Panic-Regel:** Kein `unwrap()`, `expect()` oder `panic!` in Produktivpfaden.
- **Fehlerbehandlung:** Alle Schnittstellen liefern typisierte Ergebnisse (`Result<T, BanditError>`, `Result<T, FcTsError>`).
- **Determinismus (P28):** Reiner Seeded-Zufall via `ports::Rng`, Zeitmessung über `ports::Clock` bzw. übergebene `Duration`.

## Baukasten-Module
- **`control`** (`pid`, `pid_latency_controller`, `homeostat`, `lyapunov`):
  Anti-Windup PID-Regler für SLA-Einhaltung, Homeostase und Lyapunov-Stabilitätsanalyse.
- **`bandit_algorithms`** (`bandit`, `flow_thompson`, `rie_greedy`):
  LinUCB (Sherman-Morrison Matrix-Update & Sketched Projection), Flow-Corrected Thompson Sampling, RIE-Greedy.
- **`policy`** (`offpolicy`, `shadow_mode`):
  Inverse Propensity Scoring (IPS) für unverzerrte Offline-Evaluation und Schatten-Ausführung.
- **`drift_detection`** (`drift`):
  O(1) robuste Mittelwertschätzung via Catoni-M-Estimator und Ensemble-Drift-Wächter.
- **`decay`** (`decay_controller`):
  Adaptives Abklingen von Relevanz-Scores über Zeit und Zugriffsmuster (ohne Datenlöschung).

## Abhängigkeiten
- **Interne Abhängigkeiten:** `contextra-types`, `contextra-ports`.
- **Externe Abhängigkeiten:** `blake3`, `serde`, `serde_json`, `thiserror`, `tracing`.

## Features
- `default`: `["bandit-routing", "flow-corrected-thompson"]`
- `bandit-routing`: Aktiviert LinUCB-Bandit-Routinen.
- `flow-corrected-thompson`: Aktiviert Flow-Corrected Thompson Sampling (`flow_thompson`).
- `sketched-bandit`: Aktiviert dimensionsreduzierte Sketched-Projection-Bandits.
- `rie-greedy-personalization`: Aktiviert RIE-Greedy-Profiloptimierung.

## Minimalbeispiel (Autark)
```rust
use std::time::Duration;
use contextra_adapt::pid::PidController;
use contextra_adapt::bandit::{BanditProfileState, BanditImplementation};

// 1. PID-Regler mit Anti-Windup
let mut pid = PidController::new(25.0, 50, 500, Some(100));
let new_pool = pid.update(Duration::from_millis(100), 40.0);
assert!(new_pool <= 100);

// 2. LinUCB-Bandit mit Sherman-Morrison-Update
let mut bandit = BanditProfileState::cold_start(4, 0.2);
bandit.implementation = BanditImplementation::ShermanMorrison;
let x = vec![0.5, 0.1, 0.8, 0.2];
let score = bandit.score(&x, 0.05, false).expect("valid score");
bandit.update(&x, 1.0, 0.05, false).expect("valid update");
```

Ein vollständiges, ausführbares Beispiel befindet sich unter:
```bash
cargo run -p contextra-adapt --example adapt_standalone
```
