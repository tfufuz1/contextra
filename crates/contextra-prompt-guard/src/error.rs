//! Fehler-Typen für Prompt-Injection-Erkennung und Validierung.

use thiserror::Error;

/// Fehler beim Laden, Parsen oder Verarbeiten von Prompt-Injection-Regeln und -Audits.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum GuardError {
    /// Dateizugriffs- oder I/O-Fehler beim Laden von Konfigurationen oder Schreiben von Audit-Logs.
    #[error("I/O-Fehler beim Guard-Betrieb: {0}")]
    Io(String),

    /// Serialisierungs- oder Deserialisierungsfehler im Konfigurations- oder Audit-Format.
    #[error("Format- oder Parsefehler im Guard-Schema: {0}")]
    Serialization(String),

    /// Validierungsfehler in Mustern oder Konfigurationsparametern.
    #[error("Ungültige Guard-Konfiguration: {0}")]
    InvalidConfiguration(String),

    /// Fehler beim Dekodieren verschachtelter Payloads (z.B. ungültiges UTF-8 nach Base64-Dekodierung).
    #[error("Fehler beim Dekodieren von Inhalten: {0}")]
    DecodeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_error_display() {
        let err_io = GuardError::Io("file not found".to_string());
        assert_eq!(
            err_io.to_string(),
            "I/O-Fehler beim Guard-Betrieb: file not found"
        );

        let err_ser = GuardError::Serialization("invalid json".to_string());
        assert_eq!(
            err_ser.to_string(),
            "Format- oder Parsefehler im Guard-Schema: invalid json"
        );

        let err_cfg = GuardError::InvalidConfiguration("empty pattern".to_string());
        assert_eq!(
            err_cfg.to_string(),
            "Ungültige Guard-Konfiguration: empty pattern"
        );

        let err_dec = GuardError::DecodeError("invalid utf8".to_string());
        assert_eq!(
            err_dec.to_string(),
            "Fehler beim Dekodieren von Inhalten: invalid utf8"
        );
    }

    #[test]
    fn test_guard_error_table_driven_display() {
        let test_cases: Vec<(GuardError, &'static str)> = vec![
            (
                GuardError::Io("test-io-error".into()),
                "I/O-Fehler beim Guard-Betrieb: test-io-error",
            ),
            (
                GuardError::Serialization("test-ser-error".into()),
                "Format- oder Parsefehler im Guard-Schema: test-ser-error",
            ),
            (
                GuardError::InvalidConfiguration("test-cfg-error".into()),
                "Ungültige Guard-Konfiguration: test-cfg-error",
            ),
            (
                GuardError::DecodeError("test-dec-error".into()),
                "Fehler beim Dekodieren von Inhalten: test-dec-error",
            ),
        ];

        for (err, expected_display) in test_cases {
            assert_eq!(err.to_string(), expected_display);
        }
    }
}
