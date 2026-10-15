# ADR-E12: **Fence-Protokoll B als Drain-Fence** mit `inflight`-Zähler

* **Status:** Akzeptiert (aus Spezifikation §13 E12)
* **Datum:** 2026-10-15

## Entscheidung
**Fence-Protokoll B als Drain-Fence** mit `inflight`-Zähler (§5.8), Loom mit 3 Szenarien als Merge-Blocker.

## Begründung
Der Check am Eingang plus „Atomic-Load im Leader" ließe ein TOCTOU-Fenster zwischen WAL-I/O und Publish; der Drain beweist die Lückenfreiheit ohne WAL-Formatänderung.

## Verworfen
Reiner Eingangs-Check; Void-Records im WAL
