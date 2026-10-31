//! # Kalibrierungssubsystem (`calibration`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul bündelt Algorithmen zur Wahrscheinlichkeitskalibrierung von Roh-Scores
//! sowie zur konformen Vorhersage (Conformal Prediction).

// ANCHOR[contextra-rank:CAL-MOD-001]
pub mod conformal;
pub mod isotonic;
pub mod platt;

pub use conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};
pub use isotonic::IsotonicCalibrator;
pub use platt::PlattScaler;
