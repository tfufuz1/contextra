// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deterministic Simulation Testing (DST) module exports.
// INVARIANTEN: Fully deterministic, seed-reproducible test environment.

pub mod engine;
pub mod invariants;
pub mod op;
pub mod p07_runner;
pub mod p07_sut;
pub mod rng;
pub mod runner;
pub mod scenario;

pub use engine::{DstEngine, StagedOp};
pub use invariants::{DstModel, InvariantChecker};
pub use op::DstOp;
pub use p07_runner::{generate_p07_ops, P07DivergenceRecord, P07DstReport, P07DstRunner};
pub use p07_sut::{NaiveWalSut, SystemUnderTest};
pub use rng::{DstRng, EntropyRng, SeededRng};
pub use runner::{DstConfig, DstResult, DstRunner, FaultMode};
pub use scenario::DstScenario;
