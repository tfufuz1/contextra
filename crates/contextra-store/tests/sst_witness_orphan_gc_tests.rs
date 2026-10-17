// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Tests für die Orphan- und Temp-Datei GC (D4.0 / AP-10).
// AP-10 BEFUND: Simulierter Abbruch (leftover .sst / .tmp Dateileichen) wird sicher erkannt und per
// durable_remove entfernt, während geschützte running_ops und Unknown Dateisystemobjekte unberührt bleiben.

use contextra_store::sstable::{plan_orphan_removal, remove_orphans, SstInventory, SstScanInputs};
use tempfile::tempdir;

#[tokio::test]
async fn test_orphan_gc_removes_leftovers_and_protects_running_ops_and_unknown() {
    let dir = tempdir().expect("tempdir");

    // 1. Create a valid live SSTable file
    let live_file = dir.path().join("sst_live_01.sst");
    tokio::fs::write(&live_file, b"valid_live_data_content")
        .await
        .expect("write live");

    // 2. Create an orphan SSTable file (simulated crash leftover)
    let orphan_file = dir.path().join("sst_orphan_99.sst");
    tokio::fs::write(&orphan_file, b"leftover_orphan_sst_data")
        .await
        .expect("write orphan");

    // 3. Create a temp file (simulated compaction leftover)
    let temp_file = dir.path().join("compact_abort.tmp");
    tokio::fs::write(&temp_file, b"leftover_temp_data")
        .await
        .expect("write temp");

    // 4. Create a temp file associated with a RUNNING compaction operation
    let running_temp = dir.path().join("active_flush.tmp");
    tokio::fs::write(&running_temp, b"active_flush_data")
        .await
        .expect("write running temp");

    // 5. Create an unknown file in data directory
    let unknown_file = dir.path().join("custom_user_blob.dat");
    tokio::fs::write(&unknown_file, b"user_custom_data")
        .await
        .expect("write unknown");

    let inputs = SstScanInputs {
        data_dirs: vec![dir.path().to_path_buf()],
        live_files: vec![live_file.clone()],
        pending_delete_files: vec![],
        manifest_generation: 1,
        as_of_tx: 100,
        depth: 0,
    };

    let inventory = SstInventory::build(&inputs).expect("build inventory");

    assert_eq!(inventory.live.len(), 1);
    assert_eq!(inventory.orphan.len(), 1);
    assert_eq!(inventory.temp.len(), 2);
    assert_eq!(inventory.unknown.len(), 1);

    // Plan orphan removal protecting running_temp
    let running_ops = vec![running_temp.clone()];
    let plan = plan_orphan_removal(&inventory, &running_ops);

    // Plan must include orphan_file and temp_file, BUT NOT running_temp and NOT unknown_file
    assert_eq!(plan.files_to_remove.len(), 2);
    assert!(plan.files_to_remove.contains(&orphan_file));
    assert!(plan.files_to_remove.contains(&temp_file));
    assert!(!plan.files_to_remove.contains(&running_temp));
    assert!(!plan.files_to_remove.contains(&unknown_file));

    // Execute removal
    let receipt = remove_orphans(&plan).expect("remove orphans");

    assert_eq!(receipt.files_removed, 2);
    assert!(receipt.bytes_freed > 0);

    // Assert files on disk:
    assert!(live_file.exists(), "Live file must be preserved");
    assert!(!orphan_file.exists(), "Orphan file must be deleted");
    assert!(!temp_file.exists(), "Temp file must be deleted");
    assert!(
        running_temp.exists(),
        "Running operation file must be protected and preserved"
    );
    assert!(
        unknown_file.exists(),
        "AP-10 Invariant: Unknown file must NEVER be deleted"
    );
}
