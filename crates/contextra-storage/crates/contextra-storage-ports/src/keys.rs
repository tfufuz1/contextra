//! Schlüsseltypen und Provider-Port (SP-PORTS, E-02, SP-KEYS §2).
use crate::error::Error;
use crate::ids::{BatchId, ClusterId};
use core::fmt;
use zeroize::Zeroizing;

pub const MAX_KEYS_PER_BATCH: usize = 256;

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

/// Missing ist nicht Shredded (F-6 KeyLost).
pub enum KeyLookup {
    Present(Key32),
    Shredded,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCreateFailure { Shredded, Unavailable { retry_after_secs: Option<u64> } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCreateResult {
    Created { cluster: ClusterId, epoch: u32 },
    Failed { cluster: ClusterId, error: KeyCreateFailure },
}

/// Disabled wird wie Shredded behandelt; Erase gilt erst bei Absent als beendet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProbe {
    Present,
    Absent,
    Disabled,
}

pub trait KeyProvider: Send + Sync {
    /// Idempotent batch keyed by request ID, at most 256 clusters. Per-cluster
    /// outcomes allow retrying partial successes; every success is durable.
    fn create_cluster_keys(&self, request_id: BatchId, clusters: &[ClusterId]) -> Result<Vec<KeyCreateResult>, Error>;
    /// Batch-Lesen (E-08), höchstens MAX_KEYS_PER_BATCH je Aufruf.
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error>;
    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyProbe, Error>;
    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<(), Error>;
}
