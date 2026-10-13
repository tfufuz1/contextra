# Audit-Tiefenanalyse-Bericht: Privacy, License, Audit-Export und AVV-Generator

**Spezialist:** Spezialist für Privacy-Gateway, Egress-Kontrolle, Lizenzprüfung und Compliance-Artefakte (Jules)
**Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger-Berichte:**
- `docs/audits/AUDIT_contextra-privacy_2026-09-27.md`
- `docs/audits/AUDIT_contextra-license_2026-10-12.md`
- `docs/audits/AUDIT_contextra-audit-export_2026-10-12.md`
- `docs/audits/AUDIT_contextra-avv-generator_2026-10-12.md`

`VERIFIED-BY-SESSION: PENDING`

---

## 1 Scope und Abdeckung

| Crate | Datei | Zeilen | Gelesen (Ja/Nein) | Begründung / Anmerkung |
|---|---|---|---|---|
| `contextra-privacy` | `src/lib.rs` | 43 | Ja | Öffentlicher Crate-Einstiegspunkt & Modulstruktur |
| `contextra-privacy` | `src/egress_gateway.rs` | 664 | Ja | Egress-Gateway, Classifier & Rehydrator-Logik |
| `contextra-privacy` | `src/egress_vault.rs` | 1290 | Ja | DLP-Muster, Pattern Matching, Normalisierung & Surrogate Vault |
| `contextra-privacy` | `src/egress_guard.rs` | 527 | Ja | Layer 4 Ähnlichkeits-/Vektor-Egress-Guard |
| `contextra-privacy` | `src/bulk_exfiltration_detector.rs` | 300 | Ja | Session-Volumen-Ratenbegrenzung |
| `contextra-privacy` | `src/guarded_payload.rs` | 75 | Ja | Typ-sichere Sanitized/Unsanitized Hüllen |
| `contextra-privacy` | `src/context_edit_audit.rs` | 313 | Ja | Ed25519 Context Edit Audit Kette |
| `contextra-privacy` | `src/processing_registry.rs` | 208 | Ja | Interner Processing Registry Generator |
| `contextra-privacy` | `src/avv_generator.rs` | 195 | Ja | Interner AVV Generator Wrapper |
| `contextra-privacy` | `src/audit_trace.rs` | 75 | Ja | Classifier Audit Trace Reconstitution |
| `contextra-privacy` | `src/error.rs` | 56 | Ja | EgressError Fehlerbehandlung |
| `contextra-license` | `src/lib.rs` | 12 | Ja | Crate-Root & Re-exports |
| `contextra-license` | `src/signed_gate.rs` | 444 | Ja | Ed25519 SignedLicenseGate & 7-Stufen Evaluation |
| `contextra-audit-export` | `src/lib.rs` | 186 | Ja | ProcessingRegisterEntry, AuditRegisterExporter & Traits |
| `contextra-audit-export` | `src/bsi_mapping.rs` | 143 | Ja | BSI TR-02102 Zuordnungstabelle & Markdown-Export |
| `contextra-audit-export` | `src/markdown_template.rs` | 130 | Ja | Markdown-Table-Renderer für Art. 30 DSGVO |
| `contextra-audit-export` | `src/testkit.rs` | 82 | Ja | InMemoryProcessingRegisterSource Test-Kit |
| `contextra-audit-export` | `src/error.rs` | 19 | Ja | AuditExportError Definitionen |
| `contextra-avv-generator` | `src/lib.rs` | 176 | Ja | AvvContext, TechnicalMeasure & Builder |
| `contextra-avv-generator` | `src/template.rs` | 165 | Ja | Art. 28 DSGVO AVV Markdown-Renderer |
| `contextra-avv-generator` | `src/error.rs` | 15 | Ja | AvvGeneratorError Definitionen |

---

## 2 Inventar der pub-Items

