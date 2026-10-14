# ADR-E10: purgestate-repr-u8-enum

* **Status:** Akzeptiert (aus Spezifikation §13 E10)
* **Datum:** 2026-10-14

## Entscheidung
`PurgeState` als persistiertes `#[repr(u8)]`-Enum mit Golden-Test (§5.6).

## Begründung
Zustand muss crash-fest und formatstabil sein.

## Verworfen
Implizite Zustände aus Dateien
