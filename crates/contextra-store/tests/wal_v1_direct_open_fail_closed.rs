use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_direct_wal_open_with_v1_payload_fails_closed() {
    let dir = tempdir().expect("create tempdir");
    let wal_path = dir.path().join("unencrypted_v1.wal");

    // 1. Manually construct an unencrypted V1 WAL segment (no MFW3/MFW2 header prefix)
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"v1_direct_key".to_vec(),
        value: b"v1_direct_val".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &Wal::legacy_integrity_key_for_test(), [0u8; 32])
        .expect("create v1 entry");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write unencrypted v1 wal");

    // 2. Direct open via Wal::open (without using open_for_legacy_migration or migrate_legacy_wal_keys)
    // Expectation according to Audit A-02: fail-closed, because WalConfig::default() enforces min_wal_version = V3.
    let open_res = Wal::open(&wal_path).await;

    // Note / Security Finding: As documented in task report T-2026-0230, Wal::open currently opens V1 files.
    // Per strict task directives ("Weicht ein Testergebnis vom im Audit behaupteten Ist-Zustand ab: NICHT den Test anpassen.
    // Als sicherheitsrelevante Abweichung im PR eskalieren und stoppen"), this assertion asserts fail-closed behavior.
    assert!(
        open_res.is_err(),
        "Directly opening V1 WAL via Wal::open MUST fail closed under default WalConfig"
    );
}
