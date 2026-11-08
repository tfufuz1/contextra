# contextra-adapt

Autarker mathematischer Kontroll- und Adaptionsbaustein (Klasse A) für adaptive Regelung, Bandits, Drift-Erkennung und Feedback-Schleifen.

## Zweck & Baukasten-Profil

`contextra-adapt` regelt Laufzeitparameter (wie Latenzbudgets, Candidate-Pool-Sizing und Routing-Gewichte) deterministisch ohne eigene Hintergrund-Threads oder I/O.
Es erfüllt alle Kriterien der **Klasse A** gemäß Contextra Baukasten-Spezifikation (SPEC v4.0):
- **B1 / B2:** Autarker Standalone-Build ohne interne Abhängigkeiten (`contextra-*`).
- **B3 / B4:** API in Standardtypen (`&[f32]`, `u64`, `Duration`) und eigenem Fehlertyp `AdaptError`.
- **B8 / B11:** Eigenständig dokumentiert mit Unit- und Criterion-Benchmarks, strikter Determinismus (P28).
- **B9 / B10:** Zero-Panic in Produktion (`#![forbid(unsafe_code)]`).

## Schnelleinstieg & Export

Der Baustein kann über das Export-Tool für die Verwendung in externen Projekten extrahiert werden:

```bash
python3 tools/export_crate.py contextra-adapt --out /tmp/contextra-adapt
```

## Minimalbeispiel (Autark)

```rust
use contextra_adapt::pid::PidController;
use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. PID Latency-Controller initialisieren
    let mut pid = PidController::new(100.0, 50, 500, Some(100));
    let dt = Duration::from_millis(50);
    let next_pool = pid.update(dt, 120.0);
    assert!(next_pool >= 50);

    // 2. LinUCB Bandit für Profile-Routing initialisieren
    let dim = 16;
    let mut bandit = BanditProfileState::cold_start(dim, 0.5);
    bandit.implementation = BanditImplementation::ShermanMorrison;

    let x = vec![0.1f32; dim];
    let score = bandit.score(&x, 0.05, false)?;
    println!("Bandit score: {score}");

    bandit.update(&x, 1.0, 0.05, false)?;

    Ok(())
}
```

## Features

- `bandit-routing`: LinUCB Contextual Bandits mit Sherman-Morrison und Sketched Projection.
- `flow-corrected-thompson`: Flow-Corrected Thompson Sampling (FC-TS) mit Drift-Kompensation.
- `rie-greedy-personalization`: Regularization-Induced Exploration Greedy Mechanism.
- `types-compat`: Transitionale Anbindung an `contextra-types` (Deprecation in Welle 6).
- `ports-compat`: Transitionale Anbindung an `contextra-ports` (Deprecation in Welle 6).

## Nicht-Ziele

- Keine eigene Hintergrund-Thread-Verwaltung oder asynchrone Laufzeitbindung.
- Keine Speicherung oder Datenbank-Persistenz (rein in-memory / parameterbasiert).
- Keine direkte Ausführung von Vektorsuchen oder Modellen.
