//! Fehlerbrücke zwischen `TextError` und `ContextraError` (SPEC §5.3).
// ANCHOR[contextra-text:COMPAT-ERR-001]

use crate::error::TextError;
use contextra_types::ContextraError;

impl From<TextError> for ContextraError {
    fn from(err: TextError) -> Self {
        match err {
            TextError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            TextError::Text(msg) => ContextraError::Text(msg),
            TextError::CapacityExceeded { requested, maximum } => {
                ContextraError::InvalidInput(format!(
                    "Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}"
                ))
            }
            TextError::Storage(msg) => ContextraError::Storage(msg),
            TextError::Serialization(msg) => ContextraError::Storage(msg),
            TextError::Io(e) => ContextraError::Io(e),
            TextError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}

impl From<ContextraError> for TextError {
    fn from(err: ContextraError) -> Self {
        match err {
            ContextraError::InvalidInput(msg) => TextError::InvalidInput(msg),
            ContextraError::Text(msg) => TextError::Text(msg),
            ContextraError::Storage(msg) => TextError::Storage(msg),
            ContextraError::Io(e) => TextError::Storage(e.to_string()),
            other => TextError::Internal(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_mapping_table_driven_bitgleich() {
        let cases: Vec<(TextError, fn(&ContextraError) -> bool)> = vec![
            (
                TextError::InvalidInput("bad query".into()),
                |ce| matches!(ce, ContextraError::InvalidInput(msg) if msg == "bad query"),
            ),
            (
                TextError::Text("regex fail".into()),
                |ce| matches!(ce, ContextraError::Text(msg) if msg == "regex fail"),
            ),
            (
                TextError::CapacityExceeded {
                    requested: 10,
                    maximum: 5,
                },
                |ce| matches!(ce, ContextraError::InvalidInput(msg) if msg == "Kapazitätsgrenze überschritten: gefordert 10, maximal 5"),
            ),
            (
                TextError::Storage("corrupt wal".into()),
                |ce| matches!(ce, ContextraError::Storage(msg) if msg == "corrupt wal"),
            ),
            (
                TextError::Serialization("bincode err".into()),
                |ce| matches!(ce, ContextraError::Storage(msg) if msg == "bincode err"),
            ),
            (
                TextError::Internal("state lock".into()),
                |ce| matches!(ce, ContextraError::Internal(msg) if msg == "state lock"),
            ),
        ];

        for (te, predicate) in cases {
            let te_display = te.to_string();
            let ce: ContextraError = te.into();
            assert!(predicate(&ce));
            // Display-Verhalten prüfen
            match &ce {
                ContextraError::InvalidInput(m) | ContextraError::Text(m) | ContextraError::Storage(m) | ContextraError::Internal(m) => {
                    assert!(te_display.contains(m));
                }
                _ => {}
            }
        }

        // Reverse conversion
        let c_inv = ContextraError::InvalidInput("inv".into());
        let t_inv: TextError = c_inv.into();
        assert!(matches!(t_inv, TextError::InvalidInput(s) if s == "inv"));
    }
}
