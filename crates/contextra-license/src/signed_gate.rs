//! Cryptographic signed license gate implementation (§10.2, §14.6, §15).

use std::collections::BTreeMap;
use std::sync::Arc;

use bincode::Options;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_types::TenantId;
pub use ed25519_dalek::VerifyingKey;
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle::ConstantTimeEq;

mod bytes_64 {
    use super::*;

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = serde::Deserialize::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes for signature"))
    }
}

/// Local installation identity wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstallationId([u8; 32]);

impl InstallationId {
    /// Creates a new [`InstallationId`] from a 32-byte hash array.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns an unbound installation identifier hash (`[0u8; 32]`) representing an unbound state
    /// that never matches an installation binding.
    pub const fn unbound() -> Self {
        Self([0u8; 32])
    }

    /// Returns the underlying 32-byte hash array.
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Returns a reference to the underlying 32-byte hash array.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Individual verifying key entry in a [`LicenseKeyring`] with key identifier and validity window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseKeyEntry {
    /// Unique identifier for this key (e.g. `"key-2026-v1"`).
    pub key_id: String,
    /// Ed25519 public verifying key.
    pub verifying_key: VerifyingKey,
    /// Optional Unix timestamp (seconds) starting from which this key is valid.
    pub valid_from_unix: Option<i64>,
    /// Optional Unix timestamp (seconds) until which this key is valid.
    pub valid_until_unix: Option<i64>,
    /// Flag indicating whether this key has been explicitly revoked.
    pub is_revoked: bool,
}

/// Managed collection of Ed25519 verifying keys supporting key rotation and revocation.
#[derive(Debug, Clone, Default)]
pub struct LicenseKeyring {
    keys: std::collections::BTreeMap<String, LicenseKeyEntry>,
}

impl LicenseKeyring {
    /// Constructs an empty [`LicenseKeyring`].
    pub fn new() -> Self {
        Self {
            keys: std::collections::BTreeMap::new(),
        }
    }

    /// Adds or updates a key entry in the keyring.
    pub fn add_key(&mut self, entry: LicenseKeyEntry) {
        self.keys.insert(entry.key_id.clone(), entry);
    }

    /// Revokes a key in the keyring by its `key_id`.
    pub fn revoke_key(&mut self, key_id: &str) {
        if let Some(entry) = self.keys.get_mut(key_id) {
            entry.is_revoked = true;
        }
    }

    /// Looks up an active verifying key by `key_id` at the given timestamp.
    ///
    /// # Errors
    /// Returns [`LicenseError::InvalidSignature`] if the key is unknown, revoked, or outside its validity window.
    pub fn resolve_key(&self, key_id: &str, now_unix: i64) -> Result<&VerifyingKey, LicenseError> {
        let entry = self
            .keys
            .get(key_id)
            .ok_or(LicenseError::InvalidSignature)?;

        if entry.is_revoked {
            return Err(LicenseError::InvalidSignature);
        }

        if let Some(from) = entry.valid_from_unix {
            if now_unix < from {
                return Err(LicenseError::InvalidSignature);
            }
        }

        if let Some(until) = entry.valid_until_unix {
            if now_unix >= until {
                return Err(LicenseError::InvalidSignature);
            }
        }

        Ok(&entry.verifying_key)
    }

    /// Returns an iterator over all key entries in the keyring.
    pub fn keys(&self) -> impl Iterator<Item = &LicenseKeyEntry> {
        self.keys.values()
    }
}

/// Signed activation record binding a feature ring activation to a specific machine/deployment installation hash (§10.2, §15).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedActivation {
    /// Activated feature ring.
    pub ring: FeatureRing,
    /// 32-byte hash of the target local installation identity.
    pub installation_id_hash: [u8; 32],
    /// Expiration Unix timestamp in seconds.
    pub expires_at_unix: i64,
    /// Ed25519 signature over `(ring, installation_id_hash, expires_at_unix)`.
    #[serde(with = "bytes_64")]
    pub signature: [u8; 64],
    /// Optional key ID identifying the signing key in a [`LicenseKeyring`].
    #[serde(default)]
    pub key_id: Option<String>,
}

