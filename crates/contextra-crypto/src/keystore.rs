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
            (17_usize)
                .saturating_add(kdf_bytes.len())
                .saturating_add(self.wrapped_deks.len())
                .saturating_add(self.registry_log.len()),
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
        if bytes.get(0..4) != Some(MFKS_MAGIC.as_slice()) {
            return Err(CryptoError::InvalidInput(
                "Ungültiger MFKS Keystore Magic Marker".to_string(),
            ));
        }
        let version = *bytes.get(4).ok_or_else(|| {
            CryptoError::InvalidInput("Fehler beim Lesen der MFKS Version".to_string())
        })?;
        if version != MFKS_VERSION_1 {
            return Err(CryptoError::InvalidInput(format!(
                "Nicht unterstützte MFKS Keystore Version: {version}"
            )));
        }

        let kdf_len = u32::from_be_bytes(
            bytes.get(5..9)
                .ok_or_else(|| CryptoError::InvalidInput("KDF Len Read Fail".to_string()))?
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("KDF Len Read Fail".to_string()))?,
        ) as usize;

        let kdf_end = (9_usize).saturating_add(kdf_len);
        if bytes.len() < kdf_end.saturating_add(8) {
            return Err(CryptoError::InvalidInput(
                "Inkompletter MFKS KDF Header".to_string(),
            ));
        }

        let kdf_slice = bytes.get(9..kdf_end).ok_or_else(|| {
            CryptoError::InvalidInput("Fehler beim Lesen des KDF Headers".to_string())
        })?;
        let kdf_header = KdfHeader::from_bytes(kdf_slice)?;

        let deks_offset = kdf_end;
        let deks_len_slice = bytes.get(deks_offset..deks_offset.saturating_add(4)).ok_or_else(|| {
            CryptoError::InvalidInput("DEKs Len Read Fail".to_string())
        })?;
        let deks_len = u32::from_be_bytes(
            deks_len_slice
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("DEKs Len Read Fail".to_string()))?,
        ) as usize;

        let deks_start = deks_offset.saturating_add(4);
        let deks_end = deks_start.saturating_add(deks_len);
        if bytes.len() < deks_end.saturating_add(4) {
            return Err(CryptoError::InvalidInput(
                "Inkomplette MFKS DEKs".to_string(),
            ));
        }

        let wrapped_deks = bytes.get(deks_start..deks_end).ok_or_else(|| {
            CryptoError::InvalidInput("Fehler beim Lesen der DEKs".to_string())
        })?.to_vec();

        let log_offset = deks_end;
        let log_len_slice = bytes.get(log_offset..log_offset.saturating_add(4)).ok_or_else(|| {
            CryptoError::InvalidInput("Registry Log Len Read Fail".to_string())
        })?;
        let log_len = u32::from_be_bytes(
            log_len_slice
                .try_into()
                .map_err(|_| CryptoError::InvalidInput("Registry Log Len Read Fail".to_string()))?,
        ) as usize;

        let log_start = log_offset.saturating_add(4);
        let log_end = log_start.saturating_add(log_len);
        if bytes.len() < log_end {
            return Err(CryptoError::InvalidInput(
                "Inkompletter MFKS Registry Log".to_string(),
            ));
        }

        let registry_log = bytes.get(log_start..log_end).ok_or_else(|| {
            CryptoError::InvalidInput("Fehler beim Lesen des Registry Logs".to_string())
        })?.to_vec();

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
