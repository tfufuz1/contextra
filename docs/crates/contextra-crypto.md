> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-crypto — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-crypto`  
**Referenzdokumente:** `docs/SPEC.md` §4 (Schlüsselhierarchie), §5 (Löschkette), §13; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-crypto` ist der fundamentale kryptographische Dienstleister des Contextra-Systems. Es implementiert die gesamte kryptographische Pyramide für Authenticated Encryption with Associated Data (AEAD), deterministische Key Derivation (KDF) und Zeroize-geschützte Schlüsselverwaltung:
- **AEAD Verschlüsselung:** AES-256-GCM-SIV (RFC 8452) für Chiffrierung im Ruhezustand (Data at Rest) für die Datenklassen K und P.
- **Hierarchische Schlüsselableitung:** HKDF-SHA-256 (RFC 5869) mit kryptographischer Domänentrennung.
- **Passphrase KDF:** Argon2id (RFC 9106) für die Ableitung des MasterKey aus Passphrase und persistiertem Salt.
- **Crypto-Shredding Unterstützung:** Atomare Nullung (`zeroize`) von DEKs und SubjectKeys für den DSGVO-Löschpfad.

### Beseitigung von Ebenenverstößen & Altlasten (R1, R3, R4)
1. **Keine Abhängigkeiten auf E2:** Die optionalen Abhängigkeiten auf `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden **vollständig entfernt**.
2. **Workspace-Lints einbinden:** Die lokaen Clippy-Overrides (`unwrap_used = "allow"`, `panic = "allow"`) werden entfernt und durch `[lints] workspace = true` ersetzt (INV-4 Zero-Panic).
3. **Leere Default-Features (R3):** `default = []` (Entfernung von `default = ["kv-encryption"]`).
4. **Altlasten-Bereinigung (`AP-21`):** Entfernung aller Referenzen auf `contextra-kvcache`, `contextra-infer-candle` in `lib.rs` und `kv_cipher.rs`.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-11)
1. **`#![forbid(unsafe_code)]`:** Vollständig sicheres Rust.
2. **Zero-Panic-Policy:** Keine Panics; alle kryptographischen Fehler liefern `CryptoError`.
3. **Fail-Closed (SPEC §4.3):** Fehlt ein Schlüssel oder scheitert AEAD, bricht die Operation sofort mit Fehler ab. Es gibt **keine stillen Fallbacks**, keine XOR-Masken, keine deterministischen Notfallschlüssel.
4. **ZeroizeOnDrop (INV-11):** Jede Datenstruktur, die Schlüsselmaterial hält, implementiert `Zeroize` und `ZeroizeOnDrop`.
5. **Kein `Clone`, kein `Debug`, kein `Serialize` für Schlüssel:** Verhindert Speicherduplizierung und Log-Leaks.
6. **Fehlbare Zufallszahlen:** Nonces (96 Bit) werden ausnahmslos via `rand::rngs::OsRng::try_fill_bytes` generiert.

---

## 2. Schlüsselhierarchie & Domänentrennung (SPEC §4)

```
MasterKey (Passphrase → Argon2id → 256 Bit)
    │
    ├── HKDF-Expand(info="contextra/tenant/v1" || tenant_id)
    │       │
    │       └── TenantKEK(tenant_id)
    │               │
    │               ├── HKDF-Expand(info="contextra/collection/v1" || collection_id || epoch)
    │               │       │
    │               │       ├── HKDF-Expand(info="contextra/doc/v1" || doc_id || version)
    │               │       │       │
    │               │       │       └── DocumentDEK (Klasse K) ──► AEAD Payload
    │               │       │
    │               │       └── HKDF-Expand(info="contextra/subject/v1" || subject_ref)
    │               │               │
    │               │               └── SubjectKey(subject_ref)
    │               │                       │
    │               │                       └── HKDF-Expand(info="contextra/subject_doc/v1" || doc_id || version)
    │               │                               │
    │               │                               └── SubjectDocDEK (Klasse P) ──► AEAD P-Payload
    │               │
    │               └── (Epochenrotation bei Collection-Neuanlage nach Drop)
    │
    └── HKDF-Expand(info="contextra/erasure/v1")
            └── ErasureKey (für Journal, Ledger, Berichte)
```

---

## 3. Kanonische Datenstrukturen & Algorithmen

### 3.1 Schlüsseltypen (`secret.rs`)

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 256-Bit geheimer Schlüssel mit garantierter Speicherbereinigung.
/// Implementiert bewusst WEDER Clone, Debug NOCH Serialize.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; 32]);

