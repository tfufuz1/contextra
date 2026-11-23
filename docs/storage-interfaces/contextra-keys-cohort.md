# `contextra-keys-cohort` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Skalierender optionaler KeyProvider.

## Schnittstelle und Semantik

Cohort-Rewrap bleibt intern; die sichtbare Semantik ist der clusteradressierte KeyProvider.

## Abnahme

Gleiche Provider-Conformance plus Rotationstest.

