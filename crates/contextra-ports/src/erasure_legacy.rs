//! Legacy erasure traits isolated prior to relocation to `contextra-erasure-spi`.
//!
//! Under decision D-E1, all erasure, scanning, and gate coordination traits are
//! decoupled into `contextra-erasure-spi`. Pending the full rollout of package E2,
//! these interfaces are encapsulated here and re-exported with deprecation notices.

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::barrier::{
    BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit,
};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::erasure::{
    EraseTarget as PortEraseTarget, ErasureLayer, ErasureTarget, LayerScope, PurgeCx, PurgeOutcome,
    StorageBackend,
};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::gate::{GateDecision, KeyBound, KeyRange, RangeGate, ReadGate};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::keys::{Envelope, FileKey, FileKeyring, FileKind, RecordSealer, SealCtx};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::ledger::{LedgerEntry, LedgerSink, RangeSet};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::observer::{ErasureEvent, ErasureObserver, ObservedPhase, TargetKind};

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::report_signer::ReportSigner;

#[deprecated(
    note = "Erasure coordination will relocate to contextra-erasure-spi (Decision D-E1)"
)]
pub use crate::scanner::{
    LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,
    WitnessLayer,
};
