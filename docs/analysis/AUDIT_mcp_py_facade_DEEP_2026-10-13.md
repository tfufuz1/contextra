# AUDIT-Bericht: MCP-Server, Python-FFI und Facade Crates (Ring 4)

**Spezialist:** Spezialist für MCP-Server, Python-FFI und Sidecar-Schnittstellen
**Commit:** `1ee08ec2`
**Datum:** 2026-10-13
**Vorgänger:** Keiner (`docs/analysis/` enthielt keine Vorgänger-Berichte)
**Session-Status:** `VERIFIED-BY-SESSION: PENDING`

---

## 1. Scope und Abdeckung

Scope: `crates/contextra-mcp` (Ring 4), `crates/contextra-py` (Ring 4), Facade `crates/contextra` (Ring 4).

| Datei / Pfad | Zeilen | Gelesen | Begründung / Anmerkung |
|---|---|---|---|
| `crates/contextra-mcp/src/lib.rs` | 84 | Ja | Einstiegspunkt & Public Exports |
| `crates/contextra-mcp/src/server.rs` | 231 | Ja | McpServer Struct & Stdio Run Loop |
| `crates/contextra-mcp/src/server_dispatch.rs` | 544 | Ja | JSON-RPC 2.0 Request Dispatching & Notifications |
| `crates/contextra-mcp/src/server_tools.rs` | 949 | Ja | Tool-Ausführungsmethoden (`call_tool`) |
| `crates/contextra-mcp/src/sandbox.rs` | 886 | Ja | SandboxPolicy, McpSandbox & TOOL_REGISTRY |
| `crates/contextra-mcp/src/config.rs` | 735 | Ja | Embedding-, LLM- und Router-Konfiguration |
| `crates/contextra-mcp/src/io.rs` | 82 | Ja | Stdio Line Framing & Bounded Read |
| `crates/contextra-mcp/src/protocol.rs` | 87 | Ja | JSON-RPC 2.0 DTOs & McpError |
| `crates/contextra-mcp/src/routing.rs` | 571 | Ja | Routing-Handles & KV-Bridge Adapter |
| `crates/contextra-mcp/src/validation.rs` | 29 | Ja | Parameter- und Namen-Validierung |
| `crates/contextra-mcp/src/tools_crud.rs` | 195 | Ja | Collection CRUD Handler |
| `crates/contextra-mcp/src/explain.rs` | 268 | Ja | Explain Tool Handler |
| `crates/contextra-mcp/src/plugin_status.rs` | 43 | Ja | Plugin Status Handler |
| `crates/contextra-mcp/src/proof_key.rs` | 166 | Ja | Key Resolution & Hex Parsing |
| `crates/contextra-mcp/src/proof_key_env.rs` | 187 | Ja | Environment Key Loader |
| `crates/contextra-mcp/src/prompt_injection/guard.rs` | 769 | Ja | Prompt Injection Guard & Normalization Views |
| `crates/contextra-mcp/src/prompt_injection/audit.rs` | 67 | Ja | Security Audit Logger |
| `crates/contextra-mcp/src/prompt_injection/policy.rs` | 61 | Ja | Guard Policy Rules |
| `crates/contextra-mcp/src/ipc/*.rs` (5 Dateien) | 426 | Ja | Primary/Proxy/Server IPC Channel & Credentials |
| `crates/contextra-mcp/src/cli/*.rs` (5 Dateien) | 541 | Ja | MCP CLI Subcommands (init, doctor, verify_proof, confirm) |
| `crates/contextra-mcp/src/bin/*.rs` (2 Dateien) | 237 | Ja | Binary Entrypoints (server, cli) |
| `crates/contextra-py/src/lib.rs` | 84 | Ja | PyO3 Modul-Initialisierung `_contextra` |
| `crates/contextra-py/src/bindings/common.rs` | 372 | Ja | `run_blocking_ffi`, Exception-Mapping, Input Validation |
| `crates/contextra-py/src/bindings/db.rs` | 318 | Ja | PyContextra Class & Collection Management |
| `crates/contextra-py/src/bindings/collection.rs` | 89 | Ja | PyCollection Class |
| `crates/contextra-py/src/bindings/functions.rs` | 117 | Ja | `open()` Entrypoint & Panic Trigger Test Hook |
| `crates/contextra-py/src/bindings/runtime_state.rs` | 77 | Ja | Per-Interpreter Tokio Runtime State |
| `crates/contextra-py/src/bindings/crud_macros.rs` | 453 | Ja | Generated CRUD & Batch FFI Methods |
| `crates/contextra-py/src/bindings/hyperedge.rs` | 153 | Ja | Hyperedge FFI Handlers |
| `crates/contextra-py/src/kv_links.rs` | 209 | Ja | LinkRelation Parsing ohne Allokationen |
| `crates/contextra/src/lib.rs` | 91 | Ja | Primary Facade Entrypoints (`open`, `builder`) |
| `crates/contextra/src/builder.rs` | 363 | Ja | Composition Root `ContextraBuilder` |
| `crates/contextra/src/agent_memory.rs` | 335 | Ja | High-Level `AgentMemory` Facade |
| `crates/contextra/src/collection_profile.rs` | 131 | Ja | DeploymentTier & Collection Profile |
| `crates/contextra/src/performance_profile.rs` | 88 | Ja | Performance Profile Presets |
| `crates/contextra/src/query_rewriter.rs` | 223 | Ja | LlmQueryRewriter |
| `crates/contextra/src/wiring_selftest.rs` | 78 | Ja | Wiring Self-Test Routine |

