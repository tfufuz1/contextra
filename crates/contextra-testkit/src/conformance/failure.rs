use std::fmt;

/// Represents a failure encountered during a port conformance test execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Name of the suite (e.g. "StorageEngine", "VectorIndex", "TextIndex").
    pub suite_name: &'static str,
    /// Name of the specific test case that failed (e.g. "delete_removes_key").
    pub test_case: &'static str,
    /// Detailed description of the contract violation.
    pub details: String,
}

impl ConformanceFailure {
    /// Constructs a new [`ConformanceFailure`].
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

/// Specialized result type for conformance test executions.
pub type ConformanceResult = Result<(), ConformanceFailure>;
