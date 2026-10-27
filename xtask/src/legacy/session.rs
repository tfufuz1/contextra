use super::common::not_implemented;

pub fn run_session_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "jules-preflight"
        | "jules-submit-gate"
        | "session-init"
        | "context-pack"
        | "crate-context"
        | "symbol-exists"
        | "task-card"
        | "plan-lint"
        | "loop-guard"
        | "ledger"
        | "session-report"
        | "explain"
        | "postmortem"
        | "init-audit-fix"
        | "jules"
        | "pre-push" => not_implemented(cmd),
        _ => Err(format!("Unknown command in session cluster: {}", cmd)),
    }
}
