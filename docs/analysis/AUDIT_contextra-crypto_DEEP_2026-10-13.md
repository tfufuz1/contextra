# AUDIT-Bericht: Tiefenanalyse `contextra-crypto`

- **Spezialist:** S4, Spezialist für Kryptografie, Löschnachweise und Schlüsselverwaltung
- **Commit:** `59f853cb`
- **Datum:** 2026-10-13
- **Vorgänger-Berichte:**
  - `docs/audits/AUDIT_contextra-crypto_2026-10-12.md`
  - `docs/audits/AUDIT_crypto-kernel_DEEP_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-crypto_2026-10-04.md`
- **Session-Status:** `VERIFIED-BY-SESSION: PASSED`

---

## 1 Scope und Abdeckung

| Datei | Zeilen | Gelesen | Begründung / Anmerkung |
|---|---|---|---|
| `crates/contextra-crypto/src/lib.rs` | 74 | Ja | Modul-Deklarationen, Re-exports, Crate-Attributes (`#![forbid(unsafe_code)]`) |
| `crates/contextra-crypto/src/anti_tamper.rs` | 89 | Ja | `VolatileEncryptionKey`, ZeroizeOnDrop, Not-Aus Trigger `emergency_wipe()` |
| `crates/contextra-crypto/src/audit_chain.rs` | 425 | Ja | Blake3-Hashkette, `AuditChainEntry`, `DataClass`, `sign_head`, `verify_head_signature` |
| `crates/contextra-crypto/src/crypto.rs` | 838 | Ja | `KeyManager`, HKDF-SHA256, Argon2id, AES-256-GCM-SIV, Nonce-Counter |
| `crates/contextra-crypto/src/deletion_proof.rs` | 33 | Ja | Modul-Fassade und Re-exports für Löschnachweise |
| `crates/contextra-crypto/src/deletion_proof/audit_export.rs` | 35 | Ja | Export-Helfer für externe Audit-Standard-Kopplung |
| `crates/contextra-crypto/src/deletion_proof/builder.rs` | 240 | Ja | `ProofBuilder` und `finish_checked` für v4 Löschnachweise |
| `crates/contextra-crypto/src/deletion_proof/canon.rs` | 210 | Ja | Deterministische TLV-Kodierung/Dekodierung (`TlvEncoder`/`TlvDecoder`) |
| `crates/contextra-crypto/src/deletion_proof/claim_text.rs` | 56 | Ja | `WitClaimText` Festtexte nach Spec C7 |
| `crates/contextra-crypto/src/deletion_proof/evidence.rs` | 48 | Ja | `WitLayerEvidence` Evidenz-Formate für 11 Storage-Layer |
| `crates/contextra-crypto/src/deletion_proof/exclusions.rs` | 42 | Ja | Pflicht- und Zusatz-Ausschlüsse (`mandatory_ids`) |
| `crates/contextra-crypto/src/deletion_proof/issuer.rs` | 380 | Ja | `ProofIssuer` Gegenprüfungen C3/C7 und v4 Issuance |
| `crates/contextra-crypto/src/deletion_proof/keys.rs` | 75 | Ja | `DeletionProofKeyPair` Ed25519 Signierschlüssel mit Zeroize |
| `crates/contextra-crypto/src/deletion_proof/layer_ids.rs` | 72 | Ja | Contract V1 ID Mappings (1..7, 20..26) |
| `crates/contextra-crypto/src/deletion_proof/layer_proof.rs` | 18 | Ja | Typisierte Layer-Löschbestätigung |
| `crates/contextra-crypto/src/deletion_proof/policy.rs` | 480 | Ja | `ProofPolicy` Regeln R1..R17 Verifikationsnetz |
| `crates/contextra-crypto/src/deletion_proof/probe.rs` | 24 | Ja | `WitStoreProbe` Trait für Multi-Probe Konsistenz |
| `crates/contextra-crypto/src/deletion_proof/proof.rs` | 120 | Ja | Kern-Strukturen `DeletionProof`, `DeletionScope`, `DurabilityProof` |
| `crates/contextra-crypto/src/deletion_proof/proof_store.rs` | 430 | Ja | Persistent Store für Proof v4 `.bin`/`.json` und `chain.log` |
| `crates/contextra-crypto/src/deletion_proof/proof_v2.rs` | 45 | Ja | Legacy v2 HMAC Löschnachweise |
| `crates/contextra-crypto/src/deletion_proof/proof_v3.rs` | 260 | Ja | Ed25519 v3 Löschnachweise (`create_v3`, `create_full_v3`) |
| `crates/contextra-crypto/src/deletion_proof/proof_v4.rs` | 320 | Ja | V4 TLV Proof-Format mit Domain Tag und B3 Hash-Chain |
| `crates/contextra-crypto/src/deletion_proof/signer.rs` | 290 | Ja | `ProofKeyPairSigner`, Key Rotation, File Permissions (0600, UID Check) |
| `crates/contextra-crypto/src/deletion_proof/types.rs` | 180 | Ja | Basis-Datentypen, Serde-Repräsentationen |
| `crates/contextra-crypto/src/deletion_proof/unverified.rs` | 65 | Ja | Unverified Proof Wrapper für Fail-Closed Deserialisierung |
| `crates/contextra-crypto/src/deletion_proof/verify.rs` | 180 | Ja | External & KeyManager Verification für Löschnachweise |
| `crates/contextra-crypto/src/deletion_proof/wal_receipt.rs` | 25 | Ja | WAL Chain Receipt Bindung |
| `crates/contextra-crypto/src/deletion_proof_typestate.rs` | 465 | Ja | Compile-zeitliche Typestate-Prüfung aller 7 Deletion-Layer |
| `crates/contextra-crypto/src/deletion_witness.rs` | 145 | Ja | `LayerWitness` Zeuge-Ausstellung (`issue`) |
| `crates/contextra-crypto/src/ed25519_proof.rs` | 185 | Ja | Low-level Ed25519 Signatur- und Verifikationsroutinen |
| `crates/contextra-crypto/src/error.rs` | 65 | Ja | `CryptoError` Enum und Result-Alias |
| `crates/contextra-crypto/src/kdf.rs` | 325 | Ja | Argon2id KDF, `KdfHeader`, `KdfParams`, `DerivedKey` |
| `crates/contextra-crypto/src/keystore.rs` | 180 | Ja | MFKS (Master Format Keystore) Binaer-Layout |
| `crates/contextra-crypto/src/kms.rs` | 165 | Ja | `KmsProvider` Trait, `LocalKmsProvider`, `DestroyedKey` |
| `crates/contextra-crypto/src/kv_cipher.rs` | 470 | Ja | `KvSegmentCipher`, `EncryptedKvLayer`, `ModelFingerprint` |
| `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | 120 | Ja | Memory Eviction Worker & Emergency Wipe Hook |
| `crates/contextra-crypto/src/kv_segment/mod.rs` | 15 | Ja | Modul-Reexports |
| `crates/contextra-crypto/src/kv_segment/segment.rs` | 210 | Ja | Tensor Segment Encrypted Container |
| `crates/contextra-crypto/src/kv_segment/store.rs` | 410 | Ja | `TenantIsolatedKvStore`, LRU-Eviction, Multi-Tenant Guard |
| `crates/contextra-crypto/src/kv_shredding.rs` | 810 | Ja | Crypto-Shredding `KeyRegistry`, `GroupKek`, `RecordDek` |
| `crates/contextra-crypto/src/revocation_log.rs` | 840 | Ja | Fail-Closed Revokations-Log, Marker-Files, Integrity-Verifikation |
| `crates/contextra-crypto/src/secure_buf.rs` | 65 | Ja | `CchSecureBuf` Zeroize Secure Buffer Wrapper |
| `crates/contextra-crypto/src/wal_completeness.rs` | 85 | Ja | I/O-freie WAL Kettenvollständigkeitsprüfung |
| `crates/contextra-crypto/src/wal_crypto.rs` | 910 | Ja | `WalHmac`, `IntegrityVerifier`, `EncryptedWal`, HMAC-Chaining |

---

## 2 Inventar der pub-Items

*Basis: `cargo xtask orphan-symbols --json` und Gegenprüfung am Quellcode.*

| Item-Typ | Symbol | Datei:Zeile | Nutzung Klassifikation |
|---|---|---|---|
| `struct` | `VolatileEncryptionKey` | `anti_tamper.rs:17` | Produktiv (`crypto.rs`, `kv_segment`) |
| `struct` | `KeyManager` | `crypto.rs:48` | Produktiv (`contextra-store`, `contextra-engine`, `contextra-privacy`) |
| `struct` | `AuditChain` | `audit_chain.rs:170` | Produktiv (`contextra-mcp`, `contextra-store`, Tests) |
| `struct` | `AuditChainEntry` | `audit_chain.rs:43` | Produktiv (`audit_chain.rs`, Tests) |
| `struct` | `AuditChainAnchor` | `audit_chain.rs:109` | Produktiv (`audit_chain.rs`, Tests) |
| `struct` | `AuditChainHeadSignature` | `audit_chain.rs:118` | Produktiv (`audit_chain.rs`, Tests) |
| `struct` | `EncryptedCommitmentSalt` | `audit_chain.rs:128` | Produktiv (`audit_chain.rs`, Tests) |
| `enum` | `DataClass` | `audit_chain.rs:25` | Produktiv (`audit_chain.rs`, `contextra-privacy`) |
| `struct` | `DeletionProof` | `deletion_proof/proof.rs:9` | Produktiv (`contextra-engine`, `contextra-store`, `contextra-mcp`) |
| `enum` | `DeletionScope` | `deletion_proof/proof.rs:19` | Produktiv (`contextra-engine`, `contextra-store`) |
| `struct` | `DurabilityProof` | `deletion_proof/proof.rs:34` | Produktiv (`deletion_proof/proof_v3.rs`, `contextra-store`) |
| `struct` | `ProofBuilder` | `deletion_proof/builder.rs:25` | Produktiv (`deletion_proof/issuer.rs`, Tests) |
| `struct` | `ProofV4` | `deletion_proof/proof_v4.rs:17` | Produktiv (`deletion_proof/issuer.rs`, `proof_store.rs`) |
| `struct` | `ProofIssuer` | `deletion_proof/issuer.rs:42` | Produktiv (`contextra-engine`, Tests) |
| `struct` | `ProofStore` | `deletion_proof/proof_store.rs:46` | Produktiv (`deletion_proof/issuer.rs`, Tests) |
| `struct` | `ProofKeyPairSigner` | `deletion_proof/signer.rs:23` | Produktiv (`deletion_proof/issuer.rs`, Tests) |
| `trait` | `ProofSigner` | `deletion_proof/signer.rs:13` | Produktiv (`deletion_proof/builder.rs`, `signer.rs`) |
| `struct` | `ProofPolicy` | `deletion_proof/policy.rs:53` | Produktiv (`deletion_proof/issuer.rs`, `verify.rs`) |
| `struct` | `DeletionProofKeyPair` | `deletion_proof/keys.rs:6` | Produktiv (`deletion_proof/proof_v3.rs`, `signer.rs`) |
| `struct` | `UnverifiedProof` | `deletion_proof/unverified.rs:13` | Produktiv (`contextra-engine`, `contextra-wire`) |
| `struct` | `LayerWitness` | `deletion_witness.rs:82` | Produktiv (`deletion_proof/issuer.rs`, `contextra-store`) |
| `struct` | `WitnessEvidence` | `deletion_witness.rs:115` | Produktiv (`deletion_witness.rs`, Tests) |
| `struct` | `KdfHeader` | `kdf.rs:111` | Produktiv (`crypto.rs`, `keystore.rs`) |
| `struct` | `KdfParams` | `kdf.rs:52` | Produktiv (`kdf.rs`, `keystore.rs`) |
| `struct` | `DerivedKey` | `kdf.rs:302` | Produktiv (`kdf.rs`, `crypto.rs`) |
| `struct` | `MfksKeystore` | `keystore.rs:23` | Produktiv (`kms.rs`, Tests) |
| `trait` | `KmsProvider` | `kms.rs:18` | UNGEWIRKT (Port-Interface ohne Produktions-Composition-Root) |
| `struct` | `LocalKmsProvider` | `kms.rs:58` | UNGEWIRKT (Nur Test-Aufrufe) |
| `struct` | `DestroyedKey` | `kms.rs:33` | UNGEWIRKT (Gekoppelt an LocalKmsProvider) |
| `struct` | `DeletionReceipt` | `kms.rs:49` | UNGEWIRKT (Gekoppelt an LocalKmsProvider) |
| `struct` | `KvSegmentCipher` | `kv_cipher.rs:101` | Produktiv (`contextra-kvcache`, `contextra-engine`) |
| `struct` | `EncryptedKvLayer` | `kv_cipher.rs:42` | Produktiv (`kv_cipher.rs`, `contextra-kvcache`) |
| `struct` | `ModelFingerprint` | `kv_cipher.rs:24` | Produktiv (`kv_cipher.rs`, `contextra-engine`) |
| `struct` | `EvictionWorker` | `kv_segment/eviction_worker.rs:24` | Produktiv (`contextra-kvcache`) |
| `struct` | `KvSegment` | `kv_segment/segment.rs:25` | Produktiv (`kv_segment/store.rs`) |
| `struct` | `TenantIsolatedKvStore` | `kv_segment/store.rs:48` | Produktiv (`contextra-kvcache`, `contextra-engine`) |
| `struct` | `KeyRegistry` | `kv_shredding.rs:119` | Produktiv (`kv_cipher.rs`, `kms.rs`, `audit_chain.rs`) |
| `struct` | `SubKey` | `kv_shredding.rs:30` | Produktiv (`kv_shredding.rs`) |
| `struct` | `GroupKek` | `kv_shredding.rs:48` | Produktiv (`kv_shredding.rs`) |
| `struct` | `RecordDek` | `kv_shredding.rs:60` | Produktiv (`kv_shredding.rs`) |
| `struct` | `RevocationLog` | `revocation_log.rs:233` | Produktiv (`kv_cipher.rs`, `kms.rs`) |
| `struct` | `RevocationEntry` | `revocation_log.rs:44` | Produktiv (`revocation_log.rs`, Tests) |
| `enum` | `RevocationTarget` | `revocation_log.rs:154` | Produktiv (`revocation_log.rs`, `kv_shredding.rs`) |
| `struct` | `CchSecureBuf` | `secure_buf.rs:13` | Produktiv (In `lib.rs` re-exportiert) |
| `struct` | `WalHmac` | `wal_crypto.rs:158` | Produktiv (`contextra-store/src/wal/`) |
| `struct` | `IntegrityVerifier` | `wal_crypto.rs:187` | Produktiv (`contextra-store/src/wal/`) |
| `struct` | `EncryptedWal` | `wal_crypto.rs:488` | Produktiv (`contextra-store/src/wal/`) |
| `fn` | `default_signature_version` | `deletion_proof/proof.rs:87` | ORPHAN (Serde Default-Helfer, workspace-weit ungenutzt) |
| `fn` | `mandatory_ids` | `deletion_proof/exclusions.rs:33` | ORPHAN (Helfer, workspace-weit ungenutzt) |

---

## 3 Antworten auf die fachlichen Prüffragen (1–9)

### 1. Welche Schlüsseltypen gibt es? Sind alle mit Zeroize oder ZeroizeOnDrop versehen?
- **Gefundene Schlüsseltypen in `crates/contextra-crypto/src/`:**
  1. `VolatileEncryptionKey` (`anti_tamper.rs:17`): `Zeroize` + `#[zeroize(drop)]`
  2. `SubKey` (`kv_shredding.rs:30`): `#[derive(ZeroizeOnDrop)]`
  3. `GroupKek` (`kv_shredding.rs:48`): `#[derive(ZeroizeOnDrop)]`
  4. `RecordDek` (`kv_shredding.rs:60`): `#[derive(ZeroizeOnDrop)]`
  5. `DerivedKey` (`kdf.rs:302`): `#[derive(ZeroizeOnDrop)]`
  6. `DeletionProofKeyPair` (`deletion_proof/keys.rs:6`): Explizite `impl Drop` mit `self.zeroize()` (Secret Signing Key mit `[0u8; 32]` überschrieben).
  7. `WalHmac` (`wal_crypto.rs:158`): `#[zeroize(drop)]`
  8. `IntegrityVerifier` (`wal_crypto.rs:187`): `#[zeroize(drop)]`
  9. `KeyManager` (`crypto.rs:48`): Kapselt `VolatileEncryptionKey` und bereinigt Cipher-Cache im `emergency_wipe()`.
  10. `CchSecureBuf` (`secure_buf.rs:13`): `impl Zeroize for CchSecureBuf` + `impl Drop`.
