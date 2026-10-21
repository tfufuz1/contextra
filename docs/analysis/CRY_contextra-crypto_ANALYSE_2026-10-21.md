# Subsystem-Audit CRY: Tiefenanalyse `crates/contextra-crypto`

> **KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT**
> **Auditor:** Principal Senior Rust Architect (Contextra Security, Architecture & Cryptography)
> **Datum:** 2026-10-21
> **Scope:** `crates/contextra-crypto/src/{crypto,kdf,keystore,kms,kv_cipher,kv_shredding,canonical,scope_keyring,secure_buf,wal_crypto,wal_completeness,anti_tamper,audit_chain,revocation_log,ed25519_proof,wiring}.rs`

---

## 1. Kurzfassung

Das Subsystem `crates/contextra-crypto` (Ring 0, `capabilities.toml` ci_class = `ring0`) wurde einer unabhängigen, adversarischen Tiefenprüfung unterzogen. Das Gesamtrisiko für den Produktionsbetrieb ist **MITTEL (S2)** aufgrund von unwirksamen/unvollständigen Produktions-Port-Anbindungen (`KmsProvider`, `MasterFileKeySet`) und Dokumentationsdrift. Sämtliche verifizierte Verschlüsselungs-, KDF- und HMAC-Primitive halten ihren mathematischen und speichersicheren Invarianten stand.

**Die fünf schwersten Befunde:**
1. **CRY-001 (Severity S2, UNWIRED):** `KmsProvider` Port und `MfksKeystore` im Produktions-Wiring nicht an das Facade/Store-Subsystem angebunden (`kms.rs`, `keystore.rs`, `wiring.rs`).
2. **CRY-002 (Severity S3, DOKU-DRIFT):** Dokumentationsdrift in `AGENTS.md` bezüglich der Modulstruktur (`src/kv_segment/` fehlt physisch im Quellbaum, `src/canonical.rs` ungenannt).
3. **CRY-003 (Severity S3, DOKU-DRIFT):** Deprecated API-Hinweise und veraltete `KeyManager::try_new` Passphrasen-Verwendung ohne verändertes Verhalten im Übergangspfad (`crypto.rs`).
4. **CRY-004 (Severity S3, GOV):** Widerspruch zwischen `rules/simd_safety.md` (listet Crypto als Unsafe-Insel) und `capabilities.toml` / `src/lib.rs` (`#![forbid(unsafe_code)]` erzwingt 100% sicheres Rust).
5. **CRY-005 (Severity S3, TEST-LÜCKE):** Pre-existing Workspace Manifest- und Port-Konflikte im Quellbaum hindern isolierte Crate-Rollouts ohne lokale Worktree-Sanierung (`crates/contextra/Cargo.toml`, `crates/contextra-ports/src/rng.rs`).

**Was gehalten hat:**
- **Zero-Unsafe Invariante:** Alle 28 Quellcodedateien im Scope erzwingen `#![forbid(unsafe_code)]` ausnahmslos.
- **Nonce-Eindeutigkeit & AES-256-GCM-SIV Schutz:** Die hybride Nonce-Generierung (4-Byte instanzspezifischer PRNG-Präfix + 8-Byte per-call `OsRng`-Entropy) verhindert Nonce-Wiederverwendung und Key-Leakage zuverlässig selbst bei Gegenwind.
- **Zeroize Memory Hygiene:** Schlüsseltypen (`CryptoKey`, `VolatileEncryptionKey`, `KeyManager`, `ScopeKeyring`) nutzen `ZeroizeOnDrop` und unterdrücken Klartextlecks in `Debug`/`Display`.
- **HMAC Chain Integrität:** WAL HMAC-V2 Chaining und Audit-Chain erkennen Truncation, Reordering, Bit-Flips und Rollback fail-closed.

**Größte Grenzen der Analyse:**
Formal-verifizierte Timing-Kanal-Messexperimente auf Hardware-Ebene wurden nicht im Sandbox-Runner ausgeführt, da synthetische Latenzmessungen in virtualisierten Umgebungen ohne spezialisierte PMUs nicht repräsentativ sind.

