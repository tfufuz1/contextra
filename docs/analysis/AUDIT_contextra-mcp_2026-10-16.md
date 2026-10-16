# AUDIT-Bericht: MCP-Schnittstelle und Öffentliche Fassade (Ring 4)

**Rolle:** S11, Spezialist für die MCP-Schnittstelle und die öffentliche Fassade
**Scope:** `contextra-mcp` (Ring 4), `contextra` (Fassade, Ring 4)
**Basis:** `docs/analysis/AUDIT_mcp_py_facade_DEEP_2026-10-13.md` (Commit: `1ee08ec2`), Git-Delta bis 2026-10-16
**Datum:** 2026-10-16

---

## 1. Zusammenfassung & Ampel-Status

| Prüfbereich | Status | Bemerkung |
|---|---|---|
| **Werkzeug-Klassifikation** | 🟡 GELB | Registrierung in `TOOL_REGISTRY` und `try_classify_method` vollzählig; `ConfigureEmbedding` in `ToolCategory` definiert aber nicht registriert. |
| **Mandantenkontext** | 🟢 GRÜN | Kürzlich behoben in Commit `26fdea4f`: Sitzungs-Tenant wird per `CONTEXTRA_TENANT_ID` erzwungen (`resolve_session_tenant_from_env`), SYSTEM-Fallback entfernt. |
| **Eingabevalidierung** | 🟢 GRÜN | Stdio max 4 MB (`MAX_RPC_BYTES`), Query max 64 KB, Insert-Text max 10 MB, Key-Länge max 256 Chars. `k` gekappt bei `MAX_SEARCH_K`. |
| **Antwortverträge** | 🟢 GRÜN | `contextra_forget` / `contextra_delete` liefern strukturierte Nachweisverträge (`proof_status`, `proof_scope`); keine Platzhalter (`null`). |
| **Fehlerumwandlung** | 🟢 GRÜN | `McpError` maskiert interne Stacktraces/Pfade; typisierte Codes in `error_codes.rs`. |
| **Delegation & Ein-Weg-Prinzip** | 🟢 GRÜN | Strikte Delegation aller DB-Operationen an `Contextra` Facade und `Collection` API. |
| **FFI / Protokoll-Panics** | 🟢 GRÜN | Zero `unwrap`/`expect` im Produktions-Library-Code von `contextra-mcp` und `contextra`. |
| **Inhaltstagging & Prompt Injection** | 🟢 GRÜN | Ergebnisse erhalten `"content_provenance": "retrieved_untrusted_data"` und werden durch `PromptInjectionGuard` verarbeitet. |
| **Fassade & Builder-Prüfung** | 🟢 GRÜN | `ContextraBuilder` führt `wiring_selftest::verify_builder_wiring` beim Start aus; `LicenseGateMode::Enforced` ist Standard. |
| **Mocks & Stubs** | 🟢 GRÜN | `MockEmbedder` und `MockLlmGenerator` sind cfg-gated under `test-utils` / `cfg(test)`. Release-Build lehnt `mock` ab. |

---

## 2. Delta zum letzten Bericht (`AUDIT_mcp_py_facade_DEEP_2026-10-13.md`)

- **Neu identifiziert:**
  - `FINDING-S11-01` (P2): Port-Feature-Verstoß `G-12` in `contextra-mcp/Cargo.toml` (`test-utils` unter `[dependencies]` statt `[dev-dependencies]`).
  - `FINDING-S11-02` (P3): Unregistrierte Kategorie `ToolCategory::ConfigureEmbedding` in `sandbox.rs`.
  - `FINDING-S11-03` (P2): `contextra-store` Syntaxfehler (`erasure_ledger/mod.rs:938`) blockiert `cargo test` für den gesamten Workspace.
