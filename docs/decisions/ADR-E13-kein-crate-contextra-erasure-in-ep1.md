# ADR-E13: Kein Crate `contextra-erasure` in EP1; Ledger, Purge, Resume

* **Status:** Akzeptiert (aus Spezifikation §13 E13)
* **Datum:** 2026-10-15

## Entscheidung
Kein Crate `contextra-erasure` in EP1; Ledger, Purge, Resume, RestoreGuard bleiben im Store. Crate-Schnitt von Crypto in M5, Evidence/Verifier in EP2.

## Begründung
Ring 1 darf nur auf Ring 0 hängen; ein Plugin mit Store-Abhängigkeit müsste Ring 3 sein und brächte in EP1 keinen Nutzen.

## Verworfen
Eigenes Erasure-Crate vor Attested