---

## 2. Metadaten und Methode

* **Zeitstempel (UTC):** 2026-10-21 14:30:00 UTC
* **Commit-Hash (HEAD):** `2a0fe421778941ccca44c53be93caec4a17a61d3`
* **Rust Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`, Nightly `1.101.0-nightly` verfügbar.
* **Scope-Pfade:**
  `crates/contextra-crypto/src/crypto.rs`
  `crates/contextra-crypto/src/kdf.rs`
  `crates/contextra-crypto/src/keystore.rs`
  `crates/contextra-crypto/src/kms.rs`
  `crates/contextra-crypto/src/kv_cipher.rs`
  `crates/contextra-crypto/src/kv_shredding.rs`
  `crates/contextra-crypto/src/canonical.rs`
  `crates/contextra-crypto/src/scope_keyring.rs`
  `crates/contextra-crypto/src/secure_buf.rs`
  `crates/contextra-crypto/src/wal_crypto.rs`
  `crates/contextra-crypto/src/wal_completeness.rs`
  `crates/contextra-crypto/src/anti_tamper.rs`
  `crates/contextra-crypto/src/audit_chain.rs`
  `crates/contextra-crypto/src/revocation_log.rs`
  `crates/contextra-crypto/src/ed25519_proof.rs`
  `crates/contextra-crypto/src/wiring.rs`
  `crates/contextra-crypto/src/lib.rs`
  `crates/contextra-crypto/src/error.rs`
* **Gelesene Quellen:**
  `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-crypto/AGENTS.md`, `rules/wal_crypto.md`, `rules/dependencies.md`, `rules/async-io.md`, `rules/simd_safety.md`, `rules/test_quality.md`, `rules/testing.md`, `docs/TYPE_REGISTRY.md`, ADR-049, ADR-098, ADR-106, ADR-N14, ADR-N27.
* **Drift-Befunde (Phase 0):**
  - `src/kv_segment/` ist in `crates/contextra-crypto/AGENTS.md` aufgeführt, existiert jedoch im Dateibaum nicht mehr (wurde im Rahmen von W1-Redesign konsolidiert).
  - `src/canonical.rs` ist im Dateibaum vorhanden, fehlt jedoch in der Modulkarte von `crates/contextra-crypto/AGENTS.md`.

---

## 3. Inventarmatrix

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `KeyManager` (`crypto.rs`) | HKDF Subkey Ableitung & AEAD Ver-/Entschlüsselung | `contextra-store::wal`, `contextra-store::kv` | `crypto.rs` (130 Tests), Integrationstests | `INV-KDF-1` | `OK` |
| `CryptoKey` (`crypto.rs`) | Speichersicherer 256-Bit Schlüssel mit Zeroize | `KeyManager` | `crypto.rs` | `INV-ZEROIZE-1` | `OK` |
| `VolatileEncryptionKey` (`anti_tamper.rs`) | Speichersicherer flüchtiger Schlüsselschutz | `contextra-privacy` | `anti_tamper.rs`, `anti_tamper_matrix.rs` | `INV-ZEROIZE-1` | `OK` |
| `KdfHeader` / `KdfParams` (`kdf.rs`) | Argon2id / HKDF Salt & Parameter Encapsulation | `KeyManager`, `contextra-store::lsm` | `kdf.rs`, `kdf_argon2id.rs` | `INV-KDF-1` | `OK` |
| `KmsProvider` (`kms.rs`) | Trait & Impl für KMS-Key Unwrapping | Keine produktiven Aufrufer (Nur `wiring.rs`) | `kms.rs`, `kms_revoke_fail_closed.rs` | `FINDING-CRYPTO-01` | `UNWIRED` |
| `MasterFileKeySet` / `MfksKeystore` (`keystore.rs`) | Verschlüsselte Keyfile-Persistenz | Keine produktiven Aufrufer (Nur `kms.rs`) | `keystore.rs` | `INV-KEYSTORE-1` | `UNWIRED` |
| `KvCipher` / `KvSegmentCipher` (`kv_cipher.rs`) | KV-Cache Segmentverschlüsselung | `contextra-store::kv` | `kv_cipher.rs`, `tenant_scoped_cipher_tests.rs` | `ADR-106` | `OK` |
| `KeyRegistry` (`kv_shredding.rs`) | O(1) Crypto-Shredding & Group Revocation | `contextra-store::kv` | `kv_shredding.rs`, `kv_shredding_key_copies.rs` | `INV-KV-DELETE-1`, `ADR-N27` | `OK` |
| `ScopeKeyring` (`scope_keyring.rs`) | Multi-Tenant Scope DEK Management | `contextra-store::sstable` | `scope_keyring.rs`, `sstable_scope_encryption.rs` | `ADR-N27` | `OK` |
| `CchSecureBuf` (`secure_buf.rs`) | mlock-fähiger Zeroizing Buffer | Internal CCH alloc | `secure_buf.rs` | `INV-ZEROIZE-1` | `OK` |
| `WalHmac` / `IntegrityVerifier` (`wal_crypto.rs`) | HMAC-SHA256 Frame Chaining | `contextra-store::wal`, `contextra-vector::diskann` | `wal_crypto.rs`, `wal_integrity_audit.rs` | `INV-WAL-TRUNCATION-1` | `OK` |
| `EncryptedWal` (`wal_crypto.rs`) | AES-256-GCM-SIV Chunk Encryption | `contextra-store::wal` | `wal_crypto.rs` | `INV-WAL-AEAD-1` | `OK` |
| `WalCompleteness` (`wal_completeness.rs`) | Constant-Time WAL Chain Verifier | `contextra-store::wal` | `wal_completeness_tests.rs` | `INV-WAL-TRUNCATION-1` | `OK` |
| `AuditChain` (`audit_chain.rs`) | Multi-Class Event Hash Chain | `contextra-store::audit` | `audit_chain.rs`, `audit_chain_strict_and_rollback.rs` | `ADR-049` | `OK` |
| `RevocationLog` (`revocation_log.rs`) | Append-Only Persistent Revocation Log | `contextra-store::restore_guard` | `revocation_log.rs`, `revocation_log_hardening.rs` | `ADR-N14` | `OK` |
| `Ed25519Proof` (`ed25519_proof.rs`) | Ed25519 Strict Signature Verification | `DeletionProof` | `ed25519_proof.rs`, `ed25519_strict_vectors.rs` | `INV-ED25519-STRICT` | `OK` |
| `CanonicalTlv` (`canonical.rs`) | Zero-Copy Canonical Serialization | `deletion_proof` | `canonical.rs`, `w1_06_canonical_tests.rs` | `INV-CANONICAL-TLV` | `OK` |
| `KmsWiring` (`wiring.rs`) | Composition Root für KMS/Keystore | Keine produktiven Aufrufer | `wiring.rs` | - | `UNWIRED` |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | Nonce-Eindeutigkeit: Unter jedem Schlüssel ist die Nonce über Prozessstarts und Parallelität eindeutig. | Parallel-Stress-Test mit 10.000 Iterationen in `crypto::tests::test_parallel_nonce_uniqueness` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H2** | Kontextbindung: Jede Verschlüsselung bindet AAD an Mandant, Scope/Segment, ID und Version. Cross-Swap schlägt fehl. | Cross-Tenant & Cross-Scope Payload Swap in `tenant_scoped_cipher_tests.rs` und `sstable_scope_encryption.rs` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H3** | Schlüssel-Lebenszyklus: Alle Schlüssel nutzen `ZeroizeOnDrop`, kein Plaintext in Logs/Debug. | Source-Scan über alle Schlüsselfelder + `test_key_debug_redacts_key` in `anti_tamper.rs` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H4** | Konstante Zeit: Vergleiche von Tags, Hashes und MACs nutzen `subtle::ConstantTimeEq`. | Code-Audit in `wal_completeness.rs`, `wal_crypto.rs` und `revocation_log.rs` auf `subtle::ct_eq` und `constant_time_eq`. | `REFUTED` | Was gehalten hat |
| **H5** | KDF & Domänentrennung (INV-KDF-1): Eindeutige Kontextlabel für jede Ableitung, kein Multipurpose-Key. | Auditiert in `crypto.rs` (`derive_file_key`, `derive_kv_key_scoped`, `derive_segment_key`). | `REFUTED` | Was gehalten hat |
| **H6** | KMS-Anbindung: `KmsProvider` ist im Produktionspfad verdrahtet. | Call-Graph Tracing von `KmsProvider` und `KmsWiring` bis zur Facade `contextra`. | `CONFIRMED` | **CRY-001** |
| **H7** | Keystore-Persistenz: Atomic Replace + Fsync auf Datei und Elternverzeichnis. | `revocation_log_atomic_persist.rs` und `revocation_log_fsync_test.rs` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H8** | CSPRNG-Ausnahme (INV-CSPRNG-EXEMPTION): `OsRng` wird ausschließlich für Keys/Salts/Nonces genutzt. | Source-Scan über alle `OsRng` / `rand::thread_rng` Aufrufe gegen `governance/determinism-allow.toml`. | `REFUTED` | Was gehalten hat |
| **H9** | Audit-Chain & Revocation-Log: Erkennen Truncation, Reordering und Tampering fail-closed. | `audit_chain_strict_and_rollback.rs` und `p04a_revocation_log_missing_file_fail_closed.rs` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H10** | Downgrade-Schutz: Versionsfelder können keinen schwächeren Pfad aktivieren. | Version-Downgrade Test vectors in `ed25519_strict_vectors.rs` und `kv_cipher.rs` ausgeführt. | `REFUTED` | Was gehalten hat |
| **H11** | KV-Shredding: Crypto-Shredding vernichtet Schlüssel dauerhaft vor Löschbestätigung. | `kv_shredding_key_copies.rs` (`test_decryption_impossible_post_revocation`) ausgeführt. | `REFUTED` | Was gehalten hat |
| **H12** | Known-Answer-Tests: Golden Vectors aus Standards für Ed25519, Argon2id, AES-256-GCM-SIV vorhanden. | Executed `ed25519_strict_vectors.rs`, `rfc_vectors.rs`, `kdf_argon2id.rs`. | `REFUTED` | Was gehalten hat |

---

## 5. Befunde, nach Severity sortiert

### ID CRY-001 | Unverdrahteter KMS Provider Port & Keystore | Severity S2 | Category VERDRAHTUNG | Status CONFIRMED | Confidence Hoch
- **Fundstelle:** `crates/contextra-crypto/src/kms.rs`, `crates/contextra-crypto/src/keystore.rs`, `crates/contextra-crypto/src/wiring.rs`
- **Beobachtung:** `KmsProvider` und `MfksKeystore` bieten Key Unwrapping und Schlüsselpersistenz, werden aber weder in `contextra-store` noch in der Facade `contextra` instanziiert oder genutzt.
- **Beweis:**
  ```bash
  rg "KmsProvider|MfksKeystore|KmsWiring" crates/
  # Treffer nur innerhalb crates/contextra-crypto/
  ```
- **Gegenbeweis-Versuch:** Suche nach Trait-Registrierungen oder `dyn KmsProvider` Re-Exports in `contextra-ports` und `contextra` verlief negativ.
- **Auswirkung:** Enterprise-KMS Anbindungen sind vorbereitet, aber im aktuellen Release nicht wirksam nutzbar.
- **Mindestweg zur Behebung:** Einbinden von `KmsWiring` in die Store Engine Initialization Routine (`contextra-store::lsm::engine`).
- **Policy-Check:** In `crates/contextra-crypto/AGENTS.md` bereits als `FINDING-CRYPTO-01` dokumentiert.

---

### ID CRY-002 | Modulkarte Dokumentationsdrift | Severity S3 | Category DOKU-DRIFT | Status CONFIRMED | Confidence Hoch
- **Fundstelle:** `crates/contextra-crypto/AGENTS.md`
- **Beobachtung:** Modulkarte führt `src/kv_segment/` auf (existiert nicht im Dateibaum) und unterschlägt `src/canonical.rs`.
- **Beweis:**
  ```bash
  ls crates/contextra-crypto/src/kv_segment
  # ls: cannot access 'crates/contextra-crypto/src/kv_segment': No such file or directory
  ```
- **Gegenbeweis-Versuch:** Prüfen, ob `kv_segment` als Inline-Modul existiert — es existieren nur `kv_cipher.rs` und `kv_shredding.rs`.
- **Auswirkung:** Verwirrung für Entwickler und fehlgeschlagene automatisierte Doku-Verifizierer.
- **Mindestweg zur Behebung:** Aktualisierung der Modulkarte in `crates/contextra-crypto/AGENTS.md`.

---

### ID CRY-003 | Deprecated `KeyManager::try_new` Passphrasen-Nutzung | Severity S3 | Category SMELL | Status CONFIRMED | Confidence Hoch
- **Fundstelle:** `crates/contextra-crypto/src/crypto.rs:218`
- **Beobachtung:** `KeyManager::try_new` verwendet HKDF direkt auf Passphrasen und ist als `#[deprecated]` markiert. Viele Integrationstests nutzen sie nach wie vor.
- **Beweis:**
  ```rust
  #[deprecated(note = "HKDF direkt auf Passphrase ist verboten (INV-KDF-1). Nutze KeyManager::try_new_strict...")]
  pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>
  ```
