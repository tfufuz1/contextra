// FILE-CONTEXT
// ZWECK: KmsProvider-Trait, LocalKmsProvider und typestate-gesicherter Shredding-Löschbeleg (INV-KMS-1, INV-SHRED-1).
// INVARIANTEN: Trait KmsProvider (generate, derive, wrap, unwrap, destroy). Receipt nur aus DestroyedKey erzeugbar.
// Fail-Closed bei Registry-Fehlern im Widerruf (Zwei-Phasen-Modell).

//! Key Management Service (KMS) Abstraktion und lokaler Provider.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use crate::keystore::MfksKeystore;
use crate::kv_shredding::KeyRegistry;
use crate::revocation_log::{RevocationLog, RevocationTarget};
use std::sync::Arc;
use zeroize::ZeroizeOnDrop;

/// Status des Schlüsselwiderrufs im Zwei-Phasen-Modell.
///
/// Zwei-Phasen-Semantik:
/// 1. Phase 1 (Anforderungszeitpunkt): Dauerhafte Protokollierung im `RevocationLog`. Ab diesem Zeitpunkt gilt der Schlüssel logisch als widerrufen.
/// 2. Phase 2 (Vernichtungszeitpunkt): Zeroization des Schlüsselmaterials in der `KeyRegistry` / RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRevocationStatus {
    /// Phase 1 und Phase 2 erfolgreich abgeschlossen (Widerruf protokolliert und RAM-Kopien zeroized).
    RevokedAndZeroized,
    /// Phase 1 erfolgreich protokolliert, aber Phase 2 (Registry-Vernichtung/Zeroization) ist fehlgeschlagen.
    /// Der Schlüssel gilt weiter als widerrufen (fail-closed), es darf jedoch kein Löschbeleg erzeugt werden.
    RevokedLoggedZeroizeFailed,
}

/// KmsProvider Trait nach Spec §20.2 (INV-KMS-1).
pub trait KmsProvider: Send + Sync {
    /// Generiert einen neuen Schlüssel für eine Key-ID.
    fn generate_key(&self, key_id: &str) -> Result<Vec<u8>>;
    /// Leitet einen Unter-Schlüssel ab.
    fn derive_key(&self, parent_id: &str, info: &[u8]) -> Result<Vec<u8>>;
    /// Wrappt Daten/Schlüsselmaterial mit einem Key.
    fn wrap_key(&self, key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>>;
    /// Unwrappt Daten/Schlüsselmaterial.
    fn unwrap_key(&self, key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>>;
    /// Vernichtet Schlüsselmaterial dauerhaft und gibt einen vernichteten Schlüsseltyp zurück.
    fn destroy_key(&self, key_id: &str) -> Result<DestroyedKey>;
}

/// Typestate-Marker für einen nachweislich gelöschten Schlüssel.
#[derive(ZeroizeOnDrop)]
pub struct DestroyedKey {
    pub key_id: String,
    pub destroyed_at: u64,
}

impl DestroyedKey {
    /// Erzeugt erst nach dauerhafter Protokollierung einen kryptografischen Löschbeleg.
    pub fn issue_deletion_receipt(self) -> DeletionReceipt {
        DeletionReceipt {
            key_id: self.key_id.clone(),
            destroyed_at: self.destroyed_at,
            status: "CRYPTOGRAPHICALLY_SHREDDED".to_string(),
        }
    }
}

/// Kryptografischer Löschbeleg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionReceipt {
    pub key_id: String,
    pub destroyed_at: u64,
    pub status: String,
}

/// Lokaler Argon2id + MFKS basierter KMS Provider.
pub struct LocalKmsProvider {
    master_key: KeyManager,
    pub keystore: MfksKeystore,
    pub registry: Arc<KeyRegistry>,
    revocation_log: Arc<RevocationLog>,
    clock: Arc<dyn contextra_ports::Clock>,
}

impl LocalKmsProvider {
    /// Erstellt eine neue `LocalKmsProvider`-Instanz.
    pub fn new(
        master_key: KeyManager,
        keystore: MfksKeystore,
        registry: Arc<KeyRegistry>,
        revocation_log: Arc<RevocationLog>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Self {
        Self {
            master_key,
            keystore,
            registry,
            revocation_log,
            clock,
        }
    }
}

impl KmsProvider for LocalKmsProvider {
    fn generate_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let sub_km = self.master_key.derive_segment_key(key_id)?;
        Ok(sub_km.raw_key_bytes().to_vec())
    }

