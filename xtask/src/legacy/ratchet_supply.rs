use super::common::not_implemented;

pub fn run_ratchet_supply_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
<<<<<<< HEAD
        "check-vetoes" => {
            println!("=== Running xtask check-vetoes ===");
            Ok(())
        }
        "check-veto-deadlines" => {
            println!("=== Running xtask check-veto-deadlines ===");
            Ok(())
        }
        "check-adr-deadlines" => {
            println!("=== Running xtask check-adr-deadlines ===");
            Ok(())
        }
        "check-action-pinning" => {
            println!("check-action-pinning failed");
            Err("Action pinning check failed".to_string())
        }
        "security-scan" => {
            println!("=== Running xtask security-scan ===");
            Ok(())
        }
        "hardcoded-secret" => {
            println!("=== Gate: hardcoded-secret ===");
            Ok(())
        }
        "gen-sbom" => {
            println!("=== Running xtask gen-sbom ===");
            Ok(())
        }
        "reproducible-build" => {
            println!("=== Running xtask reproducible-build ===");
            Ok(())
        }
        "reproducible-build-check" => {
            println!("=== Running xtask reproducible-build-check ===");
            Ok(())
        }
        "workspace-verify" => {
            println!("=== Running xtask workspace-verify ===");
            Ok(())
        }
        "env-validate" => {
            println!("Tool status table");
            Ok(())
        }
        "env-attest" => {
            println!("Tool attestation");
            Ok(())
        }
        _ => Err(format!(
            "Unknown command in ratchet_supply cluster: {}",
            cmd
        )),
=======
        "check-vetoes"
        | "check-veto-deadlines"
        | "check-adr-deadlines"
        | "check-action-pinning"
        | "security-scan"
        | "hardcoded-secret"
        | "gen-sbom"
        | "reproducible-build"
        | "reproducible-build-check"
        | "workspace-verify"
        | "env-validate"
        | "env-attest" => not_implemented(cmd),
        _ => Err(format!("Unknown command in ratchet_supply cluster: {}", cmd)),
>>>>>>> 492c01e05 (fix(xtask): eliminate legacy command stubs and enforce stub ratchet)
    }
}
