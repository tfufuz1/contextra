//! Output codes and error handling for ctx-verify.

use std::fmt;

/// Exit codes for the command line application.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID: i32 = 2;

/// Formatted evaluation result for CLI reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Valid,
    Invalid(InvalidCode),
}

impl fmt::Display for VerifyOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Invalid(code) => write!(f, "INVALID({})", code.as_str()),
        }
    }
}

/// Standardized invalid codes per SPEC §8.17, Anhang D, and task requirements.
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
