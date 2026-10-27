// FILE-CONTEXT
// STAND: 2026-10-25
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, FaultVfs, ReferenceModel, and Mocks.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

pub mod fault_vfs;
pub mod in_memory_store;
pub mod manual_clock;
pub mod mocks;
pub mod reference_model;

pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_store::InMemoryStorageEngine;
pub use manual_clock::ManualClock;
pub use mocks::*;
pub use reference_model::{RefOp, ReferenceModel};

/// Helper function to mint a `LayerWitness` strictly for test suites.
#[cfg(feature = "testkit")]
pub fn mint_witness_for_tests(
    layer: impl Into<contextra_ports::scanner::WitnessLayer>,
    scanned: u64,
    remaining: u64,
    technique: contextra_ports::scanner::ScanTechnique,
) -> contextra_ports::scanner::LayerWitness {
    contextra_ports::scanner::mint_witness_for_testkit(
        layer.into(),
        scanned,
        remaining,
        technique,
        [0xFE; 32],
        0,
    )
}
