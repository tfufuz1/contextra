# `contextra-testkit` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Referenz- und Konformanztests.

## Schnittstelle und Semantik

Stellt CanonicalStore-, Feed-, Owner-, Key- und FaultFs-Fakes bereit; Legacy-InMemoryStore ist keine Löschreferenz.

## Abnahme

Suites für Idempotenz, Gate, Watermark, Lagged und Attestation.

