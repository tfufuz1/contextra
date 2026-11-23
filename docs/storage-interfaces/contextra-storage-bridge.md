# `contextra-storage-bridge` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Einzige Adaptergrenze zum Kern.

## Schnittstelle und Semantik

Besitzt permanent `AsyncStorage` mit begrenztem Blocking-Pool/urgent Lane; optionaler `LegacyKvAdapter` ist befristet, Plain/NonPersonal-only und emuliert nie SSI oder Rollback.

## Abnahme

Cancellation+Idem, Backpressure, Lagged und Legacy-Abbau mit G-CUTOVER.

