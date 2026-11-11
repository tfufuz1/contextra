# DECOUPLING REPORT: contextra-crypto (P2)

## 1. Baseline (V0)
- **V0a (`cargo check -p contextra-crypto --no-default-features`):** Baute vor dem Umbau, aber mit harten Abhängigkeiten auf `contextra-types` und `contextra-ports`.
- **V0b (`cargo check -p contextra-crypto --all-targets`):** Baute vor dem Umbau erfolgreich.
- **V0c (`cargo test -p contextra-crypto --lib`):** 111 Tests passed, 0 failed.
- **V0d (Clippy Baseline):** 146 Pre-existing Errors unter `-D warnings`.
- **V0e (Golden Corpus):** Bekannte Diskrepanz in `dek_wrapping.bin` Hash im Manifest (in `FINDINGS.md` notiert).

## 2. Durchgeführte Arbeitspakete und Commits
- **WP-1 (Commit `f4b9ab33e`):** `contextra-ids` und `contextra-env` zu `Cargo.toml` hinzugefügt; `contextra-types` und `contextra-ports` als optional hinter Feature `workspace` gelegt; `default = ["kv-encryption", "workspace"]`; `compat` in `lib.rs` hinter `#[cfg(feature = "workspace")]` gegatet.
- **WP-2 (Commit `7a443ccc7`):** Reiner Pfadtausch `TenantId`, `DocId`, `TxId`, `TenantScoped` von `contextra_types` nach `contextra_ids` in allen Kernmodulen (`store.rs`, `segment.rs`, `eviction_worker.rs`, `crypto.rs`, `kv_cipher.rs`, `audit_chain.rs`).
- **WP-3 (Commit `cfbf86aff`):** `CryptoError::Storage` von `contextra_types::ContextraError` auf `String` umgestellt; in `compat/error.rs` `impl From<ContextraError> for CryptoError` bereitgestellt; `KvCipher` Trait und Implementierungen im Kern auf lokales `crate::Result<Vec<u8>>` / `CryptoError` umgestellt; tabellengetriebenen Test erweitert.
- **WP-4 (Commit `30316cd89`):** Lokalen `ModelFingerprint` struct im Kern (`kv_cipher.rs`) implementiert mit exakt identischen Feldern und Serde-Attributen; bidirektionale `From`-Konvertierungen in `src/compat/model_fingerprint.rs` hinter `workspace` bereitgestellt.
- **WP-5 (Commit `e279e070a`):** Alle Kern-Doc-Kommentare und Tests bereinigt, sodass keine verbotenen Crate-Namen mehr vorkommen. Null-Prüfung V5 bestätigt: 0 Treffer.

## 3. Verifikationsergebnisse (V1 bis V7)

### V1: Kern-Kompilierung
```bash
cargo check -p contextra-crypto --no-default-features
```
**Ergebnis:** Grün (Erfolg).

### V2: Alle Targets und Feature-Kombinationen
```bash
cargo check -p contextra-crypto --all-targets
cargo check -p contextra-crypto --no-default-features --features workspace --all-targets
```
**Ergebnis:** Beide Befehle grün (Erfolg).

### V3: Unit- und Integrationstests
```bash
cargo test -p contextra-crypto --lib
```
**Ergebnis:** 111 passed; 0 failed; 0 ignored.
Lange/Stress-Tests (`test_1m_nonce_uniqueness_stress`, `test_nonce_uniqueness_smoke_1m`, `test_h3_nonce_uniqueness_200k_encryptions`) laufen im Release-Profil in < 1 Sekunde erfolgreich durch.

### V4: Clippy
```bash
cargo clippy -p contextra-crypto --all-targets -- -D warnings
```
**Ergebnis:** Es traten keine neuen Warnungen oder Fehler durch die Entkopplung auf (146 unveränderte Pre-existing Lints im Crate).

