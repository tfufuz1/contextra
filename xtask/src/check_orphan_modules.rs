//! Legacy-Alias: `check-orphan-modules` delegiert an `check-module-reachability`.

#[allow(unused_imports)]
pub use crate::check_module_reachability::{
    check_crate_reachability, run_check_module_reachability, ModuleReachabilityResult,
    KNOWN_TRANSITION_UNREACHABLE,
};

#[allow(dead_code)]
pub fn run_check_orphan_modules(root: &std::path::Path) -> Result<Vec<String>, String> {
    let res = run_check_module_reachability(root)?;
    Ok(res.errors)
}
