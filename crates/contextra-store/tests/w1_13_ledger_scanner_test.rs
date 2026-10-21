// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Stage 6 Tests for ErasureLedger LayerScan implementation using keyed hashes exclusively.

#![allow(deprecated)]

use contextra_ports::{LayerScan, PhysicalScope};
use contextra_store::erasure_ledger::*;
use contextra_types::TenantId;
use tempfile::tempdir;

#[tokio::test]
async fn ledger_scanner_uses_only_keyed_hashes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db");
    tokio::fs::create_dir_all(&db_path).await.unwrap();

    let ledger_key = LedgerKey::new([0x99; 32]);
    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed(&db_path, ledger_key.clone(), false, None)
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let tenant_id = TenantId::try_new(123).unwrap();
    let raw_key = b"t:123:doc_456";
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", raw_key);

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash,
        erased_at_tx: 50,
        proof_id: [0x77; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    let physical_scope = PhysicalScope::try_new(
        tenant_id,
        vec![raw_key.to_vec()],
        vec![],
    )
    .unwrap();

    let obs = ledger.scan(&physical_scope).unwrap();

    assert_eq!(obs.layer_id, 12);
    assert!(!obs.clean, "Matching keyed hash in ledger must mark scan non-clean");
    assert_eq!(obs.item_count, 1);
}
