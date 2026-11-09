//! # Kalibrierungssubsystem (`calibration`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul bündelt Algorithmen zur Wahrscheinlichkeitskalibrierung von Roh-Scores
//! sowie zur konformen Vorhersage (Conformal Prediction).
pub mod conformal;
pub mod isotonic;
pub mod platt;

pub use conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError};
pub use isotonic::{Clock, IsotonicCalibrator, SystemClock};
pub use platt::PlattScaler;

use crate::error::{RankError, Result};
use serde::{Deserialize, Serialize};

/// Versionierter Container für Kalibrierungsmodelle (Klassenvertrag RANK-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Calibrator {
    /// Platt-Scaling (logistische Regression) mit Format-Version 1.
    #[serde(rename = "platt_v1")]
    Platt {
        /// Format-Versionsfeld für defensive Dekodierung.
        format_version: u8,
        /// Platt-Scaler-Instanz.
        scaler: PlattScaler,
    },
    /// Isotonische Regression via PAVA mit Format-Version 1.
    #[serde(rename = "isotonic_v1")]
    Isotonic {
        /// Format-Versionsfeld für defensive Dekodierung.
        format_version: u8,
        /// Isotonischer Kalibrierer.
        calibrator: IsotonicCalibrator,
    },
    /// Adaptiver konformer Kalibrierer mit Format-Version 1.
    #[serde(rename = "conformal_v1")]
    Conformal {
        /// Format-Versionsfeld für defensive Dekodierung.
        format_version: u8,
        /// Konformer Kalibrierer.
        calibrator: AdaptiveConformalCalibrator,
    },
}

impl Calibrator {
    /// Erzeugt einen neuen `Calibrator` basierend auf einem `PlattScaler`.
    pub fn platt(scaler: PlattScaler) -> Self {
        Self::Platt {
            format_version: 1,
            scaler,
        }
    }

    /// Erzeugt einen neuen `Calibrator` basierend auf einem `IsotonicCalibrator`.
    pub fn isotonic(calibrator: IsotonicCalibrator) -> Self {
        Self::Isotonic {
            format_version: 1,
            calibrator,
        }
    }

    /// Erzeugt einen neuen `Calibrator` basierend auf einem `AdaptiveConformalCalibrator`.
    pub fn conformal(calibrator: AdaptiveConformalCalibrator) -> Self {
        Self::Conformal {
            format_version: 1,
            calibrator,
        }
    }
}

/// Kalibriert einen Roh-Score anhand eines übergebenen Kalibrierers (Klassenvertrag RANK-1).
///
/// # Fehler
/// Gibt `RankError::InvalidInput` zurück, wenn `raw` nicht endlich ist (NaN oder Inf)
/// oder wenn die Format-Version des Kalibrierers inkompatibel ist.
pub fn calibrate_score(raw: f32, calibrator: &Calibrator) -> Result<f32> {
    if !raw.is_finite() {
        return Err(RankError::InvalidInput(format!(
            "Cannot calibrate non-finite score {raw}"
        )));
    }

    match calibrator {
        Calibrator::Platt {
            format_version,
            scaler,
        } => {
            if *format_version != 1 {
                return Err(RankError::Calibration(format!(
                    "Unsupported Platt calibrator format version: {format_version}"
                )));
            }
            Ok(scaler.predict(raw))
        }
        Calibrator::Isotonic {
            format_version,
            calibrator,
        } => {
            if *format_version != 1 {
                return Err(RankError::Calibration(format!(
                    "Unsupported Isotonic calibrator format version: {format_version}"
                )));
            }
            let mut cal_clone = calibrator.clone();
            cal_clone.calibrated_probability(raw).ok_or_else(|| {
                RankError::Calibration("Isotonic calibrator not yet warm".to_string())
            })
        }
        Calibrator::Conformal {
            format_version,
            calibrator,
        } => {
            if *format_version != 1 {
                return Err(RankError::Calibration(format!(
                    "Unsupported Conformal calibrator format version: {format_version}"
                )));
            }
            let threshold = calibrator.threshold();
            // Gibt die relative Marge / Distanz zum konformen Schwellenwert zurück
            Ok(raw - threshold)
        }
    }
}
