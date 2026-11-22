// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Hardened FlatBuffers buffer verification and safe accessor functions
// RELEVANTE SPEZIFIKATION: REQ-WIR-001, REQ-WIR-002, P7 (Zero-Panic)

//! Safe and hardened FlatBuffers verification wrappers.

use crate::generated::contextra_generated::contextra::ipc::{
    Embedding, HyperEdge, HyperEdgeId, RoleBinding, RoleId, ScoredDocument, SearchResponse,
    VectorIndexUpdate,
};
use flatbuffers::VerifierOptions;
use std::fmt;

/// Error type for FlatBuffers verification failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatBufferError {
    /// Buffer is empty.
    EmptyBuffer,
    /// Buffer exceeds configured maximum message size.
    BufferTooLarge { size: usize, max_size: usize },
    /// FlatBuffers verifier check failed.
    VerificationFailed(String),
    /// Expected file identifier was missing or mismatched.
    IdentifierMismatch,
}

impl fmt::Display for FlatBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBuffer => write!(f, "FlatBuffer payload is empty"),
            Self::BufferTooLarge { size, max_size } => write!(
                f,
                "FlatBuffer size ({size} bytes) exceeds maximum limit ({max_size} bytes)"
            ),
            Self::VerificationFailed(reason) => {
                write!(f, "FlatBuffer verification failed: {reason}")
            }
            Self::IdentifierMismatch => write!(f, "FlatBuffer file identifier mismatch"),
        }
    }
}

impl std::error::Error for FlatBufferError {}

/// Configuration parameters for FlatBuffers verification.
#[derive(Debug, Clone)]
pub struct FlatBufferConfig {
    /// Maximum allowed buffer size in bytes (default: 10 MiB).
    pub max_message_size: usize,
    /// Maximum depth of nested FlatBuffers tables (default: 64).
    pub max_depth: usize,
    /// Maximum number of tables evaluated during verification (default: 1,000,000).
    pub max_tables: usize,
}

impl Default for FlatBufferConfig {
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024,
            max_depth: 64,
            max_tables: 1_000_000,
        }
    }
}

/// Generic verified root accessor for any FlatBuffers root table `T`.
pub fn verify_flatbuffer<'a, T>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<T::Inner, FlatBufferError>
where
    T: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a,
{
    if buf.is_empty() {
        return Err(FlatBufferError::EmptyBuffer);
    }
    if buf.len() > config.max_message_size {
        return Err(FlatBufferError::BufferTooLarge {
            size: buf.len(),
            max_size: config.max_message_size,
        });
    }

    let opts = VerifierOptions {
        max_depth: config.max_depth,
        max_tables: config.max_tables,
        max_apparent_size: config.max_message_size,
        ..Default::default()
    };

    flatbuffers::root_with_opts::<T>(&opts, buf)
        .map_err(|e| FlatBufferError::VerificationFailed(format!("{e:?}")))
}

/// Verifies and parses a `SearchResponse` root table from bytes.
pub fn verify_and_parse_search_response<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<SearchResponse<'a>, FlatBufferError> {
    verify_flatbuffer::<SearchResponse<'a>>(buf, config)
}

/// Verifies and parses an `Embedding` root table from bytes.
pub fn verify_and_parse_embedding<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<Embedding<'a>, FlatBufferError> {
    verify_flatbuffer::<Embedding<'a>>(buf, config)
}

/// Verifies and parses a `ScoredDocument` root table from bytes.
pub fn verify_and_parse_scored_document<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<ScoredDocument<'a>, FlatBufferError> {
    verify_flatbuffer::<ScoredDocument<'a>>(buf, config)
}

/// Verifies and parses a `VectorIndexUpdate` root table from bytes.
pub fn verify_and_parse_vector_index_update<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<VectorIndexUpdate<'a>, FlatBufferError> {
    verify_flatbuffer::<VectorIndexUpdate<'a>>(buf, config)
}

/// Verifies and parses a `RoleId` root table from bytes.
pub fn verify_and_parse_role_id<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<RoleId<'a>, FlatBufferError> {
    verify_flatbuffer::<RoleId<'a>>(buf, config)
}

/// Verifies and parses a `HyperEdgeId` root table from bytes.
pub fn verify_and_parse_hyper_edge_id<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<HyperEdgeId<'a>, FlatBufferError> {
    verify_flatbuffer::<HyperEdgeId<'a>>(buf, config)
}

/// Verifies and parses a `RoleBinding` root table from bytes.
pub fn verify_and_parse_role_binding<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<RoleBinding<'a>, FlatBufferError> {
    verify_flatbuffer::<RoleBinding<'a>>(buf, config)
}

/// Verifies and parses a `HyperEdge` root table from bytes.
pub fn verify_and_parse_hyper_edge<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<HyperEdge<'a>, FlatBufferError> {
    verify_flatbuffer::<HyperEdge<'a>>(buf, config)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_and_parse_search_response_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_search_response(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_search_response_oversized_buffer() {
        let config = FlatBufferConfig {
            max_message_size: 10,
            ..Default::default()
        };
        let buf = vec![0u8; 20];
        let err = verify_and_parse_search_response(&buf, &config).unwrap_err();
        assert_eq!(
            err,
            FlatBufferError::BufferTooLarge {
                size: 20,
                max_size: 10
            }
        );
    }

    #[test]
    fn test_verify_and_parse_embedding_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_embedding(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_scored_document_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_scored_document(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_vector_index_update_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_vector_index_update(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_role_id_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_role_id(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_hyper_edge_id_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_hyper_edge_id(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_role_binding_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_role_binding(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    #[test]
    fn test_verify_and_parse_hyper_edge_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_hyper_edge(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }
}
