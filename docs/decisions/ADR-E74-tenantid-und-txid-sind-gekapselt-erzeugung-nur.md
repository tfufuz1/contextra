# ADR-E74: `TenantId` und `TxId` sind gekapselt. Erzeugung nur mit Prüf

* **Status:** Akzeptiert (aus Spezifikation §13 E74)
* **Datum:** 2026-10-15

## Entscheidung
`TenantId` und `TxId` sind gekapselt. Erzeugung nur mit Prüfung (§5.11).

## Begründung
Öffentliche Felder erlauben Mandanten- und Transaktionswerte ohne Prüfung (`ids.rs`).

## Verworfen
Öffentliche Felder mit Konvention
