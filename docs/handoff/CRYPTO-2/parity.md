# Paritätsbericht MISSION `CRYPTO-2`: Krypto- vs. Erasure-Module

**Datum:** 2026-11-08
**Scope:** `contextra-crypto` (`deletion_proof`, `ed25519_proof`, `kv_shredding`, `revocation_log`) vs. `contextra-erasure`
**Autor:** Principal Rust Engineer (Prompt CRYPTO-2)

---

## 1. Übersicht der Module

| Modul in `contextra-crypto` | Entsprechung in `contextra-erasure` | Status |
|---|---|---|
| `src/deletion_proof/` | `src/deletion_proof.rs` | Größtenteils äquivalent / v3+v5 in Erasure erweitert. Einige Test- und Verifikations-Helper nur in `crypto`. |
| `src/ed25519_proof.rs` | `src/ed25519_proof.rs` | Funktionell identisch; `erasure` nutzt `verify_strict` und generischen `CryptoRng`. |
| `src/kv_shredding.rs` | `src/kv_shredding.rs` | **Stark abweichend**: In `crypto` befindet sich die vollständige Envelope Encryption Engine (`KeyRegistry`, `GroupKek`, `RecordDek`, `encrypt_record`, `decrypt_record`, `encrypt_with_group`, `decrypt_with_group`). In `erasure` liegt eine deterministische BLAKE3-KDF-Variante. Vollständige Übergabe an `ERASE-2` erforderlich. |
| `src/revocation_log.rs` | `src/revocation_log.rs` | Weitgehend identisch; `crypto` nutzt `CryptoError` und `contextra_ports::{Clock, Vfs}`, `erasure` nutzt `ErasureError` und lokale Traits. Zusätzliche Tests in `crypto`. |

---

## 2. Detaillierte Funktions- und Typen-Parität

### 2.1 Modul `ed25519_proof`

| Funktion / Symbol | Sichtbarkeit | Status | Bemerkungen / Diff |
|---|---|---|---|
| `DeletionProofKeyPair::generate` | `pub` | Identisch | Erzeugt Schlüsselpaar via Standard-RNG (`thread_rng`). |
| `DeletionProofKeyPair::generate_with_rng` | `pub` | Leicht abweichend | `crypto`: `rng: &mut impl RngCore`<br>`erasure`: `rng: &mut R` mit `R: RngCore + CryptoRng`. |
| `DeletionProofKeyPair::from_signing_key` | `pub` | Identisch | Konstruktion aus `SigningKey`. |
| `DeletionProofKeyPair::verifying_key_bytes` | `pub` | Identisch | 32-Byte Public Key Repräsentation. |
| `DeletionProofKeyPair::signing_key` | `pub` | Identisch | Referenz auf internen `SigningKey`. |
| `DeletionProofKeyPair::verifying_key` | `pub` | Nur in `crypto` | Gibt Referenz auf `&VerifyingKey` zurück. In `erasure` nicht exportiert (nur `verifying_key_bytes()`). |
| `sign_deletion_proof_v3` | `pub` | Identisch | Signiert Byte-Payload mit Ed25519. |
| `verify_deletion_proof_v3` | `pub` | Leicht abweichend | `crypto`: `verifying_key.verify()`<br>`erasure`: `verifying_key.verify_strict()`. |
| `SignatureVersion` | `pub` | Identisch | Repräsentiert die Versionen V1, V2, V3, V5. |
| `SignatureVersion::as_u8` | `pub` | Identisch | Konvertiert Enum zu `u8`. |
| `u8::from(SignatureVersion)` | `priv` | Identisch | Standard From-Trait. |
| `SignatureVersion::try_from` | `priv` | Identisch | TryFrom für Validierung. |
| `hash_deleted_keys_length_prefixed` | `pub` | Identisch | In `crypto` re-exportiert aus `deletion_proof/types.rs`, in `erasure` direkt in `ed25519_proof.rs` definiert. |
| Unit-Tests (`test_keypair_generation_and_bytes`, `test_signature_version_conversions`, `test_v3_sign_and_verify_roundtrip`, `test_v3_tampered_payload_fails_verification`) | `test` | Nur in `crypto` | Ausführliche Unit-Tests in `crypto`. Werden im Übergabepaket bereitgestellt. |

---

### 2.2 Modul `kv_shredding`

