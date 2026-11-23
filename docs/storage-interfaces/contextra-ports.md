# `contextra-ports` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Besitzt Domain-Ports, nicht Persistenzports.

## Schnittstelle und Semantik

`StorageEngine` und tx-basierte Indexmethoden sind deprecated; neue DerivedIndex-/VisibilityOracle-Ports kennen Feed, Watermark, Gate und View-Position.

## Abnahme

Keine neue Persistenz-Port-Parallelwelt; Adapter-Abbau getestet.

