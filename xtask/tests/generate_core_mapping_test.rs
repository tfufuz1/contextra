use std::fs;
use std::path::Path;

#[test]
fn test_generate_core_mapping_execution_and_determinism() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let json_path = repo_root.join("docs/refactor/core-mapping.json");
    let md_path = repo_root.join("docs/refactor/core-mapping.md");

    // Execute first run
    let status1 = std::process::Command::new("cargo")
        .current_dir(&repo_root)
        .args(["xtask", "generate-core-mapping"])
        .status()
        .expect("cargo xtask generate-core-mapping failed to launch");
    assert!(status1.success(), "First run of generate-core-mapping failed");

    assert!(json_path.exists(), "core-mapping.json was not generated");
    assert!(md_path.exists(), "core-mapping.md was not generated");

    let json_content1 = fs::read_to_string(&json_path).expect("failed to read core-mapping.json");
    let md_content1 = fs::read_to_string(&md_path).expect("failed to read core-mapping.md");

    // Execute second run to test determinism
    let status2 = std::process::Command::new("cargo")
        .current_dir(&repo_root)
        .args(["xtask", "generate-core-mapping"])
        .status()
        .expect("second cargo xtask generate-core-mapping failed to launch");
    assert!(status2.success(), "Second run of generate-core-mapping failed");

    let json_content2 = fs::read_to_string(&json_path).expect("failed to read core-mapping.json");
    let md_content2 = fs::read_to_string(&md_path).expect("failed to read core-mapping.md");

    assert_eq!(
        json_content1, json_content2,
        "Determinism check failed: core-mapping.json differs between consecutive runs"
    );
    assert_eq!(
        md_content1, md_content2,
        "Determinism check failed: core-mapping.md differs between consecutive runs"
    );

    // Parse JSON report structure
    let report: serde_json::Value = serde_json::from_str(&json_content1).expect("Failed to parse core-mapping.json");

    let total_symbols = report["total_symbols"].as_u64().expect("total_symbols missing");
    let entries = report["entries"].as_array().expect("entries missing");

    assert!(total_symbols > 0, "Expected at least 1 re-exported symbol in contextra-core");
    assert_eq!(total_symbols as usize, entries.len(), "total_symbols does not match entries array length");

    // Check specific required re-export types & traits from contextra-core
    let required_symbols = ["ContextraError", "TxBuffer", "IndexOp", "StorageEngine", "WireWiring"];
    for sym in required_symbols {
        let entry = entries.iter().find(|e| e["symbol"] == sym);
        assert!(
            entry.is_some(),
            "Required symbol {} was not found in core-mapping.json",
            sym
        );
    }
}