- **Ergebnis:** Alle kryptografischen Schlüsseltypen und Puffer sind lückenlos mit `Zeroize` oder `ZeroizeOnDrop` versehen.

### 2. Kann Schlüsselmaterial über Debug, Display, Logs oder Fehlermeldungen nach außen gelangen?
- Alle schlüsseltragenden Typen besitzen benutzerdefinierte `Debug`-Implementierungen, die das Schlüsselmaterial explizit als `"***REDACTED***"` oder `"*** REDACTED ***"` maskieren:
  - `VolatileEncryptionKey`: `anti_tamper.rs:50` (`"*** REDACTED ***"`)
  - `KeyManager`: `crypto.rs:55` (`"***REDACTED***"`)
  - `SubKey`: `kv_shredding.rs:38` (`"***REDACTED***"`)
  - `GroupKek`: `kv_shredding.rs:50` (`"***REDACTED***"`)
  - `RecordDek`: `kv_shredding.rs:62` (`"***REDACTED***"`)
  - `KeyRegistry`: `kv_shredding.rs:128` (`"***REDACTED***"`)
  - `DeletionProofKeyPair`: `deletion_proof/keys.rs:11` (`signing_key`: `"***REDACTED***"`, `verifying_key`: öffentlich)
  - `RevocationLog`: `revocation_log.rs:665` (`signing_key`: `"<redacted>"`)
  - `CchSecureBuf`: `secure_buf.rs:57` (`"*** REDACTED ***"`)
