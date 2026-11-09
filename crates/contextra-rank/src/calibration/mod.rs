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
pub use isotonic::{Clock, IsotonicCalibrator, SystemClock};
pub use platt::PlattScaler;

use crate::error::{RankError, Result};
use serde::{Deserialize, Serialize};

/// Unified versioned container for calibration models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibrator {
    /// Format version field.
    pub format_version: u32,
    /// Model.
    pub model: CalibratorModel,
}

/// Supported calibrator models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalibratorModel {
    /// Platt.
    Platt(PlattScaler),
    /// Isotonic.
    Isotonic(IsotonicCalibrator),
    /// Conformal.
    Conformal(AdaptiveConformalCalibrator),
}

impl Calibrator {
    /// Current version.
    pub const CURRENT_VERSION: u32 = 1;

    /// Platt.
    pub fn platt(scaler: PlattScaler) -> Self {
        Self {
            format_version: Self::CURRENT_VERSION,
            model: CalibratorModel::Platt(scaler),
        }
    }

    /// Isotonic.
    pub fn isotonic(calibrator: IsotonicCalibrator) -> Self {
        Self {
            format_version: Self::CURRENT_VERSION,
            model: CalibratorModel::Isotonic(calibrator),
        }
    }

    /// Conformal.
    pub fn conformal(calibrator: AdaptiveConformalCalibrator) -> Self {
        Self {
            format_version: Self::CURRENT_VERSION,
            model: CalibratorModel::Conformal(calibrator),
        }
    }

    /// Calibrate.
    pub fn calibrate(&self, raw: f32) -> Result<f32> {
        if self.format_version != Self::CURRENT_VERSION {
            return Err(RankError::InvalidInput("Unsupported format version".into()));
        }
        if !raw.is_finite() {
            return Err(RankError::InvalidInput("Raw score must be finite".into()));
        }
        match &self.model {
            CalibratorModel::Platt(s) => Ok(s.predict(raw)),
            CalibratorModel::Isotonic(c) => {
                let mut iso = c.clone();
                Ok(iso
                    .calibrated_probability(raw)
                    .unwrap_or(raw.clamp(0.0, 1.0)))
            }
            CalibratorModel::Conformal(c) => {
                use conformal::ConformalCalibrator;
                let thresh = c.threshold();
                if raw >= thresh {
                    Ok(1.0)
                } else if thresh > 0.0 {
                    Ok((raw / thresh).clamp(0.0, 1.0))
                } else {
                    Ok(0.0)
                }
            }
        }
    }
}

/// Calibrate score helper.
pub fn calibrate_score(raw: f32, calibrator: &Calibrator) -> Result<f32> {
    calibrator.calibrate(raw)
}
