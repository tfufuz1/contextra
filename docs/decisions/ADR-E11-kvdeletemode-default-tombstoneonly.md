# ADR-E11: `KvDeleteMode::default() = TombstoneOnly`; Builder setzt `Cr

* **Status:** Akzeptiert (aus Spezifikation §13 E11)
* **Datum:** 2026-10-15

## Entscheidung
`KvDeleteMode::default() = TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`.

## Begründung
Verborgene Kopplung des Defaults an Verschlüsselung.

## Verworfen
Default `CryptoShred`