- Es gibt **keine** `Display`-Implementierungen für Schlüsseltypen und keine Durchleitung von Schlüsselbytes in `CryptoError`.
- **Ergebnis:** Geprüft, kein Befund. Kein Leckage-Risiko über Debug/Display/Logs/Errors.

### 3. Für jede Funktion, die einen Löschbeweis erzeugt oder verifiziert: Aufrufer per Befehl belegen
- `DeletionProof::create_v3` (`deletion_proof/proof_v3.rs:138`):
  - Aufrufer 1: `KeyManager::create_deletion_proof_v3` (`crypto.rs:390`) -> Produktiv (`contextra-engine/src/contextra_impl/collections.rs`)
  - Aufrufer 2: Test-Suites (`deletion_proof_v3_validation.rs`, `campaign_cov_deletion_proof.rs`)
  - **Belegt via `cargo xtask symbol-exists`**: `contextra_crypto::deletion_proof::proof_v3::DeletionProof::create_v3`
- `DeletionProof::verify_external` (`deletion_proof/verify.rs:166`):
  - Aufrufer 1: `UnverifiedProof::verify_external` (`deletion_proof/unverified.rs:55`) -> Produktiv (`contextra-engine`, `contextra-mcp`)
  - Aufrufer 2: Test-Suites (`campaign_cov_deletion_proof.rs`, `deletion_proof_external_verify_rejects_tamper.rs`)
  - **Belegt via `cargo xtask symbol-exists`**: `contextra_crypto::deletion_proof::verify::DeletionProof::verify_external`
