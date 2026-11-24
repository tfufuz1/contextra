//! Derived segment contracts, owners, and erasure attestations (SPEC §12, SP-EXT, INV-VIS-1).
#![forbid(unsafe_code)]

use contextra_storage_ports::{ClusterId, Error, InstanceId, Stamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmentClass {
    ClusterSections,
    MixedPerClusterKey,
    MixedPerSubjectKey,
    Rebuildable {
        max_age_secs: u64,
        max_rewrite_bytes: u64,
    },
    NoPersonalData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseAttestation {
    pub owner_id: &'static str,
    pub generation: u64,
    pub scope_digest: [u8; 32],
    pub applied_position: (InstanceId, Stamp),
    pub keys_destroyed: bool,
    pub segments_rewritten: u64,
    pub evidence_digest: [u8; 32],
}

pub trait SegmentOwner: Send + Sync {
    fn owner_id(&self) -> &'static str;
    fn class(&self) -> SegmentClass;
    fn on_gate(&self, cluster: ClusterId, stamp: Stamp) -> Result<(), Error>;
    fn rewrite_without(&self, clusters: &[ClusterId]) -> Result<u64, Error>;
    fn verify_absent(&self, clusters: &[ClusterId]) -> Result<bool, Error>;
    fn attest(&self, generation: u64) -> Result<EraseAttestation, Error>;
}
