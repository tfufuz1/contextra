# Handoff MCP-3: Entkopplungs- und Baustein-Architektur für contextra-mcp

Commit-Kennung: MC3
Stand: 2026-11-10

## 1. Status der Architektur & Baustein-Entkopplung

| Aspekt | Status | Nachweis | Anmerkung |
|---|---|---|---|
| Normative Rolle | Ring 4 (apps, Level 3) | `docs/SPEC.md` §3.1 | MCP-Server über stdio/JSON-RPC 2.0 |
| Kern-Closure | Konform | `Cargo.toml`, `capabilities.fragment.toml` | Abhängigkeiten beschränkt auf erlaubte interne Crates (`contextra`, `types`, `ports`, `wire`, `rank`, `crypto`, `privacy`, `router`, `prompt-guard`) |
| Prompt-Guard Entkopplung | Abgeschlossen (MCP-2) | `crates/contextra-prompt-guard` | Re-Export aller Typen via `contextra-prompt-guard` als autarker E0-Sicherheitsbaustein |
| Port-Migration | Abgeschlossen (S-62) | `crates/contextra-mcp/src/routing.rs` | Keine internen Abhängigkeiten zu `ports_local`; direkte Anbindung über `contextra-ports` |
| Zero-Panic Doktrin | Konform | `tools/pattern_diagnostic.py` | 0 unwrap/expect/panic in Produktionspfaden |
| Erasure / DSGVO Leaks | Konform | `tools/erasure_leak_prober.py` | 0 Leaks in Produktionsdateien |
| Capabilities-Deklaration | Vollständig | `crates/contextra-mcp/capabilities.fragment.toml` | Formale Baustein-Spezifikation für G-L4 Abnahme hinterlegt |

## 2. Baustein-Verträge & Schnittstellen

- **INV-STDOUT-1:** `stdout` gehört ausschließlich dem MCP-JSON-RPC-Protokoll; Logs und Fehler werden ausschließlich über `stderr` oder strukturierte Tracing-Subscriber ausgegeben.
- **Fail-Closed Sandbox:** Sämtliche Tool-Aufrufe passieren die `McpSandbox`; schreibende Operationen erfordern zwingend `CONTEXTRA_MCP_ALLOW_WRITE=true`.
- **Typen-Kompatibilität:** Sämtliche öffentlichen Typen (`McpServer`, `SandboxPolicy`, `PromptInjectionGuard`, `ExplainRequest`, `ExplainResponse`) bleiben 100 % quellkompatibel.

## 3. Empfehlungen für nachfolgende Entwickler

- Nachfolgende Erweiterungen des MCP-Servers sollten neue Inferenz-Backends ausschließlich hinter optionalen Feature-Gates (analog zu `candle`, `ollama`, `onnx`) kapseln.
- Tool-Registrierungen und Dispatch-Handler verbleiben isoliert in `server_tools.rs` und `server_dispatch.rs`.
