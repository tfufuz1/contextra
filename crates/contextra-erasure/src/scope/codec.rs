//! Serialisierung und Deserialisierung von Scope-Dateien (SPEC §11.1, Anhang F.1, ER-17).
//!
//! Scope-Dateien haben kind = 9, zufälligen Dateischlüssel (F-Slot im Tresor).
//! Kleine Umfänge werden direkt in den Journal-Eintrag eingebettet.

use super::set::LayerScopeSet;
use crate::error::ErasureError;

/// Serialisiert einen `LayerScopeSet` als JSON (Placeholder; produktiv: AEAD-Datei mit kind=9).
pub fn encode_scope(scope: &LayerScopeSet) -> Result<Vec<u8>, ErasureError> {
    serde_json::to_vec(scope).map_err(|e| ErasureError::Json(e.to_string()))
}

/// Deserialisiert einen `LayerScopeSet` aus JSON.
pub fn decode_scope(data: &[u8]) -> Result<LayerScopeSet, ErasureError> {
    serde_json::from_slice(data).map_err(|e| ErasureError::Json(e.to_string()))
}
