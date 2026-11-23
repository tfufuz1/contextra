//! Schlüsseltypen und Provider-Port (SP-PORTS, E-02, SP-KEYS §2).
use crate::error::Error;
use crate::ids::ClusterId;
use core::fmt;
use zeroize::Zeroizing;

/// Opaque identifier for a provider-managed key-encryption key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KekId([u8; 16]);

impl KekId {
    pub const fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
}

impl fmt::Debug for KekId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("KekId(<redacted>)") }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyBehavior {
    Immediate,
    Scheduled { max_delay_secs: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyProviderInfo {
    pub destroy: DestroyBehavior,
    pub keys_in_backups: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState { Present, Disabled, Absent }

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
    /// Describes key destruction timing and backup coverage.
    fn info(&self) -> KeyProviderInfo;
    /// Returns an available KEK by opaque provider identifier.
    fn kek(&self, id: KekId) -> Result<Key32, Error>;
    /// Creates or returns the current durable key epoch for a cluster.
    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn crate::traits::Entropy) -> Result<u32, Error>;
    /// Fetches one cluster epoch. `None` means the provider has no usable key;
    /// recovery must distinguish legitimate destruction from key loss.
    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error>;
    /// Reports provider state across every epoch for a cluster.
    fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error>;
    /// Lists provider-owned cluster identifiers for orphan reconciliation.
    fn list(&self) -> Result<Vec<ClusterId>, Error>;
    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error>;
}
