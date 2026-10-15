# ADR-E61: clippy-Ausnahmen nur über `#[allow]` mit Kommentar; die Rats

* **Status:** Akzeptiert (aus Spezifikation §13 E61)
* **Datum:** 2026-10-15

## Entscheidung
clippy-Ausnahmen nur über `#[allow]` mit Kommentar; die Ratsche zählt sie (§7.5, G-09).

## Begründung
`clippy.toml` gilt workspaceweit und kann nicht pro Crate ausnehmen.

## Verworfen
Crate-spezifische `clippy.toml` (nicht möglich)
