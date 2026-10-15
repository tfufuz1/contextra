# ADR-E69: REG-18: Keine Stubs mit Erfolgsrückgabe in Produktion (G-30)

* **Status:** Akzeptiert (aus Spezifikation §13 E69)
* **Datum:** 2026-10-15

## Entscheidung
REG-18: Keine Stubs mit Erfolgsrückgabe in Produktion (G-30).

## Begründung
DLP-Sanitizer und `merge()` geben heute unveränderte oder leere Erfolge zurück (R7-15, R7-16). Das ist fail-open.

## Verworfen
Stubs bis zur Umsetzung hinnehmen
