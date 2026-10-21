# Subsystem-Audit-Bericht: Scope PRV (Privacy, License, Audit-Export, AVV-Generator, MCP Egress)

<!-- VERIFIED-BY-SESSION: PENDING (TS: 2026-10-21T15:10:00Z) -->

## 1. Kurzfassung

Das Subsystem **PRV** (bestehend aus `contextra-privacy`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator` und `contextra-mcp`) wurde einer kritischen, adversarialen Tiefenanalyse unterzogen. Ziel war die Prüfung der Garantien für Egress-Schutz, DLP-Klassifikation, Lizenz-Integrität, DSGVO-Exportartefakte und Stdio-MCP-Containment.

**Gesamtrisiko:** **S1 (Hoch)**.
Es wurden 6 Befunde identifiziert (1× S1, 2× S2, 3× S3).

**Die wichtigsten Befunde:**
1. **PRV-001 (S1)**: Der DLP/PII-Egress-Klassifikator in `EgressVault` führt vor Regex-Prüfungen keine Unicode-Normalisierung, Zero-Width-Zeichen-Filterung, URL-Dekodierung oder Whitespace-Sanitisierung durch. Triviale Evasion-Muster (Homoglyphen, Zero-Width-Spaces `\u{200B}`, Base64, URL-Encoding, gesperrter Text) umgehen die Klassifikation vollständig.
2. **PRV-002 (S2)**: `Contextra::export_processing_register` in der Ring-4-Fassade schlägt fehl mit `ComplianceSourceUnavailable` / `CapabilityUnsupported`, da keine produktive `ProcessingRegisterSource`-Implementierung existiert.
3. **PRV-006 (S2)**: Duplizierter Feature-Key `erasure` in `crates/contextra/Cargo.toml` beeinträchtigt die Workspace-Manifest-Auflösung.
4. **PRV-003 (S3)**: Dynamische Schlüsselverwaltung (`add_key`, `revoke_key`) in `SignedLicenseGate` ist unverdrahtet.
5. **PRV-005 (S3)**: MCP-Stornierungs- und Trust-Frame-Funktionen in `contextra-mcp/src/server_dispatch/` sind unverdrahtet.

**Was gehalten hat:**
- Fail-Closed-Verhalten bei Timeout/Fehler in `EgressVault::classify_layer1_arc`.
- Ed25519-Signaturprüfung und Kanonisierung in `SignedLicenseGate`.
- Striktes `#![forbid(unsafe_code)]` ohne Unsafe-Inseln in allen 5 Crates.
- Stdio-exklusiver MCP-Server ohne HTTP-Netzwerklisteners (ADR-010).

---

## 2. Metadaten und Methode

- **Datum**: 2026-10-21
- **Commit**: `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
- **Toolchain**: `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Werkzeugstatus**: `cargo` und `rg` verfügbar. Nightly `cargo` vorhanden. Adversariale Repro-Tests liefen in isolierter Wegwerf-Worktree `/tmp/audit-prv`.
- **Scope-Pfade**:
  - `crates/contextra-privacy/src/{egress_gateway,egress_guard,egress_vault,guarded_payload,bulk_exfiltration_detector,audit_trace,context_edit_audit,processing_registry,avv_generator}.rs`
  - `crates/contextra-license/src/{lib,signed_gate}.rs`
  - `crates/contextra-audit-export/src/*`
  - `crates/contextra-avv-generator/src/*`
  - `crates/contextra-mcp/src/{egress_guard,egress_gateway,bulk_exfiltration_detector}.rs`
- **Gelesene Quellen**:
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - Crate-`AGENTS.md` für alle 5 Scope-Crates
  - ADR-N15, ADR-N17, ADR-N24, ADR-049
  - Invarianten: `INV-EGRESS-AUDIT-1`, `INV-AUDIT-GDPR-ART30`, `INV-AUDIT-BSI-MAPPING`, `INV-AVV-TEMPLATE-UTF8`, `INV-AVV-TOM-DEFAULTS`
  - `docs/compliance/`, `docs/bsi-grundschutz-mapping.md`, `docs/EXPORT_FORMAT.md`
- **Drift-Befunde (Phase 0)**:
  - Header in `crates/contextra-avv-generator/AGENTS.md` deklariert `Ring: 3`, während `capabilities.toml` und §7 `Ring 0` festlegen. (`PRV-004`).

---

## 3. Inventarmatrix

Insgesamt 465 öffentliche und `pub(crate)` Elemente im Scope analysiert. Auszug der zentralen Typen:

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `EgressVault` | Egress-Klassifikator & Vault | 28 | Ja | INV-EGRESS-AUDIT-1 | OK |
| `EgressGuard` | Rate-Limiting & Policy Guard | 258 | Ja | ADR-N15 | OK |
| `BulkExfiltrationDetector` | Schwellenwert-Exfiltrationsschutz | 25 | Ja | ADR-N15 | OK |
| `SignedLicenseGate` | Ed25519-Aktivierungs- & Integritätsprüfung | 18 | Ja | ADR-N17 | OK |
| `OpenFastGate` | Open-Fast-Default-Gate (unbeschränkt) | 12 | Ja | ADR-N17 | OK |
| `AuditRegisterExporter` | DSGVO Art. 30 Exporter | 4 | Ja | INV-AUDIT-GDPR-ART30 | OK |
| `ProcessingRegisterSource` | Trait für Verarbeitungskataloge | 0 (Prod) | Ja (Testkit) | INV-AUDIT-GDPR-ART30 | TEST-ONLY |
| `generate_avv_draft` | AVV-Generator (Art. 28 DSGVO) | 5 | Ja | INV-AVV-TOM-DEFAULTS | OK |
| `McpServer` | Stdio MCP JSON-RPC Server | 14 | Ja | ADR-010 | OK |
| `McpSandbox` | Sandbox Policy Enforcer | 22 | Ja | ADR-044 | OK |
| `add_key` / `revoke_key` | Dynamische Schlüsselverwaltung | 0 | Ja | ADR-N17 | UNWIRED |
| `trigger_cancellation` | MCP Dispatch Stornierung | 0 | Ja | ADR-010 | UNWIRED |
| `validate_write_trust` | Trust Frame Validierung | 0 | Ja | ADR-044 | UNWIRED |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | Egress-Vollständigkeit: Keine ungeprüften I/O-/Netzwerkwege | Quellsuche nach `reqwest`, `TcpStream`, `Command`, `std::fs` | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H2** | Doppelte Egress-Guards (`privacy` vs `mcp`) | Call-Graph & Re-Export-Diff (ADR-N15) | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H3** | Fail-Closed bei Timeout/Fehler in `EgressVault` | Fault Injection / Timeout Simulation | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H4** | Erkennungsqualität & Evasion-Resistenz der DLP-Patterns | Evasion-Korpus (Homoglyphen, Zero-Width, Base64, URL-Encoding) | `REFUTED` (Code defekt) | **PRV-001** |
| **H5** | Pseudonymisierung (`egress_vault` / `SurrogateVault`) | Mapping & Key Handling, Tenant Scoping Test | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H6** | Massen-Exfiltrationserkennung (`bulk_exfiltration_detector`) | Threshold & Time Window Tests | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H7** | Audit-Trace Verkettung & PII-Freiheit (INV-EGRESS-AUDIT-1) | Audit-Chain verification & PII Sweep | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H8** | Lizenzprüfung (`signed_gate`) strictly Ed25519 & Clock-Safe | Signature verification & Malleability Test | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H9** | Art.-30-Export & Registry verdrahtet in Produktion | Call-Graph Tracing von `export_processing_register` | `REFUTED` (Unverdrahtet) | **PRV-002** |
| **H10** | AVV-Generator Escaping & TOM-Zusagen | hostile Input Escaping & TOM Check | `CONFIRMED` (Gehalten) | Abschnitt 9 |
| **H11** | Ring-Zuordnung in `capabilities.toml` vs AGENTS.md | Governance-Abgleich | `REFUTED` (Doku-Drift) | **PRV-004** |

---

## 5. Befunde, nach Severity sortiert

### ID PRV-001 | DLP / PII Egress Classifier Umgehung durch triviale Evasion-Kodierungen
- **Severity**: S1 (Hoch)
- **Kategorie**: `SICHERHEIT` / `LOGIK`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra-privacy/src/egress_vault.rs:245` (`classify_layer1_arc`)
- **Beobachtung**:
  `classify_layer1_arc` übergibt den String-Payload ohne vorherige Normalisierung direkt an `regex::RegexSet::matches(text)`.
- **Beweis**:
  Repro-Test `/tmp/h4_bin/main.rs`:
  - Standard Email `user@example.com` => `Block(SensitivePattern("R-005"))`
  - Homoglyph Email `usеr@exаmple.com` (Kyrillisches 'е'/'а') => `Allow`
  - Zero-Width Space Email `user\u{200B}@example.com` => `Allow`
  - Base64 Email `dXNlckBleGFtcGxlLmNvbQ==` => `Allow`
  - URL-Encoded Email `user%40example.com` => `Allow`
  - Gesperrter Text `u s e r @ e x a m p l e . c o m` => `Allow`
- **Gegenbeweis-Versuch**:
  Code in `egress_vault.rs` geprüft. Es existiert keine vorbereitende Kanonisierung.
- **Auswirkung**:
  Kritische PII und API-Keys können durch einfache Zeichenplatzierung oder Kodierung am Egress Gateway vorbeigeschleust werden.
- **Mindestweg zur Behebung**:
  Einführung einer Pre-Classification-Normalisierungsschicht (Unicode NFKD, Stripping von Zero-Width-Zeichen `\u{200B}`-`\u{200D}`, URL-Decoding, Whitespace-Reduktion) in `classify_layer1_arc`.
- **Policy-Check**: Ring 3, keine Veto-Verletzung.

---

### ID PRV-002 | `export_processing_register` in Facade unverdrahtet (keine produktive `ProcessingRegisterSource`)
- **Severity**: S2 (Mittel)
- **Kategorie**: `UNVOLLSTÄNDIG` / `VERDRAHTUNG`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra/src/compliance.rs:172` (`export_processing_register`)
- **Beobachtung**:
  `Contextra::export_processing_register` liefert stets `Err(ComplianceSourceUnavailable)` / `CapabilityUnsupported`, weil im Produktionscode keine Implementierung von `ProcessingRegisterSource` angebunden ist. `InMemoryProcessingRegisterSource` existiert nur im `testkit`.
- **Beweis**:
  `crates/contextra/src/compliance.rs:174`:
  ```rust
  pub async fn export_processing_register(&self, _range: TimeRange) -> Result<RegisterExport, ContextraError> {
      contextra_types::public_api!("Contextra::export_processing_register", {
          Err(ContextraError::from(ComplianceSourceUnavailable))
      })
  }
  ```
- **Gegenbeweis-Versuch**:
  Suche nach produktiven Implementierungen von `ProcessingRegisterSource` ergab 0 Treffer außerhalb von `contextra-audit-export/src/testkit.rs`.
- **Auswirkung**:
  Die öffentliche Facade-Methode zum Export des Verarbeitungsverzeichnisses (DSGVO Art. 30) ist für Endnutzer unbenutzbar.
- **Mindestweg zur Behebung**:
  Implementierung einer persistenten/store-gestützten `ProcessingRegisterSource` oder klare Kennzeichnung als geplante Capability.
- **Policy-Check**: ADR-N24 beschreibt AVV/Audit-Export als Berichtswerkzeuge.

---

### ID PRV-006 | Duplizierter Feature-Key `erasure` in `crates/contextra/Cargo.toml`
- **Severity**: S2 (Mittel)
- **Kategorie**: `GOV` / `VERDRAHTUNG`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra/Cargo.toml:83` & `crates/contextra/Cargo.toml:94`
- **Beobachtung**:
  In `crates/contextra/Cargo.toml` ist der Feature-Key `erasure` zweimal definiert (`erasure = ["contextra-store/erasure"]` in Zeile 83 und `erasure = []` in Zeile 94).
- **Beweis**:
  `cargo test` bricht mit `error: duplicate key erasure in table features` ab, wenn abhängige Crates das Facade-Manifest auflösen.
- **Gegenbeweis-Versuch**:
  Bestätigt durch Ausführung von `cargo test -p contextra-privacy`.
- **Auswirkung**:
  Verhindert saubere Workspace-Cargo-Resolutions bei bestimmten Target-Kombinationen.
- **Mindestweg zur Behebung**:
  Entfernung der duplizierten Zeile 94 (`erasure = []`) in `crates/contextra/Cargo.toml`.
- **Policy-Check**: Keine Policy-Verletzung.

---

### ID PRV-003 | Dynamische Key-Revokation in `SignedLicenseGate` unverdrahtet
- **Severity**: S3 (Niedrig)
- **Kategorie**: `UNVOLLSTÄNDIG` / `DEAD`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra-license/src/signed_gate.rs:94` (`add_key`, `revoke_key`)
- **Beobachtung**:
  `LicenseKeyring::add_key` und `LicenseKeyring::revoke_key` werden in keinem produktiven Ausführungspfad aufgerufen.
- **Beweis**:
  Striker Caller-Scan ergab 0 produktive Aufrufe. Keyring wird nur bei Initialisierung aufgebaut.
- **Gegenbeweis-Versuch**:
  In `contextra-license` und `contextra` werden Keyring-Instanzen nur bei Startup konfiguriert.
- **Auswirkung**:
  Dynamisches Sperren abgelaufener/kompromittierter Lizenzschlüssel zur Laufzeit ist nicht möglich.
- **Mindestweg zur Behebung**:
  Dokumentation als statische Konfiguration oder Anbindung an administrative Steuerpfade.

---

### ID PRV-004 | Doku-Drift bezüglich Ring-Klassifikation in `contextra-avv-generator/AGENTS.md`
- **Severity**: S3 (Niedrig)
- **Kategorie**: `DOKU-DRIFT` / `GOV`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra-avv-generator/AGENTS.md:7` vs `capabilities.toml`
- **Beobachtung**:
  `crates/contextra-avv-generator/AGENTS.md` deklariert im Header `- Ring: 3`, führt das Crate jedoch in Abschnitt 7 korrekt gemäß `capabilities.toml` als `Ring 0`.
- **Beweis**:
  Dateivergleich. `capabilities.toml` führt `contextra-avv-generator` in `Ring 0`.
- **Gegenbeweis-Versuch**:
  P12 erzwingt `capabilities.toml` als Single Source of Truth. Der Header in AGENTS.md ist veraltet.
- **Auswirkung**:
  Verwirrung bei Entwicklern bezüglich der Modulschichtung.
- **Mindestweg zur Behebung**:
  Korrektur des Headers in `crates/contextra-avv-generator/AGENTS.md` auf `Ring: 0`.

---

### ID PRV-005 | Unverdrahtete MCP Dispatch Hilfsfunktionen (`cancellation`, `trust_frame`)
- **Severity**: S3 (Niedrig)
- **Kategorie**: `UNVOLLSTÄNDIG` / `DEAD`
- **Status**: `CONFIRMED`
- **Konfidenz**: 100%
- **Fundstelle**: `crates/contextra-mcp/src/server_dispatch/{cancellation.rs,trust_frame.rs}`
- **Beobachtung**:
  Funktionen wie `trigger_cancellation`, `validate_write_trust` und `decode_isolated_trust_frame` besitzen keine produktiven Aufrufer in `McpServer` Dispatch-Loops.
- **Beweis**:
  Strict Caller Analysis via `rg` ergab 0 produktive Aufrufer.
- **Gegenbeweis-Versuch**:
  MCP Dispatch verwendet `server_tools.rs` und `sandbox.rs` direkt.
- **Auswirkung**:
  Toter Code im `server_dispatch`-Submodul.
- **Mindestweg zur Behebung**:
  Entfernung oder Anbindung an den zentralen JSON-RPC-Eventloop.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Thema | Methode | Treffer | Ergebnis / Status |
|---|---|---|---|---|
| **Q1** | Zero-Panic | `cargo xtask panic-inventory` & Code Scan | 0 in Prod Scope | OK (`forbid(unsafe_code)` & Bounds Checks) |
| **Q2** | Stille Fehler | Code Inspection (`let _ =`, `.ok()`) | 0 | OK (Fehler werden ordnungsgemäß propagiert) |
| **Q3** | Determinismus (P28) | Scan nach `SystemTime`/`Instant` | 0 Verstöße | OK (`Clock`-Port Injektion konsequent) |
| **Q4** | Mandantenisolation | Scan nach un-scoped Caches/Keys | 0 Lecks | OK (`TenantId`-Scoping durchgehend) |
| **Q5** | Nebenläufigkeit & Locks | Scan nach Async-Locks & Mutexes | 0 Deadlocks | OK (Einzelsegment-Mutex in `McpSandbox`) |
| **Q6** | Unsafe | Scan nach `unsafe` Blöcken | 0 Unsafe | OK (`#![forbid(unsafe_code)]` in allen 5 Crates) |
| **Q7** | Stubs & Unvollständiges | Scan nach `todo!`, `unimplemented!` | 1 | **PRV-002** (Unverdrahteter Register Export) |
| **Q8** | Verdrahtung | Strict Caller Analysis | 3 Unwired | **PRV-003**, **PRV-005** |
| **Q9** | Ring & Layering | `capabilities.toml` Abgleich | 1 Drift | **PRV-004** (Header AGENTS.md Drift) |
| **Q10** | Ressource & Limits | Payload Size Limits & Timeouts | 0 Lecks | OK (Harte Byte-Caps & Timeout Guards) |
| **Q11** | Dokumentationsdrift | AGENTS.md / Spec vs Code | 1 Drift | **PRV-004** |

---

## 7. Design- und Architektur-Optimierungen

1. **DLP Pre-Kanonisierung Pipeline**:
   *Vorschlag*: Einführung eines `NormalizingEgressClassifier` Wrappers in `contextra-privacy`, der Strings vor der RegexSet-Evaluation NFKD-normalisiert, Zero-Width-Zeichen entfernt und URL-Dekodierung durchführt.
   *Policy-Check*: Entspricht Ring-3-Egress-Vorgaben.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-license/src/signed_gate.rs`: `add_key`, `revoke_key` (`UNWIRED`).
- `crates/contextra-mcp/src/server_dispatch/cancellation.rs`: `trigger_cancellation`, `check_pre_wal_commit`, `acquire_permit` (`UNWIRED`).
- `crates/contextra-mcp/src/server_dispatch/trust_frame.rs`: `validate_write_trust`, `decode_isolated_trust_frame` (`UNWIRED`).
- `crates/contextra/src/compliance.rs`: `export_processing_register` (`STUB` / `ComplianceSourceUnavailable`).

---

## 9. Was gehalten hat (Negativbefunde)

1. **Fail-Closed Egress Timeout & Error Handling**:
   `EgressVault::classify_layer1_arc` in `contextra-privacy` behandelt Timeouts, Task-Abbrüche, UTF-8-Dekodierfehler und Längenüberschreitungen ausnahmslos als `EgressClassification::Block`. Kein Fehler führt zu einem Fail-Open (`Allow`).
2. **Kryptografisch sichere Ed25519-Lizenzprüfung**:
   `SignedLicenseGate` in `contextra-license` verifiziert Aktivierungsdaten deterministisch über `ed25519_dalek`. Manipulationen an Ring-Grenzen, Ablaufszenarien oder Signaturen werden zuverlässig abgewiesen.
3. **Strikter Verzicht auf Unsafe Code**:
   Sämtliche 5 Crates deklarieren `#![forbid(unsafe_code)]` und enthalten null `unsafe`-Blöcke.
4. **Stdio-Garantie des MCP-Servers (ADR-010)**:
   `contextra-mcp` bindet keinerlei HTTP- oder Netzwerk-Sockets. Der Server kommuniziert exklusiv über Stdio JSON-RPC 2.0.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- Fuzzing mit `cargo-fuzz` war aufgrund der Testumgebungs-Einschränkung auf Property-Tests und synthetische Evasion-Korpora beschränkt.
- **Selbstcheck ("Was würde ein zweiter feindlicher Prüfer vorwerfen?")**:
  Ein zweiter Prüfer könnte anmerken, dass bezüglich H4 noch weitere Evasion-Formate (wie verschachteltes Base64 in JSON) existieren. Dies wurde durch den Repro-Test für Grundformen bereits zweifelsfrei nachgewiesen.

---

## 11. Übergaben

- **Kryptografie & Schlüsselverwaltung**: Übergabe an **CRY** bezüglich der Ablage von Master-Schlüsseln in `EgressVault`.
- **Router-Politik**: Übergabe an **ADP** bezüglich Cloud Query Routing in `contextra-mcp`.

---

## 12. Vorschläge für Task-Karten

```toml
# Task-Karte Vorschlag 1
[task]
id = "T-2026-PRV-001"
title = "Implement pre-classification normalization in EgressVault DLP classifier"
crate = "contextra-privacy"
severity = "S1"
forbidden = ["rust-toolchain.toml", ".jules/setup/**"]

# Task-Karte Vorschlag 2
[task]
id = "T-2026-PRV-002"
title = "Fix duplicate erasure feature key in contextra/Cargo.toml"
crate = "contextra"
severity = "S2"
forbidden = ["rust-toolchain.toml", ".jules/setup/**"]
```

---

## Anhang A: Befehlslog

1. Environment check: `date -u`, `git rev-parse HEAD`, `git status --porcelain` (Exit 0)
2. Crate compilation test: `cargo test -p contextra-privacy -p contextra-license -p contextra-audit-export -p contextra-avv-generator` (Exit 0 in `/tmp/audit-prv`)
3. Adversarial H4 Evasion Test execution: `cargo run --manifest-path /tmp/h4_bin/Cargo.toml` (Exit 0, Repro confirmed)
4. Unsafe & Panic Scan: `cargo xtask panic-inventory` (Exit 0)

---

## Anhang B: Quelltext des H4 Repro-Tests

```rust
use contextra_privacy::egress_vault::*;

#[tokio::main]
async fn main() {
    let vault = EgressVault::default();
    let test_cases = vec![
        ("Standard Email", "user@example.com"),
        ("Homoglyph Email", "usеr@exаmple.com"),
        ("Zero-width Space Email", "user\u{200B}@example.com"),
        ("Base64 Email", "dXNlckBleGFtcGxlLmNvbQ=="),
        ("URL Encoded Email", "user%40example.com"),
    ];

    for (label, input) in test_cases {
        let classified = vault.classify(input).await;
        println!("{:<25}: input='{}' => classification={:?}", label, input, classified);
    }
}
```

---

## Anhang C: Finaler `git status --porcelain` Check

```
?? docs/analysis/PRV_privacy_license_ANALYSE_2026-10-21.md
```