impl SignedActivation {
    /// Computes the binary byte representation of the activation data fields
    /// `(ring, installation_id_hash, expires_at_unix)` used for Ed25519 signing and verification.
    ///
    /// # Errors
    /// Returns a [`bincode::Error`] if serialization of the activation fields fails.
    pub fn payload_bytes(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
    ) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&(ring, installation_id_hash, expires_at_unix))
    }

    /// Creates a signed activation record given data fields and an Ed25519 signing key.
    ///
    /// # Errors
    /// Returns a [`bincode::Error`] if serialization of the activation fields fails.
    pub fn create_signed(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self, bincode::Error> {
        let payload = Self::payload_bytes(ring, installation_id_hash, expires_at_unix)?;
        let signature = signing_key.sign(&payload).to_bytes();
        Ok(Self {
            ring,
            installation_id_hash,
            expires_at_unix,
            signature,
            key_id: None,
        })
    }

    /// Verifies whether the Ed25519 signature on this activation record is valid for the given verifying key.
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> bool {
        let Ok(payload) =
            Self::payload_bytes(self.ring, self.installation_id_hash, self.expires_at_unix)
        else {
            return false;
        };
        let sig = Signature::from_bytes(&self.signature);
        verifying_key.verify(&payload, &sig).is_ok()
    }
}

/// Determines or generates a local installation ID hash (`[u8; 32]`).
///
/// If a storage path is provided (`Some(path)`), this function reads the persisted installation
/// identifier from `path/installation.id` or generates and writes a new deterministic identifier
/// `inst-<hash>` to that file.
///
/// If no storage path is provided (`None`), this function returns `Ok(InstallationId::unbound().into_bytes())`,
/// representing an unbound state (`[0u8; 32]`) that does not match any installation binding.
///
/// # Errors
/// Returns an [`std::io::Error`] if directory creation or file writing fails at the specified `storage_path`.
///
/// # Note
/// The persisted identifier is scoped to the specified storage directory path and does not construct
/// hardware or machine-bound environment fingerprints.
pub fn derive_local_installation_id_hash(
    storage_path: Option<&std::path::Path>,
) -> Result<[u8; 32], std::io::Error> {
    if let Some(path) = storage_path {
        let id_file = path.join("installation.id");
        if let Ok(content) = std::fs::read_to_string(&id_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Ok(*blake3::hash(trimmed.as_bytes()).as_bytes());
            }
        }
        let generated = format!(
            "inst-{}",
            blake3::hash(path.to_string_lossy().as_bytes()).to_hex()
        );
        std::fs::create_dir_all(path)?;
        std::fs::write(&id_file, &generated)?;
        return Ok(*blake3::hash(generated.as_bytes()).as_bytes());
    }

    Ok(InstallationId::unbound().into_bytes())
}

/// License payload containing tenant binding, granted feature rings, optional expiration timestamp,
/// arbitrary feature flags, and optional installation ID hash (legacy schema).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    /// Tenant identity bound to this license.
    pub tenant_id: TenantId,
    /// List of feature rings activated by this license.
    pub allowed_rings: Vec<FeatureRing>,
    /// Optional Unix timestamp (in seconds) after which the license expires.
    pub expires_at: Option<i64>,
    /// Arbitrary key-value feature flags.
    pub feature_flags: BTreeMap<String, bool>,
    /// Optional 32-byte hash of a local machine or instance installation ID.
    pub installation_id_hash: Option<[u8; 32]>,
    /// Optional key ID identifying the signing key in a [`LicenseKeyring`].
    #[serde(default)]
    pub key_id: Option<String>,
}

/// Legacy payload schema without `installation_id_hash` for backwards compatibility with bincode 1.x.
#[derive(Deserialize)]
struct LegacyLicensePayload {
    tenant_id: TenantId,
    allowed_rings: Vec<FeatureRing>,
    expires_at: Option<i64>,
    feature_flags: BTreeMap<String, bool>,
}

