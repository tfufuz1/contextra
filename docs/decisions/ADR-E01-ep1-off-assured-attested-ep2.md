# ADR-E01: EP1 liefert `Off` und `Assured`; `Attested` ist EP2. Erstes

* **Status:** Akzeptiert (aus Spezifikation §13 E01)
* **Datum:** 2026-10-15

## Entscheidung
EP1 liefert `Off` und `Assured`; `Attested` ist EP2. Erstes Release v0.1.0; 1.0.0 ist ein Gate (§1.3).

## Begründung
`Attested` existiert nur als Enum-Variante, kein Port hat eine Implementierung; spart ≈ 4 Wochen und vermeidet ein Versprechen ohne Deckung.

## Verworfen
„Release 1.0 mit Attested"
