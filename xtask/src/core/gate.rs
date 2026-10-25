use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateContext {
    pub repo_root: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateOutcome {
    Pass,
    Fail { reason: String },
    NotApplicable { reason: String },
    Error { message: String },
}

impl GateOutcome {
    pub fn is_pass_or_na(&self) -> bool {
        matches!(self, GateOutcome::Pass | GateOutcome::NotApplicable { .. })
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            GateOutcome::Pass | GateOutcome::NotApplicable { .. } => 0,
            GateOutcome::Fail { .. } => 1,
            GateOutcome::Error { .. } => 2,
        }
    }
}

pub type Outcome = GateOutcome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateSpec {
    pub name: String,
    pub tier: String,
    pub from_phase: String,
    pub kind: String,
    pub enforces: Vec<String>,
    pub why: String,
    pub owner: String,
    pub cmd: Option<Vec<String>>,
    pub toolchain: Option<String>,
}

pub trait Gate {
    fn spec(&self) -> GateSpec;
    fn run(&self, ctx: &GateContext) -> GateOutcome;
    fn selftest(&self) -> GateOutcome;
}
