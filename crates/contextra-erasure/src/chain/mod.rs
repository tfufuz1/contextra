//! Löschkette nach SPEC §5 (6-Schritte-Saga), §6 (ReadGate) und §7 (ErasureLayer).
//!
//! Dieses Modul ist die einzige Implementierung der Löschkette. Es ersetzt die frühere
//! Saga (`saga/`, `orchestrator`). Abhängigkeiten nur abwärts (E0): Ports werden über
//! Traits angebunden, konkrete Speicher-Crates implementieren [`layer::ErasureLayer`].

pub mod adapters;
pub mod journal;
pub mod layer;
pub mod ports;
pub mod retention;
pub mod saga;
pub mod state;
pub mod target;

pub use journal::{FileJournal, JournalRecord};
pub use layer::{ErasureLayer, LayerFuture, LayerScope, PurgeCx, PurgeOutcome};
pub use ports::{
    CommitBarrier, DiskReserve, GateMode, GateSink, Journal, KeyVault, PositiveControl, ReportSink,
};
pub use retention::{RetentionAttestation, RetentionCode};
pub use saga::{Chain, ChainDeps, Receipt};
pub use state::{PendingReason, SagaState};
pub use target::{EraseTarget, TargetKind};
