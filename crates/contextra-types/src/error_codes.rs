//! Stable error codes (SPEC Anhang C).
//!
//! These are the wire-level codes that clients match on. The Rust error type
//! `ContextraError` maps onto them. This module defines only the stable
//! identifiers and their retry semantics.

use core::fmt;
use core::str::FromStr;

/// Retry semantics of an error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryPolicy {
    /// The operation may be retried, after the usual reset (new transaction, wait).
    Retryable,
    /// Retrying the same request gives the same result.
    NotRetryable,
    /// Not an error. The code reports a successful replay.
    NotAnError,
}

/// Every error code defined in Anhang C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Write conflict. Begin a new transaction.
    Conflict,
    /// Snapshot lease expired.
    SnapshotExpired,
    /// Time travel outside the retention window. Earliest available seq is reported.
    BeyondRetention,
    /// Backpressure. The response carries `retry_after_ms`.
    Backpressure,
    /// Handle is unknown. The response carries `did_you_mean`.
    HandleUnknown,
    /// Handle MAC check failed. The response carries `did_you_mean`.
    HandleMacInvalid,
    /// Namespace not accessible. Deliberately gives no existence information.
    ForbiddenNamespace,
    /// Vector contains NaN, infinity, or has the wrong dimension.
    VectorInvalid,
    /// Budget below the minimum. The minimum is reported.
    BudgetTooSmall,
    /// Checksum failure in the truth store. Restore required.
    CorruptTruth,
    /// A gate rejected the operation. The gate and reason are reported.
    GateRejected,
    /// Idempotency replay. Not an error: the stored result is returned.
    IdempotentReplay,
    /// Same idempotency key with a different payload.
    IdempotentMismatch,
    /// Fatal I/O failure. Restart or recovery required.
    StoreFailed,
    /// Session quota exhausted.
    QuotaExceeded,
    /// Required provenance is missing.
    ProvMissing,
    /// Namespace schema violated.
    SchemaViolation,
    /// Trust tier below `min_main_trust`.
    TrustInsufficient,
}

impl ErrorCode {
    /// All codes, in Anhang C order.
    pub const ALL: [ErrorCode; 18] = [
        Self::Conflict,
        Self::SnapshotExpired,
        Self::BeyondRetention,
        Self::Backpressure,
        Self::HandleUnknown,
        Self::HandleMacInvalid,
        Self::ForbiddenNamespace,
        Self::VectorInvalid,
        Self::BudgetTooSmall,
        Self::CorruptTruth,
        Self::GateRejected,
        Self::IdempotentReplay,
        Self::IdempotentMismatch,
        Self::StoreFailed,
        Self::QuotaExceeded,
        Self::ProvMissing,
        Self::SchemaViolation,
        Self::TrustInsufficient,
    ];

    /// The stable wire string, e.g. `"E_CONFLICT"`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Conflict => "E_CONFLICT",
            Self::SnapshotExpired => "E_SNAPSHOT_EXPIRED",
            Self::BeyondRetention => "E_BEYOND_RETENTION",
            Self::Backpressure => "E_BACKPRESSURE",
            Self::HandleUnknown => "E_HANDLE_UNKNOWN",
            Self::HandleMacInvalid => "E_HANDLE_MAC_INVALID",
            Self::ForbiddenNamespace => "E_FORBIDDEN_NAMESPACE",
            Self::VectorInvalid => "E_VECTOR_INVALID",
            Self::BudgetTooSmall => "E_BUDGET_TOO_SMALL",
            Self::CorruptTruth => "E_CORRUPT_TRUTH",
            Self::GateRejected => "E_GATE_REJECTED",
            Self::IdempotentReplay => "E_IDEMPOTENT_REPLAY",
            Self::IdempotentMismatch => "E_IDEMPOTENT_MISMATCH",
            Self::StoreFailed => "E_STORE_FAILED",
            Self::QuotaExceeded => "E_QUOTA_EXCEEDED",
            Self::ProvMissing => "E_PROV_MISSING",
            Self::SchemaViolation => "E_SCHEMA_VIOLATION",
            Self::TrustInsufficient => "E_TRUST_INSUFFICIENT",
        }
    }

    /// Retry semantics from Anhang C.
    pub const fn retry_policy(self) -> RetryPolicy {
        match self {
            Self::Conflict | Self::SnapshotExpired | Self::Backpressure => RetryPolicy::Retryable,
            Self::IdempotentReplay => RetryPolicy::NotAnError,
            _ => RetryPolicy::NotRetryable,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unknown wire string passed to [`ErrorCode::from_str`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownErrorCode(pub String);

impl fmt::Display for UnknownErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown error code {:?}", self.0)
    }
}

impl std::error::Error for UnknownErrorCode {}

impl FromStr for ErrorCode {
    type Err = UnknownErrorCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ErrorCode::ALL
            .into_iter()
            .find(|c| c.as_str() == s)
            .ok_or_else(|| UnknownErrorCode(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_codes_are_unique_and_have_e_prefix() {
        let set: HashSet<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(set.len(), 18);
        assert!(ErrorCode::ALL.iter().all(|c| c.as_str().starts_with("E_")));
    }

    #[test]
    fn wire_strings_roundtrip() {
        for c in ErrorCode::ALL {
            assert_eq!(c.as_str().parse::<ErrorCode>(), Ok(c));
        }
        assert!("E_NOPE".parse::<ErrorCode>().is_err());
    }

    #[test]
    fn retry_semantics_match_spec_table() {
        assert_eq!(ErrorCode::Conflict.retry_policy(), RetryPolicy::Retryable);
        assert_eq!(
            ErrorCode::SnapshotExpired.retry_policy(),
            RetryPolicy::Retryable
        );
        assert_eq!(
            ErrorCode::Backpressure.retry_policy(),
            RetryPolicy::Retryable
        );
        assert_eq!(
            ErrorCode::IdempotentReplay.retry_policy(),
            RetryPolicy::NotAnError
        );
        assert_eq!(
            ErrorCode::CorruptTruth.retry_policy(),
            RetryPolicy::NotRetryable
        );
        assert_eq!(
            ErrorCode::TrustInsufficient.retry_policy(),
            RetryPolicy::NotRetryable
        );
    }
}
