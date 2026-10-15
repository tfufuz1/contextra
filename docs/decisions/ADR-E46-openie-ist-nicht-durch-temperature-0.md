# ADR-E46: OpenIE ist nicht durch `temperature=0` deterministisch. Dete

* **Status:** Akzeptiert (aus Spezifikation §13 E46)
* **Datum:** 2026-10-15

## Entscheidung
OpenIE ist nicht durch `temperature=0` deterministisch. Determinismus entsteht nur durch Ereignis-Persistenz (E25).

## Begründung
Modell- und Hardware-Varianz.

## Verworfen
`temperature=0, fixed seed` (ADVANCED §2.2, ARCHITEKTUR_MASTERPLAN_v2)
