//! # Charakterisierungstests: Signal-Konsistenz, Determinismus und Export-Golden-File
//!
//! SPEC-v2: §6.3, INV-ATOMIC-IDX-1, INV-INDEX-APPLY-1, INV-5 (Determinismus), VETO-F10 (Tenant-Isolation).

#![cfg(not(loom))]

use contextra_engine::ExportDocumentV1;
use sha2::{Digest, Sha256};

/// Test 1: Determinismus & Golden-File Prüfung (INV-5 / Format-Invarianz)
#[test]
fn test_export_golden_file_sha256_parity() {
    let golden_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/export_golden_v1.json"
    );
    let manifest_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/MANIFEST.sha256");

    let golden_content = std::fs::read_to_string(golden_path).expect("Golden file must exist");

    let mut hasher = Sha256::new();
    hasher.update(golden_content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let manifest_content =
        std::fs::read_to_string(manifest_path).expect("Manifest file must exist");
    let expected_hash = manifest_content.split_whitespace().next().unwrap_or("");
    assert_eq!(
        hash, expected_hash,
        "Golden file SHA256 must match MANIFEST.sha256"
    );

    let parsed_doc: ExportDocumentV1 =
        serde_json::from_str(&golden_content).expect("Valid JSON export document");
    assert_eq!(
        parsed_doc.schema_version,
        contextra_engine::SCHEMA_VERSION_V1
    );
    assert_eq!(parsed_doc.collections.len(), 1);
    assert_eq!(parsed_doc.collections[0].name, "default");
    assert_eq!(parsed_doc.collections[0].memories.len(), 2);
}

/// Test 2: Determinismus der Rangfusion (INV-5 / Tie-Break Invariante)
#[test]
fn test_fusion_determinism_and_tie_breaking() {
    use contextra_engine::fusion::{reciprocal_rank_fusion, SearchResult, SignalContribution};

    let set1 = vec![
        SearchResult {
            id: "doc-b".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-a".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
    ];

    let set2 = vec![
        SearchResult {
            id: "doc-a".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-b".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];

    // Beide Dokumente erhalten identischen fusionierten RRF-Score
    let fused = reciprocal_rank_fusion(vec![set1, set2], 10);
    assert_eq!(fused.len(), 2);
    // Bei gleichem Score greift stabiler Tie-Break nach ID aufsteigend: "doc-a" vor "doc-b"
    assert_eq!(fused[0].id, "doc-a");
    assert_eq!(fused[1].id, "doc-b");
}
