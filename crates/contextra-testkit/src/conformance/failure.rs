//! # Fehlererfassung für Port-Konformitätsprüfungen (`conformance/failure`)
//!
//! Dieses Modul definiert Fehlerstrukturen zur genauen Diagnose von
//! Vertragsverletzungen in Port-Implementierungen.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 10 (SPEC §3.2):** Fail-Closed: Vertragsabweichungen werden strukturiert
//!   als Fehlerobjekte gemeldet, statt unkontrolliert zu panicken oder ignoriert zu werden.

use std::fmt;

// ANCHOR[contextra-testkit:CONF-FAILURE-STRUCT-001]
/// Repräsentiert einen aufgetretenen Fehler während einer Konformitätsprüfung.
///
/// **Zweck:** Enthält den Suitenamen, den konkreten Testfall und eine menschenlesbare
/// Beschreibung der Vertragsverletzung, um Junior-Entwicklern bei der Behebung
/// von Adapter-Bugs sofort Klarheit zu verschaffen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Name der ausgeführten Suite (z. B. "StorageEngine", "VectorIndex", "Clock").
    pub suite_name: &'static str,
    /// Name des spezifischen fehlgeschlagenen Testfalls (z. B. "delete_removes_key").
    pub test_case: &'static str,
    /// Detaillierte Beschreibung der festgestellten Vertragsverletzung.
    pub details: String,
}

impl ConformanceFailure {
    // ANCHOR[contextra-testkit:CONF-FAILURE-NEW-001]
    /// Erstellt eine neue [`ConformanceFailure`]-Instanz.
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

// ANCHOR[contextra-testkit:CONF-FAILURE-DISPLAY-001]
impl fmt::Display for ConformanceFailure {
    /// Formatiert den Konformitätsfehler übersichtlich mit Suite und Testfall.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.details
        )
    }
}

impl std::error::Error for ConformanceFailure {}

// ANCHOR[contextra-testkit:CONF-RESULT-TYPE-001]
/// Spezialisierter Result-Typ für Konformitätsläufe: `Ok(())` oder `Err(ConformanceFailure)`.
pub type ConformanceResult = Result<(), ConformanceFailure>;
