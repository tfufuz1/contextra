# `contextra-kvcache` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Nichtkanonischer Prompt-/KV-Cache.

## Schnittstelle und Semantik

Standard volatile; persistiert nur als registrierter Owner mit clusterlokalem Scope, keine globale Prefix-Deduplication.

## Abnahme

Gate invalidiert Cache; Persistenz hat Attest.

