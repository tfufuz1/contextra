//! Tests for check_ring_capabilities_consistency module.

#[path = "../src/check_ring_layering.rs"]
mod check_ring_layering;

#[path = "../src/check_ring_capabilities_consistency.rs"]
mod check_ring_capabilities_consistency;

use check_ring_capabilities_consistency::run_check_ring_capabilities_consistency_in_root;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ring_capabilities_consistency_against_repo() {
    let root = xtask::find_root_dir();
    let findings = run_check_ring_capabilities_consistency_in_root(&root);
    assert!(
        findings.is_ok(),
        "Consistency check execution failed: {:?}",
        findings
    );
    let findings = findings.unwrap();
    // Verify that all active crates in capabilities.toml match check_ring_layering.rs
    assert!(
        findings.is_empty(),
        "Found ring inconsistencies between capabilities.toml and check_ring_layering.rs: {:#?}",
        findings
    );
}

#[test]
fn test_ring_capabilities_consistency_detects_mismatch() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();

    // Mismatched ring for contextra-types (Ring 1 instead of Ring 0)
    let caps_content = r#"
[crates.contextra-types]
ring = 'Ring 1'

[crates.unknown-crate]
ring = 'Ring 0'
"#;
    fs::write(root.join("capabilities.toml"), caps_content).unwrap();

    let findings = run_check_ring_capabilities_consistency_in_root(root).unwrap();

    assert_eq!(findings.len(), 2);

    let types_finding = findings
        .iter()
        .find(|f| f.crate_name == "contextra-types")
        .unwrap();
    assert_eq!(types_finding.declared_in_capabilities_toml, "Ring 1");
    assert_eq!(
        types_finding.declared_in_ring_layering.as_deref(),
        Some("Ring 0")
    );

    let unknown_finding = findings
        .iter()
        .find(|f| f.crate_name == "unknown-crate")
        .unwrap();
    assert_eq!(unknown_finding.declared_in_capabilities_toml, "Ring 0");
    assert_eq!(unknown_finding.declared_in_ring_layering, None);
}