impl SecretKey {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Stark typisierte Schlüssel-Rollen zur Verhinderung von Verwechslungen im Typsystem:
pub struct MasterKey(SecretKey);
pub struct TenantKek(SecretKey);
pub struct CollectionKek(SecretKey);
pub struct DocumentDek(SecretKey);
pub struct SubjectKey(SecretKey);
pub struct SubjectDocDek(SecretKey);
pub struct ErasureKey(SecretKey);

macro_rules! impl_key_wrapper {
    ($name:ident) => {
        impl $name {
            #[inline]
            pub const fn from_secret(key: SecretKey) -> Self {
                Self(key)
            }
            #[inline]
            pub fn as_bytes(&self) -> &[u8; 32] {
                self.0.as_bytes()
            }
        }
    };
}

impl_key_wrapper!(MasterKey);
impl_key_wrapper!(TenantKek);
impl_key_wrapper!(CollectionKek);
impl_key_wrapper!(DocumentDek);
impl_key_wrapper!(SubjectKey);
impl_key_wrapper!(SubjectDocDek);
impl_key_wrapper!(ErasureKey);
```

---

### 3.2 Master-Key Ableitung via Argon2id (`kdf.rs`)

```rust
use argon2::{Algorithm, Argon2, Params, Version};

/// Leitet den MasterKey aus einer Passphrase und einem 32-Byte Salt ab (SPEC §4.4).
/// Parameter: m=256MB, t=4, p=1.
pub fn derive_master_key(passphrase: &[u8], salt: &[u8; 32]) -> Result<MasterKey, CryptoError> {
    if passphrase.is_empty() {
        return Err(CryptoError::EmptyPassphrase);
    }

    let params = Params::new(
        256 * 1024, // 256 MB RAM
        4,          // 4 Iterationen
        1,          // 1 Thread / Parallelism
        Some(32),   // 32 Byte Key Output
    ).map_err(|e| CryptoError::Argon2Config(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = [0u8; 32];

    argon2
        .hash_password_into(passphrase, salt, &mut key_bytes)
        .map_err(|e| CryptoError::Argon2Derivation(e.to_string()))?;

    Ok(MasterKey::from_secret(SecretKey::from_bytes(key_bytes)))
}
```

---

### 3.3 HKDF-Schlüsselableitung (`kdf.rs`)

```rust
use hkdf::Hkdf;
use sha2::Sha256;
use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};

/// Leitet den TenantKEK aus dem MasterKey ab.
pub fn derive_tenant_kek(master: &MasterKey, tenant_id: TenantId) -> Result<TenantKek, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, master.as_bytes());
    let mut okm = [0u8; 32];
    let info = [b"contextra/tenant/v1".as_slice(), &tenant_id.as_u64().to_le_bytes()].concat();
    hk.expand(&info, &mut okm).map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(TenantKek::from_secret(SecretKey::from_bytes(okm)))
}

/// Leitet den CollectionKEK aus dem TenantKEK ab.
pub fn derive_collection_kek(
    tenant_kek: &TenantKek,
    collection_id: CollectionId,
    epoch: u64,
) -> Result<CollectionKek, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, tenant_kek.as_bytes());
    let mut okm = [0u8; 32];
    let info = [
        b"contextra/collection/v1".as_slice(),
        &collection_id.as_u64().to_le_bytes(),
        &epoch.to_le_bytes(),
    ].concat();
    hk.expand(&info, &mut okm).map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(CollectionKek::from_secret(SecretKey::from_bytes(okm)))
}

/// Leitet den DocumentDEK (Klasse K) aus dem CollectionKEK ab.
pub fn derive_document_dek(
    coll_kek: &CollectionKek,
    doc_id: DocId,
    version: u64,
) -> Result<DocumentDek, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, coll_kek.as_bytes());
    let mut okm = [0u8; 32];
    let info = [
        b"contextra/doc/v1".as_slice(),
        &doc_id.as_u64().to_le_bytes(),
        &version.to_le_bytes(),
    ].concat();
    hk.expand(&info, &mut okm).map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(DocumentDek::from_secret(SecretKey::from_bytes(okm)))
}

/// Leitet den SubjectKey (Klasse P) aus dem CollectionKEK ab.
pub fn derive_subject_key(
    coll_kek: &CollectionKek,
    subject_ref: &SubjectRef,
) -> Result<SubjectKey, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, coll_kek.as_bytes());
    let mut okm = [0u8; 32];
    let info = [
        b"contextra/subject/v1".as_slice(),
        subject_ref.as_bytes().as_slice(),
    ].concat();
    hk.expand(&info, &mut okm).map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(SubjectKey::from_secret(SecretKey::from_bytes(okm)))
}