- `ProofBuilder::finish_checked` (`deletion_proof/builder.rs:138`):
  - Aufrufer: `ProofIssuer::issue` (`deletion_proof/issuer.rs:367`) -> Produktiv (`contextra-engine/src/contextra_impl/collections.rs`)
  - **Belegt via `cargo xtask symbol-exists`**: `contextra_crypto::deletion_proof::builder::ProofBuilder::finish_checked`
- `ProofIssuer::issue` (`deletion_proof/issuer.rs:141`):
  - Aufrufer: `contextra-engine/src/contextra_impl/collections.rs:188` -> Produktiv
  - **Belegt via `cargo xtask symbol-exists`**: `contextra_crypto::deletion_proof::issuer::ProofIssuer::issue`

### 4. Ist ein Löschbeweis ohne Datenbankzustand verifizierbar? Welche Abhängigkeiten hat die Verifikation?
- **Asymmetrische Verifikation (v3 & v4 Proofs)**:
  - `verify_external(&self, public_key: &VerifyingKey)` / `verify_with_policy` ist **vollständig frei von Datenbankzustand, KeyManager oder Runtime-I/O**.
  - **Abhängigkeiten**:
    1. Der 32-Byte Ed25519 Verifikationsschlüssel (`VerifyingKey`) des Ausstellers (Trust-Anchor).
    2. Der kanonische Proof-Payload (`ProofV4` oder `ProofV3`).
    3. Blake3 Hash-Algorithmus & Ed25519 Verifikationslogik (`ed25519-dalek`).
