
//! Versionierter KDF-Header und Argon2id-Schlüsselableitung für `contextra-crypto`.
//!
//! # Geplante Migration
//! AI-NOTE\[KDF-MIGRATION\]\[OPEN\] AGT-CRYPTO-002: Geplante Migration für Aufrufer in `contextra-store` (`recovery.rs` / LSM-Recovery):
//! 1. Beim Erstellen eines neuen verschlüsselten Datenverzeichnisses wird ein `KdfHeader` generiert und als
//!    `kdf_header.bin` im Metadaten-Verzeichnis abgelegt.
//! 2. Beim Öffnen / Recovery wird geprüft, ob `kdf_header.bin` existiert.
//!    - Wenn ja: Schlüssel per `KeyManager::try_new_with_kdf(passphrase, &header)` ableiten.
//!    - Wenn nein (Bestandsdaten): Abwärtskompatibler HKDF-Fallback per `KeyManager::try_new(passphrase, salt)`.

#![forbid(unsafe_code)]

use crate::error::{CryptoError, Result};
use crate::secret::{
    CollectionKek, DocumentDek, ErasureKey, MasterKey, SecretKey32, SubjectDocDek, SubjectKey,
    TenantKek,
};
use argon2::{Algorithm, Argon2, Params, Version};
use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};
use zeroize::ZeroizeOnDrop;

/// Magischer Header-Marker "MFKD" (Contextra Key Derivation)
pub const KDF_HEADER_MAGIC: &[u8; 4] = b"MFKD";

/// Aktuelle KDF Header Version
pub const KDF_HEADER_VERSION_1: u8 = 1;

/// Identifikator für Argon2id
pub const KDF_ID_ARGON2ID: u8 = 1;

/// Min. empfohlener Salt-Umfang (16 Bytes)
pub const MIN_SALT_LEN: usize = 16;

/// OWASP Mindestanforderungen für Argon2id
pub const MIN_M_COST_KIB: u32 = 19456; // ~19 MiB
/// Minimale Anzahl an Iterationen (Zeitkosten) für Argon2id gemäß OWASP.
pub const MIN_T_COST: u32 = 2;
/// Minimaler Parallelitätsgrad für Argon2id gemäß OWASP.
pub const MIN_P_COST: u32 = 1;

/// OWASP Standardempfehlungen für Argon2id
pub const DEFAULT_M_COST_KIB: u32 = 65536; // 64 MiB
/// Standard-Zeitkosten (Iterationen) für Argon2id.
pub const DEFAULT_T_COST: u32 = 3;
/// Standard-Parallelitätsgrad für Argon2id.
pub const DEFAULT_P_COST: u32 = 1;

/// Konfiguration der Argon2id KDF-Kostenparameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KdfParams {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            m_cost_kib: DEFAULT_M_COST_KIB,
            t_cost: DEFAULT_T_COST,
            p_cost: DEFAULT_P_COST,
        }
    }
}

impl KdfParams {
    /// Erstellt neue `KdfParams` mit Validierung der Untergrenzen.
    pub fn new(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Result<Self> {
        if m_cost_kib < MIN_M_COST_KIB {
            return Err(CryptoError::InvalidInput(format!(
                "m_cost_kib ({m_cost_kib}) liegt unter dem Minimum von {MIN_M_COST_KIB} KiB"
            )));
        }
        if t_cost < MIN_T_COST {
            return Err(CryptoError::InvalidInput(format!(
                "t_cost ({t_cost}) liegt unter dem Minimum von {MIN_T_COST}"
            )));
        }
        if p_cost < MIN_P_COST {
            return Err(CryptoError::InvalidInput(format!(
                "p_cost ({p_cost}) liegt unter dem Minimum von {MIN_P_COST}"
            )));
        }
        Ok(Self {
            m_cost_kib,
            t_cost,
            p_cost,
        })
    }

    /// Erstellt `KdfParams` mit kleineren Kostenparametern ausschließlich für Testzwecke.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_for_test(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Self {
        Self {
            m_cost_kib,
            t_cost,
            p_cost,
        }
    }
}

