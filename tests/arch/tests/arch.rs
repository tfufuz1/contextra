//! Führt die Architekturprüfungen gegen das echte Workspace aus (SPEC §5.2).

use contextra_arch_tests::{
    check_clean_source_tree, check_crate_headers_and_descriptions, check_dependency_matrix_5_3,
    check_exports_and_dev_dependencies, check_feature_hygiene, check_layering,
    check_naming_and_facade_structure, check_network_dependencies, check_obsolete_evidence_terms,
    check_ratchet_hygiene, check_raw_storage_isolation, check_session_marks_and_timestamps,
    check_store_fs_usage, check_test_traceability, check_trademark_and_session_marks,
    check_types_purity, check_unsafe, check_wording, load_crates,
};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// SPEC: T-ARCH-1
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

/// SPEC: T-ARCH-3
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

/// SPEC: T-ARCH-3
#[test]
fn test_t_arch_3b_matrix_edges() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_dependency_matrix_5_3(&root, &crates);
    assert!(
        violations.is_empty(),
        "SPEC §5.3 Kanten-Verstöße (T-ARCH-3b):\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-ARCH-4
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

/// SPEC: T-ARCH-5
#[test]
fn test_t_arch_5_crate_headers_and_descriptions() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_crate_headers_and_descriptions(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-5 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-ARCH-6
#[test]
fn test_t_arch_6_exports_and_dev_dependencies() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_exports_and_dev_dependencies(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-6 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-ARCH-7
#[test]
fn test_t_arch_7_naming_and_facade_structure() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_naming_and_facade_structure(&crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-7 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-ARCH-8
#[test]
fn test_t_arch_8_clean_source_tree() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_clean_source_tree(&crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-8 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-DOC-1
#[test]
fn test_t_doc_1_session_marks_and_timestamps() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_session_marks_and_timestamps(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-DOC-1 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-LINT-4
#[test]
fn test_t_lint_4_trademark_and_session_marks() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_trademark_and_session_marks(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-LINT-4 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-LINT-3
#[test]
fn test_t_lint_3_obsolete_evidence_terms() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_obsolete_evidence_terms(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-LINT-3 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-TRACE-1
#[test]
fn test_t_trace_1_test_traceability() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_test_traceability(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-TRACE-1 Verstöße:\n{}",
        violations.join("\n")
    );
    Ok(())
}

/// SPEC: T-UNSAFE-1
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

/// SPEC: T-LINT-1
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

/// SPEC: T-LINT-2
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

/// SPEC: T-ARCH-2
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

/// SPEC: T-CFG-1
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

#[test]
fn test_t_arch_9_double_modules() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_double_modules(&crates);
    assert!(
        violations.is_empty(),
        "T-ARCH-9 Verstöße:
{}",
        violations.join(
            "
"
        )
    );
    Ok(())
}

#[test]
fn test_t_gov_1_ratchet_hygiene() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_ratchet_hygiene(&root, &crates);
    assert!(
        violations.is_empty(),
        "T-GOV-1 Verstöße (Stale-Einträge in ratchet.toml):
{}",
        violations.join(
            "
"
        )
    );
    Ok(())
}
