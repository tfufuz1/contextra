# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT
## Subsystem-Analyse: MCP (Model Context Protocol, PyO3 Python FFI & Hauptfassade)

**Scope-ID:** `MCP` / `MCP_mcp`
**Datum:** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Contextra Independent Audit Team)

---

## 1. Kurzfassung

Dieses unabhängige adversarial Audit bewertet die äußere Vertrauensgrenze von Contextra im Subsystem `MCP` (`crates/contextra-mcp`, `crates/contextra-py`, `crates/contextra`).
Die schwersten belegten Befunde sind:
1. **MCP-001 (S0 / LOGIK & SICHERHEIT):** Fehlende Mandanten-Validierung in allen MCP CRUD-Tools (`handle_search`, `handle_get`, `handle_insert_upsert`, `handle_delete`, `handle_relate`, `handle_explain` u. a.). `validate_request_tenant` wird ausschließlich bei `handle_forget` aufgerufen. Anfragen mit abweichender `tenant_id` passieren ungeprüft.
2. **MCP-002 (S1 / LOGIK & ZERO-PANIC):** Laufzeit-Panic durch ungeschützte UTF-8-Byte-Slicing in `RecallTruncator::truncate` (`&text[..approx_chars_limit]`). Bei Multibyte-UTF-8-Zeichen (Umlaute, Emojis, Symbole) panickt die Methode mit Index-Out-Of-Bounds auf Char-Boundary.
3. **MCP-003 (S2 / UNVOLLSTÄNDIG & UNWIRED):** `ForgetGuard` und `ConfirmHandler` zur Bestätigung destruktiver Aktionen via Token, Principal-Bindung und LocalIpc-Pfad sind vollständig entkoppelt (`UNWIRED`). Das produktive `McpServer` prüft stattdessen ein triviales `"confirm": true` Argument im JSON-RPC-Payload.
4. **MCP-004 (S2 / VERDRAHTUNG & SMELL):** Inkonsequente Tool-Verdrahtung: `contextra_explain` ist in `TOOL_REGISTRY` registriert und in `server_dispatch.rs` behandelt, fehlt jedoch in `McpServer::call_tool`'s `match name` in `server_tools.rs`.
5. **MCP-005 (S3 / BUILD & GOV):** Doppelter Feature-Schlüssel in `crates/contextra/Cargo.toml` (`erasure = []`), der unbehandelt workspace-weite Cargo-Befehle blockiert.

**Was gehalten hat:**
Die FFI-Panic-Boundary (`run_blocking_ffi`) in `contextra-py` fängt Panics verlässlich ab und setzt die Engine auf `poisoned` (INV-PY-PANIC-BOUNDARY). Sub-Interpreter werden strikt abgelehnt (INV-PY-SUBINTERPRETER-GUARD). Prompt-Injection-Guards (`PromptInjectionGuard`) und Release-Gates gegen Mock-Instanzen (`config.rs`) funktionieren fail-closed.

---

## 2. Metadaten und Methode

- **Zeitstempel:** 2026-10-21T15:20:00Z
- **Commit SHA:** `a96cb9f6851f3255ddd0f99fed72e5516ddc7786`
- **Toolchain:** `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)`
- **Werkzeugstatus:** `cargo` (vorhanden), `rg` (vorhanden), `cargo-mutants` (nicht vorinstalliert, Miri/Fuzz nightly via isolierte worktree-Experimente in `/tmp/audit-mcp`).
- **Geprüfte Scope-Pfade:**
  - `crates/contextra-mcp/src/**`
  - `crates/contextra-py/src/**`
  - `crates/contextra/src/**`
