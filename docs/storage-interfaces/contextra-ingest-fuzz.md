# `contextra-ingest-fuzz` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md).

## Vertrag

Fuzzt Parser und deterministische Ingest-Pläne; keine direkten Storage-Writes.

## Abnahme

Ungültige Klassifikation, Grenzen und Codec-Eingaben scheitern vor I/O.
