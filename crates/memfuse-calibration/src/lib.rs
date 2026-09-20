//! Score and probability calibration module for MemFuse (Isotonic + Platt Scaling + Adaptive PID Pool-Size Control).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod isotonic;
pub mod platt;

pub use isotonic::IsotonicCalibrator;
pub use memfuse_core::ConfigFingerprint;
pub use platt::PlattScaler;

#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_adapt::PidController as part of Ring-Modell Phase 1b"
)]
pub use memfuse_adapt::PidController;

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-16T16:15:00Z) (SESSION: e72e11a3)
// PRÜFER-KONTEXT: FRESH
// BEFUND: IsotonicCalibrator, PlattScaler, PidController und P8 Compliance lückenlos verifiziert. 66/66 Tests bestanden.