- **Gelesene Quellen & Regelwerke:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-mcp/AGENTS.md`, `crates/contextra-py/AGENTS.md`, `crates/contextra/AGENTS.md`, `docs/TYPE_REGISTRY.md`, ADR-010, ADR-044, ADR-056, ADR-059, ADR-N22, ADR-N25, `AUDIT_INTAKE_PROTOCOL.md`, `COMMON_LLM_ERRORS.md`.
- **ADR-Kette & Governance-Prüfung:**
  - ADR-010 (MCP Stdio Transport): Eingehalten. Keine HTTP/Axum Server im Code.
  - ADR-044 (MCP Write Authorization): Im Code durch `SandboxPolicy.allow_db_writes` und `CONTEXTRA_MCP_ALLOW_WRITE` abgedeckt.
  - ADR-056 / ADR-059 (Python FFI Panic Isolation): Im Code durch `run_blocking_ffi` & `AtomicBool` Poisoning umgesetzt.
  - ADR-N22 (Standard Build Core Crates): Eingehalten.
  - ADR-N25 (Panic Boundaries): Im FFI-Layer vollständig eingehalten.
- **Drift-Befund:** `crates/contextra-mcp/AGENTS.md` Modulkarte führt nur 7 von 22 Quelldateien auf (15 Quelldateien fehlen). `crates/contextra/AGENTS.md` besitzt keine Modulkarte.

---

## 3. Inventarmatrix (Scope: MCP, Py, Facade)

| Element | Zweck (1 Zeile) | produktive Aufrufer | Testabdeckung | zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `McpServer` | Haupt-MCP-Server-Instanz und Event-Loop | `contextra-mcp-server` binary | `tests/mcp_test.rs` | ADR-010 | OK |
| `McpSandbox` | Isolation, Policy-Prüfung & Volatile Outputs | `McpServer` | `sandbox.rs` inline tests | ADR-044 | OK |
| `SandboxPolicy` | Granulare Berechtigungsschalter (Read, Write, Egress) | `McpSandbox` | `sandbox.rs` inline tests | ADR-044 | OK |
| `TOOL_REGISTRY` | Single Source of Truth für Tool-Definitionen | `McpSandbox::try_classify_method` | `sandbox.rs` tests | H1 | OK |
| `PromptInjectionGuard` | Multi-Stage Normalisierung & Quarantäne-Filter | `McpServer`, `egress_gateway` | `prompt_injection/tests.rs` | H6 | OK |
| `SecurityAuditLogger` | Append-only Audit-Log für Injection-Events | `PromptInjectionGuard` | `audit.rs` tests | H6 | OK |
| `VolatileToolResult` | AES-GCM-SIV verschlüsselter RAM-Output mit Zeroize | `McpSandbox` | `sandbox.rs` tests | H5 / Zeroize | OK |
| `PrimaryDetector` | Multi-Prozess Role Detection (Primary / Proxy) | `db_open.rs` | `tests/ipc_primary_and_proxy.rs` | H4 / Single-Node | OK |
| `IpcServer` | Unix Domain Socket / Named Pipe IPC Listener | `db_open.rs` | `tests/ipc_primary_and_proxy.rs` | H4 / INV-IPC-1 | OK |
| `IpcProxy` | Forwarder für Proxy-Modus an Primary-Endpoint | `db_open.rs` | `tests/ipc_primary_and_proxy.rs` | H4 | OK |
| `validate_unix_peer_credentials` | UID Peer Credentials Pruefung auf Unix Sockets | `IpcServer` | `peer_cred.rs` tests | H4 | OK |
| `resolve_deletion_proof_key` | DeletionProof Key Resolver aus Datei / Env | `server.rs`, `db_open.rs` | `proof_key.rs` tests | H8 | OK |
| `ForgetGuard` | Token-basierter Bestätigungsschutz für Löschungen | **KEINE** | `forget_guard.rs` tests | H3 | UNWIRED |
| `ConfirmHandler` | LocalIpc Token-Validierungs-Handler | **KEINE** | `confirm_handler.rs` tests | H3 | UNWIRED |
| `RecallTruncator` | Bounded Output Truncator mit Token-Schätzung | **KEINE** | `recall_truncator.rs` tests | H5 | UNWIRED / DEFECT |
| `TrustFrame` | Vertrauensrahmen für Eingabe-/Ausgabedaten | **KEINE** | `trust_frame.rs` tests | H1 | UNWIRED |
| `ToolHints` | Metadaten-Hinweise für MCP-Tools | **KEINE** | `tool_hints.rs` tests | H1 | UNWIRED |
| `validate_request_tenant` | Pruefung ob Request-Tenant = Session-Tenant | `server_tools.rs` (`handle_forget`) | `validation.rs` tests | H2 / Q4 | DEFECT |
| `_contextra` | PyO3 CPython C-Extension Modul-Einstieg | Python `import contextra` | `tests/test_bindings.py` | INV-PY-PANIC-BOUNDARY | OK |
| `run_blocking_ffi` | FFI Panic Boundary Wrapper mit Engine-Poisoning | `PyContextra`, `PyCollection` | `tests/test_panic_isolation.py` | INV-PY-PANIC-BOUNDARY | OK |
| `check_subinterpreter_guard` | Verhindert Laden in Sub-Interpretern (PEP 684) | `_contextra` init | `tests/test_subinterpreter.py` | INV-PY-SUBINTERPRETER-GUARD | OK |
| `PyContextra` | Python-Binding-Klasse für Contextra DB | Python user API | `tests/test_bindings.py` | INV-PY-GIL-RELEASE | OK |
| `PyCollection` | Python-Binding-Klasse für Collection CRUD | Python user API | `tests/test_bindings.py` | INV-PY-GIL-RELEASE | OK |
| `ContextraBuilder` | Facade Composition Root Builder (Ring 4) | `contextra::builder()`, `open()` | `tests/builder_levers_contract.rs` | INV-WIRING-2 | OK |
| `verify_builder_wiring` | Verdrahtungs-Selbsttest für Required Ports | `ContextraBuilder::build` | `tests/w1_11_verify_builder_wiring.rs` | INV-WIRING-2 | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment / Methode | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Jedes Tool ist explizit klassifiziert; unklassifizierte Tools werden abgewiesen. | Code-Inspektion `TOOL_REGISTRY` & `sandbox.rs` tests | **CONFIRMED** (Erfüllt) | - |
| **H2** | Mandant stammt aus Authentisierungskontext; kein Fremdzugriff möglich. | Code-Inspektion `server_tools.rs` & `validation.rs` | **CONFIRMED** (Defekt) | `MCP-001` |
| **H3** | Destruktive Aktionen nutzen einmalige, gebundene Tokens (`ForgetGuard`). | Call-Graph & Code-Inspektion `forget_guard.rs` | **CONFIRMED** (Unwired) | `MCP-003` |
| **H4** | IPC prüft Peer Credentials (UID) & Socket-Rechte fail-closed. | STATIC Code-Inspektion `ipc/` & `peer_cred.rs` | **CONFIRMED** (Erfüllt) | - |
| **H5** | JSON-RPC Parser ist gehärtet; `recall_truncator` schneidet UTF-8 sauber. | STATIC Code-Inspektion `recall_truncator.rs` | **CONFIRMED** (Defekt) | `MCP-002` |
| **H6** | Prompt-Injection-Guard schützt gegen Evasion (Encodings, Base64, Unicode). | Code-Inspektion `prompt_injection/guard.rs` | **CONFIRMED** (Erfüllt) | - |
| **H7** | Fehlermeldungen enthalten keine internen Pfade oder Schlüssel. | Code-Inspektion `protocol.rs` & `McpError` | **CONFIRMED** (Erfüllt) | - |
| **H8** | Schlüsselmaterial wird aus Dateien/Env sicher geladen & zeroized. | Code-Inspektion `proof_key.rs` & `proof_key_env.rs` | **CONFIRMED** (Erfüllt mit Smell) | `MCP-006` |
| **H9** | Python-FFI verhindert Panics, gibt GIL frei, schützt Subinterpreter. | Code-Inspektion `bindings/common.rs` & `lib.rs` | **CONFIRMED** (Erfüllt) | - |
| **H10** | Mock-/Platzhalter-Typen sind in Release-Builds deaktiviert. | Code-Inspektion `config.rs` (`#[cfg(not(...))]`) | **CONFIRMED** (Erfüllt) | - |
| **H11** | Facade Re-Exports sind bewusst gewählt; interne Typen verborgen. | Code-Inspektion `crates/contextra/src/lib.rs` | **CONFIRMED** (Erfüllt) | - |
| **H12** | Tool-Liste & Verhalten entsprechen Spezifikation & Doku. | Diff `AGENTS.md` vs. Quellcode-Dateibaum | **CONFIRMED** (Drift) | `MCP-004`, `MCP-007` |

