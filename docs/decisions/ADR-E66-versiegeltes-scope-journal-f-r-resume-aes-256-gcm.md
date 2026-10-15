# ADR-E66: Versiegeltes Scope-Journal für Resume: AES-256-GCM-SIV, vor

* **Status:** Akzeptiert (aus Spezifikation §13 E66)
* **Datum:** 2026-10-15

## Entscheidung
Versiegeltes Scope-Journal für Resume: AES-256-GCM-SIV, vor `LedgerAppended` fsync-fest, nach `Completed` vernichtet (§5.18).

## Begründung
Resume braucht den Scope. Das Ledger kennt nur Hashes. Ohne Journal ist Resume nicht möglich (FE-18).

## Verworfen
Scope aus dem Ledger; Scope aus Hashes rekonstruieren
