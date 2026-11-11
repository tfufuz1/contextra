//! Kompatibilitätsbrücke für `contextra-infer-candle`.

pub mod attention;
pub mod calibrator;
#[cfg(feature = "candle")]
pub mod embedding;
pub mod error;