impl From<LegacyLicensePayload> for LicensePayload {
    fn from(legacy: LegacyLicensePayload) -> Self {
        Self {
            tenant_id: legacy.tenant_id,
            allowed_rings: legacy.allowed_rings,
            expires_at: legacy.expires_at,
            feature_flags: legacy.feature_flags,
            installation_id_hash: None,
            key_id: None,
        }
    }
}

/// Alias for [`SignedLicenseGate`].
///
/// Verifies that the payload matches the supplied verifying key; it is not license enforcement.
pub type IntegrityGate = SignedLicenseGate;

/// Cryptographic license gate verifying Ed25519 signatures over activation records ([`SignedActivation`])
/// or legacy [`LicensePayload`].
///
/// Verifies that the payload matches the supplied verifying key; it is not license enforcement.
#[derive(Clone)]
pub struct SignedLicenseGate {
    verifying_key: Option<VerifyingKey>,
    keyring: Option<LicenseKeyring>,
    activation: Option<SignedActivation>,
    license_payload: Option<LicensePayload>,
    local_installation_id: Option<[u8; 32]>,
    clock: Arc<dyn Clock>,
    mode: crate::LicenseGateMode,
}

impl std::fmt::Debug for SignedLicenseGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedLicenseGate")
            .field("verifying_key", &self.verifying_key)
            .field("activation", &self.activation)
            .field("license_payload", &self.license_payload)
            .field("local_installation_id", &self.local_installation_id)
            .finish()
    }
}

impl SignedLicenseGate {
    /// Constructs a [`SignedLicenseGate`] without any active activation record.
    pub fn no_activation() -> Self {
        Self::no_activation_with_clock(Arc::new(SystemClock::new()))
    }