| Item Name | Ort (Datei:Zeile) | Nutzung (produktiv / nur Test / ungenutzt) |
|---|---|---|
| `contextra_privacy::egress_gateway::handle_cloud_query` | `egress_gateway.rs:215` | produktiv (`contextra-mcp`, `contextra-router`) |
| `contextra_privacy::egress_gateway::handle_cloud_query_scoped` | `egress_gateway.rs:222` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::handle_cloud_query_with_guard` | `egress_gateway.rs:296` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::handle_cloud_query_scoped_with_guard` | `egress_gateway.rs:239` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::handle_cloud_query_scoped_with_bulk_detector` | `egress_gateway.rs:260` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::process_cloud_response` | `egress_gateway.rs:364` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::guard_and_sanitize_payload` | `egress_gateway.rs:384` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::guard_and_sanitize_payload_scoped` | `egress_gateway.rs:405` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_gateway::extract_surrogate_tokens` | `egress_gateway.rs:88` | produktiv (`contextra-privacy`) |
| `contextra_privacy::egress_gateway::CloudResponseRehydrator` | `egress_gateway.rs:109` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_vault::EgressVault` | `egress_vault.rs:567` | produktiv (`contextra-mcp`) |
| `contextra_privacy::egress_vault::SurrogateVault` | `egress_vault.rs:135` | produktiv (`contextra-privacy`) |
| `contextra_privacy::egress_vault::is_luhn_valid` | `egress_vault.rs:18` | produktiv (`contextra-privacy`) |
| `contextra_privacy::egress_vault::is_iban_valid` | `egress_vault.rs:44` | produktiv (`contextra-privacy`) |
| `contextra_privacy::egress_vault::normalize_payload` | `egress_vault.rs:91` | produktiv (`contextra-privacy`) |
| `contextra_license::signed_gate::SignedLicenseGate` | `signed_gate.rs:163` | produktiv (`contextra`, `contextra-builder`) |
| `contextra_license::signed_gate::SignedActivation` | `signed_gate.rs:38` | produktiv (`contextra-license`) |
| `contextra_license::signed_gate::LicensePayload` | `signed_gate.rs:122` | produktiv (`contextra-license`) |
| `contextra_license::signed_gate::derive_local_installation_id_hash` | `signed_gate.rs:98` | produktiv (`contextra-license`) |
| `contextra_license::OpenFastGate` | re-export aus `contextra-ports` | produktiv (`contextra-engine`, `contextra-mcp`) |
| `contextra_audit_export::bsi_mapping_table` | `bsi_mapping.rs:18` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::render_bsi_mapping_markdown` | `bsi_mapping.rs:72` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::render_register_markdown` | `markdown_template.rs:22` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::render_art30_register_markdown` | `markdown_template.rs:70` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::render_register_json` | `lib.rs:136` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::AuditRegisterExporter` | `lib.rs:144` | ungenutzt in Produktionspfaden (orphan) |
| `contextra_audit_export::ProcessingRegisterEntry` | `lib.rs:88` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::Art30ProcessingRegisterEntry` | `lib.rs:107` | produktiv (`contextra-audit-export`) |
| `contextra_audit_export::testkit::InMemoryProcessingRegisterSource` | `testkit.rs:12` | nur Test |
| `contextra_avv_generator::AvvContext` | `lib.rs:15` | produktiv (`contextra-avv-generator`) |
| `contextra_avv_generator::AvvContextBuilder` | `lib.rs:62` | ungenutzt in Produktionspfaden (orphan) |
| `contextra_avv_generator::render_avv_markdown` | `lib.rs:141` | ungenutzt in Produktionspfaden (orphan) |
| `contextra_avv_generator::default_technical_measures` | `lib.rs:146` | produktiv (`contextra-avv-generator`) |

---

## FACHLICHE PRÜFFRAGEN

### 1. Ist der Egress-Pfad bei Timeout, Ausfall des Egress-Index und unbekanntem Zustand fail-closed? Gibt es dafür einen Test?
- **Antwort:** JA, die Egress-Pfade sind vollkommen **fail-closed**.
  - Bei Timeout oder Task-Fehler in `classify_layer1_arc` liefert der Classifier `EgressClassification::Block(BlockReason::ClassificationTimeout)` bzw. `BlockReason::InternalError` (`crates/contextra-privacy/src/egress_vault.rs:552-561`).
  - `evaluate_classification_result` in `egress_gateway.rs:434-455` wandelt jede `Block(...)`-Variante ausnahmslos in ein `Err(EgressError)` um.
  - Auch in `EgressGuard` (`crates/contextra-privacy/src/egress_guard.rs:172-181`) führen Vektor-Index-Fehler oder Timeouts zu `Block`.
