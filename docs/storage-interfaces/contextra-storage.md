# `contextra-storage` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Synchroner, alleiniger kanonischer Speicherkern.

## Schnittstelle und Semantik

Implementiert `Change`, Views, Feed, Branch, Erase, Checkpoint und Owner-Register gemäß R9.1; keine RAG-Algorithmen.

## Abnahme

G-0 bis G-F der Storage-SPEC vor Verbraucher-Cutover.

