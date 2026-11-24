//! Lokale Fehlertypen für `contextra-simd` (B4-Konformität).
//!
//! Dieses Modul definiert den primären Fehlertyp `SimdError` für alle Operationen
//! in diesem Crate. Alle Fehlerbedingungen werden deterministisch und ohne Panics signalisiert.

use thiserror::Error;

// SECTION ERROR-ENUM
// ANCHOR[contextra-simd:SIMD-ERROR-ENUM]
/// Aufzählungs-Typ für alle im Crate auftretenden Fehlerzustände.
///
/// Alle Varianten sind Fail-Closed konstruiert.
#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum SimdError {
    /// Ungültige Vektoreingaben, beispielsweise `NaN` oder `±Inf` (INV-SIMD-FINITE-1).
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    /// Ungleiche Vektordimensionen zwischen linkem und rechtem Operanden.
    #[error("dimension mismatch: left={left}, right={right}")]
    DimensionMismatch {
        /// Länge des linken Vektors.
        left: usize,
        /// Länge des rechten Vektors.
        right: usize,
    },

    /// Eine geforderte Metrik wird für die gewählte Operation nicht unterstützt.
    #[error("unsupported metric")]
    UnsupportedMetric,
}
// !SECTION ERROR-ENUM

// SECTION RESULT-ALIAS
// ANCHOR[contextra-simd:SIMD-RESULT-ALIAS]
/// Crate-lokales Result-Alias mit `SimdError` als Standard-Fehlertyp.
pub type Result<T> = core::result::Result<T, SimdError>;

/// Typ-Alias `Error` für B4-Konformität (genau ein primärer Fehlertyp `Error`).
pub type Error = SimdError;
// !SECTION RESULT-ALIAS
