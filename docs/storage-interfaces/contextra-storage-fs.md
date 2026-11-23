# `contextra-storage-fs` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Produktions-`Fs`-Adapter.

## Schnittstelle und Semantik

Stellt Range-I/O, Preallocation, Locking und geordnete Datei-/Directory-Syncs bereit; Kernzugriffe laufen ausschließlich über diesen Port.

## Abnahme

FaultFs-/Crash-Matrix und Plattform-Durability-Konformanz.

