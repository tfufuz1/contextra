use super::common::not_implemented;

pub fn run_arch_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "check-ring-layering"
        | "check-ring-layering-full"
        | "check-dag"
        | "check-ring-capabilities-consistency"
        | "check-ring0-async-purity"
        | "check-unsafe-islands"
        | "lint-unsafe-slices"
        | "check-manifest-completeness"
        | "check-module-reachability"
        | "check-orphan-modules"
        | "check-crate-references"
        | "feature-matrix"
        | "check-duplicate-core-primitives"
        | "check-duplicate-symbols"
        | "check-duplicate-symbols-cross-file"
        | "check-type-registry" => not_implemented(cmd),
        _ => Err(format!("Unknown command in arch cluster: {}", cmd)),
    }
}