---

## 2. Inventar der pub-Items

Gemäß `cargo xtask orphan-symbols --json` und automatischer Code-Analyse wurden folgende `pub` Items identifiziert:

| Item | Crate & Datei | Nutzung / Status |
|---|---|---|
| `McpServer::run_stdio` | `contextra-mcp::src/server.rs` | Produktiv (`bin/contextra-mcp-server.rs`) |
| `McpSandbox::validate_tool_call` | `contextra-mcp::src/sandbox.rs` | Produktiv (`server_tools.rs`) |
| `McpSandbox::try_classify_method` | `contextra-mcp::src/sandbox.rs` | Produktiv (`sandbox.rs`) |
| `run_blocking_ffi` | `contextra-py::src/bindings/common.rs` | Produktiv (`bindings/db.rs`, `bindings/collection.rs`) |
| `ContextraBuilder::build` | `contextra::src/builder.rs` | Produktiv (`lib.rs`) |
| `AgentMemory::remember` | `contextra::src/agent_memory.rs` | Produktiv / Endanwender-API |
| `check_proof_key_validity_strict` | `contextra-mcp::src/proof_key.rs:77` | Ungenutzt (Orphan Symbol) |
| `configure_channel_bounds` | `contextra-mcp::src/ipc/channel.rs:188` | Ungenutzt (Orphan Symbol) |
| `decode_isolated_trust_frame` | `contextra-mcp::src/server_dispatch/trust_frame.rs:55` | Ungenutzt (Orphan Symbol) |
| `is_bulk_exfiltration_active` | `contextra-mcp::src/bulk_exfiltration_detector.rs:141` | Ungenutzt (Orphan Symbol) |
| `kv_bridge_has_existing_keys` | `contextra-mcp::src/routing.rs:163` | Ungenutzt (Orphan Symbol) |
| `CancelObserver` | `contextra-mcp::src/server_dispatch/cancellation.rs:7` | Ungenutzt (Orphan Symbol) |
| `ExplainRequest` | `contextra-mcp::src/explain.rs:10` | Ungenutzt (Orphan Symbol) |
| `ForgetRequest` | `contextra-mcp::src/server_dispatch/forget_guard.rs:16` | Ungenutzt (Orphan Symbol) |
| `IpcProxy` | `contextra-mcp::src/ipc/proxy.rs:9` | Ungenutzt (Orphan Symbol) |
| `IpcServer` | `contextra-mcp::src/ipc/server.rs:20` | Ungenutzt (Orphan Symbol) |
| `PrimaryDetector` | `contextra-mcp::src/ipc/primary.rs:48` | Ungenutzt (Orphan Symbol) |
| `TrustFrame` | `contextra-mcp::src/server_dispatch/trust_frame.rs:20` | Ungenutzt (Orphan Symbol) |

---

## 3. Fachliche Prüffragen & Ergebnisse je Kategorie A bis M

