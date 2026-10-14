# ADR-E11: kvdeletemode-default-tombstoneonly

* **Status:** Akzeptiert (aus Spezifikation §13 E11)
* **Datum:** 2026-10-14

## Entscheidung
`KvDeleteMode::default() = TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`.

## Begründung
Verborgene Kopplung des Defaults an Verschlüsselung.

## Verworfen
Default `CryptoShred`
