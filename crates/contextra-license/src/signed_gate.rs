//! Cryptographic signed license gate implementation (§14.6, §15).

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_types::TenantId;
pub use ed25519_dalek::VerifyingKey;
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};

/// License payload containing tenant binding, granted feature rings, optional expiration timestamp,
/// arbitrary feature flags, and optional installation ID hash.
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

/// Cryptographic license gate verifying Ed25519 signatures over serialized [`LicensePayload`].
///
/// Note: License verification acts as an auditable compliance gate to prevent license drift in commercial deployments,
/// not as copy protection/DRM. In open-source software under MIT/Apache 2.0, client-side checks can be bypassed by
/// custom implementations or forks (see Section 5 of `docs/refactor/license-binding-decision.md`).
#[derive(Clone)]
pub struct SignedLicenseGate {
    verifying_key: VerifyingKey,
    license_payload: LicensePayload,
    local_installation_id: Option<[u8; 32]>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for SignedLicenseGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedLicenseGate")
            .field("verifying_key", &self.verifying_key)
            .field("license_payload", &self.license_payload)
            .field("local_installation_id", &self.local_installation_id)
            .finish()
    }
}

impl SignedLicenseGate {
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

        // bincode 1.x is positional and non-self-describing. Try deserializing the current
        // LicensePayload schema first. If that fails (e.g. legacy payload bytes ended before the
        // new installation_id_hash field), fallback to LegacyLicensePayload and map it with
        // installation_id_hash: None. Newer readers deserializing older payload bytes would otherwise
        // fail if not explicitly caught and handled via a positional fallback struct.
        let license_payload: LicensePayload = bincode::deserialize(payload_bytes)
            .or_else(|_| {
                bincode::deserialize::<LegacyLicensePayload>(payload_bytes).map(Into::into)
            })
            .map_err(|_| LicenseError::InvalidSignature)?;

        Ok(Self {
            verifying_key,
            license_payload,
            local_installation_id: None,
            clock,
        })
    }

    /// Binds a local installation ID hash (e.g. BLAKE3 hash of machine ID or MAC address) to this gate instance.
    pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self {
        self.local_installation_id = Some(id_hash);
        self
    }

    /// Returns a reference to the verified [`LicensePayload`].
    pub fn license_payload(&self) -> &LicensePayload {
        &self.license_payload
    }

    /// Returns a reference to the public Ed25519 verifying key.
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
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
        // INVARIANTE INV-LICENSE-2: The Fast ring must always pass unconditionally.
        if ring == FeatureRing::Fast {
            return Ok(());
        }

        if !self.license_payload.allowed_rings.contains(&ring) {
            return Err(LicenseError::NotActivated(ring));
        }

        if let Some(expires_at) = self.license_payload.expires_at {
            let now_secs = (self.clock.now_unix_nanos() / 1_000_000_000) as i64;
            if now_secs >= expires_at {
                return Err(LicenseError::Expired(expires_at));
            }
        }

        if let Some(expected_hash) = self.license_payload.installation_id_hash {
            match self.local_installation_id {
                Some(local_hash) if local_hash == expected_hash => {}
                _ => return Err(LicenseError::NotActivated(ring)),
            }
        }

        Ok(())
    }
}
