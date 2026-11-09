# contextra-mcp

Model Context Protocol (MCP) Server über stdio/JSON-RPC 2.0 für Contextra.

## Rolle & Zweck
`contextra-mcp` stellt einen gehärteten, zustandslosen Model Context Protocol (MCP) Server bereit, der über `stdio` mittels JSON-RPC 2.0 mit KI-Hostumgebungen (Claude Desktop, AI-Agenten, IDE-Erweiterungen) kommuniziert.

- **Sicherheits-Invarianten:** `#![forbid(unsafe_code)]` strikt erzwungen; keine unwrap/expect/panic im Produktionspfad; Fail-Closed-Sicherheitsgates für Prompt-Injection und Cloud-Egress.
- **Deterministische Puffergrenzen:** Bounded RPC Message Size (`MAX_RPC_BYTES`).

## Features
- `candle`: Optionales Inferenz-Backend via `contextra-infer-candle`
- `ollama`: Optionales Loopback-Backend via `contextra-infer-ollama`
- `onnx`: Optionales ONNX-Runtime-Backend via `contextra-infer-onnx`
- `test-utils`: Hilfsmittel für Tests

## Verfügbare Tools
- `contextra_search`: Hybride semantische Suche (Vektor + Text + Graph)
- `contextra_get`: Dokument anhand der ID abrufen
- `contextra_collections`: Kollektionen auflisten
- `contextra_insert`: Neues Dokument indizieren
- `contextra_upsert`: Dokument idempotent einfügen oder aktualisieren
- `contextra_delete`: Einzelnes Dokument löschen
- `contextra_forget`: Alias für `contextra_delete` (Löschen ohne Nachweisobjekt)
- `contextra_create_collection`: Neue Kollektion anlegen
- `contextra_drop_collection`: Kollektion löschen
- `contextra_explain`: Erklärung und Provenienz für Suchergebnisse ausgeben

## Konfiguration & Umgebungsvariablen
- `CONTEXTRA_MCP_ALLOW_WRITE`: Steuert Schreibberechtigungen (`1`, `true`, `yes`). Standardmäßig `false` (Nur-Lese-Modus).
- `CONTEXTRA_MCP_IDLE_TIMEOUT_SECS`: Inaktivitäts-Timeout in Sekunden für stdio (Standard: 30s).
- `CONTEXTRA_INJECTION_POLICY`: Sicherheitsrichtlinie für Prompt-Injection (`quarantine`, `warn`, `disabled`).

## Bekannte Grenzen
- [V] Ein Nur-Lese-Betrieb ist per vorhandener Konfiguration bereits vollständig vorgesehen und aktiv:
  - Über die Umgebungsvariable `CONTEXTRA_MCP_ALLOW_WRITE` (Standardwert: `false`) wird `allow_db_writes` in der `SandboxPolicy` gesteuert.
  - Ohne explizites Setzen von `CONTEXTRA_MCP_ALLOW_WRITE=true` sind alle mutierenden Schreibwerkzeuge (`contextra_insert`, `contextra_upsert`, `contextra_delete`, `contextra_forget`, `contextra_create_collection`, `contextra_drop_collection`) fail-closed durch die `McpSandbox` gesperrt.
- [H] Weitere feingranulare Filterung (z. B. lesende Beschränkung auf bestimmte Kollektionen) erfordert derzeit anwendungsspezifische Adapter oder Proxy-Schichten.

## Einbettungsbeispiel
Ein eigenständiges, autarkes Beispiel befindet sich unter `examples/mcp_standalone.rs` und kann ausgeführt werden:

```bash
cargo run --example mcp_standalone -p contextra-mcp
```

```rust
use contextra_mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use contextra_mcp::sandbox::{McpSandbox, SandboxPolicy};
use serde_json::json;

// 1. Sandbox konfigurieren
let policy = SandboxPolicy {
    allow_db_reads: true,
    allow_db_writes: false,
    allow_code_execution: false,
    allow_cloud_egress: false,
    max_execution_ms: 1000,
};
let sandbox = McpSandbox::new(policy).expect("sandbox");

// 2. JSON-RPC Request verarbeiten
let ping_req = JsonRpcRequest {
    jsonrpc: "2.0".to_string(),
    id: Some(json!(1)),
    method: "ping".to_string(),
    params: json!({}),
};
let resp = JsonRpcResponse::ok(ping_req.id, json!({}));
```
