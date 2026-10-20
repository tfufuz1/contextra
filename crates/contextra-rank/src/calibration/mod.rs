//! Score calibration submodules (Isotonic + Platt scaling + Conformal calibration).

pub mod conformal;
pub mod isotonic;
pub mod platt;

pub use conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};
pub use isotonic::IsotonicCalibrator;
pub use platt::PlattScaler;