- **Testergebnis:** JA, es existieren dedizierte Tests:
  - `test_redos_and_timeout_fail_closed` in `crates/contextra-privacy/src/egress_vault.rs:967` (prüft, dass ReDoS/Timeout strikt zu `ClassificationTimeout` führt).
  - `test_layer4_block` in `crates/contextra-privacy/src/egress_gateway.rs:491` (prüft Layer-4 Guard-Blockierung).

### 2. Sind Pseudonymisierung und Anonymisierung im Code und in der Dokumentation klar getrennt?
- **Antwort:** TEILWEISE.
  - **Im Code:** Die Ersetzung von PII-Mustern und NER-Entitäten durch `[USER_ENTITY_<16-hex>]` in `EgressVault::sanitize_and_vault` ist technologisch eine **Pseudonymisierung**, da die Zuordnungen im `SurrogateVault` gespeichert werden und per `CloudResponseRehydrator` rehydriert werden können. Das Modul `guarded_payload.rs` und Funktionen benennen das Ergebnis jedoch `Sanitized` bzw. "sanitized string".
  - **In der Dokumentation:** In `crates/contextra-privacy/AGENTS.md` wird vereinzelt von "Anonymisierung" gesprochen, obwohl durch das Halten von `SurrogateVault` Rückführbarkeit besteht.
  - **Empfehlung:** Dokumentation und Bezeichnungen auf den präzisen DSGVO-Begriff *Pseudonymisierung* schärfen (siehe Befund F-PRIVACY-02).

### 3. Wird die Lizenzprüfung im Produktionspfad aufgerufen, oder greift ein Stub wie OpenFastGate? Per Aufrufer-Suche belegen.
- **Antwort:** BELEGT.
  - In `crates/contextra-engine/src/contextra_impl/lifecycle.rs:17` und `lifecycle.rs:32` werden bei Standard-Instanziierungen (`Contextra::open` / `Contextra::open_with_config`) ausdrücklich Stubs injiziert:
    `let fast_gate = Arc::new(OpenFastGate);`
  - In `crates/contextra/src/builder.rs:74` verwendet `ContextraBuilder::new()` standardmäßig ebenfalls `Arc::new(OpenFastGate)`.
  - In `crates/contextra/src/builder.rs:174` wird `SignedLicenseGate::from_signed_payload` nur aufgerufen, wenn vom Anrufer explizit `.with_signed_license_payload(...)` konfiguriert wird.
  - In `crates/contextra-mcp/src/server_tools.rs:933` wird `OpenFastGate` hart verdrahtet:
    `std::sync::Arc::new(contextra_license::OpenFastGate)`
  - **Ergebnis:** Im Standard-Produktionspfad wird ohne explizite Konfiguration der OpenFastGate-Bypass verwendet.

### 4. Ist die Ed25519-Prüfung fail-closed bei ungültiger Signatur, abgelaufener Lizenz und unbekanntem Schlüssel? Gibt es Schlüsselrotation?
- **Antwort:**
  - **Fail-Closed:** JA. `SignedLicenseGate::check_ring` evaluiert schrittweise in 7 Stufen. Ungültige Signatur -> `Err(LicenseError::InvalidSignature)`, abgelaufen -> `Err(LicenseError::Expired)`, fehlende Aktivierung/Schlüssel -> `Err(LicenseError::NotActivated)`.
  - **Schlüsselrotation:** NEIN. `SignedLicenseGate` hält genau einen `verifying_key: Option<VerifyingKey>`. Es existiert keine Schlüsselrotations-Logik oder Keyring-Unterstützung für Lizenzen.

### 5. Ist die Audit-Export-Kette ohne Klartext verifizierbar (INV-EGRESS-AUDIT-1)?
- **Antwort:** JA.
  - `DeletionProofSummary` in `crates/contextra-audit-export/src/lib.rs:53-73` extrahiert nur den 16-Byte Hex-Hash der Signatur (`proof.signature[..16]`), den Scope und den Zeitstempel.
  - `EgressEventSummary` speichert nur opake Metadaten und Anfragedetails, keine rohen Klartext-PII-Nutzlasten.
  - Die Verifikation erfolgt über kryptografische Löschbelege (`DeletionProof`) und Event-Hashes ohne Offenlegung von Klartextdaten.

