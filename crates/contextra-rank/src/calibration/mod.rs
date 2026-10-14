//! Score calibration submodules (Conformal + Isotonic + Platt scaling).

pub mod conformal;
pub mod isotonic;
pub mod platt;

pub use conformal::{
    compute_split_conformal_threshold, AciState, AdaptiveConformalCalibrator, ConformalCalibrator,
    ConformalError, SplitConformalOutcome, ACI_DEFAULT_GAMMA,
    CONFORMAL_MIN_CALIBRATION_SAMPLES,
};
pub use isotonic::IsotonicCalibrator;
pub use platt::PlattScaler;
