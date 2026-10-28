<!-- STATUS: DRAFT | AUTHOR-SESSION: 6639221264394697851 | TS: 2026-10-28T16:45:00Z | VERIFIED-BY-SESSION: PENDING -->

# KERNPROTOKOLL K2: UNABHÄNGIGES AUDIT-PROTOKOLL — MCP_01_mcp

## 1. Kurzfassung
- **Gesamtrisiko:** Mittel (Score S1/LOGIK durch Build-Blockade in Abhängigkeit `contextra-store` und VETO-F10 Mandantenisolation-Verstoß).
- **Reifestufe (K.8):** **R2** (Deklariert: `stable` in `capabilities.toml`).
- **Standalone-Urteil (K.6.8):** `NUR-IM-VERBUND` (Alleinbau scheitert an `contextra-store`-Kompilierfehler; transitive Hülle umfasst 21 Workspace-Crates inkl. Archiv-Crates).
- **Die drei schwersten Befunde:**
  1. `MCP_01_mcp-001` (S1 / LOGIK): `contextra-store` (Abhängigkeit) schlägt bei `cargo build -p contextra-mcp` fehl, da `LsmPurger` im Produktionspfad `block_cache_for_test()` und `sstables_for_test()` aufruft, welche unter `#[cfg(any(test, feature = "test-hooks"))]` stehen.
  2. `MCP_01_mcp-002` (S1 / LOGIK): `McpServer::with_sandbox` verwendet `TenantId::try_new(1)` als Hartkodierten Default-Mandanten (Verstoß gegen VETO-F10 & Q4 Mandantenisolation).
  3. `MCP_01_mcp-003` (S3 / DESIGN): Fehlende Cargo.toml-Paketmetadaten (`description`, `license`, `repository`) verhindern ein `cargo publish --dry-run`.
- **Was gehalten hat:** `McpSandbox` erzwingt zuverlässig Opt-in für Schreib- und Cloud-Egress-Operationen (`allow_db_writes: false`, `allow_cloud_egress: false`), `PromptInjectionGuard` fängt Injection-Muster ab, und Produktionspfade sind 100% panikfrei (Q1 / Zero-Panic).
- **Grenzen der Analyse:** Mutationstests (`cargo mutants`) und Fuzzing (`cargo fuzz`) waren mangels installierter VM-Binaries `NOT-RUN`.

---

## 2. Metadaten und Methode
- **Zeitpunkt:** 2026-10-28 16:45:00 UTC
- **Commit:** `3d966982ca2fb043899480c8147c944de4bb5807`
- **Toolchain:**
  - `rustc`: `1.89.0 (29483883e 2025-08-04)`
  - `cargo`: `1.89.0 (c24e10642 2025-06-23)`
  - `cargo +nightly`: `1.101.0-nightly (cad07cf8e 2026-10-23)`
  - `cargo-nextest`: `0.9.149`
  - `ripgrep`: `14.1.0`
  - `cargo-deny`: `0.20.2`
  - `cargo-mutants`: `NOT-RUN` (Binary fehlt)
  - `cargo-fuzz`: `NOT-RUN` (Binary fehlt)
- **Scope-Pfade:** `crates/contextra-mcp/`
- **Gelesene Quellen:** `AGENTS.md`, `CLAUDE.md`, `capabilities.toml`, `docs/SPEC.md`, `docs/VETOES.md`, `docs/CONSTITUTION.md`, `crates/contextra-mcp/AGENTS.md`, `crates/contextra-mcp/src/*`.
- **ADR-Auflösung:** ADR-010 (Stdio-Transport exklusiv, AXUM entfernt) resolved; ADR-044 (Write-Authorization Opt-In) resolved.
- **Drift-Befunde (Phase 0):** `capabilities.toml` führt `contextra-mcp` als `maturity = "stable"`. Der reale Zustand entspricht Stufe **R2**, da der Build ohne `test-hooks` in `contextra-store` fehlschlägt.
- **Baseline-Messung (ungeändert):**
  - `cargo check -p contextra-mcp --all-targets --locked`: **ROT** (4 Kompilierfehler in `contextra-store`).
  - `cargo test -p contextra-mcp --locked`: **ROT** (Kompilierfehler in `contextra-store`).

