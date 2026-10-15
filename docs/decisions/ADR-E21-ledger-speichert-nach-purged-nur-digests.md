# ADR-E21: *(überholt durch E65, v1.4)* Ledger speichert nur Keyed-Hash

* **Status:** Akzeptiert (aus Spezifikation §13 E21)
* **Datum:** 2026-10-15

## Entscheidung
*(überholt durch E65, v1.4)* Ledger speichert nur Keyed-Hashes im IST-Format; keine Kompaktierung, kein Klartext-Intent (§5.9).

## Begründung
Das Ledger darf nach Abschluss keine Klartextschlüssel gelöschter Daten tragen.

## Verworfen
Dauerhafter Klartext-Scope