    /// Constructs an unactivated [`SignedLicenseGate`] with a custom [`Clock`] port.
    pub fn no_activation_with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            verifying_key: None,
            keyring: None,
            activation: None,
            license_payload: None,
            local_installation_id: None,
            clock,
            mode: crate::LicenseGateMode::Enforced,
        }
    }

    /// Constructs a [`SignedLicenseGate`] from a [`SignedActivation`].
    pub fn from_activation(activation: SignedActivation, verifying_key: VerifyingKey) -> Self {
        Self::from_activation_with_clock(activation, verifying_key, Arc::new(SystemClock::new()))
    }

    /// Constructs a [`SignedLicenseGate`] from a [`SignedActivation`] with an injected [`Clock`].
    pub fn from_activation_with_clock(
        activation: SignedActivation,
        verifying_key: VerifyingKey,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            verifying_key: Some(verifying_key),
            keyring: None,
            activation: Some(activation),
            license_payload: None,
            local_installation_id: None,
            clock,
            mode: crate::LicenseGateMode::Enforced,
        }
    }

    /// Attaches a [`LicenseKeyring`] to this gate for key rotation and revocation lookup.
    pub fn with_keyring(mut self, keyring: LicenseKeyring) -> Self {
        self.keyring = Some(keyring);
        self
    }

    /// Configures the [`LicenseGateMode`] for this gate instance.
    pub fn with_mode(mut self, mode: crate::LicenseGateMode) -> Self {
        self.mode = mode;
        self
    }

    /// Attaches a custom [`Clock`] port to this gate.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Returns the configured [`LicenseGateMode`].
    pub fn mode(&self) -> crate::LicenseGateMode {
        self.mode
    }

    /// Sets the [`SignedActivation`] for this gate.
    pub fn with_activation(mut self, activation: SignedActivation) -> Self {
        self.activation = Some(activation);
        self
    }

    /// Sets the [`LicensePayload`] for this gate.
    pub fn with_payload(mut self, payload: LicensePayload) -> Self {
        self.license_payload = Some(payload);
        self
    }

    /// Sets the public Ed25519 [`VerifyingKey`] for this gate.
    pub fn with_verifying_key(mut self, verifying_key: VerifyingKey) -> Self {
        self.verifying_key = Some(verifying_key);
        self
    }

    /// Constructs a [`SignedLicenseGate`] by verifying an Ed25519 signature over `payload_bytes`
    /// using standard system wall-clock.
    ///
    /// # Errors
    /// Returns [`LicenseError::InvalidSignature`] if signature verification or payload deserialization fails.
    pub fn from_signed_payload(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key: VerifyingKey,
    ) -> Result<Self, LicenseError> {
        Self::from_signed_payload_with_clock(
            payload_bytes,
            signature,
            verifying_key,
            Arc::new(SystemClock::new()),
        )
    }

    /// Constructs a [`SignedLicenseGate`] with an injected [`Clock`] instance for deterministic testing (P28).
    ///
    /// # Errors
    /// Returns [`LicenseError::InvalidSignature`] if signature verification or payload deserialization fails.
    pub fn from_signed_payload_with_clock(
        payload_bytes: &[u8],
        signature: &[u8; 64],
        verifying_key: VerifyingKey,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, LicenseError> {
        let sig = Signature::from_bytes(signature);
        verifying_key
            .verify(payload_bytes, &sig)
            .map_err(|_| LicenseError::InvalidSignature)?;

        let bincode_opts = bincode::options()
            .with_fixint_encoding()
            .reject_trailing_bytes();

        if let Ok(license_payload) = bincode_opts.deserialize::<LicensePayload>(payload_bytes) {
            return Ok(Self {
                verifying_key: Some(verifying_key),
                keyring: None,
                activation: None,
                license_payload: Some(license_payload),
                local_installation_id: None,
                clock,
                mode: crate::LicenseGateMode::Enforced,
            });
        }

        if let Ok(legacy) = bincode_opts.deserialize::<LegacyLicensePayload>(payload_bytes) {
            return Ok(Self {
                verifying_key: Some(verifying_key),
                keyring: None,
                activation: None,
                license_payload: Some(legacy.into()),
                local_installation_id: None,
                clock,
                mode: crate::LicenseGateMode::Enforced,
            });
        }

        Err(LicenseError::InvalidSignature)
    }

    /// Binds a local installation ID hash (e.g. BLAKE3 hash of machine ID or MAC address) to this gate instance.
    pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self {
        self.local_installation_id = Some(id_hash);
        self
    }

    /// Returns a reference to the verified [`LicensePayload`], if present.
    pub fn license_payload(&self) -> Option<&LicensePayload> {
        self.license_payload.as_ref()
    }

    /// Returns a reference to the verified [`SignedActivation`], if present.
    pub fn activation(&self) -> Option<&SignedActivation> {
        self.activation.as_ref()
    }

    /// Returns a reference to the public Ed25519 verifying key, if present.
    pub fn verifying_key(&self) -> Option<&VerifyingKey> {
        self.verifying_key.as_ref()
    }

    /// Generates a valid signed payload, signature, and verifying key bytes for testing purposes.
    ///
    /// # Errors
    /// Returns a [`bincode::Error`] if payload serialization fails.
    #[allow(clippy::type_complexity)]
    #[cfg(any(test, feature = "test-support"))]
    pub fn create_test_signed_payload(
        rings: Vec<FeatureRing>,
        expires_at: Option<i64>,
    ) -> Result<(Vec<u8>, [u8; 64], [u8; 32]), bincode::Error> {
        let secret_bytes = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key_bytes = signing_key.verifying_key().to_bytes();

        let payload = LicensePayload {
            tenant_id: TenantId::SYSTEM,
            allowed_rings: rings,
            expires_at,
            feature_flags: BTreeMap::new(),
            installation_id_hash: None,
            key_id: None,
        };

        let payload_bytes = bincode::serialize(&payload)?;
        let signature = signing_key.sign(&payload_bytes).to_bytes();

        Ok((payload_bytes, signature, verifying_key_bytes))
    }
}

