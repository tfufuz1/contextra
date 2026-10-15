# ADR-E56: Einzelprozess-Sperre über `flock` auf `<store>/LOCK`; Fehler

* **Status:** Akzeptiert (aus Spezifikation §13 E56)
* **Datum:** 2026-10-15

## Entscheidung
Einzelprozess-Sperre über `flock` auf `<store>/LOCK`; Fehler `StoreLocked` (§5.17, G-24).

## Begründung
Zwei Prozesse auf einem Store zerstören WAL und Manifest. Ohne Sperre gilt das Kernversprechen nicht.

## Verworfen
Nur Dokumentation; read-only-Fallback
