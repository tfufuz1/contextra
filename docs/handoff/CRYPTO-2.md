# Übergabebericht CRYPTO-2

## 1. Übersicht & Auftrag
Im Rahmen von **CRYPTO-2** wurden die domänenfremden und löschbezogenen Komponenten aus `contextra-crypto` analysiert und das strukturierte Übergabepaket für `ERASE-2` geschnürt:
- `deletion_proof/` (Attestationszertifikate, Beweiserzeugung, Signatur, Verifikation, Hash-Chains, Key-Derivation)
- `ed25519_proof.rs` (Ed25519 Löschbeweise und Auditoren-Signaturen)
- `kv_shredding.rs` (Crypto-Shredding Engine für KV-Segmente)
- `revocation_log.rs` (VFS-gestütztes manipulationssicheres Revokationslog)

## 2. Paritätsanalyse & Übergabepaket
- **Paritätsbericht:** Vollständig dokumentiert in [`docs/handoff/CRYPTO-2/parity.md`](parity.md).
- **Übergabepaket:** Sämtliche Module sowie alle zugehörigen Tests liegen verbatim und regressionssicher unter [`docs/handoff/CRYPTO-2/`](.) bereit:
  - `deletion_proof/` (inkl. 6 dedizierter Test-Suiten)
  - `ed25519_proof/` (inkl. externer Audit-Verifikationstests)
  - `kv_shredding/`
  - `revocation_log/` (inkl. VFS-Fault-Injection-Tests)
- `ERASE-2` kann diese Module direkt in `contextra-erasure` übernehmen.

## 3. Modulzuordnungen (ADR Proposal)
Für die Module `kv_segment`, `audit_chain` und `wal_completeness` wurde die architektonische Entscheidung in [`docs/decisions/ADR-PROPOSAL-crypto-kv-segment.md`](../decisions/ADR-PROPOSAL-crypto-kv-segment.md) ausgearbeitet:
- `kv_segment` migriert nach `contextra-kvcache` (Klasse B/C KV-Cache).
- Generische Hash-Ketten (`audit_chain`) wandern nach `contextra-crypto::anti_tamper`.
- `wal_completeness` wandert nach `contextra-store`.

## 4. Parallelsicherheit & Konvergenz
- Gemäß Scope Guard (§5) sind `Cargo.toml`, `src/lib.rs` und `src/error.rs` Eigentum von `CRYPTO-1`.
- Workspace-Konsumenten (`contextra-engine`) nutzen `contextra_crypto::deletion_proof` noch aktiv.
- Sobald `ERASE-2` und `ENG-2` die Migration auf `contextra-erasure` abgeschlossen haben, können die Module in `contextra-crypto` durch `CRYPTO-1` im Konvergenzlauf endgültig entfernt bzw. hinter ein Übergangsfeature gestellt werden, ohne dass CI durch vorzeitige `-D warnings` (Deprecations) oder gelöschte Typen bricht.

## 5. Gefundene Probleme außerhalb des Umfangs
- **Manifest-Syntaxfehler in `crates/contextra-vector/Cargo.toml`:**
  In `origin/main` (Commit `4aaf0e6bd` / Merge PR #5843) deklariert `crates/contextra-vector/Cargo.toml` das Feature `ports-compat = ["dep:contextra-ports", ...]`, während `contextra-ports` als reguläre Abhängigkeit (`workspace = true`) ohne `optional = true` definiert ist. Dies blockiert `cargo metadata` workspace-weit für alle Cargo-Befehle, die den gesamten Workspace parsen.
