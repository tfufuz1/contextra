# OPEN_ISSUES (WP-12 / P30)

## Externe Verwendungen von archivierten Beweis-Symbolen in Fremd-Crates [V]

Gemäß Besitzprinzip (Regel 2) und Zwei-Phasen-API-Regel (Regel 4) dürfen Dateien außerhalb von BESITZ (`crates/contextra-crypto/`) nicht modifiziert werden.

Folgende Alttests in fremden Crates referenzieren noch archivierte Beweis-Symbole aus `contextra-crypto`:
1. `crates/contextra-store/tests/campaign_j42_audit.rs`:
   - Referenziert `DeletionProofKeyPair` aus `contextra_crypto`.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`. In Phase 4 (P40) auf `contextra-erasure` oder lokalen Mock umstellen.
2. `crates/contextra-db/tests/deletion_proof_integration.rs` & `e2e_compliance_scenario.rs`:
   - Referenzieren `contextra_crypto::deletion_proof::*`.
   - `contextra-db` gehört nicht zu BESITZ.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`.
3. `crates/contextra-engine/tests/erasure_report_honesty.rs` & `proof_verify_not_constant.rs`:
   - Referenzieren `contextra_crypto::deletion_proof::*`.
   - `contextra-engine` gehört nicht zu BESITZ.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`.

## CI- und Workspace-Befunde außerhalb von BESITZ [V]
- **Cargo Deny:** `deny.toml` meldet bekannte Advisories in `lopdf` (contextra-ingest), `number_prefix` (indicatif/candle), `tokio`, und `zip` (candle-core, contextra-ingest). Da diese Abhängigkeiten in fremden Crates liegen, liegen sie außerhalb von BESITZ und dürfen nicht angefasst werden.
- **Cargo xtask:** In CI schlägt der Schritt `cargo xtask shell-commit-audit` fehl, da das Crate `xtask` im Workspace nicht existiert (dokumentiert in `BUILD_LADDER.md` und Memory: `xtask existiert nicht`).
- **Format Check (cargo fmt):** Meldet Formatierungsabweichungen in `crates/contextra-store/` und `crates/contextra-vector/`. Da beide Crates außerhalb von BESITZ liegen, sind sie read-only.
- **Unwrap Baseline Gate:** Die Baseline in `.github/unwrap_baseline.txt` steht auf 1197, während die Basis vor P30 bereits 1333 `unwrap`/`expect` enthielt. `crates/contextra-crypto/src/` hat keine neuen `unwrap()` oder `expect()` eingeführt (durch die Archivierung wurden unwrap-Stellen in crypto sogar reduziert von 1333 auf 1318). Da `.github/` außerhalb von BESITZ liegt, darf die Baseline nicht geändert werden.
- **Compile Check / proof-tests / loom:** Schlagen in `contextra-store` fehl (duplizierte Methoden in `block_cache.rs`, `key_source`/`clock`-Felder in `recovery.rs`, Move aus `Drop` in `builder.rs`, `ScanTarget::Key` / `found()` in `scanner/mod.rs`), sowie in `contextra-graph` (Namenskollision `contract`) und `contextra-vector`. Diese Fehler liegen außerhalb von BESITZ.