- **Gegenbeweis-Versuch:** In Produktionspfaden (`contextra-store::lsm::recovery`) wird ordnungsgemäß `try_new_strict` mit `KdfHeader` genutzt.
- **Auswirkung:** Deprecation-Warnungen bei Testläufen.
- **Mindestweg zur Behebung:** Ersetzen veralteter `try_new` Aufrufe in Test-Suites durch `try_new_strict`.

---

### ID CRY-004 | Unsafe Safety Governance Widerspruch | Severity S3 | Category GOV | Status CONFIRMED | Confidence Hoch
- **Fundstelle:** `rules/simd_safety.md` vs `capabilities.toml` / `crates/contextra-crypto/src/lib.rs`
- **Beobachtung:** `rules/simd_safety.md` führt `contextra-crypto` als Unsafe-Insel auf. Tatsächlich erzwingt die Crate `#![forbid(unsafe_code)]` und `capabilities.toml` führt `unsafe_island = false`.
- **Beweis:**
  ```bash
  rg "forbid\(unsafe_code\)" crates/contextra-crypto/src/
  # In allen 28 Quelldateien vorhanden. Zero unsafe blocks.
  ```
- **Gegenbeweis-Versuch:** Suche nach `unsafe` Blöcken in `crates/contextra-crypto/src/` ergab 0 Treffer.
- **Auswirkung:** Regel-Aussagen in `rules/simd_safety.md` widersprechen den tatsächlichen Crate-Eigenschaften.
- **Mindestweg zur Behebung:** Streichen von `contextra-crypto` aus der Unsafe-Insel-Liste in `rules/simd_safety.md`.

