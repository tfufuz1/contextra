# `contextra-db-fuzz` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md).

## Vertrag

Fuzzt DB-/Query-DTOS gegen View-, IndexLag- und Gate-Semantik; persistiert
keine eigene Referenzwahrheit.

## Abnahme

Kein Fuzz-Fund darf `Erased` oder `Lagged` als leeren Erfolg abbilden.