---

## 5. Befunde (sortiert nach Severity)

### ID MCP-001 | Fehlen von Tenant-ID Validierungen in MCP CRUD Tools | Severity S0 | Kategorie: SICHERHEIT & LOGIK | Status: STATIC | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/src/server_tools.rs` (Zeilen 40–600), `crates/contextra-mcp/src/tools_crud.rs`
- **Beobachtung:** Die Funktion `validate_request_tenant` aus `validation.rs` wird im gesamten MCP-Server-Code **ausschließlich** in `handle_forget` (Zeile 338 in `server_tools.rs`) aufgerufen. In allen anderen Tool-Handlern (`handle_search`, `handle_get`, `handle_insert_upsert`, `handle_delete`, `handle_relate`, `handle_relate_n_ary`, `handle_cloud_query`, `handle_explain`, `handle_consolidate`, `handle_create_collection`, `handle_drop_collection`) fehlt dieser Check vollständig.
- **Beweis:** `rg "validate_request_tenant" crates/contextra-mcp/src/` liefert genau einen Treffer in `server_tools.rs:338`.
- **Gegenbeweis-Versuch:** Geprüft, ob `self.db.collection(...)` die Validierung übernimmt. `self.db.collection(col_name)` nimmt nur den Collection-Namen als `&str` entgegen, nicht die `TenantId`. Wenn ein Aufrufer in `args` explizit `"tenant_id": 999` oder `"tenant": 999` übergibt, wird dieser Wert im Handler schlicht ignoriert und die Anfrage auf dem Server-Session-Tenant ausgeführt, ohne den Fehler abzuweisen.
- **Auswirkung:** Verletzung von Hypothese H2 und Invariante Q4 (Mandantenisolation). Mandantenbezogene Eingaben in API-Anfragen werden nicht auf Übereinstimmung mit dem aktiven Session-Mandanten geprüft.
- **Mindestweg zur Behebung:** In `McpServer::call_tool` oder zu Beginn jedes Tool-Handlers `validate_request_tenant(session_tenant, Some(args))?` ausführen.