/// Versionierter KDF-Header, der alle Parameter und den Salt für die Argon2id-Ableitung speichert.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Versionierter KDF-Metadaten-Header zur deterministischen Reproduktion von Schlüsseln.
///
/// Speichert die Argon2id-Kostenparameter (m_cost, t_cost, p_cost) und den Salt,
/// um verschlüsselte Datenbankdateien auch nach Server-Neustarts mit identischen
/// Parametern entsperren zu können.
///
/// ### Binär-Layout (Big-Endian):
/// | Offset | Länge | Feld | Beschreibung |
/// |---|---|---|---|
/// | 0..4 | 4B | Magic | "MFKD" (Contextra Key Derivation) |
/// | 4 | 1B | Version | Versionsnummer (aktuell 1) |
/// | 5 | 1B | KDF-ID | 1 = Argon2id |
/// | 6..10 | 4B | m_cost_kib | Speicherkosten in KiB (>= 19456) |
/// | 10..14 | 4B | t_cost | Iterationen / Zeitkosten (>= 2) |
/// | 14..18 | 4B | p_cost | Parallelitätsgrad (>= 1) |
/// | 18..22 | 4B | salt_len | Länge des Salts in Bytes |
/// | 22.. | variable | Salt | Kryptographischer Salt (>= 16B) |
pub struct KdfHeader {
    pub version: u8,
    pub kdf_id: u8,
    pub params: KdfParams,
    pub salt: Vec<u8>,
}

impl KdfHeader {
    /// Erstellt einen neuen `KdfHeader` v1 mit zufälligem 32-Byte-Salt und OWASP-Standardparametern.
    pub fn generate_default() -> Result<Self> {
        let mut salt = vec![0u8; 32];
        rand::RngCore::try_fill_bytes(&mut rand::rngs::OsRng, &mut salt)
            .map_err(|e| CryptoError::RngFailure(e.to_string()))?;
        Ok(Self {
            version: KDF_HEADER_VERSION_1,
            kdf_id: KDF_ID_ARGON2ID,
            params: KdfParams::default(),
            salt,
        })
    }

    /// Erstellt einen `KdfHeader` v1 mit vorgegebenen Parametern und Salt.
    pub fn new(params: KdfParams, salt: Vec<u8>) -> Result<Self> {
        if salt.len() < MIN_SALT_LEN {
            return Err(CryptoError::InvalidInput(format!(
                "Salt-Länge ({}) muss mindestens {} Bytes betragen",
                salt.len(),
                MIN_SALT_LEN
            )));
        }
        if salt.len() > 10_000 {
            return Err(CryptoError::InvalidInput(format!(
                "Salt-Länge ({}) überschreitet das Maximum von 10000 Bytes",
                salt.len()
            )));
        }
        Ok(Self {
            version: KDF_HEADER_VERSION_1,
            kdf_id: KDF_ID_ARGON2ID,
            params,
            salt,
        })
    }

    /// Serialisiert den `KdfHeader` in Binärform.
    ///
    /// Layout:
    /// `magic (4B "MFKD") | version (1B = 1) | kdf_id (1B = 1 Argon2id) | m_cost_kib (4B BE) | t_cost (4B BE) | p_cost (4B BE) | salt_len (4B BE) | salt (salt_len B)`
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + 1 + 1 + 4 + 4 + 4 + 4 + self.salt.len());
        buf.extend_from_slice(KDF_HEADER_MAGIC);
        buf.push(self.version);
        buf.push(self.kdf_id);
        buf.extend_from_slice(&self.params.m_cost_kib.to_be_bytes());
        buf.extend_from_slice(&self.params.t_cost.to_be_bytes());
        buf.extend_from_slice(&self.params.p_cost.to_be_bytes());
        buf.extend_from_slice(&(self.salt.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.salt);
        buf
    }

    /// Parst einen `KdfHeader` aus Binärdaten.
    ///
    /// Panic-Freiheit (total function) bei beliebigen Byte-Eingaben.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Minimal erforderliche Header-Länge ohne Salt: 4 (magic) + 1 (ver) + 1 (kdf_id) + 4*3 (params) + 4 (salt_len) = 22 Bytes
        const MIN_HEADER_LEN: usize = 22;

        if bytes.len() < MIN_HEADER_LEN {
            return Err(CryptoError::InvalidInput(format!(
                "KdfHeader-Bytes zu kurz: {} Bytes (Minimum: {} Bytes)",
                bytes.len(),
                MIN_HEADER_LEN
            )));
        }

        if bytes
            .get(0..4)
            .is_none_or(|magic| magic != KDF_HEADER_MAGIC)
        {
            return Err(CryptoError::InvalidInput(
                "Ungültige KdfHeader Magic Bytes".to_string(),
            ));
        }

        let version = bytes.get(4).copied().ok_or_else(|| {
            CryptoError::InvalidInput("Fehler beim Lesen von version".to_string())
        })?;
        if version != KDF_HEADER_VERSION_1 {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte KdfHeader Version: {}",
                version
            )));
        }

        let kdf_id = bytes
            .get(5)
            .copied()
            .ok_or_else(|| CryptoError::InvalidInput("Fehler beim Lesen von kdf_id".to_string()))?;
        if kdf_id != KDF_ID_ARGON2ID {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte KdfId: {}",
                kdf_id
            )));
        }

        let m_cost_kib = read_u32_be(bytes, 6, "m_cost_kib")?;
        let t_cost = read_u32_be(bytes, 10, "t_cost")?;
        let p_cost = read_u32_be(bytes, 14, "p_cost")?;
        let salt_len = read_u32_be(bytes, 18, "salt_len")? as usize;

        if salt_len < MIN_SALT_LEN {
            return Err(CryptoError::InvalidInput(format!(
                "Salt-Länge in Header ({salt_len}) unter Minimum ({MIN_SALT_LEN})"
            )));
        }

        if salt_len > 10_000 {
            return Err(CryptoError::InvalidInput(format!(
                "Salt-Länge in Header ({salt_len}) überschreitet Maximum von 10000 Bytes"
            )));
        }

        if bytes.len() < MIN_HEADER_LEN + salt_len {
            return Err(CryptoError::InvalidInput(format!(
                "KdfHeader unvollständig: Erwartet {} Bytes, vorhanden {} Bytes",
                MIN_HEADER_LEN + salt_len,
                bytes.len()
            )));
        }

        let salt = bytes
            .get(MIN_HEADER_LEN..MIN_HEADER_LEN + salt_len)
            .ok_or_else(|| CryptoError::InvalidInput("KdfHeader Salt unvollständig".to_string()))?
            .to_vec();

        Ok(Self {
            version,
            kdf_id,
            params: KdfParams {
                m_cost_kib,
                t_cost,
                p_cost,
            },
            salt,
        })
    }
}