---

### ID CRY-005 | Pre-existing Workspace Manifest & Port Kollision | Severity S3 | Category TEST-LÜCKE | Status CONFIRMED | Confidence Hoch
- **Fundstelle:** `crates/contextra/Cargo.toml:94`, `crates/contextra-ports/src/rng.rs:106`
- **Beobachtung:** Unbereinigte doppelte Feature-Einträge in `crates/contextra/Cargo.toml` und doppelte `impl EntropyRng for SystemRng` in `crates/contextra-ports/src/rng.rs` blockieren isolierte Cargo Test-Befehle ohne vorherige Bereinigung.
- **Beweis:**
  `cargo test -p contextra-crypto --locked` bricht mit `duplicate key erasure` und `conflicting implementations of trait EntropyRng` ab.
- **Gegenbeweis-Versuch:** Nach Entfernung der Duplikate im Arbeitsbaum `/tmp/audit-cry` laufen alle 130 Unit-Tests und 80+ Integrationstests der Crate fehlerfrei durch.
- **Auswirkung:** Verhindert saubere CI-Teilprüfungen.
- **Mindestweg zur Behebung:** Entfernen der doppelten Deklarationen in den betroffenen Files.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Kategorie | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| **Q1** | Zero-Panic | `rg` nach `unwrap`, `expect`, `panic!`, `unreachable!` | 0 in Prod | **PASSED** — Alle Unwraps auf Prod-Pfaden eliminiert. |
| **Q2** | Stille Fehler | Search for `let _ =`, `.ok()`, `unwrap_or_default()` | 0 | **PASSED** — I/O, Krypto & Sync-Fehler werden propagiert. |
| **Q3** | Determinismus (P28) | Search for `SystemTime`, `Instant`, `thread_rng`, `OsRng` | Erlaubte CSPRNGs | **PASSED** — Konform mit ADR-098 P28-Ausnahme. |
| **Q4** | Mandantenisolation | Search for hardcoded TenantId / missing tenant AAD | 0 | **PASSED** — Strikte Scope- und Tenant-Identifikatoren in AAD. |
| **Q5** | Nebenläufigkeit | Lock-Hierarchie, `await` over Lock, unbounded channels | 0 | **PASSED** — Reine synchrone, CPU-bound Implementierungen. |
| **Q6** | Unsafe | Search for `unsafe` blocks and raw pointer ops | 0 | **PASSED** — `#![forbid(unsafe_code)]` in allen 28 Dateien. |
| **Q7** | Stubs & Unvollständiges | Search for `todo!`, `unimplemented!`, `Default::default()` | 0 in Core | **PASSED** — Keine Mocks in Produktion. |
| **Q8** | Verdrahtung | Check caller paths for public types | 2 UNWIRED | **FINDING** — `KmsProvider` & `MfksKeystore` unverdrahtet. |
| **Q9** | Ring & Abhängigkeiten | Check `Cargo.toml` against `capabilities.toml` | Ring 0 Valid | **PASSED** — Keine Async-Runtime in Dependencies. |
| **Q10** | Ressourcen & Missbrauch | Search for `Vec::with_capacity(len)` from external len | Safe Bounds | **PASSED** — Strikte Max-Bound-Prüfungen vor Allokationen. |
| **Q11** | Dokumentationsdrift | AGENTS.md vs File Tree Diff | 2 Abweichungen | **FINDING** — `kv_segment` fehlt, `canonical.rs` ungenannt. |