---

### ID MCP-002 | Panic-Gefahr durch ungeprüftes Byte-Slicing in `RecallTruncator::truncate` | Severity S1 | Kategorie: LOGIK & ZERO-PANIC | Status: STATIC | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/src/server_dispatch/recall_truncator.rs` (Zeile 26)
- **Beobachtung:** In `RecallTruncator::truncate` wird der Teilstring mittels `&text[..approx_chars_limit]` extrahiert. `approx_chars_limit` ist ein Integer-Wert (`max_response_tokens * 4`).
- **Beweis:**
  ```rust
  let approx_chars_limit = self.max_response_tokens * 4;
  if text.len() <= approx_chars_limit { ... }
  else {
      let truncated_sub = &text[..approx_chars_limit]; // Byte-Indexing!
  }
  ```
  In Rust bewirkt das Slicing eines `&str` an einer Byte-Position, die kein UTF-8 Char Boundary ist, eine unvermeidliche Panic (`byte index X is not a char boundary`).
- **Gegenbeweis-Versuch:** Geprüft, ob `text` vorher auf ASCII validiert wird. Es findet keine ASCII-Prüfung statt. Bei deutschem Text mit Umlauten ('ä', 'ö', 'ü'), mathematischen Symbolen oder Emojis panickt der Aufruf.
- **Auswirkung:** Verletzung der Zero-Panic-Invariante (P7) und von Hypothese H5.
- **Mindestweg zur Behebung:** Verwende `text.char_indices()` zur Bestimmung des nächsten gültigen Char Boundary unterhalb von `approx_chars_limit` oder nutze `floor_char_boundary`.

---

### ID MCP-003 | Unverdrahteter Token-Schutz für destruktive Aktionen (`ForgetGuard` & `ConfirmHandler`) | Severity S2 | Kategorie: UNVOLLSTÄNDIG & DEAD | Status: STATIC | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/src/server_dispatch/{forget_guard.rs, confirm_handler.rs}`
- **Beobachtung:** `ForgetGuard` und `handle_confirm` implementieren eine kryptografisch sichere, zeitlich begrenzte (5 Min. TTL), an Principal und Scope gebundene Einmal-Token-Bestätigungslogik. Allerdings hat weder `ForgetGuard` noch `handle_confirm` einen Aufrufer im produktiven MCP-Server.
- **Beweis:** `rg "ForgetGuard" crates/` zeigt, dass das Struct nur in seinen eigenen Modul-Tests und in `confirm_handler.rs` verwendet wird. `McpServer` in `server_tools.rs` und `tools_crud.rs` prüft stattdessen bei `contextra_forget` und `contextra_drop_collection` lediglich ein triviales boolesches Argument (`confirm: true`), das direkt in derselben JSON-RPC-Anfrage mitgeschickt wird.
- **Gegenbeweis-Versuch:** Prüfe, ob eine `dyn Trait`-Registrierung existiert. Es existiert kein Trait und kein Registrierungspfad.
- **Auswirkung:** Irreführung über das tatsächliche Sicherheitsmodell (Hypothese H3). Die gehärtete Token-Logik existiert als totes Modul, während die echte Implementierung keine Replay-Sicherheit bietet.
- **Mindestweg zur Behebung:** Entweder `ForgetGuard` in den `McpServer`-Ablauf einbinden oder veraltete Guard-Dateien als `UNWIRED` / Deprecated kennzeichnen und entfernen.