/// Liest ein Big-Endian-`u32` ab `offset`, ohne Panic-Pfad.
fn read_u32_be(bytes: &[u8], offset: usize, field: &str) -> Result<u32> {
    bytes
        .get(offset..offset.saturating_add(4))
        .and_then(|s| <[u8; 4]>::try_from(s).ok())
        .map(u32::from_be_bytes)
        .ok_or_else(|| CryptoError::InvalidInput(format!("Fehler beim Lesen von {field}")))
}

/// Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird.
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey(pub [u8; 32]);

/// Leitet einen 256-Bit (32 Byte) Schlüssel aus einer Passphrase und einem `KdfHeader` via Argon2id ab.
/// Leitet einen kryptographisch starken Schlüssel mittels Argon2id aus einer Passphrase ab.
///
/// # Ablauf der Argon2id-Schlüsselableitung:
/// 1. Passphrase und Salt auf Mindestlänge prüfen.
/// 2. Kostenparameter gegen OWASP-Mindestgrenzen validieren.
/// 3. Argon2id-Instanz initialisieren und 32-Byte-Schlüssel ableiten.
/// 4. Temporäre Puffer löschen und Ergebnis im geschützten KeyManager kapseln.
pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey> {
    if passphrase.is_empty() {
        return Err(CryptoError::InvalidInput(
            "Passphrase darf nicht leer sein".to_string(),
        ));
    }

    let params = Params::new(
        header.params.m_cost_kib,
        header.params.t_cost,
        header.params.p_cost,
        Some(32),
    )
    .map_err(|e| CryptoError::KeyDerivation(format!("Ungültige Argon2id Parameter: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut derived_key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &header.salt, &mut derived_key)
        .map_err(|e| CryptoError::KeyDerivation(format!("Argon2id Hash-Fehler: {e}")))?;

    Ok(DerivedKey(derived_key))
}

/// HKDF-SHA256 expand.
pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<()> {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let hk = Hkdf::<Sha256>::from_prk(prk)
        .map_err(|_| CryptoError::Crypto("hkdf from_prk failed".into()))?;
    hk.expand(info, out)
        .map_err(|_| CryptoError::Crypto("hkdf expand failed".into()))?;
    Ok(())
}

// SECTION [epic=crypto-hierarchy,seq=2] Schlüsselhierarchie-Ableitung (Spec §3.2, §3.3)
// NOTE: Domänentrennung über versionierte Info-Strings ("contextra/…/v1"). Die bestehenden
// Funktionen `derive_key_argon2id` und `hkdf_expand` bleiben unverändert, weil persistierte
// Formate (INV-1, INV-6) von ihnen abhängen. Die Spec-Funktionen sind additiv.

/// Spec §3.2 Argon2id-Parameter: m = 256 MiB, t = 4, p = 1, 32 Byte Ausgabe.
const SPEC_ARGON2_M_COST_KIB: u32 = 256 * 1024;
const SPEC_ARGON2_T_COST: u32 = 4;
const SPEC_ARGON2_P_COST: u32 = 1;

/// Leitet den `MasterKey` aus Passphrase und 32-Byte-Salt ab (Spec §3.2).
///
/// Fail-Closed: leere Passphrase wird abgelehnt, es gibt keinen Fallback.
pub fn derive_master_key(passphrase: &[u8], salt: &[u8; 32]) -> Result<MasterKey> {
    if passphrase.is_empty() {
        return Err(CryptoError::EmptyPassphrase);
    }

    let params = Params::new(
        SPEC_ARGON2_M_COST_KIB,
        SPEC_ARGON2_T_COST,
        SPEC_ARGON2_P_COST,
        Some(32),
    )
    .map_err(|e| CryptoError::Argon2Config(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut okm = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut okm)
        .map_err(|e| CryptoError::Argon2Derivation(e.to_string()))?;

    let key = SecretKey32::from_bytes(okm);
    zeroize::Zeroize::zeroize(&mut okm);
    Ok(MasterKey::from_secret(key))
}

/// Gemeinsame HKDF-SHA-256-Expansion auf 32 Byte. Der Ausgabepuffer wird nach dem
/// Umkopieren in den `SecretKey32` genullt (INV-11).
fn hkdf_expand_32(ikm: &[u8; 32], info: &[u8]) -> Result<SecretKey32> {
    let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, ikm);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .map_err(|_| CryptoError::HkdfExpandFailed)?;
    let key = SecretKey32::from_bytes(okm);
    zeroize::Zeroize::zeroize(&mut okm);
    Ok(key)
}

/// Verkettet Info-Bestandteile zu einem HKDF-`info`-Feld.
fn info_parts(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// Leitet den `TenantKek` aus dem `MasterKey` ab (Spec §3.3).
pub fn derive_tenant_kek(master: &MasterKey, tenant_id: TenantId) -> Result<TenantKek> {
    let info = info_parts(&[b"contextra/tenant/v1", &tenant_id.as_u64().to_le_bytes()]);
    Ok(TenantKek::from_secret(hkdf_expand_32(
        master.as_bytes(),
        &info,
    )?))
}

/// Leitet den `CollectionKek` aus dem `TenantKek` ab. `epoch` erlaubt die Rotation bei Neuanlage.
pub fn derive_collection_kek(
    tenant_kek: &TenantKek,
    collection_id: CollectionId,
    epoch: u64,
) -> Result<CollectionKek> {
    let info = info_parts(&[
        b"contextra/collection/v1",
        &collection_id.as_u64().to_le_bytes(),
        &epoch.to_le_bytes(),
    ]);
    Ok(CollectionKek::from_secret(hkdf_expand_32(
        tenant_kek.as_bytes(),
        &info,
    )?))
}

/// Leitet den `DocumentDek` (Klasse K) aus dem `CollectionKek` ab.
pub fn derive_document_dek(
    coll_kek: &CollectionKek,
    doc_id: DocId,
    version: u64,
) -> Result<DocumentDek> {
    let info = info_parts(&[
        b"contextra/doc/v1",
        &doc_id.as_u64().to_le_bytes(),
        &version.to_le_bytes(),
    ]);
    Ok(DocumentDek::from_secret(hkdf_expand_32(
        coll_kek.as_bytes(),
        &info,
    )?))
}

/// Leitet den `SubjectKey` (Klasse P) aus dem `CollectionKek` ab.
pub fn derive_subject_key(
    coll_kek: &CollectionKek,
    subject_ref: &SubjectRef,
) -> Result<SubjectKey> {
    let info = info_parts(&[b"contextra/subject/v1", subject_ref.as_bytes()]);
    Ok(SubjectKey::from_secret(hkdf_expand_32(
        coll_kek.as_bytes(),
        &info,
    )?))
}

/// Leitet den `ErasureKey` direkt aus dem `MasterKey` ab (Spec §2, `contextra/erasure/v1`).
///
/// Der ErasureKey schützt Journal, Ledger und Berichte und ist unabhängig von der
/// Mandanten-/Collection-Hierarchie, damit Löschmaterial über Collection-Drops hinaus
/// verfügbar bleibt.
pub fn derive_erasure_key(master: &MasterKey) -> Result<ErasureKey> {
    let info = info_parts(&[b"contextra/erasure/v1"]);
    Ok(ErasureKey::from_secret(hkdf_expand_32(
        master.as_bytes(),
        &info,
    )?))
}

/// Leitet den `SubjectDocDek` (Klasse P) aus dem `SubjectKey` ab.
pub fn derive_subject_doc_dek(
    subject_key: &SubjectKey,
    doc_id: DocId,
    version: u64,
) -> Result<SubjectDocDek> {
    let info = info_parts(&[
        b"contextra/subject_doc/v1",
        &doc_id.as_u64().to_le_bytes(),
        &version.to_le_bytes(),
    ]);
    Ok(SubjectDocDek::from_secret(hkdf_expand_32(
        subject_key.as_bytes(),
        &info,
    )?))
}
// !SECTION
