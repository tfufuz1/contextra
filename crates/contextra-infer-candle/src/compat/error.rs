//! Fehlertransformation und Brückenkonvertierungen zwischen Candle Error und ContextraError.

use crate::error::Error;

impl From<Error> for contextra_types::ContextraError {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(io_err) => contextra_types::ContextraError::Io(io_err),
            Error::ModelLoad { reason } => contextra_types::ContextraError::Internal(reason),
            Error::Tokenizer(msg) => contextra_types::ContextraError::Internal(msg),
            Error::Inference(msg) => contextra_types::ContextraError::Internal(msg),
            Error::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            Error::ResourceExceeded(msg) => contextra_types::ContextraError::limit_exceeded(0, msg),
            Error::PolicyViolation(msg) => contextra_types::ContextraError::PolicyViolation(msg),
            Error::BackendUnavailable(msg) => contextra_types::ContextraError::NotFound(msg),
        }
    }
}

impl From<contextra_types::ContextraError> for Error {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::Io(io_err) => Error::Io(io_err),
            contextra_types::ContextraError::ModelLoad { reason, .. } => Error::ModelLoad { reason },
            contextra_types::ContextraError::InvalidInput(msg) => Error::InvalidInput(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => Error::PolicyViolation(msg),
            contextra_types::ContextraError::NotFound(msg) => Error::BackendUnavailable(msg),
            other => Error::Inference(other.to_string()),
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