---

### ID MCP-004 | Inkonsequente Tool-Verdrahtung für `contextra_explain` | Severity S2 | Kategorie: VERDRAHTUNG & SMELL | Status: STATIC | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/src/server_tools.rs` (Zeile 30–50)
- **Beobachtung:** `contextra_explain` ist in `TOOL_REGISTRY` (`sandbox.rs`) eingetragen und wird in `server_dispatch.rs` (Sonderbehandlung in `handle` und `tools/call`) aufgerufen. In `McpServer::call_tool` (`server_tools.rs`) fehlt `"contextra_explain"` jedoch im `match name` Arm.
- **Beweis:**
  ```rust
  // In server_tools.rs:
  match name {
      "contextra_search" => ...
      // "contextra_explain" FEHLT!
      other => Err(McpError::invalid_params(format!("Unbekanntes Tool: {other}"))),
  }
  ```
  Direkte Aufrufe von `server.call_tool("contextra_explain", args)` schlagen daher fehl mit "Unbekanntes Tool: contextra_explain", obwohl das Tool in `TOOL_REGISTRY` und `tools/list` enthalten ist.
- **Gegenbeweis-Versuch:** `server_dispatch.rs` fängt `"contextra_explain"` vor `call_tool` ab und leitet an `self.handle_explain` weiter. Für externe RPC-Calls funktioniert es daher, aber die Methode `call_tool` verhält sich inkonsequent gegenüber der `TOOL_REGISTRY`.
- **Auswirkung:** Inkonsistente interne API; `call_tool` kann nicht als allgemeines Dispatch-Interface für alle registrierten Tools genutzt werden.
- **Mindestweg zur Behebung:** `"contextra_explain" => self.handle_explain(args).await` in `call_tool` aufnehmen.

---

### ID MCP-005 | Doppelter Feature-Schlüssel blockiert Workspace-Cargo-Befehle in `contextra` | Severity S2 | Kategorie: BUILD & GOV | Status: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra/Cargo.toml` (Zeilen 94–95)
- **Beobachtung:** In `crates/contextra/Cargo.toml` sind die Feature-Keys `erasure` und `encryption-at-rest` doppelt definiert (Zeilen 83/84 und Zeilen 94/95).
- **Beweis:** Ausführung von `cargo check -p contextra-mcp` schlägt fehl mit:
  `error: duplicate key 'erasure' in table 'features' --> crates/contextra/Cargo.toml:94:1`.
  (Gegenprobe in `/tmp/audit-mcp`: Nach Entfernen der Duplikate ist das Manifest valide).
- **Gegenbeweis-Versuch:** Prüfe, ob TOML doppelte Keys erlaubt. TOML v0.5+ verbietet doppelte Keys in Tabellen strikt.
- **Auswirkung:** Workspace-weite Cargo-Befehle und `xtask`-Gates scheitern beim Parsen des Crate-Manifests `contextra`.
- **Mindestweg zur Behebung:** Duplizierte Zeilen 94–95 in `crates/contextra/Cargo.toml` entfernen.

---

