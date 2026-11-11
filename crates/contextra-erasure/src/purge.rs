//! Purge-Modul (SPEC §11.4).
//!
//! Steuert das Purge-Fenster, die Phasen-Zustandsmaschine, Wiederanlauf (P9), die Schichtreihenfolge und die Plattenreserve.

pub mod cx;
pub mod order;
pub mod phase;
pub mod reserve;
pub mod resume;
pub mod window;

pub use cx::PurgeCx;
pub use order::{LayerPurgeOrder, PurgePriority};
pub use phase::{
    AdmissionFailure, DisposeFailure, FenceFailure, FinalizeFailure, FlushFailure, PurgePhase,
    PurgePhaseError, ReleaseFailure, RewriteFailure, WalRetireFailure, WitnessFailure,
};
pub use reserve::{check_disk_reserve, ReserveCheck};
pub use resume::PurgeProgressTracker;
#[cfg(feature = "async")]
pub use window::run_core_window;
pub use window::{run_window, WindowOutcome};

pub use crate::types_local::{
    ErasureEvent, ErasureObserver, ObservedPhase, ObserverError, TargetKind,
};
