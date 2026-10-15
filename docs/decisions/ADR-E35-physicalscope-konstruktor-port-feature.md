# ADR-E35: `PhysicalScope`-Konstruktor nur mit Port-Feature `backend-im

* **Status:** Akzeptiert (aus Spezifikation §13 E35)
* **Datum:** 2026-10-15

## Entscheidung
`PhysicalScope`-Konstruktor nur mit Port-Feature `backend-impl`, nur vom Store aktivierbar (G-12).

## Begründung
Echte Opazität über Crate-Grenzen ohne `unsafe`.

## Verworfen
Bloße Doc-Kommentar-Opazität
