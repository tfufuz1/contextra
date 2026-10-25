use serde::{Deserialize, Serialize};

use super::evidence::EvidenceRecord;
use super::gate::Outcome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGateItem {
    pub gate: String,
    pub outcome: Outcome,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub overall_passed: bool,
    pub total_gates: usize,
    pub passed_gates: usize,
    pub failed_gates: usize,
    pub error_gates: usize,
    pub items: Vec<ReportGateItem>,
}

impl VerificationReport {
    pub fn build(records: &[(EvidenceRecord, Outcome)]) -> Self {
        let mut passed_gates = 0;
        let mut failed_gates = 0;
        let mut error_gates = 0;
        let mut items = Vec::new();

        for (rec, outcome) in records {
            match outcome {
                Outcome::Pass | Outcome::NotApplicable { .. } => passed_gates += 1,
                Outcome::Fail { .. } => failed_gates += 1,
                Outcome::Error { .. } => error_gates += 1,
            }
            items.push(ReportGateItem {
                gate: rec.gate.clone(),
                outcome: outcome.clone(),
                duration_ms: rec.duration_ms,
            });
        }

        let overall_passed = failed_gates == 0 && error_gates == 0;

        VerificationReport {
            overall_passed,
            total_gates: records.len(),
            passed_gates,
            failed_gates,
            error_gates,
            items,
        }
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("=== VERIFICATION REPORT ===\n");
        for item in &self.items {
            let status = match &item.outcome {
                Outcome::Pass => "PASS".to_string(),
                Outcome::NotApplicable { reason } => format!("N/A ({})", reason),
                Outcome::Fail { reason } => format!("FAIL ({})", reason),
                Outcome::Error { message } => format!("ERROR ({})", message),
            };
            out.push_str(&format!("{:<30} {:<30} ({} ms)\n", item.gate, status, item.duration_ms));
        }
        out.push_str("----------------------------\n");
        out.push_str(&format!(
            "Summary: {} total, {} passed/N/A, {} failed, {} errors\n",
            self.total_gates, self.passed_gates, self.failed_gates, self.error_gates
        ));
        if self.overall_passed {
            out.push_str("Result: PASSED\n");
        } else {
            out.push_str("Result: FAILED\n");
        }
        out
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to render report as JSON: {}", e))
    }
}
