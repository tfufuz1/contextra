# ADR-E38: Kein Crate `contextra-erasure`. Coordinator, Purge-Treiber u

* **Status:** Akzeptiert (aus Spezifikation §13 E38)
* **Datum:** 2026-10-15

## Entscheidung
Kein Crate `contextra-erasure`. Coordinator, Purge-Treiber und RestoreGuard bleiben in Store und Engine hinter den Ports (E13 bestätigt).

## Begründung
Ein Plugin-Crate hat in EP1 keinen Nutzen und vergrößert die Hülle (§4.4).

## Verworfen
Plugin-Crate `contextra-erasure` (IMPLEMENTIERUNGSSPEZIFIKATION §3.2)