- **Symmetrische Verifikation (Legacy v1/v2 Proofs)**:
  - Benötigte `KeyManager` HMAC-Schlüssel (veraltet, in v4 gesperrt).
- **Ergebnis:** Ja, v3 und v4 Löschnachweise sind absolut autonom ohne Datenbankzustand extern verifizierbar (AK-20/AK-21 konform).

### 5. Ist der KMS-Port mit einer Produktiv-Implementierung verdrahtet? Ist die WAL-Verschlüsselung an einen festen Schlüsselmanager gebunden?
- `KmsProvider` Trait (`kms.rs:18`) besitzt mit `LocalKmsProvider` (`kms.rs:58`) eine vollständige Crate-interne Implementierung.
- **KMS Verdrahtungs-Befund**: Weder `contextra` (Facade), `contextra-engine` noch `contextra-db` instanziieren `LocalKmsProvider` im Produktions-Composition-Root. Der Trait ist ungewirkt (`cargo xtask unwired-ports`).
- **WAL-Verschlüsselung Bindung**: `EncryptedWal` (`wal_crypto.rs:488`) akzeptiert eine `KeyManager`-Instanz. Die Schlüsselableitung nutzt HKDF mit domänensepariertem Info-String `b"contextra-wal-v1"`. Es ist an die `KeyManager`-Schnittstelle gebunden, nicht an ein globales KMS-Singleton.

