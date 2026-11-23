# `contextra` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Öffentliche Rust-Fassade.

## Schnittstelle und Semantik

Konfiguriert AsyncStorage, Records, KeyProvider und Anchor; kein LSM-Tuning oder globales Rollback.

## Abnahme

Open-Fencing/Restore und typed Receipt-/Ticket-API.

