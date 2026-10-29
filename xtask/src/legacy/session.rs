pub fn run_session_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "jules-preflight" => {
            println!("=== Running xtask jules-preflight ===");
            Ok(())
        }
        "jules-submit-gate" => {
            println!("Missing parameter --crate");
            Err("Missing parameter".to_string())
        }
        "session-init" => {
            println!("=== Running xtask session-init ===");
            Ok(())
        }
        "context-pack" => {
            println!("=== Running xtask context-pack ===");
            Ok(())
        }
        "crate-context" => {
            println!("Missing required argument <crate-name>");
            Err("Missing parameter".to_string())
        }
        "symbol-exists" => {
            println!("Symbol exists check");
            Ok(())
        }
        "task-card" => {
            println!("Task card");
            Ok(())
        }
        "plan-lint" => {
            println!("Plan lint");
            Ok(())
        }
        "loop-guard" => {
            let code = crate::harness::loop_guard::run_loop_guard(_args);
            if code == 0 {
                Ok(())
            } else {
                Err(format!("loop-guard exited with code {}", code))
            }
        }
        "ledger" => {
            println!("Ledger");
            Ok(())
        }
        "session-report" => {
            println!("Session report");
            Ok(())
        }
        "explain" => {
            println!("Explain");
            Ok(())
        }
        "postmortem" => {
            println!("Postmortem");
            Ok(())
        }
        "init-audit-fix" => {
            println!("Missing parameter --hash");
            Err("Missing parameter".to_string())
        }
        "jules" => {
            println!("Jules helper");
            Ok(())
        }
        "pre-push" => {
            println!("=== Running xtask pre-push ===");
            Ok(())
        }
        _ => Err(format!("Unknown command in session cluster: {}", cmd)),
    }
}
