# `contextra-graph` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Abgeleiteter Graph-/CSR-/Community-Index.

## Schnittstelle und Semantik

Kanten/Hyperkanten/Communities tragen Quellencluster/Generation; geteilte Strukturen sind Rebuildable, nicht clusterlokal behauptet.

## Abnahme

Gate vor Expansion, Rebuild und Owner-Attestation.

