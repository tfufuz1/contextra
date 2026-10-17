// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Integrationstest für CRASH-08 (Restore aus altem Checkpoint mit StreamingRestoreGuard).
// INVARIANTEN: Gelöschte Schlüssel werden bei Restore aus alten Checkpoints deterministisch abgewiesen (RestoreVerdict::Reject).

use contextra_ports::{RestoreGuard, RestoreVerdict};
use contextra_store::erasure_ledger::{
    compute_keyed_hash, ErasureLedger, LedgerEntry, LedgerEntryKind, LedgerKey, LedgerOpenStatus,
};
use contextra_store::StreamingRestoreGuard;
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_crash_08_anti_resurrection_streaming_restore_guard() {
    let dir = tempdir().unwrap();
    let master_secret = [0x55u8; 32];
    let ledger_key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, status) =
        ErasureLedger::open_or_fail_closed(dir.path(), ledger_key.clone(), false, None)
            .await
            .unwrap();
    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let scope = b"lsm";
    let erased_key = b"user_secret_data_101";
    let normal_key = b"user_public_data_102";

    let erased_hash = compute_keyed_hash(&ledger_key, scope, erased_key);

    // Record an erasure entry in ledger at erased_at_tx = 100
    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: erased_hash,
        erased_at_tx: 100,
        proof_id: [7u8; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    let ledger_arc = Arc::new(ledger);
    let guard = StreamingRestoreGuard::for_lsm(ledger_arc);

    // Candidate key from an old checkpoint at source_tx = 50 (BEFORE erasure at 100)
    let verdict_erased = guard.admit(erased_key, TxId::new(50));
    assert_eq!(
        verdict_erased,
        RestoreVerdict::Reject,
        "CRASH-08: Key erased at tx 100 MUST be rejected when restoring from checkpoint at tx 50"
    );

    // Normal non-erased key from checkpoint at source_tx = 50
    let verdict_normal = guard.admit(normal_key, TxId::new(50));
    assert_eq!(
        verdict_normal,
        RestoreVerdict::Allow,
        "Normal non-erased key MUST be allowed on restore"
    );

    // Key rewritten after erasure at source_tx = 150 > 100
    let verdict_rewritten = guard.admit(erased_key, TxId::new(150));
    assert_eq!(
        verdict_rewritten,
        RestoreVerdict::Allow,
        "Key rewritten after erasure transaction MUST be allowed on restore"
    );
}
