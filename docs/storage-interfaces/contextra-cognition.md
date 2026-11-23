# `contextra-cognition` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Konsolidierung und Ableitungen.

## Schnittstelle und Semantik

Schreibt abgeleitete Cluster/Links über Records; arbeitet auf TTL-Branches und merged mit Guards statt Intent/OCC-Keys.

## Abnahme

Quell-Erase kaskadiert, Merge-Konflikte und Retry sicher.

