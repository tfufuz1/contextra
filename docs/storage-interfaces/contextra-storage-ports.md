# `contextra-storage-ports` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Besitzt die einzige Persistenz-Portwelt.

## Schnittstelle und Semantik

Erweitert Fassaden-, KeyProvider-, SegmentOwner-, SegmentIo-, Owner-/Feed- und Fehlerverträge nach Storage-SPEC. `Receipt` enthält immer `InstanceId`; `Shredded` ist Lifecycle, kein Lesefehler.

## Abnahme

Port-Contract- und Referenzvektortests; kein I/O, keine Async- oder Produktpolitik.

