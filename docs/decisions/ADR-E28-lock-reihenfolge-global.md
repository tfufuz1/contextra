# ADR-E28: lock-reihenfolge-global

* **Status:** Akzeptiert (aus Spezifikation §13 E28)
* **Datum:** 2026-10-14

## Entscheidung
Lock-Reihenfolge global (§7.1); sync-Locks nie über `.await`, async-Locks erlaubt, wenn gelistet.

## Begründung
Frühere Formulierung war falsch.

## Verworfen
„Keine Locks über `.await`"
