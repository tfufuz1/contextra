# ADR-E17: formatversionierung-upgrade-on-open

* **Status:** Akzeptiert (aus Spezifikation §13 E17)
* **Datum:** 2026-10-14

## Entscheidung
Formatversionierung: Upgrade-on-open, Downgrade immer abgelehnt; Hochwassermarke monoton; `AcknowledgeToken`; Integritätsschlüssel (§5.9).

## Begründung
Vorhandener `ManifestFormatTracker` wird erweitert; ehrliche Schutzgrenze dokumentiert.

## Verworfen
Freier Moduswechsel