### V5: Null-Prüfung (muss leer sein)
```bash
grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-crypto/src --include=*.rs | grep -v '/compat/'
```
**Ergebnis:** Leer (0 Treffer).

### V6: Kern-Closure
```bash
cargo tree -p contextra-crypto -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u
```
**Ergebnis:**
- `contextra-crypto v0.1.0`
- `contextra-env v0.1.0`
- `contextra-ids v0.1.0`
Exakt die erlaubten Kern-Abhängigkeiten.

### V7: Workspace- und Konsumenten-Probe
- `contextra-infer-candle`: `cargo check -p contextra-infer-candle --all-targets` -> GRÜN.
- `contextra-kvcache`: `cargo check -p contextra-kvcache --all-targets` -> GRÜN.
- Upstream-Workspace: Enthält auf `origin/main` bekannte Compilerfehler in `contextra-store` (`sstable/builder.rs` unclosed delimiter) und `contextra-checkpoint` (fehlende Typen). Diese liegen außerhalb von `contextra-crypto` und wurden per Regel H1 nicht modifiziert.

## 4. Fassaden-Liste (T2-Fälle)
- `ModelFingerprint`: Nun als identischer lokaler Typ im Kern von `contextra-crypto` definiert. Konsumenten, die `contextra_crypto::ModelFingerprint` importieren, erhalten den Typen direkt. Unter `workspace` wird `impl From<contextra_types::ModelFingerprint> for ModelFingerprint` und `impl From<ModelFingerprint> for contextra_types::ModelFingerprint` in `compat/model_fingerprint.rs` bereitgestellt.
- `KvCipher`: Trait liefert nun `crate::Result<Vec<u8>>` mit lokalem `CryptoError`. Konsumenten mit `?` auf `ContextraError` kompilieren dank `From<CryptoError> for ContextraError` in `compat/error.rs` nahtlos weiter.
- `CryptoError::Storage`: Nutzt nun `Storage(String)` im Kern. `From<ContextraError> for CryptoError` wandelt in `Storage(e.to_string())` um.

## 5. Fremdnutzer-Inventar und Übergaben
- `crates/contextra-infer-candle/src/kv_bridge.rs`: Verwendet `KvSegmentCipher`, `KvSegment`, `TenantIsolatedKvStore`.
- `crates/contextra-kvcache/src/store.rs`: Verwendet `KvSegmentCipher`, `ModelFingerprint`, `CryptoError`.
- `crates/contextra-kvcache/src/segment.rs`: Verwendet `CryptoKey`, `CryptoError`, `EncryptedKvLayer`, `KvSegmentCipher`, `ModelFingerprint`, `KvCipher`.
- `crates/contextra-kvcache/src/service.rs`: Verwendet `EncryptedKvLayer`, `KvSegmentCipher`, `ModelFingerprint`.
- `crates/contextra-store/src/wal/encode.rs`, `replay.rs`, `io.rs`, `mod.rs`: Verwendet `WalHmac`, `IntegrityVerifier`, `KeyManager`.
- `crates/contextra-mcp/src/sandbox.rs`: Verwendet `CryptoKey`.

### Zukünftiger Default-Flip (Entfernen von `workspace` aus Defaults)
Folgende Crates müssten bei einem späteren Flip `workspace` explizit aktivieren, falls sie Features aus `compat/` (z. B. `From<ContextraError>` oder `contextra-types::ModelFingerprint`) benötigen:
- `crates/contextra-infer-candle/Cargo.toml`: `contextra-crypto = { ..., features = ["workspace"] }`
- `crates/contextra-kvcache/Cargo.toml`: `contextra-crypto = { ..., features = ["workspace"] }`

## 6. Offene Punkte und Risiken
- Keine offenen Punkte in `contextra-crypto`.
- Unveränderter Golden-Corpus-Befund `dek_wrapping.bin` bleibt in `FINDINGS.md` für zukünftige Wartung dokumentiert.
