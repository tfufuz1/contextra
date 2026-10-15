# ADR-E18: MCP: Mandant beim Prozessstart gebunden, kein `SYSTEM`-Fallb

* **Status:** Akzeptiert (aus Spezifikation §13 E18)
* **Datum:** 2026-10-15

## Entscheidung
MCP: Mandant beim Prozessstart gebunden, kein `SYSTEM`-Fallback, nie aus Tool-Argumenten.

## Begründung
Transport ist stdio; Header/TLS-Token sind dort unmöglich.

## Verworfen
Session-Header, `SYSTEM`-Default