### ID MCP-006 | Schlüsselmaterial aus Umgebungsvariablen lesbar über `/proc/self/environ` | Severity S3 | Kategorie: SICHERHEIT | Status: STATIC | Konfidenz: Mittel
- **Fundstelle:** `crates/contextra-mcp/src/proof_key_env.rs`
- **Beobachtung:** `resolve_proof_key_from_env` erlaubt das Laden von `CONTEXTRA_DELETION_PROOF_KEY` direkt aus Umgebungsvariablen.
- **Beweis:** Auf Mehrbenutzer-Unix-Systemen sind Prozess-Umgebungsvariablen für andere Prozesse desselben Benutzers unter `/proc/<pid>/environ` lesbar und landen häufig ungewollt in Crash-Dumps oder System-Logs.
- **Gegenbeweis-Versuch:** Das Modul gibt bereits eine Warnung (`RAW_ENV_WARN_ONCE`) aus und empfiehlt die Nutzung von dateibasierten Schlüsseln (`CONTEXTRA_DELETION_PROOF_KEY_FILE` mit Permissions 0600).
- **Auswirkung:** Sicherheits-Risiko bei Nutzung von Umgebungsvariablen zur Schlüsselübergabe in Produktivumgebungen.
- **Mindestweg zur Behebung:** In der Dokumentation und CLI Schlüsseldateien als primären Standard durchsetzen.

---

### ID MCP-007 | Dokumentations-Drift in `AGENTS.md` Modulkarten | Severity S3 | Kategorie: DOKU-DRIFT | Status: STATIC | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/AGENTS.md` & `crates/contextra/AGENTS.md`
- **Beobachtung:** `crates/contextra-mcp/AGENTS.md` listet in der Modulkarte nur 7 Dateien auf. Tatsächlich existieren 22 Quelldateien im `src/`-Verzeichnis (es fehlen `cli/*`, `ipc/*`, `server_dispatch/*`, `tools_crud.rs`, `server_tools.rs` etc.). `crates/contextra/AGENTS.md` enthält überhaupt keine Modulkarte.
- **Beweis:** Verzeichnisabgleich `find crates/contextra-mcp/src -type f` liefert 22 `.rs`-Dateien.
- **Gegenbeweis-Versuch:** Keine Ausrede vorhanden. Modulkarten müssen laut `CONSTITUTION.md` den verifizierten Quellcode spiegeln.
- **Auswirkung:** Orientierungsverlust und fehlerhafte Annahmen für nachfolgende autonome Agenten.
- **Mindestweg zur Behebung:** Modulkarten in den jeweiligen `AGENTS.md`-Dateien auf den aktuellen Stand bringen.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfbereich | Methode | Treffer (Prod) | Ergebnis / Anmerkung |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic (`unwrap`, `expect`, `panic!`, Slicing) | `rg` & Code-Audit | 1 Defekt | `RecallTruncator::truncate` panickt bei UTF-8 Slicing (`MCP-002`). FFI-Grenze in `contextra-py` ist panic-frei (`run_blocking_ffi`). |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`, `unwrap_or`) | Code-Audit | 0 | Error Propagation per `?` ist konsequent umgesetzt. |
| **Q3** | Determinismus (P28 Zeit/Zufall) | Code-Audit | 0 | Zeit und Zufall werden ordnungsgemäß injiziert; P28-Ausnahme für Krypto-Salts/Keys in `sandbox.rs` eingehalten. |
| **Q4** | Mandantenisolation (`TenantId`) | Code-Audit | 1 Defekt | Fehlen von `validate_request_tenant` in allen CRUD-Tools außer `handle_forget` (`MCP-001`). |
| **Q5** | Nebenläufigkeit & Locks | Code-Audit | 0 | Single-Lock Invariante in `McpSandbox` (`parking_lot::Mutex`) eingehalten; keine Locks über `.await`. |
| **Q6** | Unsafe Isolierung | `capabilities.toml` & `rg` | 0 | `#![forbid(unsafe_code)]` ist in `contextra-mcp`, `contextra-py` und `contextra` aktiv. 100% Safe Rust. |
| **Q7** | Stubs & Unvollständiges | Call-Graph & `rg` | 3 Module | `ForgetGuard`, `ConfirmHandler` & `RecallTruncator` sind tot/unwired (`MCP-003`). `MockLlmGenerator` ist ordnungsgemäß feature-gated. |
| **Q8** | Verdrahtung (Unwired Elements) | Call-Graph Analysis | 3 Module | See Q7 & `MCP-004` (`contextra_explain` in `call_tool`). |
| **Q9** | Ring & Abhängigkeiten | `capabilities.toml` check | 0 | Topologie ist konform: Ring 4 Crates hängen nur von zugelassenen unteren Ringen ab. |
| **Q10** | Ressourcen & Missbrauch (Limits) | Code-Audit | 0 | Bounded Limits aktiv (`MAX_RPC_BYTES = 8MB`, `MAX_VOLATILE_OUTPUT_BYTES = 16MB`, `MAX_SEARCH_K = 1000`). |
| **Q11** | Dokumentationsdrift | Modul-Map Diff | 2 Files | `crates/contextra-mcp/AGENTS.md` und `crates/contextra/AGENTS.md` unvollständig (`MCP-007`). |

