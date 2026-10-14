# ADR-E08: zweistufiges-crypto-erase

* **Status:** Akzeptiert (aus Spezifikation §13 E08)
* **Datum:** 2026-10-14

## Entscheidung
Crypto-Erase zweistufig; 72 h Quarantäne nur für Collection/Tenant, 0 h für Dokumente; **kein DEK-Backup**; Dokument-DEK optional (§5.13).

## Begründung
Ein Backup macht die Löschung umkehrbar; Art.-17-Unverzüglichkeit.

## Verworfen
„Backup plus 72 h"
