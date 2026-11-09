# contextra-testkit

Test Fixtures, Mock Engines, In-Memory Referenz-Implementierungen und Port-Konformitätssuiten für Contextra.

## Zweck
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores, Injektions-Fixtures und wiederverwendbare Vertragstests (Konformitätssuiten) für Port-Traits bereit. Es dient ausschließlich als Testhilfe in `[dev-dependencies]`.

## Reifegrad
- **Status:** Stabil (P1–P6 erfüllt)
- **Rolle:** Test-Hilfsbibliothek (`[dev-dependencies]`)
- **Architektur-Invariante:** Keine produktive Nutzung; enthält deterministische Doubles für Storage, Vector, Text, Graph, VFS (`FaultVfs`), Clock (`ManualClock`) und Metrics.

## Interne Abhängigkeiten
`contextra-testkit` konsumiert ausschließlich Fundament-Typen und Port-Schnittstellen:
- `contextra-types`
- `contextra-ports` (mit Feature `testkit`)

## Features
- `default = ["testkit", "erasure-conformance"]`
- `testkit`: Aktiviert Testkit-Mocks und Konformitätssuiten.
- `erasure-conformance`: Übergangs-Feature für die Bereitstellung synthetischer Witness-Hilfsfunktionen (`mint_witness_for_tests`) für bestehende Testsuiten (wird in Phase 2 durch den Integrator entfernt, siehe `docs/handoff/fragments/P09/PENDING_API_CHANGES.md`).

## Minimalbeispiel
```rust
use contextra_testkit::ManualClock;
use contextra_ports::Clock;

let clock = ManualClock::new(1000);
assert_eq!(clock.now_exact_nanos(), 1000);
clock.advance_nanos(500);
assert_eq!(clock.now_exact_nanos(), 1500);
```

## Entnahme-Rezept
1. `contextra-testkit` wird üblicherweise in der `Cargo.toml` von Test-Targets oder Downstream-Crates eingebunden:
```toml
[dev-dependencies]
contextra-testkit = { path = "../contextra-testkit" }
```
2. Nutzen der generischen Konformitätssuiten in eigenen Integrationstests:
```rust,ignore
use contextra_testkit::conformance::run_storage_engine_suite;
```

## Bekannte Grenzen
- In-Memory Doubles sind nicht für Produktivlasten gedacht und halten alle Daten unkomprimiert im flüchtigen RAM.
- `FaultVfs` simuliert Fehler deterministisch anhand von Zählern und Fehlerplänen für Testzwecke.
