# AGENTS.md — contextra-mcp
> Ring 4 · stable · Quelle: capabilities.toml · Spec: III.23 / K.17 / K.34 / L.7

## 1. Zweck
Model Context Protocol Server für Contextra Kern-Werkzeuge.
Exponiert Kern-Datenbankfunktionen über Stdio JSON-RPC 2.0 für KI-Assistenten (Claude, Cursor, Jules).
Erzwingt strikte Sandbox-Richtlinien und Quarantäne durch den PromptInjectionGuard.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | `McpServer` Initialisierung und öffentliche Re-Exports |
| `server.rs`, `server_dispatch.rs`, `server_tools.rs` | JSON-RPC 2.0 Handler, Dispatcher und Tool-Registrierung (`TOOL_REGISTRY`) |
| `sandbox.rs` | `SandboxPolicy`, Timeouts und RAM-verschlüsselte `VolatileToolResult`-Verwaltung |
| `prompt_injection/` | `PromptInjectionGuard`, `SecurityAuditLogger`, Richtlinien und Quarantäne |
| `io.rs`, `protocol.rs` | Stdio-Framing, JSON-RPC Nachrichtenprotokoll |
| `config.rs`, `validation.rs`, `tools_crud.rs` | Server-Konfiguration, Parameter-Schema-Validierung und CRUD-Tool-Logik |
| `bin/contextra-mcp-server.rs` | Executable Entrypoint für Stdio-MCP-Server |

## 3. Invarianten

- **INV-MCP-STDIO-ONLY**: Kommuniziert ausschließlich über Stdio JSON-RPC 2.0; HTTP-Server sind aus Sicherheitsgründen verboten (ADR-010).
  *Prüfung*: `cargo test -p contextra-mcp --lib`
- **INV-MCP-CLASSIFY-1**: Jedes Werkzeug in `TOOL_REGISTRY` ist kategorisiert und validiert (`check_mcp_tool_classification`).
  *Prüfung*: `cargo test -p contextra-mcp --lib`
- **INV-MCP-SANDBOX-DEFAULT**: Schreibzugriffe (`allow_db_writes`) sind standardmäßig verboten.
  *Prüfung*: `cargo test -p contextra-mcp --lib`
- **INV-MCP-VOLATILE-CONTAINMENT**: Tool-Ausgaben über `MAX_VOLATILE_OUTPUT_BYTES` (16 MB) werden RAM-verschlüsselt in `VolatileToolResult` gehalten.
  *Prüfung*: `cargo test -p contextra-mcp --lib`

## 4. Verboten / Anti-Patterns

- **Keine HTTP-Endpunkte**: Der Einbau von HTTP-Servern (`axum`, `hyper`) ist im MCP-Server ein Security-Blocker.
- **Keine unbehandelte Executortimeouts**: Tool-Ausführungen müssen zwingend über `sandbox.execute_with_timeout(...)` geleitet werden.
- **Kein Bypass des InjectionGuards**: Ungefilterter Import von Prompt-Inhalten ohne `PromptInjectionGuard` ist untersagt.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Multi-Threaded Tokio-Runtime steuert Stdio-I/O und Hintergrundaufgaben.
- `McpSandbox` schützt `VolatileToolResult` über ein einzelnes, ungeschachteltes `parking_lot::Mutex`.
- Geschachtelte Sperren über Crate-Grenzen hinweg sind strikt verboten.

## 6. Verifikation

```bash
cargo xtask crate-context contextra-mcp
cargo xtask doctrine-scan --crate contextra-mcp
cargo xtask check-unsafe-islands
cargo test -p contextra-mcp --locked
```

## 7. Bekannte Lücken / SOLL

- **Behebung B4 (DeletionProofV4 Durchreichung)**: `contextra_delete_with_proof` gibt echten `DeletionProofV4` an den Aufrufer zurück statt `proof: null`.
- **Behebung B9 (Dual-Path Eliminierung)**: Alle CRUD- und Suchoperationen delegieren strikt an `ContextraDb`; keine Re-Implementierung von Collection-Logik im MCP-Server.