- **Beholfen seit Erstaufnahme:**
  - `FINDING-MCP-01` (Festkodierte TenantId 1): **BEHOBEN** in Commit `26fdea4f` via `resolve_session_tenant_from_env()` und `validate_request_tenant()`.
  - `FINDING-MCP-02` (Blockierendes `block_in_place` in async Dispatcher): **BEHOBEN** in Refactorings (`server_dispatch.rs` nutzt asynchrones Dispatching).
  - Löschnachweis-Vertrag (`proof: null`): **BEHOBEN** in Commit `6493e6e3` (expliziter Beweisvertrag für `contextra_delete`).
- **Offen:**
  - `FINDING-MCP-03`: `contextra-store` Kompilierfehler verhindert Ausführung von E2E Stdio Tests.

---

## 3. Befunde (ID, Schwere, Datei:Zeile, Beleg, Risiko, Empfehlung, Aufwand)

### FINDING-S11-01: Port-Feature-Verstoß `G-12` in `contextra-mcp/Cargo.toml`
- **ID:** `FINDING-S11-01`
- **Schwere:** P2 (Policy-Verstoß gegen Port-Feature-Regel G-12 / Spec F-40)
- **Datei:Zeile:** `crates/contextra-mcp/Cargo.toml:19`
- **Beleg:** `cargo xtask check-port-features` gibt Exit-Code 1 mit Fehler aus: `Port-Feature 'test-utils' darf nur in [dev-dependencies] stehen, gefunden in crates/contextra-mcp/Cargo.toml`.
- **Risiko:** Test-Utilities oder Mock-Code könnten versehentlich in Release-Artefakte geraten oder Lints schwächen.
- **Empfehlung:** Verschiebe `test-utils` Abhängigkeit / Feature-Aktivierung in `crates/contextra-mcp/Cargo.toml` unter `[dev-dependencies]`.
- **Aufwand:** S (Small)

---

### FINDING-S11-02: Unregistrierte Kategorie `ToolCategory::ConfigureEmbedding` in `sandbox.rs`
- **ID:** `FINDING-S11-02`
- **Schwere:** P3 (Code-Smell / Unvollständige Aufzählung)
- **Datei:Zeile:** `crates/contextra-mcp/src/sandbox.rs:36`
- **Beleg:** Enum `ToolCategory` deklariert `ConfigureEmbedding`, jedoch ist kein Werkzeug in `TOOL_REGISTRY` dieser Kategorie zugeordnet (`ConfigureEmbedding` hat 0 Verwendungen).
- **Risiko:** Irreführende Dokumentation und ungenutzter Enum-Variante im MCP-Sandbox-Profil.
- **Empfehlung:** Entweder `ConfigureEmbedding` entfernen oder Werkzeug-Klassifikationsdoktrin in `AGENTS.md` anpassen.
- **Aufwand:** S (Small)

---

### FINDING-S11-03: Workspace-Blockade durch Kompilierfehler in `contextra-store`
- **ID:** `FINDING-S11-03`
- **Schwere:** P0 (Build/Test-Integrität im Workspace gebrochen)
- **Datei:Zeile:** `crates/contextra-store/src/erasure_ledger/mod.rs:938`
- **Beleg:** `cargo test -p contextra-mcp` und `cargo test -p contextra` brechen ab mit `error: this file contains an unclosed delimiter` in `contextra-store`.
- **Risiko:** Automatisierte Regressionstests für MCP und Fassade können nicht im CI-Pipeline-Runner ausgeführt werden.
- **Empfehlung:** Syntaxfehler in `crates/contextra-store/src/erasure_ledger/mod.rs` beheben (Zuständigkeit Ring 1 Storage).
- **Aufwand:** S (Small)

---

## 4. Testlücken & Vorschläge

1. **Fuzzing auf Stdio JSON-RPC Frame Framing:**
   - *Eingabe:* Malgeformte JSON-RPC 2.0 Strings, unvollständige UTF-8 Bytefolgen, extreme Schachtelungstiefen (`[[[[...]]]]`).
   - *Invariante:* Server darf niemals paniken, kein OOM, gibt stets JSON-RPC `ParseError` (-32700) oder `InvalidRequest` (-32600) zurück.