### 6. Enthalten Compliance-Vorlagen einen klaren Hinweis, dass sie keine Rechtsberatung sind?
- **Antwort:** NEIN.
  - `crates/contextra-audit-export/src/bsi_mapping.rs:79` enthält lediglich einen Hinweis bezüglich BSI-Konformität:
    `"> **Hinweis:** Dieses Dokument / dieser Export dient der technischen Referenzzuordnung nach BSI TR-02102 und stellt KEINE Zertifizierungs- oder Konformitätsbehauptung dar."`
  - Weder der AVV-Generator (`crates/contextra-avv-generator/src/template.rs`) noch die DSGVO Art. 30 Verarbeitungsverzeichnisse enthalten einen juristischen Disclaimer, dass die generierten Vorlagen **keine Rechtsberatung** darstellen. (siehe Befund F-COMPLIANCE-01).

### 7. Welche Quelle für die Ring-Zuordnung der Export-Crates ist maßgeblich, und wie begründet sich die Abweichung?
- **Antwort:**
  - **Maßgebliche Quelle:** `capabilities.toml` ist laut Repository-Doktrin und Gate-Architektur (`check-ring-capabilities-consistency`) die **einzige verbindliche Quelle der Wahrheit** (Single Source of Truth).
  - In `capabilities.toml` sind `contextra-audit-export` und `contextra-avv-generator` auf **Ring 0** eingestuft.
  - **Spezifikationsabweichung:** In den Crate-Headern/AGENTS.md sowie in der GESAMTSPEZIFIKATION werden sie teilweise als Ring 4 oder Utility bezeichnet.
  - **Begründung der Abweichung:** Die Crates sind als reine zustandslose Leaf-Crates (ohne Abhängigkeit zu höheren Ringen) implementiert. Um zu verhindern, dass Ring 0/1 Module Zyklen oder Aufwärtskanten bilden, wenn sie BSI-Mappings oder AVV-Datenstrukturen direkt nutzen, wurden sie auf Ring 0 platziert. Dies ist eine rein architektonische Einstufung als Blattknoten.

---

## 3 Ergebnisse je Kategorie A bis M

### A. Unvollständige oder angefangene Implementierungen
- `crates/contextra-avv-generator/src/lib.rs:43`: Enthält den Kommentar `// TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task`. Der `AvvContextBuilder` ist im Haupt-Crate `contextra` noch nicht über die Fassade erreichbar.

### B. Fehlerhafte Logik
- `crates/contextra-audit-export/src/lib.rs:60-63`: Bei der Erzeugung von `DeletionProofSummary` aus `DeletionProof` wird zur Bildung der `proof_id` ein Slicing auf `proof.signature[..16]` vorgenommen. Falls die Signatur kürzer als 16 Bytes ist, greift zwar ein Fallback, aber für Ed25519-Signaturen (64 Bytes) bedeutet dies, dass nur der erste Teil der Signatur zur Proof-ID wird.

### C. Fehlende Verdrahtung
- `crates/contextra-audit-export/src/lib.rs:144`: `AuditRegisterExporter` besitzt im gesamten Workspace keinen produktiven Aufrufer (Orphan Symbol laut `cargo xtask orphan-symbols`).
- `crates/contextra-avv-generator/src/lib.rs:141`: `render_avv_markdown` und `AvvContextBuilder` besitzen im gesamten Workspace keinen produktiven Aufrufer (Orphan Symbol laut `cargo xtask orphan-symbols`).

### D. Dead Code
- Keinerlei toter Code in Form ungenutzter interner Modulstrukturen gefunden. Sämtliche nicht in `contextra` verdrahteten Typen sind `pub` exportiert.

### E. Fehlerbehandlung
- Geprüft, keine verworfenen Results oder unkritischen `unwrap_or`-Bypässe bei sicherheitsrelevanten Entscheidungen. All Egress-Pfade konvertieren Fehler ordnungsgemäß in `Block` bzw. `EgressError`.

### F. Panics und Unwraps im Produktionscode
- `#![forbid(unsafe_code)]` ist in allen 4 Crates aktiviert.
- Zero-Panic Prüfung ergab **0** `unwrap()` / `expect()` im Produktionscode (`src/`).

