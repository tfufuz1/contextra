//! Saga-Modul (SPEC §11.2).
//!
//! Definiert die Zustandsmaschine des Löschablaufs und deren Koordination.
pub mod machine;
pub mod driver;
pub mod retention;
pub mod idempotency;

pub use machine::{EraseSaga, SagaState};
pub use driver::{SagaDriver, StepOutcome};
pub use contextra_types::erasure::{
    EraseMode, EraseRequest, EraseTarget, PendingReason, RetentionAttestation, RetentionCode,
    Timing,
};
