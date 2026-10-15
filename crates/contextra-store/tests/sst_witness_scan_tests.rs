// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Injektionsmatrix-Tests (Zeilen 4–12) für den SSTable Witness Scanner.

use contextra_store::sstable::{scan_sstables, SstScanInputs, SstScopeSpec, SstableBuilder};
use tempfile::tempdir;

#[tokio::test]
async fn test_matrix_line_4_put_in_l0() {
    let dir = tempdir().expect("tempdir");
    let sst_path = dir.path().join("sst_l0.sst");

    let mut builder = SstableBuilder::create(&sst_path).await.expect("builder");
    builder
        .add(b"user_1001", b"payload_active", 10, 100)
        .await
        .expect("add");
    builder.finish().await.expect("finish");

    let scope = SstScopeSpec {
        explicit: vec![b"user_1001".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![sst_path.clone()],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.versions_found, 1);
    assert!(!evidence.invariant_violations().is_empty());
}

#[tokio::test]
async fn test_matrix_line_5_older_value_in_ln_shadowed_by_tombstone_in_l0_control_test() {
    let dir = tempdir().expect("tempdir");

    // L0 SSTable has Tombstone for key user_2002 at seq 20
    let l0_path = dir.path().join("sst_l0.sst");
    let mut l0_builder = SstableBuilder::create(&l0_path).await.expect("l0 builder");
    l0_builder
        .add(b"user_2002", b"", 20 | contextra_types::TOMBSTONE_BIT, 200)
        .await
        .expect("add tombstone");
    l0_builder.finish().await.expect("finish l0");

    // Ln SSTable has Older Value for key user_2002 at seq 10
    let ln_path = dir.path().join("sst_ln.sst");
    let mut ln_builder = SstableBuilder::create(&ln_path).await.expect("ln builder");
    ln_builder
        .add(b"user_2002", b"historical_value_before_delete", 10, 100)
        .await
        .expect("add old value");
    ln_builder.finish().await.expect("finish ln");

    let scope = SstScopeSpec {
        explicit: vec![b"user_2002".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![l0_path.clone(), ln_path.clone()],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    // CRITICAL CONTROL BELEG:
    // A live-semantics point-lookup (production reader) returns None (Key Deleted by L0 tombstone).
    // BUT the Witness Scanner detects the historical version in Ln!
    assert_eq!(
        evidence.versions_found, 1,
        "Witness scanner MUST detect older value in Ln despite L0 tombstone"
    );
    assert_eq!(
        evidence.tombstones_remaining, 1,
        "Witness scanner MUST also record L0 tombstone"
    );
    assert!(
        !evidence.invariant_violations().is_empty(),
        "CT-S2V claim must be violated because historical value exists in Ln"
    );
}

#[tokio::test]
async fn test_matrix_line_6_only_tombstone() {
    let dir = tempdir().expect("tempdir");
    let sst_path = dir.path().join("sst_tombstone_only.sst");

    let mut builder = SstableBuilder::create(&sst_path).await.expect("builder");
    builder
        .add(b"user_3003", b"", 50 | contextra_types::TOMBSTONE_BIT, 500)
        .await
        .expect("add tombstone");
    builder.finish().await.expect("finish");

    let scope = SstScopeSpec {
        explicit: vec![b"user_3003".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![sst_path.clone()],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    // S2-V is OK (no live value found), BUT S2-K triggers (tombstone exists)
    assert_eq!(evidence.versions_found, 0);
    assert_eq!(evidence.tombstones_remaining, 1);
    let violations = evidence.invariant_violations();
    assert!(violations.iter().any(|v| v.contains("CT-S2K")));
}

#[tokio::test]
async fn test_matrix_line_7_scope_prefix_only_in_index_separator() {
    let dir = tempdir().expect("tempdir");
    let sst_path = dir.path().join("sst_index.sst");

    let mut builder = SstableBuilder::create(&sst_path).await.expect("builder");
    builder
        .add(b"prefix_fence_01", b"val1", 1, 10)
        .await
        .expect("add");
    builder.finish().await.expect("finish");

    let scope = SstScopeSpec {
        explicit: vec![],
        prefixes: vec![b"prefix_fence_".to_vec()],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![sst_path.clone()],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert!(evidence.identifier_hits_index > 0);
}

#[tokio::test]
async fn test_matrix_line_8_orphan_outside_manifest() {
    let dir = tempdir().expect("tempdir");

    // Orphan SSTable in data directory, NOT listed in live_files
    let orphan_path = dir.path().join("sst_orphan_99.sst");
    let mut builder = SstableBuilder::create(&orphan_path).await.expect("builder");
    builder
        .add(b"orphan_key", b"orphan_payload", 1, 10)
        .await
        .expect("add");
    builder.finish().await.expect("finish");

    let scope = SstScopeSpec {
        explicit: vec![b"orphan_key".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![], // Empty manifest live_files
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.orphan_files_found, 1);
    assert_eq!(evidence.versions_found, 1);
}

#[tokio::test]
async fn test_matrix_line_9_tmp_file_with_data() {
    let dir = tempdir().expect("tempdir");

    let tmp_path = dir.path().join("compact_inprogress.tmp");
    let mut builder = SstableBuilder::create(&tmp_path).await.expect("builder");
    builder
        .add(b"tmp_key", b"tmp_val", 1, 10)
        .await
        .expect("add");
    builder.finish().await.expect("finish");

    let scope = SstScopeSpec {
        explicit: vec![b"tmp_key".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.versions_found, 1);
}

#[tokio::test]
async fn test_matrix_line_10_old_manifest_file() {
    let dir = tempdir().expect("tempdir");

    let old_manifest_path = dir.path().join("MANIFEST.old");
    tokio::fs::write(
        &old_manifest_path,
        b"header_data_prefix_secret_target_footer",
    )
    .await
    .expect("write old manifest");

    let scope = SstScopeSpec {
        explicit: vec![],
        prefixes: vec![b"prefix_secret_target".to_vec()],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.manifests_scanned, 1);
    assert_eq!(evidence.range_hits, 1);
}

#[tokio::test]
async fn test_matrix_line_11_corrupted_footer_file() {
    let dir = tempdir().expect("tempdir");

    let corrupt_sst_path = dir.path().join("corrupt.sst");
    tokio::fs::write(
        &corrupt_sst_path,
        b"invalid_corrupt_sst_data_without_footer",
    )
    .await
    .expect("write corrupt");

    let scope = SstScopeSpec {
        explicit: vec![b"any_key".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.unclassified_files, 1);
    assert!(evidence
        .invariant_violations()
        .iter()
        .any(|v| v.contains("Fail-closed")));
}

#[tokio::test]
async fn test_matrix_line_12_unknown_file_in_data_directory() {
    let dir = tempdir().expect("tempdir");

    let unknown_file = dir.path().join("unknown_blob.dat");
    tokio::fs::write(&unknown_file, b"random_unknown_contents")
        .await
        .expect("write unknown");

    let scope = SstScopeSpec {
        explicit: vec![b"any_key".to_vec()],
        prefixes: vec![],
    };

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 1000,
        depth: 0,
    };

    let evidence = scan_sstables(&scope, &inputs, None, || 1).expect("scan");

    assert_eq!(evidence.unclassified_files, 1);
    assert!(evidence
        .invariant_violations()
        .iter()
        .any(|v| v.contains("Fail-closed")));
}