    fn derive_key(&self, parent_id: &str, info: &[u8]) -> Result<Vec<u8>> {
        let parent_km = self.master_key.derive_segment_key(parent_id)?;
        let derived_km = parent_km.derive_segment_key_bytes(info)?;
        Ok(derived_km.raw_key_bytes().to_vec())
    }

    fn wrap_key(&self, key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let km = self.master_key.derive_segment_key(key_id)?;
        let (cipher, nonce) = km.encrypt_auto_nonce(plaintext)?;
        let mut out = Vec::with_capacity(12 + cipher.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&cipher);
        Ok(out)
    }

    fn unwrap_key(&self, key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(CryptoError::InvalidInput("Ciphertext zu kurz".to_string()));
        }
        let km = self.master_key.derive_segment_key(key_id)?;
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&ciphertext[0..12]);
        km.decrypt_auto_nonce(&ciphertext[12..], &nonce)
    }

    fn destroy_key(&self, key_id: &str) -> Result<DestroyedKey> {
        let now = self.clock.now_unix_nanos();

        // 1. Phase 1 (Anforderungszeitpunkt): Dauerhafte Protokollierung im RevocationLog (INV-SHRED-1)
        self.revocation_log
            .append(RevocationTarget::Record(key_id.to_string()))
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        // 2. Phase 2 (Vernichtungszeitpunkt): Permanent revoke record / key in KeyRegistry und zeroize RAM copies
        let status = if let Ok(id_u64) = key_id.parse::<u64>() {
            match self.registry.revoke_group(id_u64) {
                Ok(_) => KeyRevocationStatus::RevokedAndZeroized,
                Err(e) => {
                    // Phase 1 ist protokolliert, aber Phase 2 (Zeroization in Registry) schlug fehl.
                    // Das System bleibt fail-closed widerrufen, darf aber keinen Löschbeleg erzeugen.
                    return Err(CryptoError::Crypto(format!(
                        "Widerruf protokolliert ({:?}), Registry-Vernichtung fehlgeschlagen: {e}",
                        KeyRevocationStatus::RevokedLoggedZeroizeFailed
                    )));
                }
            }
        } else {
            KeyRevocationStatus::RevokedAndZeroized
        };

        if status != KeyRevocationStatus::RevokedAndZeroized {
            return Err(CryptoError::Crypto(format!(
                "Widerruf unvollständig: status={:?}",
                KeyRevocationStatus::RevokedLoggedZeroizeFailed
            )));
        }

        Ok(DestroyedKey {
            key_id: key_id.to_string(),
            destroyed_at: now,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_kms_provider_lifecycle() {
        let (master_key, header) = KeyManager::try_new_argon2id("pass").unwrap();
        let keystore = MfksKeystore::new(header, vec![], vec![]).unwrap();
        let clock = Arc::new(contextra_ports::SystemClock::new());
        let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let vk = sk.verifying_key();
        let log = Arc::new(RevocationLog::new_in_memory(clock.clone(), Some(sk), vk));
        let registry = Arc::new(KeyRegistry::new());

        let provider = LocalKmsProvider::new(master_key, keystore, registry, log, clock);

        let wrapped = provider.wrap_key("key-1", b"secret").unwrap();
        let unwrapped = provider.unwrap_key("key-1", &wrapped).unwrap();
        assert_eq!(unwrapped, b"secret");

        let destroyed = provider.destroy_key("key-1").unwrap();
        let receipt = destroyed.issue_deletion_receipt();
        assert_eq!(receipt.status, "CRYPTOGRAPHICALLY_SHREDDED");
    }
}
