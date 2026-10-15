# ADR-E42: Kein Beweis mit `Uncovered`- oder Teilstatus für einen Pflic

* **Status:** Akzeptiert (aus Spezifikation §13 E42)
* **Datum:** 2026-10-15

## Entscheidung
Kein Beweis mit `Uncovered`- oder Teilstatus für einen Pflicht-Layer. Fehlt ein Layer, entsteht ein Fehler mit Layer-ID.

## Begründung
REG-14, §6.2 (`Pending` verboten), Fail-closed.

## Verworfen
`Uncovered(reason)` im signierten Beweis (Recherche Feld J)
