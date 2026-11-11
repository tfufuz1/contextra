# contextra-adapt

Autarker mathematischer Kontroll- und Adaptionsbaustein (Klasse A, Ring 0, Reifegrad R3 / Tier `beta`) für adaptive Regelung, Bandits, Drift-Erkennung und Feedback-Schleifen.

## Zweck & Baukasten-Profil

`contextra-adapt` regelt Laufzeitparameter (wie Latenzbudgets, Candidate-Pool-Sizing und Routing-Gewichte) deterministisch ohne eigene Hintergrund-Threads oder I/O.
Es erfüllt alle Kriterien der **Klasse A** gemäß Contextra Baukasten-Spezifikation (SPEC v4.0):
- **B1 / B2:** Autarker Standalone-Build ohne interne Abhängigkeiten (`contextra-*`) im Kern.
- **B3 / B4:** API in Standardtypen (`&[f32]`, `u64`, `Duration`) und eigenem Fehlertyp `AdaptError`.
- **B8 / B11:** Eigenständig dokumentiert mit Unit- und Criterion-Benchmarks, strikter Determinismus (P28).
- **B9 / B10:** Zero-Panic in Produktion (`#![forbid(unsafe_code)]`).

## Reale interne Abhängigkeiten

Ausgeführt mit `cargo tree -p contextra-adapt -e normal --no-default-features`:
- **Keine (0 interne Abhängigkeiten im Kern).**
- Das Crate ist ein autonomer Blattbaustein (Ebene 0, Tier `beta`).

Optionale Workspace-Brücken (`contextra-types`, `contextra-ports`) sind vollständig hinter dem Feature `workspace` bzw. `types-compat` und `ports-compat` in `compat/` isoliert.

## Abgrenzung (Nicht-Ziele)

- Keine eigene Hintergrund-Thread-Verwaltung oder asynchrone Laufzeitbindung.
- Keine Speicherung oder Datenbank-Persistenz (rein in-memory / parameterbasiert).
- Keine direkte Ausführung von Vektorsuchen oder Modellen.
- Keine Rechts-, Compliance- oder Sicherheitsversprechen im Sinne von INV-7.

## Schnelleinstieg & Export

Der Baustein kann über das Export-Tool für die Verwendung in externen Projekten extrahiert werden:

```bash
python3 tools/export_crate.py contextra-adapt --out /tmp/contextra-adapt
```

## Minimalbeispiel (Autark)

```rust
use contextra_adapt::pid::PidController;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. PID Latency-Controller initialisieren
    let mut pid = PidController::new(100.0, 50, 500, Some(100));
    let dt = Duration::from_millis(50);
    let next_pool = pid.update(dt, 120.0);
    assert!(next_pool >= 50);

    #[cfg(feature = "bandit-routing")]
    {
        use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
        let mut bandit = BanditProfileState::cold_start(4, 0.5);
        bandit.implementation = BanditImplementation::ShermanMorrison;
        let ctx = [0.1f32, 0.2, 0.3, 0.4];
        let score = bandit.score(&ctx, 0.05, false)?;
        bandit.update(&ctx, 1.0, 0.05, false)?;
        assert!(score.is_finite());
    }

    Ok(())
}
```

## Features

| Feature | Standard | Beschreibung |
|---|---|---|
| `std` | Ja | Standardbibliothek-Integration (Standard-Default) |
| `workspace` | Nein | Aktiviert die Kompatibilitätsbrücke (`compat/`) zu `contextra-types` und `contextra-ports` |
| `types-compat` | Nein | Alias für Typ-Konvertierungen nach `contextra_types::ContextraError` |
| `ports-compat` | Nein | Alias für Adapter zu `contextra_ports::Clock` |
| `bandit-routing` | Nein | LinUCB Contextual Bandits mit Sherman-Morrison und Sketched Projection |
| `exp-bandit-routing`| Nein | Experiment-Alias für `bandit-routing` |
| `flow-corrected-thompson` | Nein | Flow-Corrected Thompson Sampling (FC-TS) mit Drift-Kompensation |
| `exp-flow-thompson` | Nein | Experiment-Alias für `flow-corrected-thompson` |
| `rie-greedy-personalization` | Nein | Regularization-Induced Exploration Greedy Mechanism |
| `exp-rie-greedy` | Nein | Experiment-Alias für `rie-greedy-personalization` |
