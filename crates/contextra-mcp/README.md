# contextra-mcp

Model Context Protocol (MCP) Server über stdio für Contextra.

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-mcp` stellt einen vollwertigen Model Context Protocol (MCP) Server bereit, der über `stdio` mittels JSON-RPC 2.0 mit KI-Hostumgebungen (wie Claude Desktop, IDE-Plugins) kommuniziert.

## Baukasten-Einordnung & Status
- **Ring:** Ring 4 (Protokoll-Gateway)
- **Status:** 🟡 In Migration (Entflechtung von In-Core-Löschlogik gemäß `PLAN.md` Phase 3 & 4)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Entkopplung
- **Lösch- und Rechtsformulierungen:** Antworten zu Löschoperationen delegieren Bereinigungen an den modularen Löschbaustein [`contextra-erasure`](../contextra-erasure/). Direkte In-Engine-Löschprüfungen gelten als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)).

## Kernfunktionen
- **stdio MCP-Server:** Rein textbasierte stdio-Kommunikation mit JSON-RPC 2.0.
- **Session Tenant Isolation:** Feste Bindung an eine unveränderliche `TenantId` pro Server-Sitzung.
- **Core MCP Tools:** Stellt die Werkzeuge `remember`, `recall` und `forget` bereit.
