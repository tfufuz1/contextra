// FILE-CONTEXT
// ZWECK: Versionierter Key Derivation Function (KDF) Header und Argon2id Schlüsselableitung für Passphrasen.
// INVARIANTEN: OWASP Argon2id Parameter-Standards (m_cost >= 19456, t_cost >= 2, p_cost >= 1). Binär-Format KdfHeader total geparst (keine Panics).
// NICHT-OFFENSICHTLICH: Zeroizing für abgeleitete Schlüssel. KdfHeader binary layout: magic (4B "MFKD") | version (1B = 1) | kdf_id (1B = 1 Argon2id) | m_cost_kib (4B BE) | t_cost (4B BE) | p_cost (4B BE) | salt_len (4B BE) | salt (>=16B).
// HOTSPOTS: [KdfHeader::from_bytes, derive_key_argon2id]

//! Versionierter KDF-Header und Argon2id-Schlüsselableitung für `memfuse-crypto`.
//!
//! # Geplante Migration
//! AI-NOTE[KDF-MIGRATION][OPEN] AGT-CRYPTO-002: Geplante Migration für Aufrufer in `memfuse-store` (`recovery.rs` / LSM-Recovery):
//! 1. Beim Erstellen eines neuen verschlüsselten Datenverzeichnisses wird ein `KdfHeader` generiert und als
//!    `kdf_header.bin` im Metadaten-Verzeichnis abgelegt.
//! 2. Beim Öffnen / Recovery wird geprüft, ob `kdf_header.bin` existiert.
//!    - Wenn ja: Schlüssel per `KeyManager::try_new_with_kdf(passphrase, &header)` ableiten.
//!    - Wenn nein (Bestandsdaten): Abwärtskompatibler HKDF-Fallback per `KeyManager::try_new(passphrase, salt)`.

#![forbid(unsafe_code)]

use crate::error::{CryptoError, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::ZeroizeOnDrop;

/// Magischer Header-Marker "MFKD" (MemFuse Key Derivation)
pub const KDF_HEADER_MAGIC: &[u8; 4] = b"MFKD";

/// Aktuelle KDF Header Version
pub const KDF_HEADER_VERSION_1: u8 = 1;

/// Identifikator für Argon2id
pub const KDF_ID_ARGON2ID: u8 = 1;

/// Min. empfohlener Salt-Umfang (16 Bytes)
pub const MIN_SALT_LEN: usize = 16;

/// OWASP Mindestanforderungen für Argon2id
pub const MIN_M_COST_KIB: u32 = 19456; // ~19 MiB
pub const MIN_T_COST: u32 = 2;
pub const MIN_P_COST: u32 = 1;

/// OWASP Standardempfehlungen für Argon2id
pub const DEFAULT_M_COST_KIB: u32 = 65536; // 64 MiB
pub const DEFAULT_T_COST: u32 = 3;
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
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);
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
    /// Garantiert Panic-Freiheit (total function) bei beliebigen Byte-Eingaben.
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

        if &bytes[0..4] != KDF_HEADER_MAGIC {
            return Err(CryptoError::InvalidInput(
                "Ungültige KdfHeader Magic Bytes".to_string(),
            ));
        }

        let version = bytes[4];
        if version != KDF_HEADER_VERSION_1 {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte KdfHeader Version: {}",
                version
            )));
        }

        let kdf_id = bytes[5];
        if kdf_id != KDF_ID_ARGON2ID {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte KdfId: {}",
                kdf_id
            )));
        }

        let m_cost_kib = u32::from_be_bytes(bytes[6..10].try_into().map_err(|_| {
            CryptoError::InvalidInput("Fehler beim Lesen von m_cost_kib".to_string())
        })?);
        let t_cost =
            u32::from_be_bytes(bytes[10..14].try_into().map_err(|_| {
                CryptoError::InvalidInput("Fehler beim Lesen von t_cost".to_string())
            })?);
        let p_cost =
            u32::from_be_bytes(bytes[14..18].try_into().map_err(|_| {
                CryptoError::InvalidInput("Fehler beim Lesen von p_cost".to_string())
            })?);

        let salt_len =
            u32::from_be_bytes(bytes[18..22].try_into().map_err(|_| {
                CryptoError::InvalidInput("Fehler beim Lesen von salt_len".to_string())
            })?) as usize;

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

        let salt = bytes[MIN_HEADER_LEN..MIN_HEADER_LEN + salt_len].to_vec();

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

/// Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird.
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey(pub [u8; 32]);

/// Leitet einen 256-Bit (32 Byte) Schlüssel aus einer Passphrase und einem `KdfHeader` via Argon2id ab.
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
