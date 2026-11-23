# `contextra-checkpoint` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Checkpoint-/Restore-Orchestrierung.

## Schnittstelle und Semantik

Checkpoint = View-Position mit Lease/Hold; eigene Manifeste sind Owner-registrierte Backups. Kein Hardlink-/Tx-Rollback als Autorität.

## Abnahme

Restore-Reconcile ohne Erase-Wiederkehr; Floor/Lease-Tests.

