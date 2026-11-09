# Übergabepaket MISSION `CRYPTO-2` an Prompt `ERASE-2`

**Datum:** 2026-11-08
**Quell-Crate:** `contextra-crypto`
**Ziel-Crate:** `contextra-erasure` (übernommen durch Prompt `ERASE-2`)
**Status:** Verbatim gesichert, bereit zur Integration in `contextra-erasure`.

---

## 1. Inhalt des Übergabepakets

Dieses Verzeichnis enthält den vollständigen Stand der domänenfremden Lösch- und Beweislogik aus `contextra-crypto`, die gemäß Architektur-Zielbild (Klasse A Autarkie, SPEC v4.0 §6.3 / E-04) ausschließlich in `contextra-erasure` beheimatet sein darf.

### 1.1 Quellcode-Module
- **`deletion_proof/`**:
  - `mod.rs`: Hauptmodul und Fassade.
  - `creation.rs`: Erstellungsfunktionen für V1, V2, V3, V5 (`issue`, `create`, `create_v3`, `create_full_v3`, HMAC- und WAL-Delete-Receipt-Berechnung).
  - `types.rs`: DeletionProof, LayerCleanupProof, DeletionLayer, DeletionScope, CustodyLevel, SigningKeyHandle, VerificationKey.
  - `verification.rs`: Interne und externe Verifikation (`verify`, `verify_external`, `export_for_audit`).
  - `verifier.rs`: Standalone-Verifizierer für Dritte (`DeletionProofVerifier`, `verify_standalone`, `verify_standalone_for_tenant`, `verify_audit_chain_and_head`).
  - `key_derivation.rs`: Mandantenspezifische Schlüsselableitung (`derive_deletion_proof_key_for_tenant`).
  - `chain.rs`: Sequenz- und Kettungsprüfungen (`verify_proof_sequence`).
- **`ed25519_proof/`**:
  - `ed25519_proof.rs`: Asymmetrische Ed25519-Signierung und Verifikation (`DeletionProofKeyPair`, `sign_deletion_proof_v3`, `verify_deletion_proof_v3`, `SignatureVersion`).
- **`kv_shredding/`**:
  - `kv_shredding.rs`: Vollständige Envelope-Encryption-Engine für Krypto-Shredding (`KeyRegistry`, `GroupKek`, `RecordDek`, `EncryptedRecordPayload`, `encrypt_record`, `decrypt_record`, `encrypt_with_group`, `decrypt_with_group`).
- **`revocation_log/`**:
  - `revocation_log.rs`: Append-Only Widerrufslog mit Hash-Verkettung, Ed25519-Signatur und VFS-Persistenz (`RevocationLog`, `RevocationEntry`, `RevocationTarget`).

### 1.2 Zugehörige Tests (`tests/`)
Folgende Integrationstests gehören zu den Modulen und sollen nach Abschluss der Migration in `crates/contextra-erasure/tests/` ausgeführt werden:
- `deletion_proof_external_verify_rejects_tamper.rs`
- `deletion_proof_hash_collision.rs`
- `deletion_proof_negative_and_golden.rs`
- `deletion_proof_prefix_collision_proptest.rs`
- `deletion_proof_signature_version_serde.rs`
- `deletion_proof_v3_validation.rs`
- `ed25519_proof_external_verify.rs`
- `revocation_log_vfs.rs`

---

## 2. Hinweise für `ERASE-2` zur Übernahme

1. **Typ-Anpassungen:**
   - In `contextra-erasure` werden lokale Typen (bzw. Typen aus `contextra-types`) verwendet.
   - Fehlerbehandlung in `contextra-erasure` nutzt `ErasureError`.
2. **Envelope Encryption in `kv_shredding`:**
   - `contextra-erasure` hat derzeit nur eine deterministische KDF-Variante. Die hier übergebene `KeyRegistry` mit zufälligen DEKs/KEKs und Envelope Encryption ist das geforderte Produktiv-Shredding-Verfahren (Spec v17 §16.1 / AP-P0-06).
3. **Deprecation in `contextra-crypto`:**
   - Bis die Konsumenten in `contextra-engine` umgestellt sind, bleiben die Typen in `contextra-crypto` unter `#[deprecated]` und hinter dem Feature `legacy-erasure` verfügbar.
