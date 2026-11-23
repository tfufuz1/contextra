# `contextra-keys-file` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Einzelhost-Cluster-KeyProvider.

## Schnittstelle und Semantik

`destroy_many` ist endgültig und idempotent; unterscheidet Offline, Absent, Disabled und Lost; unterstützt Tomb-Epochen.

## Abnahme

KeyDestroy-/Crash-/Retry-Konformanz.

