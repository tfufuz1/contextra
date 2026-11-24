//! Port-Traits (SP-PORTS §2). Jede Fähigkeit hat genau einen Trait.
pub mod anchor;
pub mod authority;
pub mod cold;
pub mod crypto;
pub mod engine;
pub mod fs;
pub mod host;
pub mod memory;
pub mod observe;
pub mod spawn;

pub use anchor::{Anchor, AnchorStrength, AnchorValue, Clock, ClockReading, ClockSkew, Entropy};
pub use authority::{Authority, FourEyesProof, Principal};
pub use crypto::{Checksum, Cipher, Kdf, Mac};
pub use fs::{File, FileRole, Fs, FsCaps, FsLock, IoCtx, ReadRange};
pub use host::{HostProbe, HostReport, HostToggle};
pub use memory::{MemoryBudget, ResourceClass, ResourcePermit};
pub use observe::{
    EraseStage, MaintenancePhase, MetricLabel, MetricLabelKey, MetricName, MetricResult, Metrics,
    MetricSample, Observer, ObserverEvent, OwnerKind,
};
