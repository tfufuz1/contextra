//! Adaptive Decay Controller Re-export (F-01).
//!
//! Re-exports `AdaptiveDecayController`, `DecayControllerConfig`, and `DecaySignalInputs`
//! from `contextra-adapt` to eliminate duplicate implementations across Ring 0 and Layer 3.

pub use contextra_adapt::decay_controller::{
    AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs,
};
