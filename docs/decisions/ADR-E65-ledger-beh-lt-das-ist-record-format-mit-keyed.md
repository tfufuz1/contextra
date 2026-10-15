# ADR-E65: Ledger behält das IST-Record-Format mit Keyed-Hashes und erg

* **Status:** Akzeptiert (aus Spezifikation §13 E65)
* **Datum:** 2026-10-15

## Entscheidung
Ledger behält das IST-Record-Format mit Keyed-Hashes und ergänzt einen Datei-Header `CTRALEDG`. BLAKE3-keyed ist eine dokumentierte Ausnahme zur HMAC-Regel. Kein Klartext-Intent, keine Kompaktierung (§5.9).

## Begründung
Das IST-Ledger speichert bereits nur Keyed-Hashes (`erasure_ledger/entry.rs`). Ein Klartext-Intent gehört nicht ins Ledger.

## Verworfen
Neues Ledger-Format v1 mit Klartext-Intent und Kompaktierung (v1.1 bis v1.3)
