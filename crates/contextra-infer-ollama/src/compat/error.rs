//! Fehlerabbildung und Typkonvertierungen zwischen lokalen Typen und `contextra_types` / `contextra_ports`.

use crate::error::{LocalImportanceScore, LocalQueryRewriteOutput, OllamaError};

impl From<OllamaError> for contextra_types::ContextraError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            OllamaError::ModelNotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            OllamaError::Network(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                msg,
            )),
            OllamaError::Timeout(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                msg,
            )),
            OllamaError::Server(msg) => contextra_types::ContextraError::Storage(msg),
            OllamaError::Deserialization(msg) => contextra_types::ContextraError::Internal(msg),
            OllamaError::EgressViolation(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            OllamaError::CircuitBreakerOpen => {
                contextra_types::ContextraError::Internal("circuit breaker open".to_string())
            }
            OllamaError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_error_mapping_table() {
        struct Case {
            local: OllamaError,
            expected_discriminant: &'static str,
            expected_display: &'static str,
        }

        let cases = vec![
            Case {
                local: OllamaError::InvalidInput("bad input".into()),
                expected_discriminant: "InvalidInput",
                expected_display: "Invalid input: bad input",
            },
            Case {
                local: OllamaError::ModelNotFound("m1".into()),
                expected_discriminant: "NotFound",
                expected_display: "Not found: m1",
            },
            Case {
                local: OllamaError::Network("conn refused".into()),
                expected_discriminant: "Io",
                expected_display: "I/O error: conn refused",
            },
            Case {
                local: OllamaError::Timeout("deadline".into()),
                expected_discriminant: "Io",
                expected_display: "I/O error: deadline",
            },
            Case {
                local: OllamaError::Server("500".into()),
                expected_discriminant: "Storage",
                expected_display: "Storage error: 500",
            },
            Case {
                local: OllamaError::Deserialization("bad json".into()),
                expected_discriminant: "Internal",
                expected_display: "Internal error: bad json",
            },
            Case {
                local: OllamaError::EgressViolation("blocked url".into()),
                expected_discriminant: "PolicyViolation",
                expected_display: "Policy violation: blocked url",
            },
            Case {
                local: OllamaError::CircuitBreakerOpen,
                expected_discriminant: "Internal",
                expected_display: "Internal error: circuit breaker open",
            },
            Case {
                local: OllamaError::Internal("unexpected".into()),
                expected_discriminant: "Internal",
                expected_display: "Internal error: unexpected",
            },
        ];

        for case in cases {
            let ctx_err: contextra_types::ContextraError = case.local.into();
            let disc = match ctx_err {
                contextra_types::ContextraError::InvalidInput(_) => "InvalidInput",
                contextra_types::ContextraError::NotFound(_) => "NotFound",
                contextra_types::ContextraError::Io(_) => "Io",
                contextra_types::ContextraError::Storage(_) => "Storage",
                contextra_types::ContextraError::Internal(_) => "Internal",
                contextra_types::ContextraError::PolicyViolation(_) => "PolicyViolation",
                _ => "Other",
            };
            assert_eq!(disc, case.expected_discriminant);
            assert_eq!(ctx_err.to_string(), case.expected_display);
        }
    }
}

impl From<contextra_types::ContextraError> for OllamaError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => OllamaError::InvalidInput(msg),
            contextra_types::ContextraError::NotFound(msg) => OllamaError::ModelNotFound(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => {
                OllamaError::EgressViolation(msg)
            }
            contextra_types::ContextraError::Io(io_err) => {
                if io_err.kind() == std::io::ErrorKind::TimedOut {
                    OllamaError::Timeout(io_err.to_string())
                } else {
                    OllamaError::Network(io_err.to_string())
                }
            }
            contextra_types::ContextraError::Storage(msg) => OllamaError::Server(msg),
            contextra_types::ContextraError::Internal(msg) => OllamaError::Internal(msg),
            other => OllamaError::Internal(other.to_string()),
        }
    }
}

impl From<OllamaError> for contextra_ports::EmbeddingError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::ModelNotFound(msg) | OllamaError::InvalidInput(msg) => {
                contextra_ports::EmbeddingError::Unavailable(msg)
            }
            other => contextra_ports::EmbeddingError::ComputationFailed(other.to_string()),
        }
    }
}

impl From<LocalQueryRewriteOutput> for contextra_types::QueryRewriteOutput {
    fn from(val: LocalQueryRewriteOutput) -> Self {
        contextra_types::QueryRewriteOutput::new(
            val.text_query,
            val.semantic_query,
            val.anchor_entities,
        )
    }
}

impl From<contextra_types::QueryRewriteOutput> for LocalQueryRewriteOutput {
    fn from(val: contextra_types::QueryRewriteOutput) -> Self {
        LocalQueryRewriteOutput::new(val.text_query, val.semantic_query, val.anchor_entities)
    }
}

impl From<LocalImportanceScore> for contextra_types::ImportanceScore {
    fn from(score: LocalImportanceScore) -> Self {
        contextra_types::ImportanceScore::new(score.0)
    }
}

impl From<contextra_types::ImportanceScore> for LocalImportanceScore {
    fn from(score: contextra_types::ImportanceScore) -> Self {
        LocalImportanceScore(score.value())
    }
}

impl crate::error::LocalClock for contextra_ports::SystemClock {
    fn now(&self) -> std::time::SystemTime {
        let nanos = contextra_ports::Clock::now_unix_nanos(self);
        std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

impl crate::error::LocalRng for contextra_ports::SeededRng {
    fn next_u64(&self) -> u64 {
        contextra_ports::Rng::next_u64(self)
    }
}
