# `contextra-vector` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Abgeleiteter ANN-/Vektorindex.

## Schnittstelle und Semantik

Implementiert DerivedIndex+SegmentOwner. Elemente tragen Quellcluster/Generation; Feed ist idempotent, Gate vor Distanz, View-Filter vor Ausgabe. Eigene mmap-Persistenz wandert hinter SegmentIo oder wird für Personal abgeschaltet.

## Abnahme

Lagged-Rebuild, Gate-Race und attestiertes Rebuild/Rewrite.

