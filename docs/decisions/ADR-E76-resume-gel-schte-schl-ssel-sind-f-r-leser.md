# ADR-E76: Resume: Gelöschte Schlüssel sind für Leser unsichtbar, bis `

* **Status:** Akzeptiert (aus Spezifikation §13 E76)
* **Datum:** 2026-10-15

## Entscheidung
Resume: Gelöschte Schlüssel sind für Leser unsichtbar, bis `Purged` erreicht ist. Die Produktions-Dispose ist `RequireScrub`. No-op-Hooks sind nur bei `Off` zulässig (G-31).

## Begründung
Ein Hintergrund-Purge hätte sonst Lesezugriff auf noch vorhandene Daten (FE §4.9).

## Verworfen
Blockierendes `open()` bis Ende des Purge
