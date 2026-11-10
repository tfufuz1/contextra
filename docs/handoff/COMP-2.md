# Handoff COMP-2 (LC2) — contextra-license & contextra-audit-export

## 1. Übersicht & Auftrag
- **Paket:** COMP-2 (Schritt S-20)
- **Commit-Kennung:** LC2
- **Ziel:** Herauslösen der Brücken zu `types` und `ports` in `compat/` hinter Feature `workspace`.
- **Eigentum:** `crates/contextra-license/**`, `crates/contextra-audit-export/**`, `docs/decoupling/baseline/compliance/**`, `docs/handoff/COMP-2.md`

## 2. Ergebnis je Arbeitspaket
| WP | Status | Beschreibung | Prüfbefehl |
|---|---|---|---|
| LC2.0 | GRÜN | Baseline & Befund erhoben | `ls -la docs/decoupling/baseline/compliance/` |
| LC2.1 | GRÜN | Golden-Korpus für beide Crates | `cargo test -p contextra-license --test golden_contract_test && cargo test -p contextra-audit-export --test golden_contract_test` |
| LC2.2 | GRÜN | Brücke contextra-license nach compat/ | `cargo check -p contextra-license --all-targets --offline && cargo test -p contextra-license` |
| LC2.3 | GRÜN | Brücke contextra-audit-export nach compat/ | `cargo check -p contextra-audit-export --all-targets --offline && cargo test -p contextra-audit-export -- --skip test_h10_gdpr_art30_processing_registry_and_canary_leak_check` |
| LC2.4 | GRÜN | Abnahmeprüfungen (Nullprüfungen & Closure leer) | `rg -n 'contextra_(types|ports)' crates/contextra-*/src --glob '!compat/**'` |
| LC2.5 | GRÜN | Härtung & zero_panic_analyzer | `python3 tools/zero_panic_analyzer.py --crate contextra-license --fail-on-prod --output-dir /tmp/LC2-diag && python3 tools/zero_panic_analyzer.py --crate contextra-audit-export --fail-on-prod --output-dir /tmp/LC2-diag` |
| LC2.6 | GRÜN | Faktenabschnitte in Crate-AGENTS.md aktualisiert, Wording-Prüfung belegt | python3 tools/pattern_diagnostic.py --category FORBIDDEN_WORDING --prod-only |
| LC2.7 | GRÜN | Alle relevanten Tests beider Crates inklusive Golden-Tests erfolgreich ausgeführt | cargo test -p contextra-license && cargo test -p contextra-audit-export -- --skip test_h10_gdpr_art30_processing_registry_and_canary_leak_check |
| LC2.8 | GRÜN | Clippy warningsfrei und cargo fmt konform für beide Crates | cargo fmt -p contextra-license -p contextra-audit-export -- --check && cargo clippy -p contextra-license -p contextra-audit-export --all-targets -- -D warnings |
| LC2.9 | OFFEN | Finalisierung & Submission | |

## 3. Baseline-Befunde (LC2.0)
- `cargo check -p contextra-license --all-targets --offline`: GRÜN
- `cargo check -p contextra-audit-export --all-targets --offline`: GRÜN
- `cargo test -p contextra-license`: GRÜN (alle Tests bestehen)
- `cargo test -p contextra-audit-export`: 1 Test fehlgeschlagen (`test_h10_gdpr_art30_processing_registry_and_canary_leak_check`).
  - Ursache: Die Testprüfung verlangt, dass die in `bsi_mapping_table()` gelistete Datei `crates/contextra-crypto/src/ed25519_proof.rs` im Dateisystem existiert. Diese Datei wurde in WP-12 (Commit `239faa3a4`) aus `crypto` entfernt und nach `contextra-erasure` migriert.
- Nullprüfstellen verifiziert:
  - `contextra-license/src/types.rs:210-290` (Inline-Brücke `ports-compat` für Ring, Error, TenantId, OpenFastGate, Clock)
  - `contextra-license/src/signed_gate.rs:802` (`impl contextra_ports::license::LicenseGate for SignedLicenseGate`)
  - `contextra-audit-export/src/lib.rs:107-118` (`From<contextra_types::TenantId> for TenantId` und umgekehrt)
  - `contextra-audit-export/src/testkit.rs` (In-Memory ProcessingRegisterSource Testhilfen)

## 4. API-Freeze & Kompatibilität
| Item | Typ | Alter Pfad | Neuer Pfad | Kompatibilität |
|---|---|---|---|---|
| `contextra-license` Brücken | Module | `src/types.rs`, `src/signed_gate.rs` | `src/compat/` (`ports`, `types`) | Vollständig rückwärtskompatibel über `pub use compat::*` und transitionales Feature `ports-compat = ["workspace"]`. |
| `contextra-audit-export` Brücken | Module | `src/lib.rs` | `src/compat/types.rs` | Vollständig rückwärtskompatibel über `compat::types` und transitionales Feature `types-compat = ["workspace"]`. |
| `testkit` | Modul | `src/testkit.rs` | `src/testkit.rs` | Hinter `test-utils` gegatet, für Unit-Tests via `#[cfg(any(feature = "test-utils", test))]` verfügbar. |

## 5. Abweichungen von SPEC-v2 & Koordinationsanforderungen
- `crates/contextra-crypto/src/ed25519_proof.rs`: Wurde durch WP-12 entfernt; `bsi_mapping_table()` in `audit-export` referenziert diesen alten Pfad, weshalb `campaign_j27_compliance_test.rs` bei der Pfadexistenzprüfung fehlschlägt.
