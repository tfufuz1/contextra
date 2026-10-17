// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Tests für Kopienregister (CopyRegistry) — Registration, Listing, Holds, Removal, Idempotenz & Crash-Recovery.

use contextra_store::copy_registry::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_copy_registry_lifecycle() {
    let dir = tempdir().unwrap();

    let mut reg = CopyRegistry::open(dir.path()).await.unwrap();
    assert_eq!(reg.list().len(), 0);

    let file_id1 = [1u8; 16];
    let file_id2 = [2u8; 16];
    let copy_id1 = [10u8; 16];

    let entry1 = CopyRegEntry {
        copy_id: copy_id1,
        kind: CopyKind::Hardlink,
        file_ids: vec![file_id1, file_id2],
        created_tx: 100,
    };

    reg.register(entry1.clone()).await.unwrap();
    assert_eq!(reg.list().len(), 1);

    // Idempotent re-registration
    reg.register(entry1.clone()).await.unwrap();
    assert_eq!(reg.list().len(), 1);

    // Reopen registry from disk
    let reg2 = CopyRegistry::open(dir.path()).await.unwrap();
    assert_eq!(reg2.list().len(), 1);
    assert_eq!(reg2.list()[0], entry1);

    // Query holds_files
    let matched = reg2.holds_files(&[file_id1]);
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0], copy_id1);

    let unmatched = reg2.holds_files(&[[99u8; 16]]);
    assert_eq!(unmatched.len(), 0);

    // Query holds_scope_files
    let scope_matched = reg2.holds_scope_files(&[file_id1]);
    assert_eq!(scope_matched.len(), 1);
    assert_eq!(scope_matched[0], copy_id1);

    // Remove registration
    let mut reg3 = reg2;
    reg3.remove(&copy_id1).await.unwrap();
    assert_eq!(reg3.list().len(), 0);

    // Reopen after removal
    let reg4 = CopyRegistry::open(dir.path()).await.unwrap();
    assert_eq!(reg4.list().len(), 0);
}

#[test]
fn test_file_id_for_name_contract() {
    let name1 = "000001.sst";
    let name2 = "000002.sst";

    let fid1 = file_id_for_name(name1);
    let fid1_dup = file_id_for_name(name1);
    let fid2 = file_id_for_name(name2);

    assert_eq!(fid1, fid1_dup, "file_id_for_name must be deterministic");
    assert_ne!(
        fid1, fid2,
        "file_id_for_name must not collide for different file names"
    );

    // Test a batch of names for collisions
    let mut set = std::collections::HashSet::new();
    for i in 0..1000 {
        let name = format!("{:06}.sst", i);
        let fid = file_id_for_name(&name);
        assert!(
            set.insert(fid),
            "Collision detected for file_id_for_name at index {i}"
        );
    }
}

#[tokio::test]
async fn test_copy_registry_torn_write_recovery() {
    let dir = tempdir().unwrap();

    let mut reg = CopyRegistry::open(dir.path()).await.unwrap();
    let entry = CopyRegEntry {
        copy_id: [1u8; 16],
        kind: CopyKind::Reflink,
        file_ids: vec![[10u8; 16]],
        created_tx: 50,
    };
    reg.register(entry.clone()).await.unwrap();

    // Append partial frame to simulate torn write
    let path = dir.path().join(REGISTRY_FILENAME);
    let mut bytes = tokio::fs::read(&path).await.unwrap();
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x20, 0xAA, 0xBB]); // Truncated length header/payload
    tokio::fs::write(&path, &bytes).await.unwrap();

    // Reopen - should read valid entries up to torn write
    let reg_healed = CopyRegistry::open(dir.path()).await.unwrap();
    assert_eq!(reg_healed.list().len(), 1);
    assert_eq!(reg_healed.list()[0], entry);
}
