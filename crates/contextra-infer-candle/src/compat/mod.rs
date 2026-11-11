//! Kompatibilitätsbrücke für `contextra-types`, `contextra-ports`, `contextra-rank` und `contextra-crypto`.

pub mod attention;
pub mod calibrator;
#[cfg(feature = "candle")]
pub mod embedding;
pub mod error;
pub mod fingerprint;
pub mod grounding;
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
pub mod kv;
#[cfg(feature = "candle")]
pub mod llm;
#[cfg(feature = "kv-bridge")]
pub mod kv_bridge;
#[cfg(all(test, feature = "candle"))]
pub mod inference_tests;