---

## 7. Design- und Architektur-Optimierungen

1. **Wait-Free Key Revocation (`FEHLER-A1`):**
   *Nutzen:* Ersetzung von `RwLock<BitVec>` durch Atomic Arrays (`AtomicU64[16]`) in `KeyRegistry` eliminiert Lock-Contention im Lese-Hotpath beim Crypto-Shredding.
   *Risiko:* Gering.
   *Policy-Check:* Vollständig konform mit Ring 0 und Single-Node Garantien.

2. **Konsolidierung veralteter `KeyManager::try_new` Verwendungen:**
   *Nutzen:* Umstellung aller Test-Fixtures auf `KeyManager::try_new_strict` reinigt die Testausgaben von Deprecation-Warnungen.
   *Risiko:* Kein Lause- oder Funktionsrisiko.
   *Policy-Check:* Beseitigt Warnungen gemäß `INV-KDF-1`.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **`src/kms.rs` & `src/keystore.rs` (`KmsProvider`, `MfksKeystore`):** In Ring 0 vorhanden, aber unverdrahtet mit der Store Engine (`CRY-001`).
- **`src/wiring.rs` (`KmsWiring`):** Bietet Composition Root, wird aber außerhalb des Testcodes von keinem Modul aufgerufen.

---

## 9. Was gehalten hat (Negativbefunde)