impl LicenseGate for SignedLicenseGate {
    /// Checks whether the requested [`FeatureRing`] is authorized under this gate instance.
    ///
    /// # Legacy Path Handling
    /// If the gate was constructed from a legacy [`LicensePayload`] without an `installation_id_hash`,
    /// the installation binding check is skipped with a warning log. This legacy fallback exists
    /// strictly for backwards compatibility with legacy payloads and does not enforce installation binding.
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
        // (1) Step 1: Fast Ring Bypass - INVARIANTE INV-LICENSE-2
        // Ring Fast always returns Ok unconditionally, even with corrupt, missing, or expired activation.
        if ring == FeatureRing::Fast {
            return Ok(());
        }

        // (2) Step 2: Missing activation check
        if self.activation.is_none() && self.license_payload.is_none() {
            return Err(LicenseError::NotActivated(ring));
        }

        // (3) Step 3: Installation ID hash mismatch check (constant time via subtle::ConstantTimeEq, fail-closed without leaking details)
        if let Some(ref act) = self.activation {
            match self.local_installation_id {
                Some(local_hash) if local_hash.ct_eq(&act.installation_id_hash).into() => {}
                _ => return Err(LicenseError::NotActivated(ring)),
            }
        } else if let Some(ref payload) = self.license_payload {
            if let Some(ref expected_hash) = payload.installation_id_hash {
                match self.local_installation_id {
                    Some(local_hash) if local_hash.ct_eq(expected_hash).into() => {}
                    _ => return Err(LicenseError::NotActivated(ring)),
                }
            } else {
                // Legacy path: payloads created without installation_id_hash skip local installation verification.
                tracing::warn!(
                    "LicensePayload has no installation_id_hash (legacy payload); skipping installation ID check"
                );
            }
        }

        // (4) Step 4: Keyring / Key Resolution and Signature Verification
        let now_secs = (self.clock.now_unix_nanos() / 1_000_000_000) as i64;

        let active_key_id = self
            .activation
            .as_ref()
            .and_then(|a| a.key_id.as_deref())
            .or_else(|| {
                self.license_payload
                    .as_ref()
                    .and_then(|p| p.key_id.as_deref())
            });

        let resolved_key: Option<&VerifyingKey> = if let Some(key_id) = active_key_id {
            if let Some(ref keyring) = self.keyring {
                Some(keyring.resolve_key(key_id, now_secs)?)
            } else if let Some(ref vk) = self.verifying_key {
                Some(vk)
            } else {
                return Err(LicenseError::InvalidSignature);
            }
        } else if let Some(ref vk) = self.verifying_key {
            Some(vk)
        } else if let Some(ref keyring) = self.keyring {
            keyring.resolve_key("default", now_secs).ok().or_else(|| {
                keyring
                    .keys()
                    .find_map(|k| keyring.resolve_key(&k.key_id, now_secs).ok())
            })
        } else {
            None
        };

        if let Some(ref act) = self.activation {
            let sig_valid = match resolved_key {
                Some(vk) => act.verify_signature(vk),
                None => false,
            };
            if !sig_valid {
                return Err(LicenseError::InvalidSignature);
            }
        } else if self.license_payload.is_some() && resolved_key.is_none() {
            return Err(LicenseError::InvalidSignature);
        }

        // (5) Step 5: Expiration check using clock port (P28)
        let expires_at = if let Some(ref act) = self.activation {
            act.expires_at_unix
        } else if let Some(ref payload) = self.license_payload {
            payload.expires_at.unwrap_or(i64::MAX)
        } else {
            i64::MAX
        };

        if now_secs >= expires_at {
            return Err(LicenseError::Expired(expires_at));
        }

        // (6) Step 6: Ring level requirement check (explicit allowed_rings / ring matching, no automatic hierarchy)
        if let Some(ref act) = self.activation {
            if act.ring != ring {
                return Err(LicenseError::NotActivated(ring));
            }
        } else if let Some(ref payload) = self.license_payload {
            if !payload.allowed_rings.contains(&ring) {
                return Err(LicenseError::NotActivated(ring));
            }
        }

        // (7) Step 7: Success
        Ok(())
    }
}
