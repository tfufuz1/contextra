# contextra-mcp

Model Context Protocol (MCP) Server über stdio für Contextra (§21.1, REQ-MCP-001).

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; §6.0, `capabilities.toml`).

## Funktionen & Spezifikationsbezug
- **stdio MCP-Server (REQ-MCP-001):** Kommuniziert ausschließlich über stdio mittels JSON-RPC 2.0 (INV-STDOUT-1).
- **Session Tenant Isolation (REQ-TEN-001, INV-TENANT-1):** Binds an immutable `tenant_id` resolved at server startup.
- **MCP Core Tools (REQ-MCP-003):** Stellt die Kern-Schnittstellen `remember`, `recall` und `forget` bereit (§21.1).
- **Löschungs-Aussagen (INV-CLAIMS-1, REQ-DEL-006):** Antworten zu Löschoperationen folgen zwingend der Außenformulierung (§4.3, §20.12).
