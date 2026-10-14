# ADR-E06: v3-beweiserzeugung-entfernen

* **Status:** Akzeptiert (aus Spezifikation §13 E06)
* **Datum:** 2026-10-14

## Entscheidung
v3-Beweiserzeugung wird gelöscht, v2/v3 bleiben verify-only; kein Feature `legacy-proofs`.

## Begründung
Zwei Serialisierungen unter einem Domain-Tag verletzen die Domain-Trennung.

## Verworfen
`deprecated`-Markierung
