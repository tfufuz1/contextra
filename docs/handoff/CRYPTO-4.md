# Handoff CRYPTO-4: contextra-crypto Härtung, Golden-Korpus, additive KV-Schlüssel, Fehlerabbildung

**Datum:** 2026-11-10
**Commit-Kennung:** CR4
**Branch:** `jules/crypto-4-harden`
**Rolle:** Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency)

---

## 1. Ergebnis je Arbeitspaket

| Arbeitspaket | Status | Prüfbefehl | Nachweis |
|---|---|---|---|
| **CR4.0 Baseline & Befund** | GRÜN | `cargo check -p contextra-crypto --all-targets --locked` | Baseline unter `docs/decoupling/baseline/crypto/` erfasst; `autobenches = false` in `Cargo.toml` gesetzt. |
| **CR4.1 Golden-Korpus** | GRÜN | `cargo test -p contextra-crypto --test golden_corpus` | Golden-Dateien unter `tests/golden/` samt `MANIFEST.sha256` und `regen.sh`; Prüftest und Rückwärts-Lesetest ohne Panics. |
| **CR4.2 Additive KV-Schlüsselfunktionen** | GRÜN | `cargo test -p contextra-crypto --lib tests_cr42_differential` | Neue Methoden `derive_kv_key_bytes`, `derive_kv_key_scoped_bytes`, `cipher_for_bytes`, `cipher_for_scoped_bytes` mit `[u8; 32]`. Bitgleicher Erhalt bestehender Methoden. |
| **CR4.3 Fehlerabbildung nach compat/** | GRÜN | `cargo test -p contextra-crypto --lib compat::error::tests` | Brückenmodul `src/compat/error.rs` implementiert `From<CryptoError> for ContextraError`. Abbildungstabelle in `docs/decoupling/errors/crypto.md`. |
| **CR4.4 Härtung** | GRÜN | `RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --release --test loom_kv_deferred_zeroize` | Zero-Panic: 0 Panics im Produktivcode. Golden Bitflip- und Truncation-Tests grün. Loom Concurrency-Tests grün. Unsafe-Auditor: 0 Unsafe. |
| **CR4.5 Dokumentation** | GRÜN | `python3 tools/pattern_diagnostic.py --crate contextra-crypto --category FORBIDDEN_WORDING --prod-only --no-save` | `crates/contextra-crypto/AGENTS.md` aktualisiert; 0 verbotene Zusicherungen. |

---

## 2. API-Freeze & Verbraucherinventar

Alle bestehenden öffentlichen Schnittstellen von `contextra-crypto` wurden quell- und binärkompatibel beibehalten.

| Crate | Benutzte Symbole | Status Quellkompatibilität |
|---|---|---|
| `contextra-store` | `KeyManager`, `WalHmac`, `IntegrityVerifier`, `WalEntrySnapshot`, `verify_wal_chain_completeness` | Vollständig kompatibel |
| `contextra-kvcache` | `CryptoKey`, `KeyManager`, `KvSegmentCipher`, `ModelFingerprint`, `KvCipher`, `EncryptedKvLayer`, `CryptoError` | Vollständig kompatibel |
| `contextra-infer-candle` | `CryptoKey`, `KvSegmentCipher`, `TenantIsolatedKvStore`, `KvSegment`, `EncryptedKvLayer`, `EvictionWorker` | Vollständig kompatibel |
| `contextra-mcp` | `AuditChain`, `CryptoKey` | Vollständig kompatibel |

---

## 3. Kern-API-Lücken & Additive Methoden

Zur Vorbereitung der vollständigen Entkopplung wurden folgende additive Methoden auf `KeyManager` eingeführt:
- `pub fn derive_kv_key_bytes(&self, tenant_id: TenantId, fingerprint: &[u8; 32]) -> Result<Self>`
- `pub fn derive_kv_key_scoped_bytes(&self, scoped_fingerprint: TenantScoped<&[u8; 32]>, expected_tenant_id: &TenantId) -> Result<Self>`
- `pub fn cipher_for_bytes(&self, tenant_id: TenantId, fingerprint: &[u8; 32]) -> Result<Self>`
- `pub fn cipher_for_scoped_bytes<T>(&self, scoped_key: TenantScoped<T>, expected_tenant_id: &TenantId, fingerprint: &[u8; 32]) -> Result<Self>`

Bestehende Signaturen (`derive_kv_key`, `derive_kv_key_scoped`, `cipher_for`, `cipher_for_scoped`) bleiben unverändert funktionsfähig.

---

## 4. Messwerte Vorher / Nachher

| Metrik | Vorher (CR4.0) | Nachher (CR4.5) | Differenz |
|---|---|---|---|
| Produktive LOC (`contextra-crypto`) | 3.541 | 3.655 | +114 LOC |
| Unsafe-Blöcke (`unsafe_boundary_auditor`) | 0 | 0 | 0 |
| Zero-Panic Prod (`zero_panic_analyzer`) | 0 | 0 | 0 |
| Verbotene Wörter (`pattern_diagnostic`) | 1 | 0 | -1 (Bereinigt) |
| Golden-Dateien | 0 | 6 Dateien + Manifest | +6 Vektoren |
| Unit- & Integrationstests (`cargo test --lib`) | 108 passed | 111 passed | +3 Tests |
| Loom Tests | 2 passed | 2 passed | Stabil grün |

---

## 5. Abweichungen von SPEC-v2

Keine signifikanten Abweichungen. Gemäß SPEC-v2 B10 / UNABHÄNGIGKEIT verbleibt der KV-Teil an Ort und Stelle, da er von `contextra-erasure`, `contextra-kvcache` und `contextra-infer-candle` benötigt wird; die Migration nach `kvcache` erfolgt im festgelegten Nachfolgepaket S-38.

---

## 6. Blockierte Punkte & Nicht-Ziele (Folgeschritte)

- **Umzug des KV-Teils nach `kvcache` (S-38):** Blockiert bis zur synchronisierten Crate-übergreifenden Migration mit `kvcache`/`infer-candle`.
- **E0/E1 Reduktion auf `ids`/`env`:** Zurückgestellt auf Folgeschritt Integration.


## 7. Koordinationsanforderungen (Fehler außerhalb Eigentum)

- **Datei:**  (Zeilen 283, 398, 494, 514)
- **Fehler:** Im CI-Workflow `Loom Concurrency Check — contextra-store` schlägt der Build fehl mit:
  `error[E0433]: failed to resolve: could not find `lsm` in the crate root`
  Ursache: In `crates/contextra-store/src/lib.rs:58-59` ist `pub mod lsm;` mit `#[cfg(not(loom))]` gegatet. `src/wal/hmac.rs` ruft jedoch bedingungslos `crate::lsm::recovery::next_temp_file_counter()` auf.
- **Exakte Änderung erforderlich (STORE-Besitzer):** `next_temp_file_counter()` entweder nach `util.rs` verschieben oder den Aufruf in `src/wal/hmac.rs` unter `#[cfg(loom)]` mit einem loom-kompatiblen Fallback versehen.
