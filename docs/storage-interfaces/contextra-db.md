# `contextra-db` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Produktdatenbank- und Query-Fassade.

## Schnittstelle und Semantik

Nutzt Engine/RecordStore, nicht LSM. Query bindet `view_durable`, Basis+Delta/Fallback und zweimal Visibility-Filter.

## Abnahme

IndexLag/Erased/BelowFloor bleiben unterscheidbar.

