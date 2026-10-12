# AUDIT: KDF-Migration Status & Verbleibende Aufrufer-Inventarisierung

**Datum:** 2026-10-12
**Rolle:** Krypto-Architekt
**Betroffenes Modul:** `crates/contextra-crypto/src/crypto.rs` (`KeyManager::try_new`, `CryptoKey::try_new`)
**Status:** GEBLOCKT (Verbleibende Aufrufer im Workspace identifiziert per Regel 1)

---

## 1. Übersicht

Im Rahmen des KDF-Migrationsabschlusses (INV-KDF-1) soll die veraltete KDF-Funktion `KeyManager::try_new` bzw. ihr Alias `CryptoKey::try_new` (welche direkt HKDF-SHA256 auf Passphrasen anwendet) entfernt oder auf `pub(crate)` bzw. `private` beschränkt werden.

Gemäß den Abschlusskriterien (Regel 1: *"Prüfe per `rg`, dass kein Aufrufer mehr existiert. Bei Treffern: Stopp, Liste in die PR"*) wurde vor dem Entfernen eine workspace-weite Inventarisierung aller Aufrufstellen durchgeführt.

---

## 2. Befund: Identifizierte Aufrufstellen nach Crate / Modul

`rg -n "CryptoKey::try_new|KeyManager::try_new\(" crates` liefert Treffer in folgenden Modulen und Tests:

### 2.1 `crates/contextra-crypto/` (Core Crypto)
- `src/kv_segment/segment.rs`: Zeilen 277, 310, 337
- `src/wal_crypto.rs`: Zeilen 466, 480, 617, 631, 647, 656, 664, 679, 690, 708, 719
- `src/kv_shredding.rs`: Zeilen 642, 686, 750
- `src/revocation_log.rs`: Zeile 752
- `src/kv_cipher.rs`: Zeilen 241, 258, 279, 300, 322, 347, 388, 415, 436
- `tests/key_separation_and_edge_cases.rs`: Zeilen 23, 39, 50, 59, 71, 72, 84, 97, 112, 134, 135, 156, 171, 182
- `tests/kv_segment_integration.rs`: Zeile 14
- `tests/nonce_stress.rs`: Zeilen 20, 83
- `tests/proptests.rs`: Zeilen 22, 34, 53, 73, 92, 110
- `tests/audit_chain_tests.rs`: Zeile 114
- `tests/guarantee_deletion_irrecoverable.rs`: Zeilen 103, 208
- `tests/wal_integrity_audit.rs`: Zeile 178
- `tests/p06_persistent_shredding_durability_test.rs`: Zeilen 42, 84, 137, 211, 215, 224, 234
- `benches/crypto_benchmarks.rs`: Zeilen 12, 26, 46, 56

### 2.2 `crates/contextra-store/` (Storage Engine)
- `src/lsm/engine.rs`: Zeile 420
- `src/lsm/recovery.rs`: Zeile 199
- `src/wal/tests/replay_tests.rs`: Zeilen 369, 436, 509, 743, 1043, 1204
- `src/wal/tests/hmac_tests.rs`: Zeilen 182, 271
- `src/wal/tests/encode_tests.rs`: Zeile 126
- `src/wal/tests/flusher_tests.rs`: Zeile 351
- `src/wal/open_heal_tests.rs`: Zeile 119
- `tests/wal_frame_limits.rs`: Zeilen 12, 65, 154
- `tests/wal_replay_parity.rs`: Zeilen 111, 121, 145, 168
- `tests/sst_witness_reader_tests.rs`: Zeile 173
- `tests/deletion_proof_unregistered.rs`: Zeile 32
- `tests/kv_deletion_proof_registry.rs`: Zeile 16
- `tests/kv_crypto_shred_instant_unrecoverable.rs`: Zeile 10
- `tests/kv_delete_mode_wiring_test.rs`: Zeilen 14, 83
- `tests/j07closure-contextra-store_wiring_test.rs`: Zeile 38
- `tests/wal_tail_corruption_midfile.rs`: Zeile 14
- `tests/campaign_j21_hypotheses.rs`: Zeile 261
- `benches/wal_bench.rs`: Zeile 24

### 2.3 `crates/contextra-kvcache/` (KV Cache)
- `src/segment.rs`: Zeilen 46, 55, 514, 548, 571
- `tests/j06_contextra_kvcache_wiring.rs`: Zeile 155
- `tests/j06closure-contextra-kvcache_wiring.rs`: Zeile 27
- `tests/kivi_aead_order_enforced.rs`: Zeile 18

### 2.4 `crates/contextra-infer-candle/` (Inference Pipeline)
- `src/kv_bridge.rs`: Zeilen 344, 459, 472, 492, 532, 547, 589, 623, 655
- `src/inference/tests.rs`: Zeilen 176, 232, 341, 638
- `tests/kv_bridge_roundtrip.rs`: Zeile 14
- `tests/kv_bridge_tenant_key_separation.rs`: Zeile 14
- `tests/kv_bridge_version_mismatch.rs`: Zeile 14
- `tests/kv_bridge_and_stress_test.rs`: Zeilen 57, 82
- `benches/kv_bridge_bench.rs`: Zeile 17

### 2.5 `crates/contextra-mcp/` (MCP Protocol Server)
- `src/routing.rs`: Zeile 207
- `src/sandbox.rs`: Zeilen 362, 667, 678
- `src/tests.rs`: Zeile 1122

### 2.6 `crates/contextra-privacy/` (Privacy Shield)
- `tests/tenant_scoped_egress_isolation.rs`: Zeile 95

---

## 3. Handlungsempfehlungen für Folgetasks

1. **Aufrufer-Migration:**
   - In Produktion und Tests ist `KeyManager::try_new_strict` oder `KeyManager::try_new_with_kdf` mit explizitem `KdfHeader` (Argon2id) bzw. `KeyManager::try_new_argon2id` zu verwenden.
2. **Entfernung der deprecated KDF-Funktion:**
   - Sobald alle verbleibenden Aufrufstellen migriert sind, kann `KeyManager::try_new` in `crates/contextra-crypto/src/crypto.rs` ersatzlos entfernt oder privatisiert werden.
