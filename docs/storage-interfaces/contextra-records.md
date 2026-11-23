# `contextra-records` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Neues Ring-1-Crate für Record-/Scope-Verträge.

## Schnittstelle und Semantik

Besitzt versionierten Space-Katalog, binäre Schlüsselcodecs, Header v1, `ScopeResolver`, `ScopeBinding`, `RecordStore` und Guard-Aufzeichnung. Domain-Crates sprechen es statt `Storage` direkt.

## Abnahme

Compile-fail gegen rohe ClusterId; Tenant-Naht, Codec-Properties, DataClass-/Seal-Checks.

