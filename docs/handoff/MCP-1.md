# Handoff Report `MCP-1` · Model Context Protocol Server & Egress Gateway

## 1. Übersicht & Zielbild
- **Crate:** `contextra-mcp` (Ring 4, Klasse C - Komposition & Schnittstellen)
- **Auftrag:** `MCP-1` Protokollschicht härten, begrenztes JSON-RPC 2.0, Prompt-Injection-Guard mit fail-closed `confirm_required`-Semantik, Egress-Gateway, Entkopplung der Löschdomäne (`proof_key`), und Analyse der Sandbox-Ausführungslogik.

## 2. Durchgeführte Maßnahmen
1. **Entkopplung der Löschdomäne (`proof_key`, `proof_key_env`):**
   - Quelldateien `proof_key.rs` und `proof_key_env.rs` verbatim nach `docs/handoff/MCP-1/adapter_src/` ausgelagert.
   - Lokaler Port-Trait `ProofKeyProvider` in `crates/contextra-mcp/src/proof_key.rs` eingeführt mit Default-Implementierung `EnvProofKeyProvider`.
   - Tool-Ausführungen in `tools_crud.rs` und `server_tools.rs` konsumieren den Schlüssel ausschließlich über `ProofKeyProvider`.
   - Benannter Konsument für die spätere Übernahme in die Löschdomäne: **`ERASE-ADAPT-1`**.

2. **Protokoll & Validierung:**
   - Rekursionstiefen-Begrenzung (`MAX_JSON_DEPTH = 32`) und Batch-Längen-Begrenzung (`MAX_BATCH_SIZE = 100`) in `protocol.rs` definiert und in `server_dispatch.rs` erzwungen.
   - Message-Framing und Standard-JSON-RPC-Strukturen delegieren an `contextra-wire`.
   - Zero-Panic: Alle Fehlerpfade münden in standardisierte JSON-RPC-Codes (`-32700`, `-32600`, `-32601`, `-32602`, `-32603`).

3. **Prompt-Injection-Guard & Egress:**
   - Fail-closed Verhalten für Prompt-Injection mit `confirm_required`-Semantik.
   - Audit-Logging ohne Leaking von Payloads, Prompts, Schlüsseln oder Token-Inhalten.
   - Tool-Klassifikation (lesend, schreibend, destruktiv); destruktive Tools erfordern explizite Bestätigung (`confirm`).
   - Tenant-Isolation gem. VETO-F10: Mandant wird strikt aus der Session bezogen, niemals aus Tool-Parametern überschrieben.

4. **Sandbox-Audit (`sandbox.rs`, 758 Zeilen):**
   - Analyse der in `sandbox.rs` enthaltenen Ausführungslogik:
     - Enthält `McpSandbox`, `SandboxPolicy`, `ToolCategory`, `TOOL_REGISTRY`, `VolatileStore` sowie Timeout-Execution-Logik.
     - Diese Ausführungs- und Isolationslogik hat beträchtliche Substanz (758 Zeilen, Zero-Panic, Mutex-synchronisiert ohne geschachtelte Locks).
     - **Übergabe-Auftrag `SANDBOX-1`:** Diese originäre Sandbox-Ausführungslogik soll in einem späteren Arbeitspaket nach `contextra-sandbox` migriert werden. Per Substanzerhalt-Prinzip (E-05) wurde in diesem Schritt keine Löschung vorgenommen.

## 3. Schnittstellen & Ports
- `ProofKeyProvider`: Lokaler Trait für die Entkopplung von Beweisschlüsseln.
- Konsument `ERASE-ADAPT-1` übernimmt die konkrete Anbindung an `contextra-erasure`.
- Konsument `SANDBOX-1` übernimmt die Auslagerung von `sandbox.rs` nach `contextra-sandbox`.

## 4. Offene Punkte / Externe Abhängigkeiten
- Externe Crates im Workspace (`contextra-vector`, `contextra-graph`, `contextra-crypto`) wiesen temporäre Kompilierfehler im aktuellen Workspace-Branch auf (z.B. Duplikate in Vector). Diese liegen außerhalb des Scopes von `contextra-mcp` (OWNS).
- `contextra-mcp` selbst ist isoliert und modular aufgebaut.
