//! Schlüsseltypen und Provider-Port (SP-PORTS, E-02, E-74, E-77, SPEC §9.4).
use crate::error::Error;
use crate::ids::ClusterId;
use crate::traits::Entropy;
use core::fmt;
use std::time::Duration;
use zeroize::Zeroizing;

pub const KEY_PROVIDER_BATCH_MAX: usize = 256;

/// Opaque identifier for a provider-managed key-encryption key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KekId([u8; 16]);

impl KekId {
    pub const fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
    pub const fn as_bytes(&self) -> &[u8; 16] { &self.0 }
}

impl fmt::Debug for KekId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KekId(<redacted>)")
    }
}

/// Idempotency identifier for batched key creation (SPEC §9.4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchId([u8; 16]);

impl BatchId {
    pub const fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
    pub const fn as_bytes(&self) -> &[u8; 16] { &self.0 }
}

impl fmt::Debug for BatchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BatchId(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyBehavior {
    Immediate,
    Scheduled { max_delay: Duration },
}

/// Whether cluster/tomb keys may exist in provider-managed backups (E-77, SPEC §9.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeysInBackups {
    No,
    Bounded { until: Duration },
    Yes,
}

/// Assurance class of the underlying key storage medium (E-77, SPEC §9.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAssurance {
    Hardware,
    Kms,
    FilePassphrase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyProviderInfo {
    pub destroy: DestroyBehavior,
    pub keys_in_backups: KeysInBackups,
    pub assurance: KeyAssurance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Present,
    Disabled,
    Absent,
}

/// Result for one cluster in a `create_cluster_keys` batch.
#[derive(Debug)]
pub struct KeyCreateResult {
    pub cluster: ClusterId,
    pub result: Result<u32, Error>,
}

/// Result for one `(cluster, epoch)` in a `cluster_keys` batch.
#[derive(Debug)]
pub struct KeyLookup {
    pub cluster: ClusterId,
    pub epoch: u32,
    pub result: Result<Option<Key32>, Error>,
}

/// Provider-owned cluster metadata entry for orphan reconciliation (R9.1-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyListEntry {
    pub cluster: ClusterId,
    pub created_utc: u64,
}

/// Aggregate result of an idempotent destroy request. No key or cluster IDs
/// are included in this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyReceipt {
    pub requested_clusters: usize,
    pub absent_clusters: usize,
}

/// 32-Byte-Schlüssel. Zeroized bei Drop, Debug redigiert, nicht Clone.
pub struct Key32(Zeroizing<[u8; 32]>);

impl Key32 {
    pub fn from_array(raw: [u8; 32]) -> Self {
        Self(Zeroizing::new(raw))
    }
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Key32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key32(<redacted>)")
    }
}

pub trait KeyProvider: Send + Sync {
    /// Describes key destruction timing, backup coverage, and assurance class.
    fn info(&self) -> KeyProviderInfo;

    /// Returns an available KEK by opaque provider identifier.
    fn kek(&self, id: KekId) -> Result<Key32, Error>;

    /// Creates or returns durable key epochs for up to 256 clusters idempotently per `request_id`.
    fn create_cluster_keys(
        &self,
        request_id: BatchId,
        clusters: &[ClusterId],
        entropy: &dyn Entropy,
    ) -> Result<Vec<KeyCreateResult>, Error>;

    /// Fetches up to 256 `(cluster, epoch)` keys. `None` means the provider has
    /// no usable key; recovery must distinguish legitimate destruction from key loss.
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error>;

    /// Destroys all key epochs for up to 256 clusters per call, idempotently.
    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error>;

    /// Reports provider state for a specific cluster key epoch.
    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyState, Error>;

    /// Lists provider-owned cluster identifiers with creation timestamp (UTC seconds) for orphan reconciliation.
    fn list(&self) -> Result<Vec<KeyListEntry>, Error>;

    /// Creates a durable tomb epoch key (K-31). Idempotent.
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), Error>;

    /// Returns the tomb key for `epoch`, or `None` solely if that epoch has been retired.
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, Error>;

    /// Idempotently retires `epoch` and destroys its tomb key (`Absent` confirms completion).
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyState, Error>;

    /// Convenience helper for single-cluster creation using a fresh batch ID from `entropy`.
    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn Entropy) -> Result<u32, Error> {
        let mut raw_batch = [0u8; 16];
        entropy.fill(&mut raw_batch)?;
        let mut results = self.create_cluster_keys(BatchId::from_bytes(raw_batch), &[cluster], entropy)?;
        results
            .pop()
            .ok_or(Error::Corrupt { place: "key provider empty batch result" })?
            .result
    }

    /// Convenience helper for fetching a single `(cluster, epoch)` key.
    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error> {
        let mut results = self.cluster_keys(&[(cluster, epoch)])?;
        results
            .pop()
            .ok_or(Error::Corrupt { place: "key provider empty lookup result" })?
            .result
    }
}
