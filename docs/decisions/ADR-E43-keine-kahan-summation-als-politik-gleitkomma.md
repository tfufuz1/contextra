# ADR-E43: Keine Kahan-Summation als Politik. Gleitkomma-Reduktionen fo

* **Status:** Akzeptiert (aus Spezifikation §13 E43)
* **Datum:** 2026-10-15

## Entscheidung
Keine Kahan-Summation als Politik. Gleitkomma-Reduktionen folgen §7.6 Punkt 3.

## Begründung
Kahan ersetzt keine feste Reduktionsordnung. „NaN wird ans Ende sortiert" ist falsch (§7.6 Punkt 5).

## Verworfen
Kahan-Politik und NaN-Aussage (ADVANCED §8.1)
