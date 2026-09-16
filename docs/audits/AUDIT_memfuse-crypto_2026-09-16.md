# Audit & Review Report `memfuse-crypto` (Package: `memfuse-security`)

**Datum:** 2026-09-16
**Auditor / Reviewer:** Senior Rust Applied-Cryptography Engineer (Jules Agent Session `5366d74e`)
**Scope:** `crates/memfuse-crypto/src/` (alle 12 Quellcodedateien)
**Crate-Einstufung:** Layer 1 — Tier 1 (Encryption-at-Rest, HKDF Key Derivation, AES-256-GCM-SIV, HMAC-SHA256, Zeroize, Egress Security)
**Status:** COMPLETE (REVIEW & VERIFICATION)

---

## VERDICT
VERDICT: CONDITIONAL
<!-- VERIFIED-BY-SESSION: 5366d74e (TS: 2026-09-16T16:25:00Z) -->

---

## 1. Inventar-Realitätsabgleich (Schritt 0)

Der Abgleich des tatsächlichen Dateibaums unter `crates/memfuse-crypto/src/` ergab folgende Inventar-Drift gegenüber dem Prompter-Snapshot vom 2026-09-13:

- **Im Repo vorhanden (12 `.rs` Dateien):**
  - `crates/memfuse-crypto/src/anti_tamper.rs`
  - `crates/memfuse-crypto/src/crypto.rs`
  - `crates/memfuse-crypto/src/deletion_proof.rs`
  - `crates/memfuse-crypto/src/egress_vault.rs` (Gefehlt im Prompter-Snapshot)
  - `crates/memfuse-crypto/src/error.rs`
  - `crates/memfuse-crypto/src/kv_cipher.rs`
  - `crates/memfuse-crypto/src/kv_segment/eviction_worker.rs`
  - `crates/memfuse-crypto/src/kv_segment/mod.rs` (Gefehlt im Prompter-Snapshot)
  - `crates/memfuse-crypto/src/kv_segment/segment.rs` (Gefehlt im Prompter-Snapshot)
  - `crates/memfuse-crypto/src/kv_segment/store.rs`
  - `crates/memfuse-crypto/src/lib.rs`
  - `crates/memfuse-crypto/src/wal_crypto.rs`

- **Ergebnis:** Inventar-Drift dokumentiert: 3 Quellcodedateien (`egress_vault.rs`, `kv_segment/mod.rs`, `kv_segment/segment.rs`) existierten im Repo, fehlten aber in der Prompter-Aufzählung vom 2026-09-13.

---

## 2. Modul-Klassifikation & Verifikation

- `lib.rs`: Strict `#![deny(unsafe_code)]` im Produktivcode. Exportiert Krypto-Primitiven ordnungsgemäß.
- `crypto.rs`: `KeyManager` HKDF-SHA256 Key Derivation, AES-256-GCM-SIV mit 12-Byte-Nonces (`OsRng`).
- `wal_crypto.rs`: `WalHmac`, `IntegrityVerifier` (V1/V2/V3 HMAC-Chaining) und `EncryptedWal`.
- `deletion_proof.rs`: DSGVO Art. 17 `DeletionProof` mit HMAC-SHA256 Signatur Version 2 und `LayerCleanupProof`.
- `anti_tamper.rs`: `VolatileEncryptionKey` mit `ZeroizeOnDrop` und `subtle::ConstantTimeEq`.
- `egress_vault.rs`: Fail-closed Cloud-Egress-DLP Guard mit `RegexSet` und Tokio blocking task execution.
- `kv_cipher.rs`: `KvSegmentCipher` und `EncryptedKvLayer` für `(TenantId, ModelFingerprint)` Mandanten-Isolierung.
- `kv_segment/store.rs`: `TenantIsolatedKvStore` mit 32 RwLock-Shards und fair Round-Robin LRU Eviction.
- `kv_segment/segment.rs`: `KvSegment` mit `ZeroizeOnDrop` Tensor-Puffer.
- `kv_segment/eviction_worker.rs`: Dedicated OS-Thread Worker für Hot-Path LRU Eviction (verhindert Tokio-Executor Blocking beim Zeroizen).
- `error.rs`: `CryptoError` Standalone Enum.

---

## 3. Review & Gate-Stack Ergebnisse

### Test- & Gate-Ergebnisse
1. `cargo check --workspace` — **PASSED** (0 Fehler).
2. `cargo clippy -p memfuse-crypto --all-features -- -D warnings` — **PASSED** (0 Warnungen).
3. `cargo test -p memfuse-crypto --all-features` — **FAILED** (114/115 passed, 1 failed).

---

## 4. Befund-Analyse (Gegenbeweis / Testfall-Analyse)

### Failure: `egress_vault::tests::test_egress_guard_fail_closed_on_index_unavailable`

- **Fehlermeldung:**
  ```text
  thread 'egress_vault::tests::test_egress_guard_fail_closed_on_index_unavailable' panicked at crates/memfuse-crypto/src/egress_vault.rs:400:9:
  Fail-Closed-Verletzung: classify_layer1 gab bei Timeout nicht Block(ClassificationTimeout) zurück.
  Got: Block(PolicyDenied("Payload size exceeds limit: 2650000 bytes > 65536 limit"))
  ```
- **Ursache:**
  Der Test in `egress_vault.rs` erzeugt mit `"This is a benign payload without any sensitive data. ".repeat(50_000)` ein Payload von 2.650.000 Bytes.
  In `classify_layer1_arc` (Zeile 86) wird die Payload-Länge jedoch **synchron vor** dem Task-Spawn und Timeout-Tick geprüft:
  ```rust
  if payload.len() > MAX_CLASSIFY_PAYLOAD_BYTES {
      return EgressClassification::Block(BlockReason::PolicyDenied(format!(
          "Payload size exceeds limit: {} bytes > {} limit",
          payload.len(),
          MAX_CLASSIFY_PAYLOAD_BYTES
      )));
  }
  ```
  Da 2.650.000 > 65.536 (`MAX_CLASSIFY_PAYLOAD_BYTES`), bricht die Funktion sofort mit `Block(PolicyDenied)` ab, noch bevor das 1-Nanosekunden-Timeout ausgewertet werden kann.
  Der Test erwartet jedoch explizit `Block(BlockReason::ClassificationTimeout)` und schlägt fehl.

- **Empfehlung für die Implementierungs-Session:**
  Reduziere in `test_egress_guard_fail_closed_on_index_unavailable` die Payload-Wiederholung so, dass die Payload-Länge unter `MAX_CLASSIFY_PAYLOAD_BYTES` (65.536 Bytes) bleibt (z.B. `.repeat(1_000)` = 53.000 Bytes). Dadurch passert der Längencheck und der 1-Nanosekunden-Timeout wird ordnungsgemäß ausgelöst und getestet.

---

## 5. Review-Status Summary

- **Vollständigkeit:** Crate ist funktional vollständig und gut strukturiert.
- **Sicherheit & Zeroize:** Ausgezeichnet. Strict `#![deny(unsafe_code)]` im Produktivcode, `ConstantTimeEq` für HMAC/Krypto-Vergleiche, Zeroize-on-Drop auf allen Key-Materialien.
- **Review Verdict:** **CONDITIONAL** (114/115 Tests passed; `test_egress_guard_fail_closed_on_index_unavailable` benötigt Anpassung der Test-Payload-Größe in der nachfolgenden Implementierungs-Session).
