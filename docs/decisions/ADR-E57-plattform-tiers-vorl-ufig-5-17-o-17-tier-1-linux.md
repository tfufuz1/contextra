# ADR-E57: Plattform-Tiers, vorläufig (§5.17, O-17): Tier 1 Linux; Tier

* **Status:** Akzeptiert (aus Spezifikation §13 E57)
* **Datum:** 2026-10-15

## Entscheidung
Plattform-Tiers, vorläufig (§5.17, O-17): Tier 1 Linux; Tier 2 macOS mit `F_FULLFSYNC` und Crash-Test; Windows nicht in v0.1.0.

## Begründung
Die Dauerhaftigkeit (§5.3) hängt am `fsync`-Verhalten des Betriebssystems. Ohne Beleg gibt es keinen Claim.

## Verworfen
Implizite Unterstützung aller Plattformen
