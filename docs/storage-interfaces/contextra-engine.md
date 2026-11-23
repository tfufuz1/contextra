# `contextra-engine` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Fachlicher Orchestrator.

## Schnittstelle und Semantik

Verwendet `contextra-records`: IngestUnit baut durable clustergebundene Changes; `TrackedView` erzeugt Guards. Das 4-Index-2PC, Intent-Keys und Kompensationen entfallen.

## Abnahme

CRUD/Guards/Indeterminate sowie Gate×Query und große Dokumente mit Live-Marker.

