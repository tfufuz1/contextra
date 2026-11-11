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

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::ContextraError;

    #[test]
    fn test_error_mapping_table() {
        let test_cases: Vec<(Error, &str, fn(&ContextraError) -> bool)> = vec![
            (
                Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")),
                "I/O error: file not found",
                |e| matches!(e, ContextraError::Io(_)),
            ),
            (
                Error::ModelLoad {
                    reason: "failed gguf magic".to_string(),
                },
                "Model load error: failed gguf magic",
                |e| matches!(e, ContextraError::Internal(msg) if msg == "failed gguf magic"),
            ),
            (
                Error::Tokenizer("unknown token".to_string()),
                "Tokenizer error: unknown token",
                |e| matches!(e, ContextraError::Internal(msg) if msg == "unknown token"),
            ),
            (
                Error::Inference("tensor shape mismatch".to_string()),
                "Inference execution error: tensor shape mismatch",
                |e| matches!(e, ContextraError::Internal(msg) if msg == "tensor shape mismatch"),
            ),
            (
                Error::InvalidInput("batch size zero".to_string()),
                "Invalid parameter: batch size zero",
                |e| matches!(e, ContextraError::InvalidInput(msg) if msg == "batch size zero"),
            ),
            (
                Error::ResourceExceeded("context length 8192 exceeded".to_string()),
                "Resource limit exceeded: context length 8192 exceeded",
                |e| matches!(e, ContextraError::LimitExceeded { context, .. } if context == "context length 8192 exceeded"),
            ),
            (
                Error::PolicyViolation("hallucinated fact".to_string()),
                "Grounding policy violation: hallucinated fact",
                |e| matches!(e, ContextraError::PolicyViolation(msg) if msg == "hallucinated fact"),
            ),
            (
                Error::BackendUnavailable("cuda device missing".to_string()),
                "Backend unavailable: cuda device missing",
                |e| matches!(e, ContextraError::NotFound(msg) if msg == "cuda device missing"),
            ),
        ];

        for (local_err, expected_display, match_fn) in test_cases {
            assert_eq!(local_err.to_string(), expected_display);
            let ctx_err: ContextraError = local_err.into();
            assert!(match_fn(&ctx_err));
        }
    }
}