| Funktion / Symbol | Sichtbarkeit | Status | Bemerkungen / Diff |
|---|---|---|---|
| `SubKey` | `pub` | Identisch | 32-Byte Secret Key Newtype mit `ZeroizeOnDrop`. |
| `GroupKek` | `pub` | Nur in `crypto` | 32-Byte Group Key Encryption Key mit `ZeroizeOnDrop`. In `erasure` nicht vorhanden. |
| `RecordDek` | `pub` | Nur in `crypto` | 32-Byte Record Data Encryption Key mit `ZeroizeOnDrop`. In `erasure` nicht vorhanden. |
| `EncryptedRecordPayload` | `pub` | Nur in `crypto` | Serialisierbare Struktur für Ciphertext, Nonces und gewrappte Schlüssel. |
| `derive_subkey` | `pub` | Abweichend | `crypto`: `derive_subkey(registry, master_key, group_id) -> Result<SubKey>`<br>`erasure`: `derive_subkey(master_key, group_id, epoch) -> SubKey` (deterministische BLAKE3 KDF). |
| `KeyRegistry::new` | `pub` | Identisch | Initialisiert Registry. |
| `KeyRegistry::with_revocation_log` | `pub` | Identisch | Bindet Revocation-Log an. |
| `KeyRegistry::get_wrapped_kek` | `pub` | Nur in `crypto` | Liefert gewrappten KEK für Gruppe. |
| `KeyRegistry::get_wrapped_dek` | `pub` | Nur in `crypto` | Liefert gewrappten DEK für Record. |
| `KeyRegistry::is_group_revoked` | `pub` | Identisch | Prüft Gruppenstatus im Revocation-Log. |
| `KeyRegistry::get_or_derive` | `pub` | Nur in `crypto` | Erzeugt oder lädt Group KEK. |
| `KeyRegistry::revoke_group` | `pub` | Identisch | Widerruft KEK einer gesamten Gruppe in O(1). |
| `KeyRegistry::revoke_subkey` | `pub` | Nur in `crypto` | Alias für Gruppenwiderruf (`revoke_group`). |
| `KeyRegistry::revoke_record` | `pub` | Identisch | Widerruft DEK eines einzelnen Records in O(1). |
| `KeyRegistry::is_group_active` | `pub` | Identisch | Prüft Aktivität der Gruppe. |
| `KeyRegistry::is_key_active` | `pub` | Nur in `crypto` | Alias für `is_group_active`. |
| `KeyRegistry::is_record_active` | `pub` | Identisch | Prüft Aktivität des Einzel-Records. |
| `KeyRegistry::encrypt_record` | `pub` | Abweichend | In `crypto` vollständige Envelope Encryption mit Zufalls-DEK und KEK-Wrapping. In `erasure` vereinfachtes Schema. |
| `KeyRegistry::decrypt_record` | `pub` | Abweichend | Entschlüsselt DEK und Record-Ciphertext. |
| `KeyRegistry::encrypt_with_group` | `pub` | Nur in `crypto` | Verschlüsselt direkt unter dem Gruppen-KEK (genutzt von `kv_segment`). |
| `KeyRegistry::decrypt_with_group` | `pub` | Nur in `crypto` | Entschlüsselt direkt unter dem Gruppen-KEK (genutzt von `kv_segment`). |
| Tests (`test_envelope_group_deletion_and_attack_simulation`, `test_envelope_single_record_deletion_64_records`) | `test` | Nur in `crypto` | Liegen im Übergabepaket. |

---

### 2.3 Modul `revocation_log`

| Funktion / Symbol | Sichtbarkeit | Status | Bemerkungen / Diff |
|---|---|---|---|
| `RevocationTarget` | `pub` | Identisch | Enum: `Group(u64)`, `Record(u64)`. |
| `RevocationTarget::canonical_bytes` | `pub` | Identisch | Kanonische Bytefolge für Hash-Berechnung. |
| `RevocationEntry` | `pub` | Identisch | Serialisierbarer Eintrag mit Hash-Verkettung und Ed25519-Signatur. |
| `compute_entry_hash` | `pub` | Identisch | Berechnet BLAKE3 Hash über Kettenelement. |
| `RevocationLog::new_in_memory` | `pub` | Identisch | In-Memory Konstruktor. |
| `RevocationLog::open_or_create` | `pub` | Identisch | Öffnet Logdatei über VFS und prüft Integrität. |
| `RevocationLog::verify_integrity` | `pub` | Identisch | Verifiziert Kette und Signaturen. |
| `RevocationLog::verify_chain_entries` | `priv` | Identisch | Interne Kettenvalidierung. |
| `RevocationLog::append` | `pub` | Identisch | Hängt Widerruf an und persistiert atomar. |
| `RevocationLog::revoked_targets` | `pub` | Identisch | Liefert alle widerrufenen Targets. |
| `RevocationLog::is_revoked` | `pub` | Identisch | Prüft ob Target widerrufen ist. |
| `RevocationLog::len` / `is_empty` | `pub` | Identisch | Längenabfragen. |
| Tests (`test_persistence_and_simulated_restart`, `test_in_memory_revocation_log_append_and_query`, `test_persisted_log_tamper_detection_fails_open`, `test_get_or_derive_fails_with_dedicated_error_on_revoked_group`, `test_vfs_conformance`) | `test` | Nur in `crypto` | Liegen im Übergabepaket. |