/// Leitet den SubjectDocDEK (Klasse P) aus dem SubjectKey ab.
pub fn derive_subject_doc_dek(
    subject_key: &SubjectKey,
    doc_id: DocId,
    version: u64,
) -> Result<SubjectDocDek, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, subject_key.as_bytes());
    let mut okm = [0u8; 32];
    let info = [
        b"contextra/subject_doc/v1".as_slice(),
        &doc_id.as_u64().to_le_bytes(),
        &version.to_le_bytes(),
    ].concat();
    hk.expand(&info, &mut okm).map_err(|_| CryptoError::HkdfExpandFailed)?;
    Ok(SubjectDocDek::from_secret(SecretKey::from_bytes(okm)))
}
```

---

### 3.4 Authenticated Encryption mit AES-256-GCM-SIV (`aead.rs`)

```rust
use aes_gcm_siv::{
    aead::{Aead, KeyInit, Payload},
    Aes256GcmSiv, Nonce,
};
use contextra_ids::{CollectionId, DocId, TenantId};
use rand::RngCore;

/// AAD-Kontextkonstruktion (SPEC §4.3).
#[inline]
pub fn build_aad(tenant: TenantId, collection: CollectionId, doc: DocId, version: u64) -> [u8; 32] {
    let mut aad = [0u8; 32];
    aad[0..8].copy_from_slice(&tenant.as_u64().to_le_bytes());
    aad[8..16].copy_from_slice(&collection.as_u64().to_le_bytes());
    aad[16..24].copy_from_slice(&doc.as_u64().to_le_bytes());
    aad[24..32].copy_from_slice(&version.to_le_bytes());
    aad
}

/// Verschlüsselt Payload mit AES-256-GCM-SIV und bindet AAD ein.
/// Nonce (96-Bit) wird zwingend zufällig aus dem OS CSPRNG gezogen.
pub fn encrypt_aead(
    key: &SecretKey,
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256GcmSiv::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::CipherInit(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::RngFailure(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let payload = Payload {
        msg: plaintext,
        aad,
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Format: Nonce (12 Bytes) || Ciphertext mit Tag
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Entschlüsselt einen AES-256-GCM-SIV Chiffretext unter Validierung der AAD.
pub fn decrypt_aead(
    key: &SecretKey,
    aad: &[u8],
    encrypted_blob: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if encrypted_blob.len() < 12 + 16 {
        return Err(CryptoError::CiphertextTooShort);
    }

    let (nonce_bytes, ciphertext) = encrypted_blob.split_at(12);
    let cipher = Aes256GcmSiv::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::CipherInit(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let payload = Payload {
        msg: ciphertext,
        aad,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}
```

---

## 4. Fehlerbehandlung (`CryptoError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Passphrase cannot be empty")]
    EmptyPassphrase,

    #[error("Argon2 configuration error: {0}")]
    Argon2Config(String),

    #[error("Argon2 key derivation failed: {0}")]
    Argon2Derivation(String),

    #[error("HKDF expansion failed: output length invalid or key rejected")]
    HkdfExpandFailed,

    #[error("Cipher initialization failed: {0}")]
    CipherInit(String),

    #[error("OS CSPRNG failure: {0}")]
    RngFailure(String),

    #[error("AEAD encryption operation failed")]
    EncryptionFailed,

    #[error("AEAD decryption failed: tag mismatch, corrupted data, or invalid AAD context")]
    DecryptionFailed,

    #[error("Ciphertext blob too short (missing 12-byte nonce or 16-byte tag)")]
    CiphertextTooShort,

    #[error("Key slot verification failed: slot is not zeroed")]
    SlotNotZeroed,
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-crypto"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Cryptographic primitives, AES-256-GCM-SIV AEAD, HKDF key derivation, and Argon2id for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
thiserror = { workspace = true }
aes-gcm-siv = "0.11"
argon2 = { version = "0.5", default-features = false, features = ["alloc"] }
hkdf = "0.12"
sha2 = "0.10"
subtle = "2.6"
zeroize = { version = "1.8", features = ["zeroize_derive"] }
rand = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports` aus `[dependencies]` und `[dev-dependencies]`.
   - Entfernen der Features `workspace`, `types-compat`, `ports-compat`, `kv-encryption`, `cloud-egress-guard`.
   - `default = []` setzen (R3).
   - Clippy-Lints durch `[lints] workspace = true` ersetzen (Zero-Panic INV-4).
2. **Altlasten-Bereinigung (`AP-21`):**
   - Veraltete Bezüge auf `contextra-kvcache` und `contextra-infer-candle` in `src/lib.rs` und `src/kv_cipher.rs` entfernen.
3. **Schlüsselhierarchie-Implementierung:**
   - Typisierte Ableitungsfunktionen `derive_tenant_kek`, `derive_collection_kek`, `derive_document_dek`, `derive_subject_key`, `derive_subject_doc_dek` etablieren.
4. **Verifikation:**
   - `cargo test -p contextra-crypto`
