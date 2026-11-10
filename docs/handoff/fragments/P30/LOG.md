# P30 LOG: Krypto-Entkopplung & Archivierung von Beweis-Modulen

## Baseline & Start-Check [V]
- Commit-Basis: `6f7454d14ef71570b6502b223aa0de51cdfbf948` (Merge pull request #5884)
- Rust Toolchain: `cargo 1.89.0`
- BUILD_LADDER Check: Vor Beginn der Arbeit baute `contextra-crypto` bereits mit einem Fehler (`unexpected cfg condition value: serde` in `src/secret.rs:94, 105`).
- Fremdfehler dokumentiert in `docs/handoff/fragments/P00/BUILD_LADDER.md`.

## Symbol-Verwendung außerhalb von `contextra-crypto` [V]

Ermittelt via `git grep -E "contextra_crypto::|use contextra_crypto" crates/ benchmarks/ examples/`:

| Symbol | Nutzer (Dateipfad) | Status / Bemerkung |
|---|---|---|
| `wal_crypto::WalHmac` | `crates/contextra-store/src/wal/encode.rs`, `io.rs`, `tests/hmac_tests.rs` | Verbleibt in `crypto` |
| `wal_crypto::IntegrityVerifier` | `crates/contextra-store/src/wal/io.rs`, `replay.rs` | Verbleibt in `crypto` |
| `wal_crypto::WalEntrySnapshot` | `crates/contextra-store/src/wal/io.rs`, `replay.rs` | Verbleibt in `crypto` |
| `crypto::KeyManager` | `crates/contextra-store/src/wal/mod.rs`, `tests.rs` | Verbleibt in `crypto` |
| `verify_wal_chain_completeness` | `crates/contextra-store/src/lsm/recovery.rs` | Verbleibt in `crypto` |
| `CryptoKey` (`KeyManager`) | `crates/contextra-mcp/src/sandbox.rs`, `contextra-kvcache/src/segment.rs`, `tests/`, `contextra-infer-candle/src/`, `tests/`, `benches/` | Verbleibt in `crypto` |
| `KvCipher` | `crates/contextra-kvcache/src/segment.rs`, `tests/kivi_aead_order_enforced.rs` | Verbleibt in `crypto` |
| `KvSegmentCipher` | `crates/contextra-kvcache/src/segment.rs`, `store.rs`, `service.rs`, `tests/`, `contextra-infer-candle/src/kv_bridge.rs`, `tests/`, `benches/` | Verbleibt in `crypto` |
| `EncryptedKvLayer` | `crates/contextra-kvcache/src/segment.rs`, `service.rs`, `contextra-infer-candle/src/kv_bridge.rs` | Verbleibt in `crypto` |
| `ModelFingerprint` | `crates/contextra-kvcache/src/segment.rs`, `store.rs`, `service.rs`, `tests/`, `contextra-infer-candle/src/kv_bridge.rs` | Re-export verbleibt in `crypto` |
| `TenantIsolatedKvStore` | `crates/contextra-infer-candle/src/kv_bridge.rs`, `tests/`, `benches/` | Verbleibt in `crypto` |
| `KvSegment` | `crates/contextra-infer-candle/src/kv_bridge.rs`, `tests/` | Verbleibt in `crypto` |
| `EvictionWorker` | `crates/contextra-infer-candle/src/kv_bridge.rs` | Verbleibt in `crypto` |
| `CryptoError` | `crates/contextra-kvcache/src/segment.rs`, `store.rs` | Verbleibt in `crypto` |
| `AuditChain` | `crates/contextra-mcp/tests/campaign_j42_mcp_audit.rs`, `crates/contextra-store/tests/campaign_j42_audit.rs` | Verbleibt in `crypto` |
| `DataClass` | `crates/contextra-store/tests/campaign_j42_audit.rs` | Verbleibt in `crypto` |
| `DeletionProofKeyPair` | `crates/contextra-store/tests/campaign_j42_audit.rs` | Alttest in fremdem Crate; dokumentiert in OPEN_ISSUES.md |
| `deletion_proof::*` | `crates/contextra-db/tests/deletion_proof_integration.rs`, `e2e_compliance_scenario.rs`, `crates/contextra-engine/tests/erasure_report_honesty.rs`, `proof_verify_not_constant.rs` | Alttests in fremden Crates (nicht in `src/`); dokumentiert in OPEN_ISSUES.md & PENDING_API_CHANGES.md |

## Durchgeführte Arbeiten (P30 / WP-12) [V]
1. **Archivierung via git mv**:
   - `crates/contextra-crypto/src/deletion_proof/` -> `archive/contextra-crypto-proofs/deletion_proof/`
   - `crates/contextra-crypto/src/ed25519_proof.rs` -> `archive/contextra-crypto-proofs/ed25519_proof.rs`
   - `crates/contextra-crypto/src/kv_shredding.rs` -> `archive/contextra-crypto-proofs/kv_shredding.rs`
   - `crates/contextra-crypto/src/revocation_log.rs` -> `archive/contextra-crypto-proofs/revocation_log.rs`
   - Benches & Fuzz & reine Proof-Tests -> `archive/tests-legacy/contextra-crypto/`
   - Mischtests (`campaign_j25.rs`, `campaign_mut_j17.rs`, `crypto_vectors_extended.rs`) sauber zerlegt und Ed25519/Shredding-Anteile archiviert.
2. **Entkopplung Quellcode**:
   - `src/secret.rs`: `#[cfg(feature = "serde")]` entfernt.
   - `src/kv_segment/`: `KvDeleteMode` und `KvSegmentManager` von `KeyRegistry` entkoppelt.
   - `src/kv_cipher.rs`: `KvSegmentCipher` von `KeyRegistry` entkoppelt, direkte Delegierung für Segmentverschlüsselung.
   - `src/audit_chain.rs`: `ed25519-dalek` und `KeyRegistry` entfernt, reine Blake3-Hashkette.
   - `src/crypto.rs` & `src/error.rs`: `derive_deletion_proof_key` und DeletionProof-Fehlertypen entfernt.
   - `src/lib.rs`: Re-Exporte und Moduldeklarationen bereinigt.
3. **Features & Defaults**:
   - `ed25519-dalek` und `legacy-erasure` vollständig aus `Cargo.toml` entfernt.
   - `default = ["kv-encryption"]` gesetzt.
4. **Prüfergebnisse**:
   - `cargo tree -p contextra-crypto -e normal | grep -c ed25519` → 0
   - `grep -rniE "deletion_proof|kv_shredding|revocation_log" crates/contextra-crypto/src` → 0
   - `cargo check -p contextra-crypto --no-default-features` → grün
   - `cargo check -p contextra-crypto --all-features` → grün
   - `cargo test -p contextra-crypto --release` → grün (108 Unit-Tests + alle 21 Integrationstest-Suiten bestanden)
   - `cargo check -p contextra-kvcache` → grün
