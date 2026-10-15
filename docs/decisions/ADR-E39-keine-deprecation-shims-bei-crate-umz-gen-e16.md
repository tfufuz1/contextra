# ADR-E39: Keine Deprecation-Shims bei Crate-Umzügen (E16 bleibt).

* **Status:** Akzeptiert (aus Spezifikation §13 E39)
* **Datum:** 2026-10-15

## Entscheidung
Keine Deprecation-Shims bei Crate-Umzügen (E16 bleibt).

## Begründung
Shims halten alte Pfade in Ring 0 und 1 und erhöhen die Hülle.

## Verworfen
`pub use`-Shims mit `#[deprecated]` (IMPLEMENTIERUNGSSPEZIFIKATION §3.4, ARCHITEKTUR_MASTERPLAN_v2 §10.3)
