// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Test für Anti-Resurrection Restore-Filter (CpyRestoreFilter) mit ErasureLedger Integration.

use contextra_checkpoint::{filter_restore_records, CpyRestoreFilter};
use contextra_store::erasure_ledger::{
    compute_keyed_hash, ErasureLedger, LedgerEntry, LedgerEntryKind, LedgerKey,
};
use tempfile::tempdir;

pub struct ErasureLedgerRestoreFilter<'a> {
    ledger: &'a ErasureLedger,
    scope: &'static [u8],
}

impl<'a> ErasureLedgerRestoreFilter<'a> {
    pub fn new(ledger: &'a ErasureLedger) -> Self {
        Self {
            ledger,
            scope: b"lsm",
        }
    }
}

impl<'a> CpyRestoreFilter for ErasureLedgerRestoreFilter<'a> {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        matches!(
            self.ledger.decide(self.scope, key, tx),
            contextra_store::erasure_ledger::LedgerDecision::Drop
        )
    }
}

#[tokio::test]
async fn test_anti_resurrection_restore_filter_with_erasure_ledger() {
    let dir = tempdir().unwrap();
    let master_secret = [0x42u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, status) = ErasureLedger::open_or_fail_closed(dir.path(), ledger_key.clone(), false, None)
        .await
        .unwrap();
    assert_eq!(status, contextra_store::erasure_ledger::LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let scope = b"lsm";
    let erased_key = b"user_101_sensitive_data";
    let normal_key = b"user_102_public_data";

    let erased_hash = compute_keyed_hash(&ledger_key, scope, erased_key);

    // Record an erasure entry for erased_key at tx = 100
    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: erased_hash,
        erased_at_tx: 100,
        proof_id: [1u8; 16],
    };

    ledger.append(&[entry]).await.unwrap();

    // Prepare candidate records from a pre-erasure checkpoint and subsequent operations
    let rec_erased_old = (
        erased_key.to_vec(),
        b"secret_data_v1".to_vec(),
        50u64, // Created at tx = 50 (BEFORE erasure at tx = 100)
    );
    let rec_normal = (
        normal_key.to_vec(),
        b"public_data_v1".to_vec(),
        60u64,
    );
    let rec_erased_rewritten = (
        erased_key.to_vec(),
        b"consent_renewed_v2".to_vec(),
        150u64, // Rewritten at tx = 150 (AFTER erasure at tx = 100)
    );

    let candidate_records = vec![rec_erased_old.clone(), rec_normal.clone(), rec_erased_rewritten.clone()];

    let filter = ErasureLedgerRestoreFilter::new(&ledger);

    // Execute restore record filtering
    let filtered_records = filter_restore_records(candidate_records, &filter);

    // Record A (user_101 at tx = 50 <= 100) MUST be dropped (Anti-Resurrection)
    assert!(
        !filtered_records.iter().any(|(k, _, tx)| k == erased_key && *tx == 50),
        "Erased record prior to erasure transaction must be dropped on restore"
    );

    // Record B (user_102 at tx = 60) MUST be kept
    assert!(
        filtered_records.iter().any(|(k, _, tx)| k == normal_key && *tx == 60),
        "Unerased normal record must be retained on restore"
    );

    // Record C (user_101 rewritten at tx = 150 > 100) MUST be kept
    assert!(
        filtered_records.iter().any(|(k, _, tx)| k == erased_key && *tx == 150),
        "Record rewritten after erasure transaction must be retained on restore"
    );

    assert_eq!(filtered_records.len(), 2);
}
