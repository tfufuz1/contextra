# Spezifikation: `contextra-mcp`

**Ring:** Ring 4 (Boundary & Composition Roots)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.4
**Klasse:** C (Protokoll-Adapter & Boundary Root)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-mcp` implementiert das Model Context Protocol (MCP) für Contextra als Server und Client:
- **Scope:** Bereitstellung von Contextra-Ressourcen, Prompts und Tools über JSON-RPC 2.0 (Stdio und SSE-Transports).
- **Sicherheitsgrenze:** Integrierte Prompt-Injection-Filterung (`prompt-guard`), PII-Egress-Kontrolle und Sandboxing von Tool-Aufrufen.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-mcp`:*
- **LOC:** 4.844 LOC über 22 Rust-Dateien.
- **Typen-Inventar:** 0 Traits, 27 Structs, 3 Enums, 79 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 11 interne Crates (u. a. `contextra-types`, `contextra-router`, `contextra-db`, `contextra-privacy`, `contextra-sandbox`).
- **Audit-Befund:** 0 Deletion-Leaks. Protokoll-Adapter mit sauberer Kapselung externer Requests.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `server.rs` | MCP-Server-Zustandsmaschine und Request-Router | `McpServer`, `ServerConfig` |
| `protocol.rs` | JSON-RPC 2.0 Nachrichtendefinitionen für MCP | `McpRequest`, `McpResponse`, `McpNotification` |
| `transport/` | Stdio- und HTTP/SSE-Transportlayer | `StdioTransport`, `SseTransport` |
| `prompt_injection/` | Filterung feindlicher Prompts und Escape-Sequenzen | `PromptGuard`, `ThreatLevel` |
| `egress_*.rs` | Mandanten- und Datenschutz-Egress-Prüfungen | `EgressFilter`, `EgressDecision` |
| `sandbox.rs` | Abgesicherte Ausführung von Code- und Shell-Tools | `McpSandbox`, `SandboxResult` |

---

## 4. Invarianten (INV-MCP-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-MCP-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird strikt erzwungen. | `cargo check -p contextra-mcp` |
| `INV-MCP-EGRESS-GUARD` | Keine Daten verlassen den MCP-Server, bevor die konfigurierte `EgressPolicy` die Freigabe erteilt. | `tests/egress_guard_tests.rs` |
| `INV-MCP-RPC-CONFORMANCE` | Protokollnachrichten erfüllen exakt die MCP-JSON-RPC-2.0-Spezifikation. | `tests/protocol_conformance.rs` |
| `INV-MCP-FAIL-CLOSED` | Unbekannte Tools oder ungültige JSON-RPC-Nachrichten resultieren in Fehlercodes, niemals Panics. | `tests/adversarial_json.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.4:
1. `prompt_injection/` konsolidieren mit `prompt-guard`.
2. Umstellung der Importe von `contextra_router::ports_local` auf `contextra_ports`.
