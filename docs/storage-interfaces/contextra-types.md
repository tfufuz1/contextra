# `contextra-types` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Stabile Fach- und Außen-DTOs.

## Schnittstelle und Semantik

Positionen sind `(InstanceId, Stamp)`; keine Key-/Kernformattypen. TxId/Seq werden Legacy, nicht globale Ordnung.

## Abnahme

Serde-/Wire-Kompatibilität ohne Diagnose-Leaks.

