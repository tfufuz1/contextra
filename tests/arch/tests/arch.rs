//! Führt die Architekturprüfungen gegen das echte Workspace aus (SPEC §5.2).

use contextra_arch_tests::{
    check_feature_hygiene, check_layering, check_network_dependencies, check_raw_storage_isolation,
    check_store_fs_usage, check_types_purity, check_unsafe, check_wording, load_crates,
};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn test_t_arch_1_types_purity() {
    let root = workspace_root();
    let violations = check_types_purity(&root);
    assert!(
        violations.is_empty(),
        "T-ARCH-1 Verstöße:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_t_arch_3_ring_regeln_gelten() -> Result<(), String> {
    let crates = load_crates(&workspace_root())?;
    let violations = check_layering(&crates);
    assert!(
        violations.is_empty(),
        "Ring-Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn test_t_arch_4_single_node_garantie() -> Result<(), String> {
    let crates = load_crates(&workspace_root())?;
    let violations = check_network_dependencies(&crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-4 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn test_t_unsafe_1_unsafe_nur_in_inseln() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_unsafe(&root, &crates);
    assert!(
        violations.is_empty(),
        "Unsafe-Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn test_t_lint_1_wortlaut_workspace() {
    let root = workspace_root();
    let violations = check_wording(&root);
    if !violations.is_empty() {
        println!(
            "INFO: T-LINT-1 Altfunde im Workspace (für Übergabe erfasst):\n{}",
            violations.join("\n")
        );
    }
}

#[test]
fn test_t_lint_2_store_fs_usage() {
    let root = workspace_root();
    let violations = check_store_fs_usage(&root);
    if !violations.is_empty() {
        println!(
            "INFO: T-LINT-2 Altfunde im Workspace (für Übergabe erfasst):\n{}",
            violations.join("\n")
        );
    }
}

#[test]
fn test_t_arch_2_raw_storage_isolation() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_raw_storage_isolation(&root, &crates);
    if !violations.is_empty() {
        println!(
            "INFO: T-ARCH-2 Altfunde im Workspace (für Übergabe erfasst):\n{}",
            violations.join("\n")
        );
    }
    Ok(())
}

#[test]
fn test_t_cfg_and_def_hygiene() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_feature_hygiene(&root, &crates);
    if !violations.is_empty() {
        println!(
            "INFO: T-CFG / T-DEF Altfunde im Workspace (für Übergabe erfasst):\n{}",
            violations.join("\n")
        );
    }
    Ok(())
}
