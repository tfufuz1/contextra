# ADR-E49: ESSN ist Kandidat hinter Benchmark-Gate. Default bleibt SSI.

* **Status:** Akzeptiert (aus Spezifikation §13 E49)
* **Datum:** 2026-10-15

## Entscheidung
ESSN ist Kandidat hinter Benchmark-Gate. Default bleibt SSI. Latch-freie Varianten sind verboten.

## Begründung
Beleg fehlt, Commit-Overhead auf 4 Kernen ungemessen, P28.

## Verworfen
ESSN als Default (Recherche Feld F), latch-freie SSN (ARCHITECTURE_PLAN_v1 §19.2 verbietet sie selbst)
