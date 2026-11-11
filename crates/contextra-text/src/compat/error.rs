//! Kompatibilitätsbrücke für Fehlerkonvertierungen (SPEC §4.3 / §5.3 / H7).

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
    fn test_error_mapping_table_driven() {
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
            (
                TextError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")),
                |ce| matches!(ce, ContextraError::Io(e) if e.kind() == std::io::ErrorKind::NotFound),
            ),
        ];

        for (te, predicate) in cases {
            let ce: ContextraError = te.into();
            assert!(predicate(&ce));
        }

        // Reverse conversions
        let c_inv = ContextraError::InvalidInput("inv".into());
        let t_inv: TextError = c_inv.into();
        assert!(matches!(t_inv, TextError::InvalidInput(s) if s == "inv"));

        let c_txt = ContextraError::Text("txt".into());
        let t_txt: TextError = c_txt.into();
        assert!(matches!(t_txt, TextError::Text(s) if s == "txt"));

        let c_stor = ContextraError::Storage("stor".into());
        let t_stor: TextError = c_stor.into();
        assert!(matches!(t_stor, TextError::Storage(s) if s == "stor"));

        let c_io = ContextraError::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"));
        let t_io: TextError = c_io.into();
        assert!(matches!(t_io, TextError::Storage(s) if s == "denied"));

        let c_other = ContextraError::Internal("internal err".into());
        let t_other: TextError = c_other.into();
        assert!(matches!(t_other, TextError::Internal(s) if s.contains("internal err")));
    }
}