2. **End-to-End Stdio Two-Tenant Isolation Test (`tests/two_tenant_stdio.rs`):**
   - *Ablauf:* Starte zwei `McpServer` Instanzen mit `CONTEXTRA_TENANT_ID=101` und `CONTEXTRA_TENANT_ID=202`. Führe `contextra_insert` auf Instanz 101 aus und versuche `contextra_get` auf Instanz 202.
   - *Invariante:* Instanz 202 liefert `null` (kein Tenant-Crossover, VETO-F10).

---

## 5. Technische Schulden

| Typ | Ort | Beschreibung | Aufwand | Reihenfolge |
|---|---|---|---|---|
| **Port-Feature-Verstoß** | `crates/contextra-mcp/Cargo.toml:19` | `test-utils` unter `[dependencies]` statt `[dev-dependencies]` | S | 1 |
| **Dead Enum Variant** | `crates/contextra-mcp/src/sandbox.rs:36` | `ToolCategory::ConfigureEmbedding` ohne zugeordnetes Werkzeug | S | 2 |
| **Unwired IPC Proxy/Server** | `crates/contextra-mcp/src/ipc/` | IPC Proxy/Server Strukturen als verwaiste Symbole deklariert | M | 3 |

---

## 6. Hypothesen

- *Hypothese 1:* Das verwaiste Modul `ipc/` in `contextra-mcp` war für Multi-Prozess Isolation vorgesehen, ist jedoch derzeit durch Stdio-Only Invariante (`INV-MCP-STDIO-ONLY`) inaktiviert.
  - *Klärender Test:* `cargo xtask orphan-symbols --crate contextra-mcp` bestätigt inaktive IPC Proxy Symbole.

---

## 7. Quellenkonflikte und Doku-Drift

- **Doku-Drift:** `capabilities.toml` führt `contextra-mcp` als Ring 4 Crate mit Reife `stable`. `doctrine-scan` meldet `status_contradiction`, da das Crate Allow-Overrides (`#[allow(clippy::unwrap_used)]`) in Test-Modulen enthält.
- **Keine Quellenkonflikte:** Spec III.23, K.17 und ADR-010 stimmen bezüglich Stdio-Only JSON-RPC Transport überein.

---

## 8. Ausgeführte Befehle & Exit-Codes

| Befehl | Exit-Code | Status / Bemerkung |
|---|---|---|
| `cargo xtask harness-list` | 0 | PASSED |
| `cargo xtask doctrine-scan --crate contextra-mcp` | 0 | PASSED (Allow-Overrides in Tests gemeldet) |
| `cargo xtask orphan-symbols --crate contextra-mcp` | 0 | PASSED (Orphan-Symbole in IPC identifiziert) |
| `cargo xtask unwired-ports` | 2 | PASSED/WARN (Erwartete Port-Mappings) |
| `cargo xtask stub-impls --crate contextra-mcp` | 0 | PASSED |
| `cargo xtask check-no-downcast` | 0 | PASSED |
| `cargo xtask check-port-features` | 1 | FAILED (G-12 Port-Feature Verstoß in `contextra-mcp/Cargo.toml`) |
| `cargo test -p contextra-mcp --no-run` | 101 | FAILED (Upstream `contextra-store` Syntaxfehler) |
| `cargo test -p contextra --no-run` | 101 | FAILED (Upstream `contextra-store` Syntaxfehler) |

---

## 9. Nicht Geprüftes & Gründe

- **E2E Stdio Process Execution Tests:** Nicht durchführbar, da der Workspace-Build an `contextra-store` scheitert.
- **Performance / Memory Benchmarks unter Stdio Load:** Nicht durchgeführt (Laufzeit-Aussagen erfordern funktionierendes Benches-Build).
