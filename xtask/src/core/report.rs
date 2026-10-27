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
            // Outcome::NotDue and Outcome::OtherTier are not included in counted total/passed/failed/error totals.
            if matches!(outcome, Outcome::NotDue { .. } | Outcome::OtherTier { .. }) {
                items.push(ReportGateItem {
                    gate: rec.gate.clone(),
                    outcome: outcome.clone(),
                    duration_ms: rec.duration_ms,
                });
                continue;
            }

            match outcome {
                Outcome::Pass | Outcome::NotApplicable { .. } => passed_gates += 1,
                Outcome::Fail { .. } => failed_gates += 1,
                Outcome::Error { .. } | Outcome::NotRun { .. } => error_gates += 1,
                Outcome::NotDue { .. } | Outcome::OtherTier { .. } => unreachable!(),
            }
            items.push(ReportGateItem {
                gate: rec.gate.clone(),
                outcome: outcome.clone(),
                duration_ms: rec.duration_ms,
            });
        }

        let counted_total = passed_gates + failed_gates + error_gates;
        let overall_passed = failed_gates == 0 && error_gates == 0;

        VerificationReport {
            overall_passed,
            total_gates: counted_total,
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
            let status = item.outcome.status_str();
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