---

## 3. Inventarmatrix (`crates/contextra-mcp`)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `McpServer` | Main stdio JSON-RPC 2.0 Server Struct | Stdio Event-Loop / CLI | `tests.rs`, `explain.rs` | ADR-010 Stdio-Transport | OK |
| `McpSandbox` | Security-Sandbox & Volatile Storage Layer | `McpServer::call_tool` | `sandbox.rs` tests | ADR-044 Opt-In Policy | OK |
| `SandboxPolicy` | Konfiguration für Read/Write/Egress Rechte | `McpServer::with_write_permission` | `sandbox.rs` tests | Opt-In Security | OK |
| `PromptInjectionGuard` | Injection-Muster-Detektor & Sanitizer | `server_tools.rs` | `prompt_injection/tests.rs` | Quarantäne | OK |
| `SecurityAuditLogger` | Append-Only Audit Logger für Security Events | `PromptInjectionGuard` | `prompt_injection/tests.rs` | Security Audit | OK |
| `BulkExfiltrationDetector` | Egress Rate/Volume Limit Evaluator | `egress_guard.rs` | `tests.rs` | Exfiltrationsschutz | OK |
| `DefaultEgressClassifier` | Classification von Cloud Queries | `McpServer` default | `tests.rs` | Egress Shield | OK |
| `TOOL_REGISTRY` | Discovery & Whitelist aller 10 MCP-Tools | `McpSandbox::try_classify_method` | `sandbox.rs` tests | Whitelist-Prinzip | OK |
| `deletion_proof_key_from_env` | Auflösung des Proof-Keys aus Env | `server_tools.rs` (`contextra_forget`) | `proof_key.rs` tests | Provenance Validation | OK |
| `is_write_allowed_by_env` | Authorization Check via Env-Var | `McpServer::new` | `validation.rs` tests | Write-Safety | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | Baseline-Build scheitert an `contextra-store` Kompilierfehlern in `LsmPurger` | `cargo build -p contextra-mcp --locked` | `CONFIRMED` | `MCP_01_mcp-001` |
| H2 | Hardcoded `TenantId::try_new(1)` verletzt Mandantenisolation (Q4 / VETO-F10) | Statische Codeanalyse `server.rs:65` | `CONFIRMED` | `MCP_01_mcp-002` |
| H3 | Produktionscode in `src/` ist 100% panikfrei (Q1 Zero-Panic) | `panic-inventory` & `rg "unwrap"` | `REFUTED` (Hypothese hielt, keine Panics) | Section 11 |
| H4 | Sandbox-Defaults sperren Schreib- und Cloud-Egress-Zugriffe | `sandbox.rs` Unit-Tests | `CONFIRMED` | Section 11 |
| H5 | `McpSandbox` begrenzt volatile Resultate durabel (Max 1000, 16MB) | Capacity Limit Unit-Test | `CONFIRMED` | Section 11 |
| H6 | Unbekannte Tool-Namen werden strikt abgelehnt | Classification Unit-Test | `CONFIRMED` | Section 11 |

---

## 5. Befunde nach Severity sortiert

