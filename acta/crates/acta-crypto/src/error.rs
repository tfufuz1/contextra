//! Fehlertyp von acta-crypto (HR-4: ein Fehlertyp je Crate, `#[non_exhaustive]`).
//!
//! Die Meldungen enthalten weder Schlüsselmaterial noch Daten oder Kennungen (Z15).

use acta_base::{ClassifiedError, EntropyError, ErrorClass};

/// Fehler der kryptografischen Operationen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CryptoError {
    /// Die Entropiequelle hat versagt. Es entsteht kein Umschlag und kein Schlüssel (Z10).
    #[error("entropy source failed; nothing was produced")]
    Entropy,
    /// Authentisierung oder Entschlüsselung ist fehlgeschlagen. Die Ursache wird nicht unterschieden.
    #[error("decryption failed")]
    DecryptionFailed,
    /// Der Umschlag ist strukturell fehlerhaft (Länge, Padding, Format).
    #[error("sealed envelope is malformed")]
    Malformed,
    /// Unbekannte Version, unbekanntes Verfahren oder falsche Ebene.
    #[error("unsupported envelope version, algorithm or plane")]
    Unsupported,
    /// Die Nutzlast überschreitet die Längengrenze des Formats.
    #[error("payload exceeds the record length limit")]
    TooLarge,
    /// Mehr Betroffene, als das Ableitungsformat kodieren kann (höchstens 255).
    #[error("too many subjects for one document version")]
    TooManySubjects,
    /// Eine Betroffenen-Kennung kommt mehrfach vor.
    #[error("duplicate subject tag")]
    DuplicateSubject,
    /// Eine interne kryptografische Operation ist fehlgeschlagen.
    #[error("internal cryptographic operation failed")]
    Internal,
    /// Die Signatur ist ungültig.
    #[error("signature verification failed")]
    SignatureInvalid,
}

impl From<EntropyError> for CryptoError {
    fn from(_: EntropyError) -> Self {
        Self::Entropy
    }
}

impl ClassifiedError for CryptoError {
    fn class(&self) -> ErrorClass {
        match self {
            Self::Entropy | Self::DecryptionFailed | Self::Internal => ErrorClass::Crypto,
            Self::Malformed | Self::SignatureInvalid => ErrorClass::Integrity,
            Self::Unsupported | Self::TooLarge | Self::TooManySubjects | Self::DuplicateSubject => {
                ErrorClass::Invalid
            }
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Entropy => "crypto.entropy",
            Self::DecryptionFailed => "crypto.decryption_failed",
            Self::Malformed => "crypto.malformed",
            Self::Unsupported => "crypto.unsupported",
            Self::TooLarge => "crypto.too_large",
            Self::TooManySubjects => "crypto.too_many_subjects",
            Self::DuplicateSubject => "crypto.duplicate_subject",
            Self::Internal => "crypto.internal",
            Self::SignatureInvalid => "crypto.signature_invalid",
        }
    }
}
