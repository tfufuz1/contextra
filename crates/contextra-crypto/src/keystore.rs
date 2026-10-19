// FILE-CONTEXT
// ZWECK: Keystore-Format MFKS (Master Format Keystore) für persistent gewrappte DEKs und KeyRegistry-Audit.
// INVARIANTEN: Binary layout: magic (4B "MFKS") | version (1B = 1) | kdf_len (4B BE) | kdf_bytes | deks_len (4B BE) | wrapped_deks | log_len (4B BE) | log_bytes.
// NICHT-OFFENSICHTLICH: HMAC-chained log integrity checks enforce fail-closed behavior on missing or altered log state.

//! Master Format Keystore (`MFKS`) Implementation.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use crate::kdf::KdfHeader;
use crate::kv_shredding::KeyRegistry;

/// Magischer Keystore-Marker "MFKS" (Master Format Keystore)
pub const MFKS_MAGIC: &[u8; 4] = b"MFKS";

/// Aktuelle MFKS Version
pub const MFKS_VERSION_1: u8 = 1;

/// Keystore Payload im MFKS-Format
#[derive(Debug, Clone)]
pub struct MfksKeystore {
    pub version: u8,
    pub kdf_header: KdfHeader,
    pub wrapped_deks: Vec<u8>,
    pub registry_log: Vec<u8>,
}

impl MfksKeystore {
    /// Erzeugt ein neues Keystore-Objekt.
    pub fn new(
        kdf_header: KdfHeader,
        wrapped_deks: Vec<u8>,
        registry_log: Vec<u8>,
    ) -> Result<Self> {
        if !kdf_header.meets_policy() {
            return Err(CryptoError::InvalidInput(
                "KdfHeader für MFKS Keystore erfüllt nicht die INV-KDF-1 Policy".to_string(),
            ));
        }
        Ok(Self {
            version: MFKS_VERSION_1,
            kdf_header,
            wrapped_deks,
            registry_log,
        })
    }

    /// Serialisiert das `MFKS` Keystore-Format in Bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let kdf_bytes = self.kdf_header.to_bytes();
        let mut out = Vec::with_capacity(
            4 + 1 + 4 + kdf_bytes.len() + 4 + self.wrapped_deks.len() + 4 + self.registry_log.len(),
        );
        out.extend_from_slice(MFKS_MAGIC);
        out.push(self.version);

        let kdf_len = u32::try_from(kdf_bytes.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&kdf_len.to_be_bytes());
        out.extend_from_slice(&kdf_bytes);

        let deks_len = u32::try_from(self.wrapped_deks.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&deks_len.to_be_bytes());
        out.extend_from_slice(&self.wrapped_deks);

        let log_len = u32::try_from(self.registry_log.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&log_len.to_be_bytes());
        out.extend_from_slice(&self.registry_log);

        out
    }

    /// Parst eine `MFKS` Keystore-Struktur aus Bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 17 {
            return Err(CryptoError::InvalidInput(
                "MFKS Keystore Bytes zu kurz".to_string(),
            ));
        }
        if &bytes[0..4] != MFKS_MAGIC {
            return Err(CryptoError::InvalidInput(
                "Ungültiger MFKS Keystore Magic Marker".to_string(),
            ));
        }
        let version = bytes[4];
        if version != MFKS_VERSION_1 {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte MFKS Keystore Version: {version}"
            )));
        }

        let kdf_len = u32::from_be_bytes(
            bytes[5..9]
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("KDF Len Read Fail".to_string()))?,
        ) as usize;

        if bytes.len() < 9 + kdf_len + 8 {
            return Err(CryptoError::InvalidInput(
                "Inkompletter MFKS KDF Header".to_string(),
            ));
        }

        let kdf_header = KdfHeader::from_bytes(&bytes[9..9 + kdf_len])?;

        let deks_offset = 9 + kdf_len;
        let deks_len = u32::from_be_bytes(
            bytes[deks_offset..deks_offset + 4]
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("DEKs Len Read Fail".to_string()))?,
        ) as usize;

        if bytes.len() < deks_offset + 4 + deks_len + 4 {
            return Err(CryptoError::InvalidInput(
                "Inkomplette MFKS DEKs".to_string(),
            ));
        }

        let wrapped_deks = bytes[deks_offset + 4..deks_offset + 4 + deks_len].to_vec();

        let log_offset = deks_offset + 4 + deks_len;
        let log_len = u32::from_be_bytes(
            bytes[log_offset..log_offset + 4]
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("Registry Log Len Read Fail".to_string()))?,
        ) as usize;

        if bytes.len() < log_offset + 4 + log_len {
            return Err(CryptoError::InvalidInput(
                "Inkompletter MFKS Registry Log".to_string(),
            ));
        }

        let registry_log = bytes[log_offset + 4..log_offset + 4 + log_len].to_vec();

        Ok(Self {
            version,
            kdf_header,
            wrapped_deks,
            registry_log,
        })
    }

    /// Entschlüsselt den Keystore mit der Passphrase und verifiziert gleichzeitig die Integrität des
    /// an das `KeyRegistry` angebundenen `RevocationLog`s.
    ///
    /// # Errors
    /// Returns [`CryptoError`] falls die Passphrase falsch ist, die KDF-Parameter nicht der Policy
    /// entsprechen oder das RevocationLog Manipulationen aufweist.
    pub fn open_and_verify(&self, passphrase: &str, registry: &KeyRegistry) -> Result<KeyManager> {
        let km = KeyManager::try_new_strict(passphrase, &self.kdf_header)?;
        if let Some(ref log) = registry.revocation_log {
            log.verify_integrity()?;
        }
        Ok(km)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_mfks_roundtrip_tamper_and_truncation() {
        let (_, header) = KeyManager::try_new_argon2id("passphrase").unwrap();
        let deks = vec![1, 2, 3, 4, 5];
        let log_data = vec![10, 20, 30];
        let keystore = MfksKeystore::new(header, deks.clone(), log_data.clone()).unwrap();

        let bytes = keystore.to_bytes();
        let parsed = MfksKeystore::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.version, MFKS_VERSION_1);
        assert_eq!(parsed.wrapped_deks, deks);
        assert_eq!(parsed.registry_log, log_data);

        // Tampered magic bytes
        let mut tampered = bytes.clone();
        tampered[0] = b'X';
        assert!(MfksKeystore::from_bytes(&tampered).is_err());

        // Truncated bytes
        for len in 0..bytes.len() - 1 {
            assert!(MfksKeystore::from_bytes(&bytes[..len]).is_err());
        }
    }

    #[test]
    fn test_mfks_keystore_open_and_verify() {
        let passphrase = "correct-passphrase-mfks";
        let (_, header) = KeyManager::try_new_argon2id(passphrase).unwrap();
        let keystore = MfksKeystore::new(header.clone(), vec![1, 2, 3], vec![4, 5, 6]).unwrap();
        let registry = KeyRegistry::new();

        // Positive path
        let open_res = keystore.open_and_verify(passphrase, &registry);
        assert!(open_res.is_ok());

        // Negative path: Corrupted KdfHeader policy failure MUST fail open_and_verify
        let mut weak_header = header;
        weak_header.params.m_cost_kib = 100; // violates MIN_M_COST_KIB (65536)
        let invalid_keystore = MfksKeystore {
            version: MFKS_VERSION_1,
            kdf_header: weak_header,
            wrapped_deks: vec![1, 2, 3],
            registry_log: vec![4, 5, 6],
        };
        let policy_err = invalid_keystore.open_and_verify(passphrase, &registry);
        assert!(policy_err.is_err());
    }
}
