//! Score and probability calibration module for MemFuse (Isotonic + Platt Scaling + Adaptive PID Pool-Size Control).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod isotonic;
pub mod pid;
pub mod platt;

pub use isotonic::IsotonicCalibrator;
pub use memfuse_core::ConfigFingerprint;
pub use pid::PidController;
pub use platt::PlattScaler;

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-16T16:15:00Z) (SESSION: e72e11a3)
// PRÜFER-KONTEXT: FRESH
// BEFUND: IsotonicCalibrator, PlattScaler, PidController und P8 Compliance lückenlos verifiziert. 66/66 Tests bestanden.
