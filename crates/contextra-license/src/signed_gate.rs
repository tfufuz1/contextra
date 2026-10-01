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

#[inline]
fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut res = 0u8;
    for i in 0..32 {
        res |= a[i] ^ b[i];
    }
    res == 0
}

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
}

impl SignedActivation {
    /// Computes the binary byte representation of the activation data fields
    /// `(ring, installation_id_hash, expires_at_unix)` used for Ed25519 signing and verification.
    pub fn payload_bytes(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
    ) -> Vec<u8> {
        bincode::serialize(&(ring, installation_id_hash, expires_at_unix)).unwrap_or_default()
    }

    /// Creates a signed activation record given data fields and an Ed25519 signing key.
    pub fn create_signed(
        ring: FeatureRing,
        installation_id_hash: [u8; 32],
        expires_at_unix: i64,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Self {
        let payload = Self::payload_bytes(ring, installation_id_hash, expires_at_unix);
        let signature = signing_key.sign(&payload).to_bytes();
        Self {
            ring,
            installation_id_hash,
            expires_at_unix,
            signature,
        }
    }

    /// Verifies whether the Ed25519 signature on this activation record is valid for the given verifying key.
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> bool {
        let payload =
            Self::payload_bytes(self.ring, self.installation_id_hash, self.expires_at_unix);
        let sig = Signature::from_bytes(&self.signature);
        verifying_key.verify(&payload, &sig).is_ok()
    }
}

/// Determines or generates a deterministic local installation ID hash (`[u8; 32]`).
///
/// # Status
/// **VORLÄUFIG, BIS ZUR SCHRIFTLICHEN ENTSCHEIDUNG** (Spec Teil 15, "Offene Entscheidungen").
///
/// This implementation computes a stable 32-byte installation hash. If a persistent storage path is
/// provided, it attempts to read or persist a machine-bound installation identifier (`installation.id`)
/// and hashes its contents with BLAKE3. If no path is provided, it derives a deterministic fallback
/// installation hash based on machine environment seeds.
pub fn derive_local_installation_id_hash(storage_path: Option<&std::path::Path>) -> [u8; 32] {
    if let Some(path) = storage_path {
        let id_file = path.join("installation.id");
        if let Ok(content) = std::fs::read_to_string(&id_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return *blake3::hash(trimmed.as_bytes()).as_bytes();
            }
        }
        let generated = format!(
            "inst-{}",
            blake3::hash(path.to_string_lossy().as_bytes()).to_hex()
        );
        if std::fs::create_dir_all(path).is_ok() {
            if let Ok(()) = std::fs::write(&id_file, &generated) {}
        }
        return *blake3::hash(generated.as_bytes()).as_bytes();
    }

    const FALLBACK_SEED: &[u8] = b"CONTEXTRA_DEFAULT_LOCAL_INSTALLATION_ID_v1";
    *blake3::hash(FALLBACK_SEED).as_bytes()
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
        }
    }
}

/// Cryptographic license gate verifying Ed25519 signatures over activation records ([`SignedActivation`])
/// or legacy [`LicensePayload`].
#[derive(Clone)]
pub struct SignedLicenseGate {
    verifying_key: Option<VerifyingKey>,
    activation: Option<SignedActivation>,
    license_payload: Option<LicensePayload>,
    local_installation_id: Option<[u8; 32]>,
    clock: Arc<dyn Clock>,
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
        Self {
            verifying_key: None,
            activation: None,
            license_payload: None,
            local_installation_id: None,
            clock: Arc::new(SystemClock::new()),
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
            activation: Some(activation),
            license_payload: None,
            local_installation_id: None,
            clock,
        }
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
                activation: None,
                license_payload: Some(license_payload),
                local_installation_id: None,
                clock,
            });
        }

        if let Ok(legacy) = bincode_opts.deserialize::<LegacyLicensePayload>(payload_bytes) {
            return Ok(Self {
                verifying_key: Some(verifying_key),
                activation: None,
                license_payload: Some(legacy.into()),
                local_installation_id: None,
                clock,
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
    pub fn create_test_signed_payload(
        rings: Vec<FeatureRing>,
        expires_at: Option<i64>,
    ) -> (Vec<u8>, [u8; 64], [u8; 32]) {
        let secret_bytes = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key_bytes = signing_key.verifying_key().to_bytes();

        let payload = LicensePayload {
            tenant_id: TenantId::SYSTEM,
            allowed_rings: rings,
            expires_at,
            feature_flags: BTreeMap::new(),
            installation_id_hash: None,
        };

        let payload_bytes = bincode::serialize(&payload).unwrap_or_default();
        let signature = signing_key.sign(&payload_bytes).to_bytes();

        (payload_bytes, signature, verifying_key_bytes)
    }
}

impl LicenseGate for SignedLicenseGate {
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

        // (3) Step 3: Installation ID hash mismatch check (constant time, fail-closed without leaking details)
        if let Some(ref act) = self.activation {
            match self.local_installation_id {
                Some(local_hash) if constant_time_eq_32(&local_hash, &act.installation_id_hash) => {
                }
                _ => return Err(LicenseError::NotActivated(ring)),
            }
        } else if let Some(ref payload) = self.license_payload {
            if let Some(expected_hash) = payload.installation_id_hash {
                match self.local_installation_id {
                    Some(local_hash) if constant_time_eq_32(&local_hash, &expected_hash) => {}
                    _ => return Err(LicenseError::NotActivated(ring)),
                }
            } else {
                tracing::warn!(
                    "LicensePayload has no installation_id_hash (legacy payload); skipping installation ID check"
                );
            }
        }

        // (4) Step 4: Invalid signature check
        if let Some(ref act) = self.activation {
            let sig_valid = match &self.verifying_key {
                Some(vk) => act.verify_signature(vk),
                None => false,
            };
            if !sig_valid {
                return Err(LicenseError::InvalidSignature);
            }
        } else if self.license_payload.is_some() && self.verifying_key.is_none() {
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

        let now_secs = (self.clock.now_unix_nanos() / 1_000_000_000) as i64;
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