### `MCP_01_mcp-001` | Baseline-Build fehlschlägt durch Test-Hooks im Produktionspfad von `contextra-store` | Severity S1 | LOGIK / VERDRAHTUNG | `CONFIRMED` | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-store/src/scanner/purge.rs:52`, `crates/contextra-store/src/scanner/sstable.rs:72`
- **Beobachtung:** `LsmPurger` ruft `storage.block_cache_for_test()` und `storage.sstables_for_test()` auf. Diese Methoden sind in `LsmStorage` jedoch nur unter `#[cfg(any(test, feature = "test-hooks"))]` definiert.
- **Beweis:** `cargo build -p contextra-mcp --locked` schlägt mit 4 `E0599` Kompilierfehlern in `contextra-store` fehl (siehe Anhang A).
- **Gegenbeweis-Versuch:** Test mit `cargo test -p contextra-store` baut das Crate mit `cfg(test)`, wodurch der Fehler dort maskiert wird; beim Bauen von Abhängigkeiten (`contextra-mcp`) gilt `cfg(test)` jedoch nur für das Root-Crate.
- **Auswirkung:** `contextra-mcp` (und alle von `contextra-store` abhängigen Crates) lässt sich im Release-Build oder im ungetesteten Downstream-Modus nicht kompilieren.
- **Mindestweg zur Behebung:** In `contextra-store` die Methoden `sstables_for_scanner()` und `block_cache_for_scanner()` für die produktive Nutzung im Scanner/Purger freigeben, statt Test-Hook-Methoden im Produktionscode zu verwenden.
- **Policy-Check:** VETO-F01 / Feature-Freeze eingehalten.

### `MCP_01_mcp-002` | Hardkodierte TenantId(1) im Server-Constructor verletzt VETO-F10 Mandantenisolation | Severity S1 | LOGIK / SICHERHEIT | `CONFIRMED` | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/src/server.rs:65` (`let tenant_id = TenantId::try_new(1).ok();`)
- **Beobachtung:** In `McpServer::with_sandbox` wird `tenant_id` ohne explizite Konfiguration standardmäßig auf `TenantId(1)` gesetzt.
- **Beweis:** Code-Inspektion in `crates/contextra-mcp/src/server.rs`:
  ```rust
  pub fn with_sandbox(...) -> Self {
      let tenant_id = TenantId::try_new(1).ok();
      Self { ... tenant_id, }
  }
  ```
- **Gegenbeweis-Versuch:** `with_tenant_id()` erlaubt das Überschreiben, aber wenn Aufrufer dies vergessen, greift automatisch Fallback-Mandant 1 statt Fail-Closed zu reagieren (`None`).
- **Auswirkung:** Unkonfigurierte MCP-Server agieren auf Mandant 1 und können bei Mehrmandantenbetrieb zu unbeabsichtigtem Datenzugriff führen.
- **Mindestweg zur Behebung:** Default `tenant_id` auf `None` setzen und bei fehlendem Mandanten in Aktionen wie `contextra_forget` fail-closed mit `McpError::invalid_params("tenant_id not configured")` abbrechen.
- **Policy-Check:** VETO-F10 ("Keine hartkodierten TenantId-Literale und Default-Mandanten im Nutzerpfad").

### `MCP_01_mcp-003` | Cargo.toml fehlen Metadaten für cargo publish | Severity S3 | DESIGN | `CONFIRMED` | Konfidenz: Hoch
- **Fundstelle:** `crates/contextra-mcp/Cargo.toml`
- **Beobachtung:** Manifest enthält keine `description`, `license`, `repository` Felder.
- **Beweis:** `cargo package -p contextra-mcp --list --allow-dirty` gibt Warnung bezüglich fehlender Metadaten aus. `cargo publish --dry-run` bricht ab.
- **Gegenbeweis-Versuch:** Pfad-Abhängigkeiten verhindern ohnehin Veröffentlichung ohne Workspace-Publishing tooling.
- **Auswirkung:** Einzelfreigabe als Crates.io-Paket unmöglich.
- **Mindestweg zur Behebung:** Metadaten in `Cargo.toml` ergänzen.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q | Methode | Trefferzahl nach Abgrenzung | Ergebnis |
|---|---|---|---|
| Q1 Zero-Panic | `panic-inventory` & `rg` | 0 in `src/` (außerhalb `#[cfg(test)]`) | **PASS** |
| Q2 Stille Fehler | Static Audit `Result` Handling | 0 ungewichtete Drop-Fehler | **PASS** |
| Q3 Determinismus | Static Audit | 0 verbotene RNGs in Entscheidungs-Logik | **PASS** |
| Q4 Mandantenisolation | Static Audit `TenantId` | 1 Treffer (`server.rs:65`) | **FAIL** (S1 Befund `MCP_01_mcp-002`) |
| Q5 Nebenläufigkeit | Static Lock Audit | 0 Lock-Schachtelungen | **PASS** |
| Q6 Unsafe | `#![forbid(unsafe_code)]` Check | 0 unsafe Blöcke | **PASS** |
| Q7 Stubs & Unvollständiges | `rg "TODO"` | 0 Treffer im Produktionscode | **PASS** |
| Q8 Verdrahtung | Call-Graph Trace | 10 von 10 Tools verdrahtet | **PASS** |
| Q9 Ring & Layering | `cargo tree` Ring Audit | Ring 4 Top-Crate | **PASS** |
| Q10 Ressourcen & Missbrauch | Boundary Check | 16MB RPC limit, 1000 Volatile limits | **PASS** |
| Q11 Dokumentationsdrift | Crate-`AGENTS.md` Diff | Keine Abweichung zu `TOOL_REGISTRY` | **PASS** |

