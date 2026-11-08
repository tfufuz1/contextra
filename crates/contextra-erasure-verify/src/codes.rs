//! Ausgangscodes und strukturierte Fehlerbehandlung fuer contextra-erasure-verify.

use std::fmt;

/// Exit-Codes fuer die CLI-Anwendung (0 = VALID, 2 = INVALID).
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID: i32 = 2;

/// Standardisierte Pruefcodes nach SPEC §8.17, Anhang D und Baukasten-Anforderung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidCode {
    MissingKey,
    IoError,
    NonCanonicalJson,
    BadSignature,
    BrokenHashChain,
    MissingPositiveControl,
    WitnessMatchesNonZero,
    IncompleteResidualRisks,
    TemporalOrderViolation,
    LedgerMismatch,
    PassphraseFdMissing,
    Mk1StructureResidue,
    Mk2RawResidue,
    Mk3KeyPresent,
}

impl InvalidCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MissingKey => "missing_key",
            Self::IoError => "io_error",
            Self::NonCanonicalJson => "non_canonical_json",
            Self::BadSignature => "bad_signature",
            Self::BrokenHashChain => "broken_hash_chain",
            Self::MissingPositiveControl => "missing_positive_control",
            Self::WitnessMatchesNonZero => "witness_matches_nonzero",
            Self::IncompleteResidualRisks => "incomplete_residual_risks",
            Self::TemporalOrderViolation => "temporal_order_violation",
            Self::LedgerMismatch => "ledger_mismatch",
            Self::PassphraseFdMissing => "passphrase_fd_missing",
            Self::Mk1StructureResidue => "mk1_structure_residue",
            Self::Mk2RawResidue => "mk2_raw_residue",
            Self::Mk3KeyPresent => "mk3_key_present",
        }
    }
}

/// Formatiertes Pruefergebnis fuer das Reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Valid,
    Invalid(InvalidCode),
}

impl std::error::Error for InvalidCode {}

impl fmt::Display for InvalidCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for VerifyOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Invalid(code) => write!(f, "INVALID({})", code.as_str()),
        }
    }
}
