# `contextra-mcp` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

MCP-/Tool-Protokollgrenze.

## Schnittstelle und Semantik

Ruft öffentliche Engine/Fassade auf und überträgt typed Fehler, Tickets und Stages vollständig.

## Abnahme

RPC-Fälle für Indeterminate, Lagged, Erased, IndexLag, Blocked.

