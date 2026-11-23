# `contextra-durable-fs` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Plattformmechanik, kein zweiter VFS-Vertrag.

## Schnittstelle und Semantik

Lieferant für Sync-/Poison-/Scrub-Bausteine des Storage-Fs; Vfs/Pending-Delete-Journal laufen nicht parallel zum Catalog.

## Abnahme

Keine Default-Kernabhängigkeit; übernommene Crash-Tests.

