//! Score calibration submodules (Isotonic + Platt scaling).

pub mod isotonic;
pub mod platt;

pub use isotonic::IsotonicCalibrator;
pub use platt::PlattScaler;
