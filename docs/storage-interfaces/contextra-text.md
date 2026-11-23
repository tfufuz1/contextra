# `contextra-text` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Abgeleiteter Text-/BM25-Index.

## Schnittstelle und Semantik

Implementiert DerivedIndex+SegmentOwner; Postings tragen Cluster/Generation und werden vor Scoring gegated.

## Abnahme

Feed-Replay, Sichtbarkeit und verify_absent über Postings.

