use super::common::not_implemented;

pub fn run_code_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "check-max-results-unbound"
        | "check-toctou-defaults"
        | "check-nan-hot-loop"
        | "check-result-dropped-io"
        | "check-ffi-panic-boundary"
        | "std-fs-in-async"
        | "panic-inventory"
        | "debt-audit"
        | "check-compile" => not_implemented(cmd),
        _ => Err(format!("Unknown command in code cluster: {}", cmd)),
    }
}
