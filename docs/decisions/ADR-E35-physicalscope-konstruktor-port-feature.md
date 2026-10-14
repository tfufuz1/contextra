# ADR-E35: physicalscope-konstruktor-port-feature

* **Status:** Akzeptiert (aus Spezifikation §13 E35)
* **Datum:** 2026-10-14

## Entscheidung
`PhysicalScope`-Konstruktor nur mit Port-Feature `backend-impl`, nur vom Store aktivierbar (G-12).

## Begründung
Echte Opazität über Crate-Grenzen ohne `unsafe`.

## Verworfen
Bloße Doc-Kommentar-Opazität
