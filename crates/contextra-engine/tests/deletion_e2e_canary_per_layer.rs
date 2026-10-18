// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: E2E-Canary-Test je Layer mit Rohsweep Orakel fuer die Loeschgarantie (Feld 5, 10).
// INVARIANTEN: Zero-Panic, Fail-Closed, Scope: files_only.

#![forbid(unsafe_code)]

use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use contextra_testkit::deletion_oracle::{raw_sweep, OracleLayer, OrcMarkerSet};

#[test]
fn test_deletion_e2e_canary_per_layer_sweep() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    // 1. Generate canary marker set for all 7 canonical layers
    let marker_set = OrcMarkerSet::generate_per_layer("e2e_canary_sweep", 98765);
    let needles = marker_set.needles_multi_encoding();

    let memtable_marker = marker_set
        .layer_marker(OracleLayer::Memtable)
        .expect("memtable marker present");
    let sstable_marker = marker_set
        .layer_marker(OracleLayer::SSTable)
        .expect("sstable marker present");
    let wal_marker = marker_set
        .layer_marker(OracleLayer::WAL)
        .expect("wal marker present");
    let checkpoint_marker = marker_set
        .layer_marker(OracleLayer::Checkpoint)
        .expect("checkpoint marker present");
    let hnsw_marker = marker_set
        .layer_marker(OracleLayer::HNSW)
        .expect("hnsw marker present");
    let text_marker = marker_set
        .layer_marker(OracleLayer::Text)
        .expect("text marker present");
    let graph_marker = marker_set
        .layer_marker(OracleLayer::Graph)
        .expect("graph marker present");

    // 2. Populate DB directory with layer-specific files
    let memtable_path = root.join("memtable.buf");
    let sstable_path = root.join("000001.sst");
    let wal_path = root.join("000001.wal");
    let checkpoint_path = root.join("chk_01.snapshot");
    let hnsw_path = root.join("hnsw_index.bin");
    let text_path = root.join("text_index.bm25");
    let graph_path = root.join("graph.csr");

    // Write raw and multi-encoded representations across layers
    fs::write(&memtable_path, memtable_marker.as_bytes()).expect("write memtable");
    fs::write(&sstable_path, sstable_marker.as_bytes()).expect("write sstable");
    fs::write(&wal_path, wal_marker.as_bytes()).expect("write wal");
    fs::write(&checkpoint_path, checkpoint_marker.as_bytes()).expect("write checkpoint");
    fs::write(&hnsw_path, hnsw_marker.as_bytes()).expect("write hnsw");
    fs::write(&text_path, text_marker.as_bytes()).expect("write text");
    fs::write(&graph_path, graph_marker.as_bytes()).expect("write graph");

    // 3. Simulate physical deletion / purge pass:
    // Physical purge removes Memtable and SSTable layers.
    // Layers without physical purge support remain on disk as expected_residue.
    fs::remove_file(&memtable_path).expect("purge memtable file");
    fs::remove_file(&sstable_path).expect("purge sstable file");

    let expected_residue = vec![
        "000001.wal",
        "chk_01.snapshot",
        "hnsw_index.bin",
        "text_index.bm25",
        "graph.csr",
    ];

    // 4. Run raw sweep over DB directory
    let report = raw_sweep(root, &needles).expect("raw_sweep succeeded");

    // 5. Verify report invariants
    assert_eq!(
        report.scope, "files_only",
        "raw_sweep report scope must be files_only"
    );
    assert_eq!(report.files_scanned, 5, "5 remaining residue files scanned");
    assert_eq!(
        report.hits.len(),
        5,
        "5 hits detected in expected_residue files"
    );

    // Verify no hits exist for physically purged layers (Memtable & SSTable)
    for hit in &report.hits {
        let path_str = hit.relative_path.to_string_lossy();
        assert!(
            !path_str.contains("memtable.buf"),
            "purged memtable file must not appear in hits"
        );
        assert!(
            !path_str.contains("000001.sst"),
            "purged sstable file must not appear in hits"
        );
        assert!(
            expected_residue.iter().any(|r| path_str.contains(r)),
            "hit file must be in expected_residue list"
        );
    }

    // 6. Test-Selbstprüfung: Mutation "Orakel gibt immer sauber zurück" must fail
    assert!(
        report.assert_clean().is_err(),
        "Oracle assert_clean must fail when residual hits exist"
    );
}

#[test]
fn test_deletion_e2e_all_layers_purged_clean_sweep() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    let marker_set = OrcMarkerSet::generate_per_layer("e2e_clean_sweep", 12345);
    let needles = marker_set.needles_multi_encoding();

    // Create files for all 7 layers
    for layer in OracleLayer::all() {
        let marker = marker_set.layer_marker(*layer).expect("marker present");
        let file_path = root.join(format!("{}.dat", layer.as_str()));
        fs::write(&file_path, marker.as_bytes()).expect("write layer file");
    }

    // Simulate complete physical purge across all layers
    for layer in OracleLayer::all() {
        let file_path = root.join(format!("{}.dat", layer.as_str()));
        fs::remove_file(&file_path).expect("remove layer file");
    }

    let report = raw_sweep(root, &needles).expect("raw_sweep clean directory");

    assert_eq!(report.scope, "files_only");
    assert_eq!(report.files_scanned, 0);
    assert!(
        report.assert_clean().is_ok(),
        "report must be clean after complete purge"
    );
}

#[test]
#[ignore = "wird grün, wenn Purge (A08/A10) gemergt"]
fn test_deletion_e2e_unpurged_layer_fix_pending() {
    let dir = tempdir().expect("failed to create tempdir");
    let root = dir.path();

    let marker_set = OrcMarkerSet::generate_per_layer("e2e_pending_purge", 54321);
    let needles = marker_set.needles_multi_encoding();

    // Placeholder for E2E sweep after Purge (A08/A10) feature merge
    let report = raw_sweep(root, &needles).expect("raw_sweep pending purge");
    assert!(report.assert_clean().is_ok());
}
