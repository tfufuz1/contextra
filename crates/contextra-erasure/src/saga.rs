//! Saga-Modul (SPEC §11.2).
//!
//! Definiert die Zustandsmaschine des Löschablaufs und deren Koordination.

pub mod core;
pub mod driver;
pub mod idempotency;
pub mod machine;
pub mod retention;
pub mod seams;

pub use core::{EraseExecutionRequest, ErasureConfig, ErasureCore, ErasureCoreStatus, ErasureDeps};
pub use driver::{SagaDriver, SagaEvent, StepOutcome};
pub use idempotency::{derive_open_key, is_same_request};
pub use machine::{EraseRequest, EraseSaga, SagaState};
pub use retention::{is_restrict_allowed, is_retention_active};
pub use seams::{
    SagaGate, SagaJournal, SagaLedger, SagaMeasure, SagaReport, SagaVault, SnapshotRevoker,
};

pub use crate::types_local::{
    EraseMode, EraseTarget, PendingReason, RetentionAttestation, RetentionCode, Timing,
};