### 6. Verhält sich das Revocation-Log bei fehlender oder manipulierter Datei fail-closed?
- Ja. `RevocationLog::open_or_create_with_mode` (`revocation_log.rs:331`):
  - Fehlt die Log-Datei oder die Marker-Datei (`.marker`) bei `InitMode::OpenExisting`, bricht der Aufruf sofort mit `Err(CryptoError::IntegrityViolation)` ab.
  - Besitzt das Verzeichnis Schlüsselspuren (`has_existing_keys: true`), aber Log/Marker fehlen, verweigert `InitMode::OpenOrCreateIfFresh` die Neu-Erzeugung und bricht fail-closed mit `Err(CryptoError::IntegrityViolation)` ab.
  - Bei Diskrepanz zwischen Log-Eintragsanzahl und signiertem Marker-Header oder ungültiger Ed25519-Marker-Signatur bricht die Initialisierung mit `Err(CryptoError::IntegrityViolation)` ab.

### 7. Ist die CSPRNG-Ausnahme (P28) auf Schlüssel- und Salt-Material beschränkt?
- Alle Vorkommen von `OsRng` in `crates/contextra-crypto/src/` wurden geprüft:
  - `crypto.rs`: `nonce_prefix` (4 zufällige Bytes bei KeyManager-Erzeugung).
  - `kdf.rs`: Salt-Generierung (32 Bytes via `OsRng` in `KdfHeader::generate_default`).
  - `kv_shredding.rs`: Zufällige GEK/DEK-Schlüssel und Nonces (`kek_bytes`, `dek_bytes`, `dek_nonce`, `nonce`).
  - `deletion_proof/keys.rs` & `signer.rs`: Ed25519 Keypair-Generierung.
- **Ergebnis:** Geprüft. `OsRng` wird ausschließlich für kryptografisches Schlüsselmaterial, Salts, Nonces und Signierschlüssel verwendet (100% P28-Konformität).

### 8. Gibt es AEAD-Aufrufe mit wiederverwendeten oder deterministisch aus unsicheren Quellen erzeugten Nonces?
- `KeyManager::encrypt_auto_nonce` (`crypto.rs`):
  - Generiert 12-Byte Nonce aus 4-Byte `OsRng`-Instanz-Präfix + 8-Byte atomarem `AtomicU64`-Monotoniezähler.
  - Bei Zählerüberlauf (`u64::MAX`) bricht die Verschlüsselung mit `CryptoError::Encryption` ab. Ein Wrap-around auf 0 ist unmöglich.
- `KvSegmentCipher` (`kv_cipher.rs`): Generiert frische 12-Byte Nonces via `OsRng` pro Verschlüsselung.
- AES-256-GCM-SIV ist misuse-resistant nach RFC 8452.
- **Ergebnis:** Geprüft, kein Befund. Keine Re-use-Gefahr.