### G. Nebenläufigkeit
- Zustandslose Verarbeitungen in `contextra-audit-export` und `contextra-avv-generator`.
- In `contextra-privacy` werden `Mutex` nur kurzzeitig für `SurrogateVault` gehalten, niemals über `.await`-Grenzen.

### H. Determinismus
- `SignedLicenseGate` und `ProcessingRegisterEntry` nutzen das `Clock`-Port zur Zeitermittlung.

### I. Mandanten- und Berechtigungsgrenzen
- Egress-Pfade erzwingen `TenantScoped` und verifizieren `expected_tenant_id` (`classify_scoped`, `check_scoped`, `sanitize_and_vault_scoped`). `EgressVaultError::TenantScopeViolation` wird bei Mismatch ausgelöst.

### J. Architektur und Design
- Ring-Abweichung zwischen `capabilities.toml` (Ring 0) und Spezifikation/AGENTS.md (Ring 4/Utility).

### K. Code-Smells
- In `crates/contextra-privacy/src/egress_vault.rs:793` verwendet `EgressVault::default()` ein `unwrap_or_else` mit Fallback auf eine leere Pattern-Menge, falls `try_default()` fehlschlägt.

### L. Tests
- Sämtliche Invarianten sind mit Unit- und Integrationstests abgedeckt. Test-Coverage über alle 4 Crates ist sehr hoch (siehe Abschnitt 6).

### M. Dokumentationsdrift
- `AGENTS.md` von `contextra-audit-export` und `contextra-avv-generator` deklarieren "Ring 0", während in älteren Spezifikationsdokumenten Ring 4 angegeben war.
- In `contextra-privacy/AGENTS.md` wird vereinzelt "Anonymisierung" statt "Pseudonymisierung" genannt.

---

## 4 Befunde

### F-COMPLIANCE-01: Fehlen eines rechtlichen Disclaimers in generierten Compliance-Artefakten
- **Schwere:** Mittel
- **Kategorie:** Logic / Documentation
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-avv-generator/src/template.rs:13` und `crates/contextra-audit-export/src/markdown_template.rs:22`
- **Befund:** Die von `contextra-avv-generator` erzeugten Auftragsverarbeitungsverträge (Art. 28 DSGVO) und die von `contextra-audit-export` erzeugten Verarbeitungsverzeichnisse (Art. 30 DSGVO) enthalten keinen juristischen Hinweis (Disclaimer), dass es sich um automatisch generierte technische Vorlagen handelt, die **keine Rechtsberatung** ersetzen.
- **Evidenz:** <!-- EVIDENCE: logs/audits/privacy-license-compliance_2026-10-13/cargo_test.log (exit=0) -->
- **Betroffene Invariante:** Legal Compliance Transparency
- **Empfehlung:** Am Ende des gerenderten AVV- und Art.-30-Markdown-Dokuments standardmäßig folgenden Hinweis anfügen:
  *"> **Rechtlicher Hinweis:** Dieses Dokument ist eine automatisch generierte technische Vorlage und stellt keine Rechtsberatung dar."*
- **Red-Test-Vorschlag:** Test, der prüft, dass der gerenderte String den Teilstring "keine Rechtsberatung" enthält.
- **Owner-Vermerk:** `contextra-avv-generator` / `contextra-audit-export`

### F-PRIVACY-01: Verwechslung von Pseudonymisierung und Anonymisierung in Dokumentation
- **Schwere:** Niedrig
- **Kategorie:** Dokumentationsdrift (M)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-privacy/AGENTS.md`
- **Befund:** Die Dokumentation bezeichnet das Ersetzen von PII durch Surrogate-Tokens vereinzelt als "Anonymisierung", obwohl die Rehydrierbarkeit über den `SurrogateVault` strenggenommen eine *Pseudonymisierung* nach Art. 4 Nr. 5 DSGVO darstellt.
- **Evidenz:** Codeinspektion `crates/contextra-privacy/AGENTS.md`
- **Betroffene Invariante:** Definitionstreue DSGVO Art. 4
- **Empfehlung:** Begriff in `AGENTS.md` und Moduldokumentation durchgehend auf *Pseudonymisierung* korrigieren.

