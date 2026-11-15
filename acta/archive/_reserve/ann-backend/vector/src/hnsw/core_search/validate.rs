// FILE-CONTEXT
// ZWECK: Validierung von Suchanfragen und Parametern (K-Grenzen, Dimension, NaN/Inf).
// INVARIANTEN: Zero-Panic; k <= MAX_SEARCH_K; finite f32.
// HOTSPOTS: validate_search_params
// STAND: 2026-11-12

//! Validierungsroutinen für Vektorsuchanfragen.

use crate::error::{Result, VectorError};
use crate::types::MAX_SEARCH_K;

/// Validiert Dimension, K-Wert und Komponenten einer Vektorsuchanfrage.
#[inline]
pub(crate) fn validate_search_params(
    validation_error: Option<&String>,
    expected_dim: usize,
    query: &[f32],
    k: usize,
) -> Result<()> {
    if let Some(err) = validation_error {
        return Err(VectorError::invalid_input(format!(
            "Invalid index configuration: {err}"
        )));
    }
    if query.len() != expected_dim {
        return Err(VectorError::invalid_input(format!(
            "Expected dimension {expected_dim}, got {}",
            query.len()
        )));
    }
    if k > MAX_SEARCH_K {
        return Err(VectorError::invalid_input(format!(
            "Requested k ({k}) exceeds maximum allowed search limit ({MAX_SEARCH_K}). \
             Use Collection::query() with appropriate k bounds."
        )));
    }
    for (i, &val) in query.iter().enumerate() {
        if !val.is_finite() {
            return Err(VectorError::invalid_input(format!(
                "Query vector element at index {i} is not finite (value: {val}). \
                 NaN/Inf values corrupt HNSW distance computation and heap ordering. \
                 Validate embedding outputs before search."
            )));
        }
    }
    Ok(())
}