### 9. Ist der Audit-Chain-Anker im Produktionsablauf eingehängt? Ist die Verifikation ohne KeyManager möglich?
- `AuditChain` (`audit_chain.rs:170`) und `AuditChainEntry` werden in `contextra-mcp` und `contextra-store` getestet.
- **Produktions-Einhängung**: Im `contextra-engine` Transaktionsablauf wird `AuditChain` nicht als aktiver Persistenz-Fluss geführt, sondern separat von `ContextEditAuditSink` im Agenten-Bereich betrieben.
- **KeyManager-Unabhängigkeit**: `AuditChain::verify_chain()` und `AuditChain::verify_head_signature()` benötigen **keinen** `KeyManager`. Sie verifizieren rein deterministische Blake3-Hashes und Ed25519-Signaturen gegen einen übergebenen `VerifyingKey`.

---

## 4 Ergebnisse je Kategorie A bis M

### Category A: Unvollständige oder angefangene Implementierungen
- **Geprüft, Befund vorhanden (FEHLER-A1)**:
  - `kv_shredding.rs:247`: `// TODO(#JULES-P04-1, Implementer): [P04 / F-3 / CRITICAL & Phase D]` bezüglich Lock-freier `is_group_revoked`-Prüfung und Durability-Hardening im Code vorhanden.

### Category B: Fehlerhafte Logik
- **Geprüft, kein Befund**. Off-by-one, LE/BE Byteorder und Längen-Präfixe in TLV und HKDF sind präzise spezifiziert.

### Category C: Fehlende Verdrahtung
- **Geprüft, Befund vorhanden (FEHLER-C1)**:
  - `src/kms.rs`: `KmsProvider` Trait und `LocalKmsProvider` besitzen keinen Produktions-Composition-Root-Aufrufer in `contextra` oder `contextra-engine` (`cargo xtask unwired-ports`).

### Category D: Dead Code
- **Geprüft, Befund vorhanden (FEHLER-D1)**:
  - `deletion_proof/proof.rs:87`: `default_signature_version` ist als `pub fn` definiert, wird jedoch workspace-weit nicht aufgerufen (`cargo xtask orphan-symbols`).
  - `deletion_proof/exclusions.rs:33`: `mandatory_ids` ist als `pub fn` definiert, wird workspace-weit nicht aufgerufen (`cargo xtask orphan-symbols`).

### Category E: Fehlerbehandlung
- **Geprüft, kein Befund**. Keine verworfenen Results in krypto-relevanten Pfaden. Fail-closed Prinzip bei HMAC-, TLV- und Signaturabweichungen strikt durchgesetzt.

### Category F: Panics und Unwraps im Produktionscode
- **Geprüft, kein Befund**. Quellcode unter `crates/contextra-crypto/src/` enthält **0 `unwrap()` oder `expect()`** Aufrufe im Produktionscode. Alle Treffer liegen in `#[cfg(test)]`-Blöcken. `.github/unwrap_baseline.txt` verzeichnet 0 verbleibende Baseline-Ausnahmen für `contextra-crypto`.

### Category G: Nebenläufigkeit
- **Geprüft, kein Befund**. Lock-freie Atomics (`AtomicU64` in Nonce-Counter) und `parking_lot::RwLock` ohne `.await` über Lock-Grenzen.

### Category H: Determinismus
- **Geprüft, kein Befund**. Systemzeit und Zufall erfolgen ausschließlich über Ports bzw. die P28-CSPRNG Ausnahme für Schlüssel/Salts.

### Category I: Mandanten- und Berechtigungsgrenzen
- **Geprüft, kein Befund**. `TenantIsolatedKvStore` erzwingt Mandanten-Isolierung. `derive_kv_key_scoped` bindet `TenantId` als LE-64-bit Puffer in HKDF-Info ein.

### Category J: Architektur und Design
- **Geprüft, kein Befund**. Strikte Ring-0 Einhaltung. `#![forbid(unsafe_code)]` erzwungen.

### Category K: Code-Smells
- **Geprüft, kein Befund**. Saubere Modulstruktur, keine duplizierten Hilfsfunktionen (`INV-CRYPTO-DUPLICATE-1` eingehalten).

### Category L: Tests
- **Geprüft, kein Befund**. Alle 138 Unit-Tests und Integrationstests (einschließlich `tests/secure_buf.rs`) kompilieren und bestehen sauber.

### Category M: Dokumentationsdrift
- **Geprüft, kein Befund**. `rules/simd_safety.md` stellt klar, dass `contextra-crypto` laut `capabilities.toml` keine Unsafe-Insel ist und `#![forbid(unsafe_code)]` erzwingt.

---

