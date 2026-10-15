# ADR-E30: Snapshot-Pin-Policy mit `max_snapshot_age` und `pin_grace` (

* **Status:** Akzeptiert (aus Spezifikation §13 E30)
* **Datum:** 2026-10-15

## Entscheidung
Snapshot-Pin-Policy mit `max_snapshot_age` und `pin_grace` (§5.10).

## Begründung
Ein Langläufer-Reader blockiert sonst die Zeroisierung unbegrenzt.

## Verworfen
Unbegrenztes Warten