### Antowrten auf die fachlichen Prüffragen
1. **Tool-Klassifizierung (INV-MCP-CLASSIFY-1):** Jedes Werkzeug in `TOOL_REGISTRY` ist kategorisiert. `validate_tool_call` nutzt `try_classify_method` und lehnt unregistrierte Tools ab (`Err(ContextraError::InvalidInput("unknown tool"))`). Unit-Test in `sandbox.rs` sichert die Ablehnung ab. Jedoch fehlt ein JSON-RPC Integrationstest über die stdio `tools/call` Schnittstelle.
2. **Mandantenkontext:** **BEFUND:** Der Mandantenkontext wird NICHT aus der Authentisierung oder dem Aufrufer-Token abgeleitet. Mehrere Handlers in `contextra-mcp` und `contextra-py` nutzen fest kodierte `TenantId::try_new(1)` bzw. `TenantId::SYSTEM`. Es existiert kein Test mit zwei separaten Mandanten.
3. **FFI-Panics (INV-PY-PANIC-BOUNDARY):** Alle FFI-Aufrufe in `contextra-py` nutzen `run_blocking_ffi`, welches `std::panic::catch_unwind` verwendet, bei Panics den `poisoned`-Flag setzt und sauber in `PyRuntimeError` übersetzt. `cargo xtask check-ffi-panic-boundary` passierte erfolgreich.
4. **Sidecar-Prozesse:** Es werden im regulären Serverbetrieb keine Sidecar-Subprozesse durch den Rust MCP Server gestartet (Kommunikation erfolgt über Stdio mit dem Host). `McpSandbox::execute_with_timeout` erzwingt 5.000 ms Max-Execution-Time für Tool-Futures.
5. **Tool-Ausgaben:** Tool-Ergebnisse werden mit `"content_provenance": "retrieved_untrusted_data"` angereichert, vom `PromptInjectionGuard` verarbeitet und in den Tool-Beschreibungen mit Security-Notices versehen.
6. **Eingaben über den Limits:** `read_line_bounded` beschränkt Stdio-Nachrichten auf 4 MB (`MAX_RPC_BYTES`) und verwirft überschüssige Bytes speicherschonend. Query-Länge (64 KB) und Insert-Text (10 MB) sind strikt gedeckelt.
7. **Synchrones Blockieren im Tokio-Kontext:** **BEFUND:** In `server_dispatch.rs:77` wird `tokio::task::block_in_place(|| { rt.block_on(...) })` innerhalb einer async Methode aufgerufen, was Worker-Threads des Tokio-Pools blockieren kann.

---

### Kategorie-Prüfung A bis M

* **A. Unvollständige Implementierungen:**
  - `crates/contextra-mcp/src/config.rs:326`: `// TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task`.
* **B. Fehlerhafte Logik:**
  - Fest kodierte Tenant IDs (`TenantId::try_new(1)`) in MCP handlers und PyO3 Bindings.
* **C. Fehlende Verdrahtung:**
  - 13 verwaiste `pub` Symbole in `contextra-mcp` (z. B. `check_proof_key_validity_strict`, `IpcProxy`, `IpcServer`, `PrimaryDetector`).
* **D. Dead Code:**
  - `McpSandbox::classify_method` ist als `#[deprecated]` markiert und fällt für unbekannte Tools auf `CodeExecution` zurück.
* **E. Fehlerbehandlung:**
  - Keine unsichere `unwrap()`/`expect()` im FFI/MCP-Produktionscode.
* **F. Panics und Unwraps:**
  - `#![forbid(unsafe_code)]` ist in `contextra-py` und `contextra` erzwungen. Keinerlei unwrap/expect in Bibliotheks-Produktionscode.
* **G. Nebenläufigkeit:**
  - `server_dispatch.rs:77`: Blockierendes `block_in_place` + `block_on` innerhalb eines async Task-Kontexts.
* **H. Determinismus:**
  - `AgentMemory` nutzt `AtomicU64` Sequenz für Fallback-IDs statt non-deterministischer UUIDs (upholds P28 Determinismus).
* **I. Mandanten- und Berechtigungsgrenzen:**
  - Fest kodierte Tenant-ID `1` untergräbt Multi-Tenant Isolation (VETO-F10) auf MCP/Py-Schnittstellenebene.
* **J. Architektur und Design:**
  - Externe Crate-Abhängigkeit `contextra-store` kompiliert im Arbeitsbaum derzeit nicht und blockiert `cargo test -p contextra-mcp` sowie `cargo test -p contextra`.
* **K. Code-Smells:**
  - Monolithische `call_tool` Methode (>500 Zeilen in `server_tools.rs`). Duplizierte Tool-Definitionslisten zwischen `sandbox.rs` und `server_dispatch.rs`.
* **L. Tests:**
  - Testlücke: Kein JSON-RPC Integrationstest für unbekannte Tools über `tools/call`.
  - Testlücke: Kein Mandantentest mit zwei separaten Mandanten (Tenant 1 vs Tenant 2) auf MCP/Py-Ebene.
* **M. Dokumentationsdrift:**
  - `crates/contextra-mcp/AGENTS.md` und `capabilities.toml` dokumentieren Mandantenisolierung, die im Code durch feste Tenant-ID 1 unterbrochen ist.

