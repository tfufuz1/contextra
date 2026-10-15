# ADR-E67: Zustandsnummern 0 bis 8 nach FE-10 (`#[repr(u8)]`, Golden-Te

* **Status:** Akzeptiert (aus Spezifikation §13 E67)
* **Datum:** 2026-10-15

## Entscheidung
Zustandsnummern 0 bis 8 nach FE-10 (`#[repr(u8)]`, Golden-Test). `Scanning` entfällt.

## Begründung
Die Nummern in v1.1 widersprachen FE-10.

## Verworfen
Werte 1 bis 9 aus v1.1