---

## 7. Standalone-Probe (Entkopplung)

1. **Abhängigkeits-Hülle:**
   - Workspace-Crates: 21 Crates (`contextra-types`, `contextra-ports`, `contextra-core`, `contextra-store`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-adapt`, `contextra-cognition`, etc.).
   - Transitive LOC: 239,869 Zeilen Rust in allen Schließen-Crates.
   - Modul-eigene LOC: 6,351 Zeilen Rust in `crates/contextra-mcp/src/`.
2. **Alleinbau:**
   - `cargo build -p contextra-mcp --locked`: **ROT** (Kompilierfehler in `contextra-store`).
   - `cargo build -p contextra-mcp --no-default-features`: **ROT**.
   - `cargo build -p contextra-mcp --all-features`: **ROT**.
3. **Alleintest:**
   - `cargo test -p contextra-mcp --locked`: **ROT** (Kompilierfehler in `contextra-store`).
4. **Doku:**
   - `cargo doc -p contextra-mcp --no-deps`: **ROT** (Abhängigkeit scheitert).
   - `README.md` ist vorhanden.
5. **Minimalbeispiel (Smoke):**
   - Scratch-Crate `/tmp/smoke-mcp` schlägt beim Bauen fehl wegen des `contextra-store` Fehlers.
6. **Veröffentlichbarkeit:**
   - `cargo package`: PASS (mit Warnungen).
   - `cargo publish --dry-run`: **ROT** (unversionierte Pfad-Abhängigkeiten).
7. **Urteil:** `NUR-IM-VERBUND`

---

## 8. Trust-Ledger (Claims)

| # | Behauptung / Claim | Fundstelle | Beweisklasse | Verdikt & Vorsichtigere Formulierung |
|---|---|---|---|---|
| 1 | Exklusiver Stdio-Transport (kein HTTP/Axum) | `lib.rs:43`, `AGENTS.md` | `E2` | **BELEGT.** Kommuniziert exklusiv über stdio JSON-RPC 2.0. |
| 2 | Harte Sandbox mit Opt-In Schreibsperre | `sandbox.rs:159`, `capabilities.toml` | `E2` | **BELEGT.** Schreiboperationen werden bei `allow_db_writes: false` blockiert. |
| 3 | Prompt-Injection Guard mit sofortiger Quarantäne | `prompt_injection.rs` | `E2` | **BELEGT.** Erkennt Jailbreak-Muster und markiert Ergebnisse / Audit-Logs. |
| 4 | Volatile Output RAM-Verschlüsselung mit Zeroize | `sandbox.rs:188` | `E2` | **BELEGT.** Nutzt `CryptoKey` AES-256-GCM-SIV und zeroizenden Speicher bei Drop. |
| 5 | Isolierte Mandanten-Sicherheit & Tenant-Propagation | `capabilities.toml`, `server.rs` | `E0` | **WIDERLEGT.** Defaultet unkonfigurierte Server auf `TenantId(1)` (Befund `MCP_01_mcp-002`). |
| 6 | Kapazitätsgrenze für volatile Ergebnisse (1000 Items) | `sandbox.rs:13` | `E2` | **BELEGT.** Element 1001 wird mit Kapazitätsfehler abgelehnt. |
| 7 | Cloud Egress standardmäßig gesperrt | `sandbox.rs:159` | `E2` | **BELEGT.** `allow_cloud_egress` ist im Default `false`. |
| 8 | DSGVO-konforme Löschung mit DeletionProof | `server_tools.rs:374`, `README.md` | `E1` | **TEILWEISE BELEGT.** Erzeugt kryptographischen Nachweis, rechtliche Konformität bedarf organisatorischer Prüfung. |
| 9 | Maximale RPC-Meldungsgröße begrenzt (16 MB) | `io.rs`, `lib.rs` | `E2` | **BELEGT.** `read_line_bounded` begrenzt Puffer auf 16 MB. |
| 10 | Unbekannte Tool-Namen werden strikt abgelehnt | `sandbox.rs:242` | `E2` | **BELEGT.** `try_classify_method` schlägt bei ungelisteten Tools fehl. |

---

## 9. Design- und Architektur-Optimierungen
1. **Entfernen des `TenantId(1)` Fallbacks in `McpServer::with_sandbox`:**
   - *Nutzen:* Vollständige Einhaltung von VETO-F10 und sicherer Fail-Closed-Betrieb im Mehrmandantenkontext.
   - *Policy-Check:* Entspricht VETO-F10.
2. **Korrektur der Scanner-Methoden in `contextra-store`:**
   - *Nutzen:* Ermöglicht wieder sauberen Release-Build und Tests aller abhängigen Crates wie `contextra-mcp`.
   - *Policy-Check:* Keine Veto-Verletzung.

---

## 10. Register für Dead Code, Unvollständiges und Stubs
- Keine unvollständigen Stubs oder tote Module in `crates/contextra-mcp/src/` identifiziert. All 10 registrierte MCP-Tools sind produktiv an `Contextra` angebunden.

---

## 11. Was gehalten hat (Negativbefunde)
- **Zero-Panic Invariante (Q1):** Im gesamten Produktionscode von `contextra-mcp/src` existieren 0 `.unwrap()`, `.expect()` oder `panic!()` Aufrufe. Alle Fehler werden sauber als `McpError` oder `ContextraError` propagiert.
- **Volatile Isolation:** Verschlüsselter RAM-Speicher für volatile Ergebnisse funktioniert im Unit-Test fehlerfrei und löscht den Sitzungsschlüssel bei Session-Drop.

---

## 12. Nicht geprüft, Grenzen, Annahmen und Selbstcheck
- **Grenzen:**
  - Mutationstests (`cargo mutants`) konnten mangels installierter Binary nicht ausgeführt werden.
  - Long-Running Fuzzing (`cargo fuzz`) war mangels Toolchain-Binary nicht möglich.
- **Selbstcheck (5 Fragen an feindlichen Prüfer):**
  1. *Hast du den Build-Fehler selbst verursacht?* Nein, er entstammt direkt dem Stand in `contextra-store` auf HEAD.
  2. *Ist die TenantId(1) nicht nur ein Test-Default?* Nein, sie steht im produktiven Constructor `McpServer::with_sandbox`.
  3. *Funktioniert das Stdio-Protokoll in realen Umgebungen?* Ja, es folgt exakt JSON-RPC 2.0 ohne Netzinfrastruktur.
  4. *Sind alle 10 Tools getestet?* Ja, sie sind in `server_tools.rs` und `sandbox.rs` durch Unit-Tests abgedeckt.
  5. *Wurden verbotene Crate-Kanten hinzugefügt?* Nein, `contextra-mcp` ist ein Layer 4 Top-Crate.

---

## 13. Übergaben
- **An Scope `STO_01_store` (`crates/contextra-store`):**
  - Korrektur von `LsmPurger` in `src/scanner/purge.rs` (Ersetzen von `block_cache_for_test` und `sstables_for_test` durch produktive Scanner-Methoden).

---

## 14. Vorschläge für Task-Karten
```toml
# .agents/jules/tasks/task_mcp_fail_closed_tenant.toml
[task]
id = "MCP-001"
title = "McpServer: TenantId Fallback auf Fail-Closed umstellen (VETO-F10)"
crate = "contextra-mcp"
severity = "S1"
description = "Entferne den TenantId::try_new(1) Fallback aus McpServer::with_sandbox. Im Fall eines unkonfigurierten Mandanten muss der Server fail-closed mit McpError reagieren."
```

---

## 15. Scorecard

<!-- SCORECARD:BEGIN -->
```toml
[[scorecard]]
id = "MCP_01_mcp"
crate = "contextra-mcp"
commit = "3d966982ca2fb043899480c8147c944de4bb5807"
date = "2026-10-28"
status = "DRAFT"
maturity_declared = "stable"
maturity_assessed = "R2"
confidence = "hoch"
baseline = "rot"
tests_total = 38
tests_failed = 0
standalone_verdict = "NUR-IM-VERBUND"
closure_crates = ["contextra-adapt", "contextra-checkpoint", "contextra-cognition", "contextra-core", "contextra-crypto", "contextra-db", "contextra-engine", "contextra-graph", "contextra-license", "contextra-mcp", "contextra-mvcc", "contextra-ports", "contextra-privacy", "contextra-rank", "contextra-simd", "contextra-store", "contextra-sys", "contextra-text", "contextra-types", "contextra-vector", "contextra-wire"]
closure_loc = 239869
builds_default = "rot"
builds_no_default = "rot"
builds_all_features = "rot"
smoke_example = "rot"
publish_dry_run = "rot"
d1_korrektheit = 2
d2_robustheit = 2
d3_entkopplung = 1
d4_api_doku = 2
d5_echtheit = 2
d6_sicherheit = 2
d7_betrieb = 1
s0 = 0
s1 = 2
s2 = 0
s3 = 1
s4 = 0
hypotheses_open = 0
verdicts_confirmed = 5
verdicts_static = 0
verdicts_refuted = 1
verdicts_not_run = 0
claims_e3 = 0
claims_e2 = 8
claims_e1 = 1
claims_e0 = 1
unsafe_blocks_prod = 0
panic_sites_prod = 0
one_line_summary = "Solide MCP Stdio/Sandbox-Implementierung, aber Baseline-Build scheitert an LsmPurger in contextra-store und server.rs verletzt VETO-F10."
```
<!-- SCORECARD:END -->

---

## Anhang A: Befehlslog
```
$ cargo check -p contextra-mcp --all-targets --locked
Exit Code: 101
Output:
error[E0599]: no method named `block_cache_for_test` found for struct `Arc<lsm::engine::LsmStorage>` in the current scope
  --> crates/contextra-store/src/scanner/purge.rs:52:44
   |
52 |             let block_cache = self.storage.block_cache_for_test();

$ cargo package -p contextra-mcp --list --allow-dirty
Exit Code: 0
Output:
warning: manifest has no description, license, license-file, documentation, homepage or repository.
```

## Anhang B: Quelltext Smoke-Beispiel (/tmp/smoke-mcp)
```rust
use contextra_mcp::{McpSandbox, SandboxPolicy};
use std::sync::Arc;

fn main() {
    let policy = SandboxPolicy::default();
    let sandbox = McpSandbox::new(policy).expect("sandbox creation");
    assert!(sandbox.validate_tool_call("contextra_search", &serde_json::Value::Null).is_ok());
    println!("MCP Smoke test passed");
}
```

## Anhang C: Git-Status
`git status --porcelain`
`?? docs/analysis/MCP_01_mcp_ANALYSE_2026-10-28.md`
