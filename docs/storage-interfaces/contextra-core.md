# `contextra-core` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Dünne Reexport-Schicht.

## Schnittstelle und Semantik

Reexportiert nur stabile Records-/Facade-/Domain-DTOs, keine LSM- oder MVCC-Interna.

## Abnahme

Layer-Test ohne Legacy-Snapshot/Store-Reexport.