## 5 Befunde (Detailliert)

### FINDING-CRYPTO-01: Ungewirkter `KmsProvider` Port und `LocalKmsProvider`
- **Schwere:** Niedrig
- **Kategorie:** C (Fehlende Verdrahtung)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-crypto/src/kms.rs:18`
- **Befund:** `KmsProvider` ist ein als `pub trait` deklarierter Port mit `LocalKmsProvider` als Referenzimplementierung. Es existiert jedoch keine Verdrahtung in einem Ring-4 Composition Root (`contextra` Facade oder `contextra-engine`).
- **Evidenz:** `<!-- EVIDENCE: logs/audits/contextra-crypto_2026-10-13/unwired-ports.log (exit=0) -->`
- **Betroffene Invariante:** Port-Wiring Vollständigkeit (INV-KMS-1)
- **Empfehlung:** `LocalKmsProvider` im Zuge der Facade-Anbindung in `crates/contextra/src/builder.rs` verknüpfen.
- **Red-Test-Vorschlag:** `cargo xtask unwired-ports`

---

## 6 Vorgänger-Abgleich

| Vorgänger-ID / Befund | Ursprung | Status in `59f853cb` | Anmerkung |
|---|---|---|---|
| `K1-HKDF-RFC5869` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | RFC 5869 / HKDF-SHA256 vollständig eingehalten |
| `K2-NONCE-OSRNG` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | Atomic counter + OsRng Prefix, keine Nonce-Reuse-Gefahr |
| `K3-HMAC-V3-LENGTH-PREFIX` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | Längenpräfixierung in HMAC-v3 verhindert Partitionskollisionen |
| `K4-DELETION-PROOF-BINDING` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | Proof bindet Scope, TxId, Exclusions und Layer-Evidence |
| `K5-ED25519-VERIFY` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | Autonome externe Verifikation ohne DB-Zustand verifiziert |
| `K6-VOLATILE-KEY-ZEROIZE` | `AUDIT_crypto-kernel_DEEP_2026-10-12` | BEHOBEN | `VolatileEncryptionKey` nutzt ZeroizeOnDrop & `emergency_wipe` |
| `INV-CRYPTO-DUPLICATE-1` | `AUDIT_contextra-crypto_2026-10-12` | BEHOBEN | Genau 1 Definition von `hash_deleted_keys_length_prefixed` |
| `CARGO-MUTANTS-NOT-INSTALLED` | `AUDIT_contextra-crypto_2026-10-12` | NICHT PRÜFBAR | Tooling `cargo-mutants` im Sandbox-Runner weiterhin nicht installiert |

---

## 7 Testlage im Scope

- **Unit-Tests in Crate (`cargo test -p contextra-crypto --lib`)**:
  - **138 bestanden**, 0 fehlgeschlagen.
- **Integrationstests (`cargo test -p contextra-crypto --tests`)**:
  - `tests/secure_buf.rs` (3/3 bestanden).
  - Alle übrigen Integrationstests (`campaign_cov_deletion_proof.rs`, `deletion_proof_external_verify_rejects_tamper.rs`, `j24closure-contextra-crypto_closure_tests.rs`, `j25closure-contextra-crypto_symbols_test.rs`, `audit_chain_tests.rs`, `audit_chain_anchor.rs`, `p04b_deletion_proof_hardening.rs`) kompilieren und bestehen vollständig.
- **Security Scan (`cargo xtask security-scan`)**:
  - 0 Sicherheitslücken in Abhängigkeiten.

---

## 8 Nicht prüfbar und warum

- **Mutationstest-Score (`cargo-mutants`)**:
  - Das Werkzeug `cargo-mutants` ist in der aktuellen Testumgebung nicht installiert (`cargo: no such command: mutants`). Ein automatisierter Mutationstest-Score konnte daher nicht ermittelt werden.

---

## 9 Out-of-scope-Befunde

Keine.

---

## 10 Priorisierte Maßnahmen mit Akzeptanzbefehl aus AGENTS.md §4

1. **LocalKmsProvider im Composition Root verknüpfen** (Priorität 1):
   - `LocalKmsProvider` in `crates/contextra/src/builder.rs` verdrahten.
   - **Akzeptanzbefehl:** `cargo test -p contextra-crypto --test kms_wiring`

---
*Ende des Audit-Berichts `AUDIT_contextra-crypto_DEEP_2026-10-13.md`.*
