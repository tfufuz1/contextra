#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Table-driven Test zur Verifikation bitgleicher Display-Ausgaben für KvCacheError und ContextraError (KV3.3).

use contextra_kvcache::error::KvCacheError;
use contextra_types::ContextraError;

#[test]
fn test_error_display_identity() {
    let cases: Vec<(KvCacheError, fn(&ContextraError) -> bool)> = vec![
        (
            KvCacheError::InvalidInput("test input error".into()),
            |ce| matches!(ce, ContextraError::InvalidInput(m) if m == "test input error"),
        ),
        (
            KvCacheError::CryptoError("encryption failed".into()),
            |ce| matches!(ce, ContextraError::Crypto(m) if m == "encryption failed"),
        ),
        (
            KvCacheError::PinBudgetExceeded("pin limit exceeded".into()),
            |ce| matches!(ce, ContextraError::PinBudgetExceeded(m) if m == "pin limit exceeded"),
        ),
        (
            KvCacheError::LimitExceeded {
                limit: 100,
                context: "capacity full".into(),
            },
            |ce| matches!(ce, ContextraError::LimitExceeded { limit, context } if *limit == 100 && context == "capacity full"),
        ),
        (
            KvCacheError::PolicyViolation("namespace mismatch".into()),
            |ce| matches!(ce, ContextraError::PolicyViolation(m) if m == "namespace mismatch"),
        ),
        (
            KvCacheError::Serialization("bincode error".into()),
            |ce| matches!(ce, ContextraError::Serialization(m) if m == "bincode error"),
        ),
        (
            KvCacheError::QuantizationError("scale non-finite".into()),
            |ce| matches!(ce, ContextraError::KvQuantization(m) if m == "scale non-finite"),
        ),
        (
            KvCacheError::Internal("corrupt state".into()),
            |ce| matches!(ce, ContextraError::Internal(m) if m == "corrupt state"),
        ),
    ];

    for (kv_err, matcher) in cases {
        let kv_display = kv_err.to_string();
        let ce: ContextraError = kv_err.into();
        assert!(
            matcher(&ce),
            "Converted variant must match expected ContextraError variant"
        );
        let ce_display = ce.to_string();
        assert_eq!(
            kv_display, ce_display,
            "Display strings must be bit-identical"
        );
    }
}
