//! Score calibration submodules (Conformal + Isotonic + Platt scaling).

pub mod conformal;
pub mod isotonic;
pub mod platt;

pub use conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};
pub use isotonic::IsotonicCalibrator;
pub use platt::PlattScaler;
