//! Fehlererfassung für Port-Konformitätsprüfungen (Spec 16.1, Z10, Z11).
//!
//! Vertragsverletzungen werden als strukturierte Werte gemeldet und nicht als Panic.
//! Nur der Testläufer macht daraus einen Fehlschlag (siehe `assert_conformance!`).

use std::fmt;

/// Ein Fehler während einer Konformitätsprüfung.
///
/// Enthält den Suitenamen, den fehlgeschlagenen Testfall und eine Beschreibung der
/// Vertragsverletzung. Die Angaben enthalten keine Personendaten und keine Kennungen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Name der ausgeführten Suite, zum Beispiel `CryptoRng`.
    pub suite_name: &'static str,
    /// Name des fehlgeschlagenen Testfalls, zum Beispiel `try_fill_ok`.
    pub test_case: &'static str,
    /// Beschreibung der festgestellten Vertragsverletzung.
    pub details: String,
}

impl ConformanceFailure {
    /// Erstellt einen neuen Konformitätsfehler.
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            details: details.into(),
        }
    }
}

impl fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.details
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Ergebnis eines Konformitätslaufs: `Ok(())` oder ein `ConformanceFailure`.
pub type ConformanceResult = Result<(), ConformanceFailure>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_suite_and_case() {
        let f = ConformanceFailure::new("CryptoRng", "try_fill_ok", "boom");
        assert_eq!(
            f.to_string(),
            "[CryptoRng] Conformance failure in test 'try_fill_ok': boom"
        );
    }

    #[test]
    fn result_alias_carries_failure() {
        let r: ConformanceResult = Err(ConformanceFailure::new("S", "c", "d"));
        assert!(matches!(r, Err(ref f) if f.test_case == "c"));
    }
}
