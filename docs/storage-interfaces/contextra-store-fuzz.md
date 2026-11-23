# `contextra-store-fuzz` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md).

## Vertrag

Der Fuzz-Workspace testet nur den eingefrorenen Legacy-Store und den
Forward-Export. Neue Fuzz-Ziele dürfen keine neue LSM-Funktion legitimieren.

## Abnahme

Nach G-CUTOVER bleiben ausschließlich reproduzierbare Import-/Korruptions-
Regressionen archiviert.
