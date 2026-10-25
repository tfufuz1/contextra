use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail { reason: String },
    NotApplicable { reason: String },
    Error { message: String },
}

impl Outcome {
    pub fn is_pass_or_na(&self) -> bool {
        matches!(self, Outcome::Pass | Outcome::NotApplicable { .. })
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Outcome::Pass | Outcome::NotApplicable { .. } => 0,
            Outcome::Fail { .. } => 1,
            Outcome::Error { .. } => 2,
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
