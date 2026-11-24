//! Change, Op, Guard, Precondition, Receipt (SP-PORTS §2). Körper werden nie gedruckt.
use crate::ids::{BranchId, ClusterId, IdemToken, SpaceId, Stamp};
use bytes::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protection {
    Sealed,
    Plain,
}

/// Physical erase placement locality for a cluster (SPEC §4.3, K-37).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseLocality {
    PackedBounded { max_file_bytes: u64 },
    ClusterLocal,
}

impl Default for EraseLocality {
    fn default() -> Self {
        Self::PackedBounded {
            max_file_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterLifecycle {
    Live,
    Retired { at: crate::ids::Stamp },
    Shredded { at: crate::ids::Stamp },
    Purged { at: crate::ids::Stamp },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    Durable,
    Relaxed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    User,
    Maintenance,
    Replication,
    Migration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallerMeta {
    pub origin: Origin,
    pub caller_seq: u64,
}

pub struct Change {
    pub ops: Vec<Op>,
    pub guards: Vec<Guard>,
    pub idem: Option<IdemToken>,
    pub durability: Durability,
    pub allow_mixed: bool,
    pub caller: CallerMeta,
}

pub enum Op {
    Put {
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: Bytes,
        header: Bytes,
        body: Bytes,
        weights: [u32; 4],
    },
    Delete {
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: Bytes,
    },
    /// start inklusiv, end exklusiv
    RangeDelete {
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        start: Bytes,
        end: Bytes,
    },
}

pub enum Guard {
    Key {
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: Bytes,
        pre: Precondition,
    },
    /// Phantom-Schutz für Bereichsscans (E-05)
    Range {
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        start: Bytes,
        end: Bytes,
        since: Stamp,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precondition {
    Absent,
    Version(Stamp),
    Unchanged { since: Stamp },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Receipt {
    pub stamp: Stamp,
    pub durable: bool,
}
