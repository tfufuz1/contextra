# contextra-mcp

Model Context Protocol (MCP) Server über stdio/JSON-RPC 2.0 für Contextra.

Normativitätsklasse: **Ring 4 (Protokoll & IPC)** (`stable`; SPEC §4, `capabilities.toml`).

## Rolle & Zweck (Baukasten-Prinzip)
`contextra-mcp` stellt einen gehärteten, zustandslosen Model Context Protocol (MCP) Server bereit, der über `stdio` mittels JSON-RPC 2.0 mit KI-Hostumgebungen (Claude Desktop, AI-Agenten, IDE-Erweiterungen) kommuniziert.

Gemäß dem Baukasten-Vertrag (M1–M11) ist `contextra-mcp` modular isoliert:
- **Abhängigkeiten abwärts (M1):** Verwendet ausschließlich wohldefinierte Schnittstellen.
- **Entkopplung der Löschkette (M7):** Keine rechtlichen Löschzusagen oder Deletion-Proof-Interna im Produktionskern; diese sind als `#[deprecated]` markiert und gehören exklusiv zur `contextra-erasure`-Familie.
- **Sicherheits-Invarianten:** `#![forbid(unsafe_code)]` strikt erzwungen; keine unwrap/expect/panic im Produktionspfad; Fail-Closed-Sicherheitsgates für Prompt-Injection und Cloud-Egress.
- **Umwelt-Injektion (M4):** Deterministische Puffergrenzen (`MAX_RPC_BYTES`), injizierbare VFS- und Konfigurationspfade.

## Features
- `candle`: Optionales Inferenz-Backend via `contextra-infer-candle`
- `ollama`: Optionales Loopback-Backend via `contextra-infer-ollama`
- `onnx`: Optionales ONNX-Runtime-Backend via `contextra-infer-onnx`
- `test-utils`: Hilfsmittel für Tests

## Einbettungsbeispiel (M10)
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

## Garantien & Invarianten
- **DbC (Design by Contract):** Strikte Eingangsparameter-Validierung und Bounded-Buffer-Reader (Schutz gegen DoS und übergroße Payloads).
- **Zero Panic:** Alle Fehler werden deterministisch in typisierten `McpError`-Ergebnissen gekapselt.
- **Isolation:** Keine unerlaubten Querverbindungen; autark im Baukastensystem integrierbar.
