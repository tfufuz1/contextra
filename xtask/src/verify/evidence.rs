// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Verification evidence recording structure and format.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    Passed,
    Failed,
    SkippedToolMissing,
    SkippedByFlag,
    Error,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Passed => write!(f, "PASSED"),
            StepStatus::Failed => write!(f, "FAILED"),
            StepStatus::SkippedToolMissing => write!(f, "SKIPPED_TOOL_MISSING"),
            StepStatus::SkippedByFlag => write!(f, "SKIPPED_BY_FLAG"),
            StepStatus::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvidence {
    pub step_id: String,
    pub status: StepStatus,
    pub argv: Vec<String>,
    pub toolchain: String,
    pub commit: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub log_path: String,
    pub log_sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub run_id: String,
    pub tier: String,
    pub timestamp_utc: String,
    pub overall_status: StepStatus,
    pub steps: Vec<StepEvidence>,
}
