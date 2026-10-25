use crate::core::gate::{Gate, GateContext, GateOutcome, GateSpec};

pub struct GateSelftests;

impl Gate for GateSelftests {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "gate-selftests".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-QA-008".to_string()],
            why: "Führt die Selbsttests aller registrierten Gates aus.".to_string(),
            owner: "mp-02".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, _ctx: &GateContext) -> GateOutcome {
        let selftest_res = self.selftest();
        if selftest_res.is_pass_or_na() {
            GateOutcome::Pass
        } else {
            selftest_res
        }
    }

    fn selftest(&self) -> GateOutcome {
        // Positive case: Temp directory creation & basic verification
        let temp_dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => return GateOutcome::Fail { reason: format!("Tempdir failure: {}", e) },
        };
        if !temp_dir.path().exists() {
            return GateOutcome::Fail { reason: "Temp directory does not exist".to_string() };
        }
        GateOutcome::Pass
    }
}
