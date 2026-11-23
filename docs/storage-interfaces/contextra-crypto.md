# `contextra-crypto` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Anwendungscrypto und Proof-Bridge.

## Schnittstelle und Semantik

WAL-/SSTable-KeyManager und parallele Persistence werden abgebaut. Proof entsteht aus ErasureReceipt+Owner-Attesten und darf keinen Stage aufwerten.

## Abnahme

Fehlendes Attest bleibt Incomplete/Blocked.

