// FILE-CONTEXT
// ZWECK: Wiring entry point KmsWiring for facade composition root (contextra/src/builder.rs).
// INVARIANTEN: Zero-panic, fail-closed on invalid KDF parameters or key derivation failures.

//! Composition root wiring module for KMS and key management.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::Result;
use crate::kdf::KdfHeader;
use crate::keystore::MfksKeystore;
use crate::kms::LocalKmsProvider;
use crate::kv_shredding::KeyRegistry;
use crate::revocation_log::RevocationLog;
use std::sync::Arc;

/// Input dependencies required to initialize KMS wiring.
pub struct KmsWiringDeps {
    /// Master passphrase used for key derivation.
    pub passphrase: String,
    /// KDF parameters and salt header.
    pub kdf_header: KdfHeader,
    /// Shared key registry for envelope crypto-shredding.
    pub registry: Arc<KeyRegistry>,
    /// Persistent revocation log.
    pub revocation_log: Arc<RevocationLog>,
    /// System clock port.
    pub clock: Arc<dyn contextra_ports::Clock>,
    /// Optional serialized wrapped DEKs for MFKS keystore.
    pub wrapped_deks: Option<Vec<u8>>,
    /// Optional serialized registry audit log for MFKS keystore.
    pub registry_log: Option<Vec<u8>>,
}

/// Productive KMS wiring container holding the initialized master `KeyManager` and `LocalKmsProvider`.
pub struct KmsWiring {
    /// Derived master key manager enforcing strict KDF policy.
    pub key_manager: KeyManager,
    /// Productive local KMS provider instance.
    pub kms_provider: Arc<LocalKmsProvider>,
}

impl KmsWiring {
    /// Builds `KmsWiring` enforcing strict INV-KDF-1 policy checks on `kdf_header`.
    pub fn build(deps: KmsWiringDeps) -> Result<Self> {
        let master_key = KeyManager::try_new_strict(&deps.passphrase, &deps.kdf_header)?;
        let keystore = MfksKeystore::new(
            deps.kdf_header,
            deps.wrapped_deks.unwrap_or_default(),
            deps.registry_log.unwrap_or_default(),
        )?;
        let kms_provider = Arc::new(LocalKmsProvider::new(
            master_key.clone(),
            keystore,
            deps.registry,
            deps.revocation_log,
            deps.clock,
        ));

        Ok(Self {
            key_manager: master_key,
            kms_provider,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]
mod tests {
    use super::*;
    use contextra_ports::SystemClock;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_kms_wiring_build_success() {
        let (master_km, header) = KeyManager::try_new_argon2id("wiring-passphrase-123456").unwrap();
        let clock = Arc::new(SystemClock::new());
        let sk = SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();
        let log = Arc::new(RevocationLog::new_in_memory(clock.clone(), Some(sk), vk));
        let registry = Arc::new(KeyRegistry::new());

        let deps = KmsWiringDeps {
            passphrase: "wiring-passphrase-123456".to_string(),
            kdf_header: header,
            registry,
            revocation_log: log,
            clock,
            wrapped_deks: None,
            registry_log: None,
        };

        let wiring = KmsWiring::build(deps).expect("build should succeed");
        assert_eq!(
            wiring.key_manager.inspect_key_bytes_for_test(),
            master_km.inspect_key_bytes_for_test()
        );
    }
}
