# ADR-E09: commit-punkt-durables-ledger-append

* **Status:** Akzeptiert (aus Spezifikation §13 E09)
* **Datum:** 2026-10-14

## Entscheidung
Commit-Punkt = durables Ledger-Append; danach Roll-forward; `Indeterminate`-Semantik; Cancellation-sicher (§5.3).

## Begründung
Ohne Vertrag widerspricht Recovery dem gemeldeten `Err` (F-02).

## Verworfen
Roll-back nach Commit