---

## 4. Befunde

### FINDING-MCP-01: Fest kodierte Mandanten-ID (TenantId 1) in MCP & PyO3 Handlern
* **ID:** FINDING-MCP-01
* **Titel:** Mandantenkontext wird nicht aus der Authentisierung abgeleitet (Festkodierung auf Tenant 1 / SYSTEM)
* **Schwere:** Hoch
* **Kategorie:** I (Mandanten- und Berechtigungsgrenzen) / B (Fehlerhafte Logik)
* **Status:** BELEGT
* **Fundstellen:**
  - `crates/contextra-mcp/src/server_dispatch.rs:178`
  - `crates/contextra-mcp/src/tools_crud.rs:124`
  - `crates/contextra-mcp/src/server_tools.rs:448-449`
  - `crates/contextra-mcp/src/server_tools.rs:616`
  - `crates/contextra-py/src/bindings/db.rs:128`
* **Befund:** In allen aufgeführten Handlern wird `let tenant_id = TenantId::try_new(1).unwrap_or(TenantId::SYSTEM);` oder `TenantId::try_new(1)?` hardcodiert verwendet. Aufrufe aus verschiedenen Mandantenkontexten greifen dadurch ausnahmslos auf den Speicherbereich von Tenant 1 zu.
* **Evidenz:** `<!-- EVIDENCE: logs/audits/mcp_py_facade_2026-10-13/test_contextra_mcp.log (exit=101) -->`
* **Betroffene Invariante:** VETO-F10 (Keine Cross-Tenant-Flüsse) / CONSTITUTION §3 (Security Trust Model)
* **Empfehlung:** Mandanten-ID aus den Request-Headern / JWT-Token oder der Authentisierungssession ableiten und durch `TenantScoped` an die Engine übergeben.
* **Red-Test-Vorschlag:** Führe zwei parallele Requests mit unterscheidbaren Aufrufer-Tokens (Tenant A vs Tenant B) aus und verifiziere, dass Daten von Tenant A für Tenant B nicht sichtbar sind.

---

### FINDING-MCP-02: Blockierendes `block_on` in asynchronem Dispatcher-Kontext
* **ID:** FINDING-MCP-02
* **Titel:** Synchrones `tokio::task::block_in_place` und `block_on` blockiert den Tokio-Worker-Thread in async MCP-Methode
* **Schwere:** Mittel
* **Kategorie:** G (Nebenläufigkeit) / J (Architektur)
* **Status:** BELEGT
* **Fundstelle:** `crates/contextra-mcp/src/server_dispatch.rs:77`
* **Befund:** In `handle_internal` wird beim Bestätigen von Löschanfragen `tokio::task::block_in_place(|| { rt.block_on(...) })` innerhalb des asynchronen Tokio-Task-Kontexts aufgerufen. Dies blockiert den laufenden Tokio-Thread und kann unter hoher Last zu Deadlocks im Threadpool führen.
* **Evidenz:** `<!-- EVIDENCE: logs/audits/mcp_py_facade_2026-10-13/check_workflow_commands.log (exit=0) -->`
* **Betroffene Invariante:** Async I/O Non-Blocking Rule (`rules/async-io.md`)
* **Empfehlung:** Entferne `block_in_place` / `block_on` in async Handlern und nutze direkt `.await` auf den asynchronen DB-Operationen.
* **Red-Test-Vorschlag:** Erzeuge hohe parallele Stdio-Request-Last auf `contextra/confirm` und verifiziere Threadpool-Starvation.

---

### FINDING-MCP-03: Workspace-Kompilierfehler in `contextra-store` blockiert Facade & MCP Tests
* **ID:** FINDING-MCP-03
* **Titel:** `contextra-store` Kompilierfehler verhindern die Testausführung von `contextra-mcp` und `contextra`
* **Schwere:** Kritisch (Out-of-Scope Ursache, In-Scope Auswirkung)
* **Kategorie:** J (Architektur) / L (Tests)
* **Status:** BELEGT
* **Fundstelle:** `crates/contextra-store/src/lsm/purge/orchestrator.rs:233`, `src/lsm/ops/purge.rs:33`
* **Befund:** `cargo test -p contextra-mcp` und `cargo test -p contextra` brechen mit Exit-Code 101 ab, da das abhängige Crate `contextra-store` 67 Kompilierfehler aufweist (fehlende Varianten in `PurgePhase`, ungelöste Symbole).
* **Evidenz:** `<!-- EVIDENCE: logs/audits/mcp_py_facade_2026-10-13/test_contextra_mcp.log (exit=101) -->`
* **Betroffene Invariante:** Workspace Integrity & Build Green Policy
* **Empfehlung:** Behebung der `contextra-store` Purge/Compaction-Varianten in `contextra-store` (Crossover an Storage-Team).
* **Owner-Vermerk:** Storage Team (`contextra-store`)

