# `contextra-mvcc` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Übergangscrate für lokale reine Algorithmik.

## Schnittstelle und Semantik

Verliert Autorität über globale Sichtbarkeit und SSI; Writes verwenden Kern-Guards, Views und Branches.

## Abnahme

Produktionsgraph ohne globale TxId-/SSI-Abhängigkeit.

