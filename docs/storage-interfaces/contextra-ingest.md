# `contextra-ingest` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

I/O-freie Extraktion und Planung.

## Schnittstelle und Semantik

Erzeugt Ingest-Pläne mit DataClass, Scope, Lineage, Header und Weights; Engine schreibt.

## Abnahme

Deterministische Pläne und Vor-I/O-Validierung.

