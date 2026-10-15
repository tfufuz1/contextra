# ADR-E54: Ringverletzungen werden nur als befristete Ausnahmen in `cap

* **Status:** Akzeptiert (aus Spezifikation §13 E54)
* **Datum:** 2026-10-15

## Entscheidung
Ringverletzungen werden nur als befristete Ausnahmen in `capabilities.toml` geduldet (§4.3a, G-21).

## Begründung
Der Crate-Split (§4.8) und der Core-Abbau (§4.9) können nicht in M1 erfolgen. Stille Verletzung würde G-04 und G-07 entwerten.

## Verworfen
Stille Duldung bis M5
