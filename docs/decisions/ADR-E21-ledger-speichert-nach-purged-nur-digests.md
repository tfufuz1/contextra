# ADR-E21: ledger-speichert-nach-purged-nur-digests

* **Status:** Akzeptiert (aus Spezifikation §13 E21)
* **Datum:** 2026-10-14

## Entscheidung
Ledger speichert nach `Purged` nur Digests; Kompaktierung mit Scrub (§5.9).

## Begründung
Das Ledger darf nach Abschluss keine Klartextschlüssel gelöschter Daten tragen.

## Verworfen
Dauerhafter Klartext-Scope