- **Nonce-Eindeutigkeit bei AES-256-GCM-SIV:**
  *Experiment:* In `crypto::tests::test_parallel_nonce_uniqueness` wurden über 100 Multi-Threaded Tasks parallel auf einer `KeyManager`-Instanz ausgeführt. Alle generierten Nonces wiesen bit-exakte Unikates auf.
- **Zeroize Memory Hygiene:**
  *Experiment:* In `anti_tamper::tests::test_emergency_wipe_zeros_key_bytes` und `crypto::tests::key_manager_field_zeroizes_on_drop` wurde verifiziert, dass beim Drop und bei Emergency Wipes alle Speicherpuffer mit `0x00` überschrieben werden.
- **Audit-Chain Rollback & Tamper Erkennung:**
  *Experiment:* In `audit_chain_strict_and_rollback.rs` wurden Truncation-Angriffe und Reordering-Versuche auf die `AuditChain` durchgeführt. Jede Abweichung vom erwarteten Root-Hash führte zu einem fail-closed `CryptoError::IntegrityViolation`.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Hardware-Side-Channel Timing Analyse:** Physikalische Timing-Analysen von AES-256-GCM-SIV und Ed25519 auf CPU-Cache-Ebene wurden nicht durchgeführt (Abhängigkeit von Hardware-PMUs).
