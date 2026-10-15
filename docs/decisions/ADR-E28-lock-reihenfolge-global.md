# ADR-E28: Lock-Reihenfolge global (§7.1); sync-Locks nie über `.await`

* **Status:** Akzeptiert (aus Spezifikation §13 E28)
* **Datum:** 2026-10-15

## Entscheidung
Lock-Reihenfolge global (§7.1); sync-Locks nie über `.await`, async-Locks erlaubt, wenn gelistet.

## Begründung
Frühere Formulierung war falsch.

## Verworfen
„Keine Locks über `.await`"