---

### 2.4 Modul `deletion_proof`

| Funktion / Symbol | Sichtbarkeit | Status | Bemerkungen / Diff |
|---|---|---|---|
| `DeletionProof` | `pub` | Identisch | Kanonische Repräsentation von Nachweisen (V1, V2, V3, V5). |
| `DeletionProof::issue` | `pub` | Identisch | Erstellt v5 Nachweis basierend auf `SagaRecord`. |
| `DeletionProof::create` | `pub` | Identisch | Erstellt v2 HMAC Nachweis. |
| `DeletionProof::create_with_wal_receipt` | `pub` | Identisch | Erstellt v2 Nachweis mit WAL-Receipt. |
| `DeletionProof::create_v3` | `pub` | Identisch | Erstellt v3 Ed25519 Nachweis. |
| `DeletionProof::create_v3_with_audit_position` | `pub` | Identisch | Erstellt v3 Nachweis mit Audit-Chain Index. |
| `DeletionProof::create_with_wal_receipt_v3` | `pub` | Identisch | Erstellt v3 Nachweis mit WAL-Receipt. |
| `DeletionProof::create_full_v3` | `pub` | Identisch | Erstellt vollständigen v3 Nachweis mit WAL-Receipt und Audit-Position. |
| `DeletionProof::verify` | `pub` | Identisch | Interne Verifikation. |
| `DeletionProof::verify_external` | `pub` | Identisch | Externe Verifikation ohne internen State. |
| `DeletionProof::export_for_audit` | `pub` | Identisch | JSON-Export für Audit-Zwecke. |
| `DeletionProofVerifier::verify_standalone` | `pub` | Identisch | Standalone Verifikation. |
| `DeletionProofVerifier::verify_standalone_for_tenant` | `pub` | Identisch | Verifikation mit Mandantenbindung. |
| `DeletionProofVerifier::verify_audit_chain_and_head` | `pub` | Nur in `crypto` | Verifiziert Bindung von `DeletionProof` an `AuditChain` und signierten Kettenkopf. |
| `verify_proof_sequence` | `pub` | Nur in `crypto` | In `deletion_proof/chain.rs`: Prüft monotone Zeitreihenfolge einer Beweiskette. |
| `derive_deletion_proof_key_for_tenant` | `pub` | Nur in `crypto` | In `deletion_proof/key_derivation.rs`: Mandantenspezifische Schlüsselableitung für HMAC-Beweise. |
| `LayerCleanupProof::for_test` | `pub` | Nur in `crypto` | Test-Helper zur Erzeugung von `LayerCleanupProof`. |

---

## 3. Fazit und Handlungsanweisung

1. Sämtliche in `contextra-crypto` vorhandenen, aber in `contextra-erasure` noch fehlenden oder abweichenden Funktionalitäten (insbesondere `encrypt_with_group`, `decrypt_with_group`, `EncryptedRecordPayload`, `verify_audit_chain_and_head`, `verify_proof_sequence`, `derive_deletion_proof_key_for_tenant` sowie alle Test-Suites) werden vollständig und lauffähig in `docs/handoff/CRYPTO-2/` abgelegt.
2. In `contextra-crypto` verbleiben diese Module vorerst hinter dem Feature `legacy-erasure` (Default an) und erhalten `#[deprecated]`-Attribute, da `contextra-engine` aktuell noch direkte Bindungen an `contextra_crypto::deletion_proof::{LayerWitnessExt, CustodyLevel, DeletionProof}` aufweist.
3. Sobald der Workspace-Grep leer ist (nach Übernahme durch `ERASE-2` und Bereinigung von `contextra-engine`), werden die Module endgültig entfernt.
