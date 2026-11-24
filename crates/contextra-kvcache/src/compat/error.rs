//! Fehlerkonvertierung zwischen `KvCacheError` und `contextra_types::ContextraError`.

use crate::error::KvCacheError;
use contextra_types::ContextraError;

impl From<KvCacheError> for ContextraError {
    fn from(err: KvCacheError) -> Self {
        match err {
            KvCacheError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            KvCacheError::CryptoError(msg) => ContextraError::Crypto(msg),
            KvCacheError::PinBudgetExceeded(msg) => ContextraError::PinBudgetExceeded(msg),
            KvCacheError::LimitExceeded { limit, context } => {
                ContextraError::LimitExceeded { limit, context }
            }
            KvCacheError::PolicyViolation(msg) => ContextraError::PolicyViolation(msg),
            KvCacheError::Serialization(msg) => ContextraError::Serialization(msg),
            KvCacheError::QuantizationError(msg) => ContextraError::KvQuantization(msg),
            KvCacheError::Internal(msg) => ContextraError::Internal(msg),
            KvCacheError::InvalidTenant(msg) => {
                ContextraError::InvalidInput(format!("Ungültiger Mandant: {msg}"))
            }
            KvCacheError::InvalidSegment(msg) => {
                ContextraError::Internal(format!("Ungültiges Segment: {msg}"))
            }
            KvCacheError::SegmentNotFound(id) => {
                ContextraError::NotFound(format!("Segment nicht gefunden: {id}"))
            }
            KvCacheError::SegmentKeyDestroyed => {
                ContextraError::Internal("Segmentschlüssel ist zerstört".into())
            }
            KvCacheError::BudgetExceeded(msg) => ContextraError::MemoryLimitExceeded(msg),
            KvCacheError::ConfigError(msg) => {
                ContextraError::Internal(format!("Konfigurationsfehler: {msg}"))
            }
            KvCacheError::IoError(err) => ContextraError::Io(err),
        }
    }
}

impl From<ContextraError> for KvCacheError {
    fn from(err: ContextraError) -> Self {
        match err {
            ContextraError::InvalidInput(msg) => KvCacheError::InvalidInput(msg),
            ContextraError::Crypto(msg) => KvCacheError::CryptoError(msg),
            ContextraError::PinBudgetExceeded(msg) => KvCacheError::PinBudgetExceeded(msg),
            ContextraError::LimitExceeded { limit, context } => {
                KvCacheError::LimitExceeded { limit, context }
            }
            ContextraError::PolicyViolation(msg) => KvCacheError::PolicyViolation(msg),
            ContextraError::Serialization(msg) => KvCacheError::Serialization(msg),
            ContextraError::KvQuantization(msg) => KvCacheError::QuantizationError(msg),
            ContextraError::Internal(msg) => KvCacheError::Internal(msg),
            ContextraError::NotFound(msg) => KvCacheError::Internal(msg),
            ContextraError::MemoryLimitExceeded(msg) => KvCacheError::BudgetExceeded(msg),
            ContextraError::Io(err) => KvCacheError::IoError(err),
            other => KvCacheError::Internal(other.to_string()),
        }
    }
}
