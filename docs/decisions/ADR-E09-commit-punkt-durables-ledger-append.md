# ADR-E09: Commit-Punkt = durables Ledger-Append; danach Roll-forward;

* **Status:** Akzeptiert (aus Spezifikation §13 E09)
* **Datum:** 2026-10-15

## Entscheidung
Commit-Punkt = durables Ledger-Append; danach Roll-forward; `Indeterminate`-Semantik; Cancellation-sicher (§5.3).

## Begründung
Ohne Vertrag widerspricht Recovery dem gemeldeten `Err` (F-02).

## Verworfen
Roll-back nach Commit