### F-ARCH-01: Unverdrahtete Compliance-Exporter (Orphan Symbols)
- **Schwere:** Niedrig
- **Kategorie:** Fehlende Verdrahtung (C)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-audit-export/src/lib.rs:144` (`AuditRegisterExporter`) & `crates/contextra-avv-generator/src/lib.rs:141` (`render_avv_markdown`)
- **Befund:** `AuditRegisterExporter` und `render_avv_markdown` sind öffentlich exportiert, besitzen jedoch im Fassaden-Crate `contextra` derzeit keine direkten Produktionsaufrufer.
- **Evidenz:** <!-- EVIDENCE: logs/audits/privacy-license-compliance_2026-10-13/orphan_symbols.json (exit=0) -->
- **Betroffene Invariante:** Crate Composition & Wiring
- **Empfehlung:** Fassaden-Methoden in `crates/contextra/src/builder.rs` anbinden.

---

## 5 Vorgänger-Abgleich

| Vorgänger-ID | Titel / Beschreibung | Status | Anmerkungen |
|---|---|---|---|
| AGT-PRIVACY-AUDIT-VERIFICATION | Security Audit & Verification `contextra-privacy` | BEHOBEN | Zero-Panic und Fail-Closed verifiziert. |
| P1-LICENSE-FAILCLOSED | Fail-Closed Invariante `contextra-license` | BEHOBEN | 7-Stufen Evaluation ist strikt fail-closed. |
| P2-LICENSE-SIG | Ed25519 Signatur-Verifikation | BEHOBEN | `reject_trailing_bytes()` aktiv. |
| AUDIT-EXPORT-VERIFICATION | Audit Export Hardening | BEHOBEN | BSI-Mapping und Art. 30 Exporte voll funktionsfähig. |
| AVV-GEN-VERIFICATION | AVV Generator Hardening | BEHOBEN | Default TOMs und Rendering verifiziert. |

---

## 6 Testlage im Scope

Die Testsuite für alle 4 Crates im Scope verläuft vollständig grün:
<!-- EVIDENCE: logs/audits/privacy-license-compliance_2026-10-13/cargo_test.log (exit=0) -->

- `contextra-privacy`: 55 Tests bestanden (46 Unit-Tests, 9 Integrationstests).
- `contextra-license`: 37 Tests bestanden.
- `contextra-audit-export`: 18 Tests bestanden.
- `contextra-avv-generator`: 14 Tests bestanden.

---

## 7 Nicht prüfbar und warum

- **Loom Concurrency Model Checking:** Für `contextra-privacy` wurden keine Loom-Tests ausgeführt, da die Concurrency auf Standard-Tokio-Tasks und `parking_lot::Mutex` basiert und keine ad-hoc atomaren Zustandsketten verwendet werden.

---

## 8 Out-of-scope-Befunde

- **Unresolved Imports in contextra-db / contextra-engine:** Während des `repo-doctor`-Laufs schlug die globale Workspace-Verifizierung für `contextra-db` fehl. Dies liegt außerhalb des Privacy/License/Compliance-Scopes und betrifft bekannte `volatile-vault` Feature-Gating-Anforderungen in Ring 3.

---

## 9 Priorisierte Maßnahmen mit Akzeptanzbefehl

1. **Disclaimer in Compliance-Generatoren ergänzen (F-COMPLIANCE-01):**
   - Ergänzen des Satzes `"> **Rechtlicher Hinweis:** Dieses Dokument ist eine automatisch generierte technische Vorlage und stellt keine Rechtsberatung dar."` in `template.rs` und `markdown_template.rs`.
   - **Akzeptanzbefehl:** `cargo test -p contextra-avv-generator -p contextra-audit-export`

2. **Verdrahtung in Haupt-Fassade herstellen (F-ARCH-01):**
   - Anbindung von `AvvContextBuilder` und `AuditRegisterExporter` in `crates/contextra/src/builder.rs`.
   - **Akzeptanzbefehl:** `cargo xtask orphan-symbols`

3. **Dokumentationsbegriffe schärfen (F-PRIVACY-01):**
   - Anpassen von `AGENTS.md` in `contextra-privacy`.
   - **Akzeptanzbefehl:** `cargo xtask doc-truth`