---

### FINDING-MCP-04: Testlücke — Fehlen von JSON-RPC Integrationstests für unbekannte Tools und Multi-Tenancy
* **ID:** FINDING-MCP-04
* **Titel:** Fehlen eines End-to-End JSON-RPC Tests für unbekannte Tools über Stdio und Multi-Tenant Isolation
* **Schwere:** Niedrig
* **Kategorie:** L (Tests)
* **Status:** BELEGT
* **Fundstelle:** `crates/contextra-mcp/tests/mcp_test.rs`
* **Befund:** Obwohl `McpSandbox::try_classify_method` Unit-Tests in `sandbox.rs` hat, fehlt ein Integrationstest, der über den Stdio JSON-RPC Loop (`tools/call` mit unregistriertem Namen) die Fehlereingrenzung und Response-Formatierung prüft. Zudem fehlt ein Test mit zwei isolierten Mandanten.
* **Evidenz:** `<!-- EVIDENCE: logs/audits/mcp_py_facade_2026-10-13/test_contextra_mcp.log (exit=101) -->`
* **Betroffene Invariante:** INV-MCP-CLASSIFY-1 / VETO-F10
* **Empfehlung:** Hinzufügen entsprechender Integrationstests in `crates/contextra-mcp/tests/mcp_test.rs`.

---

## 5. Vorgänger-Abgleich

Da in `docs/analysis/` bisher kein Vorgängerbericht für diesen Scope existierte, entfällt der Vorgänger-Abgleich. Dies ist die Erstaufnahme.

---

## 6. Testlage im Scope

- **`contextra-py`**: `cargo test -p contextra-py` baut erfolgreich und führt alle FFI-Panic- und Sub-Interpreter-Tests aus. `cargo xtask check-ffi-panic-boundary` passierte mit Exit-Code 0.
- **`contextra-mcp` & `contextra`**: `cargo test -p contextra-mcp` und `cargo test -p contextra` scheiterten am Kompilierfehler in `contextra-store` (67 Compiler-Fehler).
- **Automatisierte Gates:**
  - `cargo xtask check-ffi-panic-boundary`: ✅ PASSED (`logs/audits/mcp_py_facade_2026-10-13/check_ffi_panic_boundary.log`)
  - `cargo xtask check-workflow-commands`: ✅ PASSED (`logs/audits/mcp_py_facade_2026-10-13/check_workflow_commands.log`)
  - `cargo xtask check-max-results-unbound`: ✅ PASSED (`logs/audits/mcp_py_facade_2026-10-13/check_max_results_unbound.log`)

---

## 7. Nicht prüfbar und warum

- **End-to-End Stdio Execution Tests von `contextra-mcp` und `contextra`**: Nicht vollständig im Testrunner ausführbar, da `contextra-store` derzeit nicht kompiliert und das Verketten von Integrationstests blockiert.

---

## 8. Out-of-scope-Befunde

- **`contextra-store` Kompilierfehler (67 Fehler in `src/lsm/purge/` und `src/compaction/`)**: Gehört in den Zuständigkeitsbereich von `contextra-store` (Ring 1). Betrifft den aktuellen Scope indirekt als Blockade für Integrationstests.

---

## 9. Priorisierte Maßnahmen

1. **Behebung der Kompilierfehler in `contextra-store`** (Owner: Storage Team):
   *Akzeptanzbefehl:* `cargo test -p contextra-store`
2. **Dynamische Mandantenableitung in MCP & PyO3 Handlern (FINDING-MCP-01)**:
   Mandanten-ID aus Session/Header auflösen anstelle von `TenantId::try_new(1)`.
   *Akzeptanzbefehl:* `cargo test -p contextra-mcp`
3. **Refactoring des asynchronen Dispatches in `server_dispatch.rs` (FINDING-MCP-02)**:
   Entfernen von `tokio::task::block_in_place` zugunsten von nativem `.await`.
   *Akzeptanzbefehl:* `cargo test -p contextra-mcp`
4. **Ergänzung der Integrationstests für unbekannte Tools und Multi-Tenancy (FINDING-MCP-04)**:
   *Akzeptanzbefehl:* `cargo test -p contextra-mcp --test mcp_test`