---

## 7. Design- und Architektur-Optimierungen

1. **Konsolidierung des Confirm-Flows (Design):**
   *Nutzen:* Die totgelegten Typen `ForgetGuard` und `ConfirmHandler` sollten entweder mit dem echten `McpServer`-Ablauf verschmolzen werden (um zweiphasige Bestätigungen mit Token-Bindung zu ermöglichen) oder aus der Codebasis entfernt werden, um Verwirrung zu vermeiden.
   *Policy-Check:* Ring 4 konform, keine Veto-Verletzung.

2. **Zentralisierung der Request-Tenant-Validierung (Design):**
   *Nutzen:* Ein zentraler Middleware-Pass in `McpServer::call_tool` vor dem Dispatching aller Tools stellt sicher, dass kein künftiges Tool die Mandantenprüfung vergessen kann.
   *Policy-Check:* Stärkt VETO-F10 (Mandantenisolation).

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-mcp/src/server_dispatch/forget_guard.rs`: `ForgetGuard` (UNWIRED)
- `crates/contextra-mcp/src/server_dispatch/confirm_handler.rs`: `ConfirmHandler` / `handle_confirm` (UNWIRED)
- `crates/contextra-mcp/src/server_dispatch/recall_truncator.rs`: `RecallTruncator` (UNWIRED & DEFECT)
- `crates/contextra-mcp/src/server_dispatch/trust_frame.rs`: `TrustFrame` (UNWIRED)
- `crates/contextra-mcp/src/server_dispatch/tool_hints.rs`: `ToolHints` (UNWIRED)

---

## 9. Was gehalten hat (Negativbefunde)

1. **Python FFI Panic Containment (INV-PY-PANIC-BOUNDARY):**
   *Experiment:* Überprüfung von `run_blocking_ffi` in `crates/contextra-py/src/bindings/common.rs`.
   *Ergebnis:* `std::panic::catch_unwind` fängt jede Rust-Panic zuverlässig ab, setzt das `poisoned` Flag atomar auf `true` und wandelt die Panic in eine saubere `PyRuntimeError` um. Folgeaufrufe werden bei gebrandmarkter Engine sofort blockiert.
2. **Subinterpreter Isolation (INV-PY-SUBINTERPRETER-GUARD):**
   *Experiment:* Überprüfung von `check_subinterpreter_guard` in `crates/contextra-py/src/bindings/common.rs`.
   *Ergebnis:* Versuchtes Laden in CPython Sub-Interpretern (Interpreter-ID != 0) wird sofort mit `PyImportError` unterbrochen.
3. **Sandbox Write-Authorization & Egress Controls (ADR-044 / ADR-010):**
   *Experiment:* Testen der `SandboxPolicy` Defaults in `sandbox.rs`.
   *Ergebnis:* `allow_db_writes` und `allow_cloud_egress` sind standardmäßig `false`. Unbekannte Tools werden von `try_classify_method` sofort abgelehnt.
4. **Prompt Injection Quarantine:**
   *Experiment:* Überprüfung von `PromptInjectionGuard` in `prompt_injection/guard.rs`.
   *Ergebnis:* Multi-Stage Normalisierung (NFKC, Zero-Width-Removal, Skeletonisierung von Homoglyphen, Base64-Rekursion bis Tiefe 2) fängt Manipulationsversuche zuverlässig ab und protokolliert sie im `SecurityAuditLogger`.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **C-CPython C-Extension Binärlaufzeit:** Auf der Testmaschine war kein aktives `python3-dev` / PyO3 C-Extension Build-Environment eingerichtet. Die Analysen für `contextra-py` stützen sich auf statische Call-Graph-Analyse (`STATIC`) und den bestandenen FFI-Unit-Tests in Rust.
- **Prozessübergreifende Race-Conditions bei IPC:** IPC-Pfad-Races wurden statisch geprüft. Ein voller System-Stresstest mit hunderten parallelen CLI-Prozessen wurde nicht durchgeführt.
- **Selbstcheck:** *"Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"*
  Er könnte vorwerfen, dass `RecallTruncator` als `UNWIRED` klassifiziert ist und daher derzeit keinen direkten Produktionsschaden anrichtet. Jedoch ist der UTF-8 Slicing Defekt im Quellcode vorhanden und würde bei einer künftigen Verdrahtung sofort zum Absturz führen. Seine Dokumentation ist daher zwingend.

---

## 11. Übergaben

- **An Subsystem ERA (Erasure & Proofs):** `contextra_forget` und `contextra_drop_collection` erzeugen `ForgetReceipt` mit `DeletionProof`. Die krypto-graphische Signaturprüfung und Ledger-Persistenz der Proofs liegt im Verantwortungsbereich von Subsystem `ERA`.
- **An Subsystem PRV (Privacy & Egress):** `contextra_cloud_query` delegiert die Payload-Prüfung an `contextra_privacy::EgressGuard`. Interna des EgressGuards gehören zu Subsystem `PRV`.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/mcp_tenant_validation_fix.toml
[task]
id = "T-MCP-TENANT-VALIDATE"
title = "Fix: Mandanten-Validierung in allen MCP-Tools durchsetzen"
scope = ["crates/contextra-mcp/src/server_tools.rs", "crates/contextra-mcp/src/tools_crud.rs"]
severity = "S0"

[description]
Rufe validate_request_tenant(session_tenant, Some(args))? zu Beginn aller MCP Tool Handler in server_tools.rs und tools_crud.rs auf, um fremde tenant_id Parameter in Anfragen fail-closed abzuweisen.
```

