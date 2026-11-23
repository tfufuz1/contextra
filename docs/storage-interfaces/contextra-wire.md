# `contextra-wire` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Versionierte Wire-DTOs.

## Schnittstelle und Semantik

Kodiert Position, Cursor, Ticket/Stage und redigierte Fehler; kein Kern-/File-/Key-Zugriff.

## Abnahme

Reject von unklaren/alten Positionen und Leak-Tests.

