# `contextra-router` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Routing-/Modellsteuerung.

## Schnittstelle und Semantik

Nutzen Engine-Services; Routingcache ist volatile oder Owner-registriert und darf IndexLag nicht verschleiern.

## Abnahme

Typed Fehler und keine Schattenpersistenz.