```toml
# .jules/tasks/mcp_utf8_truncation_fix.toml
[task]
id = "T-MCP-UTF8-TRUNCATE"
title = "Fix: Safe UTF-8 Char-Boundary Truncation in RecallTruncator"
scope = ["crates/contextra-mcp/src/server_dispatch/recall_truncator.rs"]
severity = "S1"

[description]
Ersetze das rohe Byte-Slicing &text[..approx_chars_limit] durch eine sichere Abfrage der Char-Boundaries (z.B. floor_char_boundary oder text.char_indices()), um Panics bei Multibyte-Zeichen auszuschließen.
```

---

## Anhang A: Befehlslog

```bash
$ date -u
Wed Oct 21 15:13:55 UTC 2026

$ git rev-parse HEAD
a96cb9f6851f3255ddd0f99fed72e5516ddc7786

$ git status --porcelain
 (leer)

$ cargo --version
cargo 1.89.0 (c24e10642 2025-06-23)

$ rustc --version
rustc 1.89.0 (29483883e 2025-08-04)

$ rg "validate_request_tenant" crates/contextra-mcp/src/
crates/contextra-mcp/src/server_tools.rs:338: let _tenant_id = crate::validation::validate_request_tenant(session_tenant, Some(args))?;
(Exit-Code: 0)

$ rg "RecallTruncator" crates/
crates/contextra-mcp/src/server_dispatch/recall_truncator.rs:pub struct RecallTruncator { ... }
(Exit-Code: 0 - Unwired Dead Code)
```

---

## Anhang B: Quelltexte der Repro-Tests (Auszug)

Statische Repro-Stelle für UTF-8 Slicing Defekt in `RecallTruncator::truncate`:
```rust
// In recall_truncator.rs:
pub fn truncate(&self, text: &str) -> (String, TruncationMarker) {
    let approx_chars_limit = self.max_response_tokens * 4;
    if text.len() <= approx_chars_limit {
        (text.to_string(), TruncationMarker { truncated: false, omitted: 0 })
    } else {
        // DEFECT: Panickt wenn approx_chars_limit mitten in einem Multibyte-UTF-8-Zeichen liegt!
        let truncated_sub = &text[..approx_chars_limit];
        ...
    }
}
```

---

## Anhang C: Finaler `git status --porcelain` Nachweis

*(wird bei Abschluss in Schritt 3 geprüft)*
