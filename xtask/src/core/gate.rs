use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail { reason: String },
    NotApplicable { reason: String },
    Error { message: String },
    NotRun { reason: String },
    NotDue { reason: String },
    OtherTier { tier: String },
}

impl Outcome {
    pub fn is_pass_or_na(&self) -> bool {
        matches!(
            self,
            Outcome::Pass | Outcome::NotApplicable { .. } | Outcome::NotDue { .. } | Outcome::OtherTier { .. }
        )
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Outcome::Pass | Outcome::NotApplicable { .. } | Outcome::NotDue { .. } | Outcome::OtherTier { .. } => 0,
            Outcome::Fail { .. } => 1,
            Outcome::Error { .. } | Outcome::NotRun { .. } => 2,
        }
    }

    pub fn status_str(&self) -> String {
        match self {
            Outcome::Pass => "PASS".to_string(),
            Outcome::Fail { reason } => format!("FAIL ({})", reason),
            Outcome::NotApplicable { reason } => format!("N/A ({})", reason),
            Outcome::Error { message } => format!("ERROR ({})", message),
            Outcome::NotRun { reason } => format!("NOT RUN ({})", reason),
            Outcome::NotDue { reason } => format!("NOT DUE ({})", reason),
            Outcome::OtherTier { tier } => format!("OTHER TIER ({})", tier),
        }
    }
}

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
    fn run(&self, ctx: &super::ctx::Context) -> Outcome;
    fn selftest(&self) -> Outcome;
}
