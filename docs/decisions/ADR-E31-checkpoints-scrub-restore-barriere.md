# ADR-E31: checkpoints-scrub-restore-barriere

* **Status:** Akzeptiert (aus Spezifikation §13 E31)
* **Datum:** 2026-10-14

## Entscheidung
Checkpoints: bei Collection/Tenant-Löschung scrubben; bei Dokument-Löschung Restore-Barriere und Option `checkpoint_scrub_on_erase`.

## Begründung
Kosten-Nutzen; ehrlicher Claim.

## Verworfen
Immer alle Checkpoints neu schreiben
