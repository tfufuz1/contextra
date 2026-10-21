// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Generic crash matrix runner for systematic failpoint testing (Spezifikation T3).
// INVARIANTEN: Deterministic execution, zero panic, dead failpoint detection, thread-safe hit tracking.

use parking_lot::Mutex;
use std::collections::HashSet;
use std::fmt;
use std::future::Future;
use std::sync::LazyLock;

static HIT_REGISTRY: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Error types for crash matrix execution.
#[derive(Debug, PartialEq, Eq)]
pub enum CrashMatrixError {
    /// Scenario execution failed for a specific failpoint.
    ScenarioFailed {
        failpoint: &'static str,
        details: String,
    },
    /// One or more failpoints were declared but never triggered during scenario execution.
    DeadFailpoints(Vec<&'static str>),
}

impl fmt::Display for CrashMatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrashMatrixError::ScenarioFailed { failpoint, details } => {
                write!(
                    f,
                    "Scenario failed at failpoint '{}': {}",
                    failpoint, details
                )
            }
            CrashMatrixError::DeadFailpoints(dead) => {
                write!(f, "Dead failpoint(s) detected (never hit): {:?}", dead)
            }
        }
    }
}

impl std::error::Error for CrashMatrixError {}

/// Report summarizing crash matrix run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashMatrixReport {
    /// Seed used for deterministic simulation.
    pub seed: u64,
    /// Total failpoints evaluated.
    pub total_points: usize,
    /// Failpoints successfully triggered and verified.
    pub points_hit: Vec<&'static str>,
}

/// Generic crash matrix runner.
pub struct CrashMatrix {
    failpoints: &'static [&'static str],
    seed: u64,
}

impl CrashMatrix {
    /// Creates a new `CrashMatrix` with the specified failpoint list.
    pub fn new(failpoints: &'static [&'static str]) -> Self {
        Self {
            failpoints,
            seed: 0x4352_4153_4800_0000, // Default deterministic seed "CRASH\0\0\0"
        }
    }

    /// Sets the seed for deterministic execution.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Records that `failpoint` was hit/triggered during test execution.
    pub fn record_hit(failpoint: &str) {
        let mut reg = HIT_REGISTRY.lock();
        reg.insert(failpoint.to_string());
    }

    /// Clears the global failpoint hit registry.
    pub fn clear_hits() {
        let mut reg = HIT_REGISTRY.lock();
        reg.clear();
    }

    /// Returns whether `failpoint` has been recorded as hit.
    pub fn is_hit(failpoint: &str) -> bool {
        let reg = HIT_REGISTRY.lock();
        reg.contains(failpoint)
    }

    /// Runs the provided scenario function for every failpoint in sequence.
    ///
    /// Verifies that each failpoint is hit during its scenario execution.
    /// If any failpoint is never hit, returns `Err(CrashMatrixError::DeadFailpoints)`.
    pub async fn run<F, Fut, E>(
        &self,
        scenario_runner: F,
    ) -> Result<CrashMatrixReport, CrashMatrixError>
    where
        F: Fn(&'static str) -> Fut,
        Fut: Future<Output = Result<(), E>>,
        E: fmt::Display,
    {
        let mut dead_points = Vec::new();
        let mut hit_points = Vec::new();

        for &fp in self.failpoints {
            Self::clear_hits();

            if let Err(err) = scenario_runner(fp).await {
                return Err(CrashMatrixError::ScenarioFailed {
                    failpoint: fp,
                    details: err.to_string(),
                });
            }

            if Self::is_hit(fp) {
                hit_points.push(fp);
            } else {
                dead_points.push(fp);
            }
        }

        if !dead_points.is_empty() {
            Err(CrashMatrixError::DeadFailpoints(dead_points))
        } else {
            Ok(CrashMatrixReport {
                seed: self.seed,
                total_points: self.failpoints.len(),
                points_hit: hit_points,
            })
        }
    }
}