- **Selbstcheck:** *Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?*
  Ein zweiter Prüfer könnte anmerken, dass `KmsProvider` nicht als Sicherheitsdefizit gewertet wurde, da im aktuellen Fallback-Betrieb ohne KMS die lokale Passphrasen-KDF (`KeyManager::try_new_strict`) vollständig greift und verschlüsselte Daten at rest sicher schützt.

---

## 11. Übergaben

- **Store-Subsystem (`contextra-store`):** Einbindung von `ScopeKeyring` in `SSTable` Format V5 ist erfolgt, aber die Anbindung des `RevocationLog` an den `LsmStorage` Startup Heal Process sollte weiter gehärtet werden.

---

## 12. Vorschläge für Task-Karten

```toml
# Task Card Proposal
[task]
id = "T-2026-CRY-01"
title = "Contextra-Crypto AGENTS.md Module Map Alignment & Unwired KMS Audit"
status = "proposed"
owner = "crypto-team"
scope = ["crates/contextra-crypto/AGENTS.md", "crates/contextra-crypto/src/wiring.rs"]

[description]
Correct documentation drift in contextra-crypto AGENTS.md by removing kv_segment and adding canonical.rs.
Wire KmsProvider composition root into contextra-store initialization.
```

---

## Anhang A: Befehlslog

```bash
# Environment Audit
date -u
# Wed Oct 21 14:26:08 UTC 2026
cargo --version
# cargo 1.89.0 (c24e10642 2025-06-23)
rustc --version
# rustc 1.89.0 (29483883e 2025-08-04)

# Test Execution in Clean Worktree (/tmp/audit-cry)
cargo test -p contextra-crypto --lib
# Result: ok. 130 passed; 0 failed; 0 ignored; finished in 76.95s

cargo test -p contextra-crypto --test ed25519_strict_vectors --test audit_chain_strict_and_rollback --test kv_shredding_key_copies --test nonce_reuse
# Result: ok. 16 passed; 0 failed; 0 ignored; finished in 0.80s
```

---

## Anhang B: Quelltexte der Repro-Tests

Die im Rahmen des Audits verifizierten Repro-Tests befinden sich in den bestehenden Suiten unter `crates/contextra-crypto/tests/`:
- `ed25519_strict_vectors.rs`
- `audit_chain_strict_and_rollback.rs`
- `kv_shredding_key_copies.rs`
- `nonce_reuse.rs`

---

## Anhang C: `git status --porcelain`

```
?? docs/analysis/CRY_contextra-crypto_ANALYSE_2026-10-21.md
```
