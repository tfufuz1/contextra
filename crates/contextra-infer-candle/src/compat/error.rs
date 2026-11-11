//! Fehlerabbildung und Typkonvertierungen zwischen CandleError / ModelFingerprint und `contextra_types`.

use crate::error::CandleError;
use crate::model_registry::ModelFingerprint;

impl From<CandleError> for contextra_types::ContextraError {
    fn from(err: CandleError) -> Self {
        match err {
            CandleError::Io(e) => contextra_types::ContextraError::Io(e),
            CandleError::ModelLoad { reason } => contextra_types::ContextraError::ModelLoad {
                path: "unknown".to_string(),
                reason,
            },
            CandleError::Tokenizer(msg) => {
                contextra_types::ContextraError::Internal(format!("Tokenizer error: {msg}"))
            }
            CandleError::Inference(msg) => contextra_types::ContextraError::Internal(format!(
                "Inference execution error: {msg}"
            )),
            CandleError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            CandleError::ResourceExceeded(_msg) => {
                contextra_types::ContextraError::MemoryBudgetExceeded {
                    used_mb: 0,
                    limit_mb: 0,
                }
            }
            CandleError::PolicyViolation(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            CandleError::BackendUnavailable(msg) => {
                contextra_types::ContextraError::CapabilityUnsupported {
                    capability: "candle".to_string(),
                    reason: msg,
                }
            }
        }
    }
}

impl From<contextra_types::ContextraError> for CandleError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::Io(e) => CandleError::Io(e),
            contextra_types::ContextraError::ModelLoad { path: _, reason } => {
                CandleError::ModelLoad { reason }
            }
            contextra_types::ContextraError::InvalidInput(msg) => CandleError::InvalidInput(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => {
                CandleError::PolicyViolation(msg)
            }
            contextra_types::ContextraError::CapabilityUnsupported {
                capability: _,
                reason,
            } => CandleError::BackendUnavailable(reason),
            contextra_types::ContextraError::MemoryBudgetExceeded { .. } => {
                CandleError::ResourceExceeded("Memory budget exceeded".to_string())
            }
            other => CandleError::Inference(other.to_string()),
        }
    }
}

impl From<ModelFingerprint> for contextra_types::ModelFingerprint {
    fn from(fp: ModelFingerprint) -> Self {
        contextra_types::ModelFingerprint {
            hash: fp.hash,
            model_id: fp.model_id,
            quantization: fp.quantization,
        }
    }
}

impl From<contextra_types::ModelFingerprint> for ModelFingerprint {
    fn from(fp: contextra_types::ModelFingerprint) -> Self {
        ModelFingerprint {
            hash: fp.hash,
            model_id: fp.model_id,
            quantization: fp.quantization,
        }
    }
}
