//! Canonical error definitions for contextra-crypto per HR-4 / SPEC 7.5.

use thiserror::Error;

/// Cryptographic error type with closed enum variants and fixed strings.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CryptoError {
    /// Authentication failure (e.g. wrong passphrase, MAC/tag mismatch)
    #[error("Auth")]
    Auth,

    /// Data corruption or malformed header/payload
    #[error("Corrupt")]
    Corrupt,

    /// Configuration error (e.g. invalid KdfParams from configuration)
    #[error("Config")]
    Config,

    /// Boundary or resource limit reached (e.g. 2^32 block limit)
    #[error("Limit")]
    Limit,

    /// Internal unexpected error
    #[error("Internal")]
    Internal,
}

pub type Result<T> = std::result::Result<T, CryptoError>;

impl From<CryptoError> for contextra_types::ContextraError {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::Auth => Self::Crypto("Auth: Authentication failed".to_string()),
            CryptoError::Corrupt => {
                Self::Crypto("Corrupt: Cryptographic integrity failure".to_string())
            }
            CryptoError::Config => {
                Self::InvalidInput("Config: Cryptographic configuration error".to_string())
            }
            CryptoError::Limit => {
                Self::Crypto("Limit: Cryptographic counter or capacity limit reached".to_string())
            }
            CryptoError::Internal => {
                Self::Internal("Internal: Cryptographic internal error".to_string())
            }
        }
    }
}
