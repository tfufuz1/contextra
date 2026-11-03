//! Saga-Modul (SPEC §11.2).
//!
//! Definiert die Zustandsmaschine des Löschablaufs und deren Koordination.

pub mod driver;
pub mod idempotency;
pub mod machine;
pub mod retention;

pub use driver::{SagaDriver, StepOutcome};
pub use idempotency::is_same_request;
pub use machine::{EraseRequest, EraseSaga, SagaState};
pub use retention::{is_restrict_allowed, is_retention_active};

pub use contextra_types::erasure::{
    EraseMode, EraseTarget, PendingReason, RetentionAttestation, RetentionCode, Timing,
};
