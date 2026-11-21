//! # Invarianten-Validierung (Ring 1)
//!
//! // STAND: 2026-11-11
//! // ZWECK: Prueft Schluessel- und Wertgroessen auf Einhaltung der Systemgrenzen
//! // INVARIANTEN: MAX_KEY_SIZE (65535 Bytes), MAX_VALUE_SIZE (128 MB), Nicht-leere Schluessel

use super::{MAX_KEY_SIZE, MAX_VALUE_SIZE};
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use contextra_ids::DocId;

/// Validates that a key is non-empty and does not exceed `MAX_KEY_SIZE`.
pub fn validate_key(key: &[u8]) -> Result<()> {
    if key.is_empty() {
        return Err(ContextraError::InvalidInput("Key cannot be empty".into()));
    }
    if key.len() > MAX_KEY_SIZE {
        return Err(ContextraError::InvalidInput(format!(
            "Key length ({} bytes) exceeds limit of {} bytes",
            key.len(),
            MAX_KEY_SIZE
        )));
    }
    Ok(())
}

/// Validates that value bytes do not exceed `MAX_VALUE_SIZE`.
pub fn validate_value(value: &[u8]) -> Result<()> {
    if value.len() > MAX_VALUE_SIZE {
        return Err(ContextraError::InvalidInput(format!(
            "Value length ({} bytes) exceeds limit of {} bytes",
            value.len(),
            MAX_VALUE_SIZE
        )));
    }
    Ok(())
}
