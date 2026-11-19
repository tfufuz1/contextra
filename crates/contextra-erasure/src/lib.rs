//! Löschkern: Löschkette nach SPEC §5 (Saga), §6 (ReadGate), §7 (ErasureLayer).
//!
//! Ring 0 (E0). Die einzige Löschmethode ist [`chain`]. Abhängigkeiten nur abwärts (R1).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod chain;
pub mod crypto_local;
pub mod error;
pub mod gate;
pub mod io;
pub mod types_local;
pub mod vault;

pub use gate::saga_gate::{GateDecision, RangeGate, ReadGate, SagaGate};
