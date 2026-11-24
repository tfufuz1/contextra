# Baseline Suite Protokoll: `contextra-engine` (ENG-B-0)

- **Datum:** 2026-11-11
- **Toolchain:** `rustc 1.89.0`, `cargo 1.89.0`
- **Ziel:** Feststellung des Teststatus der 5 dem Auftrag ENG-B gehörenden Bestandstests vor Beginn der Arbeiten.

## Befunde vor Behebung
- `tests/decay_and_expiry.rs` (sowie `src/background_workers/tests.rs`) schlug beim Kompilieren mit `error[E0063]: missing field base_half_life_ms in initializer of DecayControllerConfig` fehl.
- Ursache: In einem früheren Auftrag (ENG-1) wurde `base_half_life_ms: Option<u64>` direkt in das struct `DecayControllerConfig` eingefügt, was die Quellkompatibilität für alle Aufrufe brach, die `DecayControllerConfig` mit den 3 Basis-Feldern initialisieren. In `contextra-adapt` existiert dieses Feld nicht.
- Behebung: `base_half_life_ms` wurde aus `DecayControllerConfig` entfernt und stattdessen sauber an `AdaptiveDecayController` über `with_base_half_life_ms` angebunden. Damit ist `DecayControllerConfig` wieder 100% quellkompatibel und feldidentisch zu `contextra-adapt`.

## Bestandstests (Default-Features)
Befehl: `cargo test -p contextra-engine --locked --test chunker_golden_tests --test chunker_hardening_tests --test chunker_parity_ingest --test chunker_robustness --test decay_and_expiry`
Ergebnis:
- `chunker_golden_tests`: 1 passed, 0 failed (0.00s)
- `chunker_hardening_tests`: 6 passed, 0 failed (0.02s)
- `chunker_parity_ingest`: 1 passed, 0 failed (0.00s)
- `chunker_robustness`: 1 passed, 0 failed (0.00s)
- `decay_and_expiry`: 3 passed, 0 failed (0.16s)
- **Gesamtergebnis: 12 passed, 0 failed**

## Bestandstests mit Feature `adaptive-candidate-pool-sizing`
Befehl: `cargo test -p contextra-engine --locked --features adaptive-candidate-pool-sizing --test chunker_golden_tests --test chunker_hardening_tests --test chunker_parity_ingest --test chunker_robustness --test decay_and_expiry`
Ergebnis:
- `chunker_golden_tests`: 1 passed, 0 failed (0.00s)
- `chunker_hardening_tests`: 6 passed, 0 failed (0.02s)
- `chunker_parity_ingest`: 1 passed, 0 failed (0.00s)
- `chunker_robustness`: 1 passed, 0 failed (0.00s)
- `decay_and_expiry`: 3 passed, 0 failed (0.15s)
- **Gesamtergebnis: 12 passed, 0 failed**

## Zero-Panic und Determinismus Guard Baseline
- In den Modulen `src/chunker/**`, `src/temporal_filter.rs` und `src/decay_controller.rs`:
  - `zero_panic_analyzer`: 0 Produktions-Panics, 0 Unwraps
  - `determinism_guard`: 0 Verstöße
