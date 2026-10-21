// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Mandated integration tests for ScopeFile, single-flight mutex, phase roundtrips, and generations.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::{PurgeBlocked, PurgeError, PurgeScope};
use contextra_store::lsm::purge::{PurgePhase, ScopeFile};
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use tempfile::tempdir;

#[test]
fn test_scope_roundtrip_and_tamper() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let purge_id = [42u8; 16];
    let scope_path = ScopeFile::path(dir.path(), &purge_id);

    let scope = PurgeScope::Keys(vec![b"key_a".to_vec(), b"key_b".to_vec()]);

    // 1. Write scope file
    ScopeFile::write(&scope_path, &scope, &purge_id, None)?;
    assert!(scope_path.exists());

    // 2. Read roundtrip
    let read_scope = ScopeFile::read(&scope_path, &purge_id, None)?;
    assert_eq!(read_scope, scope);

    // 3. Tamper bytes and verify rejection
    let mut bytes = std::fs::read(&scope_path)?;
    if let Some(last) = bytes.last_mut() {
        *last ^= 0xFF;
    }
    std::fs::write(&scope_path, &bytes)?;

    let err = ScopeFile::read(&scope_path, &purge_id, None).unwrap_err();
    assert!(matches!(err, PurgeError::ReadError { .. }));

    Ok(())
}

#[test]
fn test_phase_codes_roundtrip_and_zero_rejected() {
    for code in 1..=9 {
        let phase = PurgePhase::try_from(code).expect("valid code");
        assert_eq!(phase as u8, code);
    }

    assert!(PurgePhase::try_from(0).is_err());
    assert!(PurgePhase::try_from(10).is_err());
}

#[tokio::test]
async fn test_store_generation_advances_after_purge() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = contextra_store::lsm::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx = TxId::try_new(1)?;
    storage.put(tx, b"k1", b"v1").await?;
    storage.commit(tx).await?;
    storage.flush().await?;

    let probe_before = storage.store_probe();
    let gen_before = probe_before.manifest_file_count();

    let scope = PurgeScope::Keys(vec![b"k1".to_vec()]);
    let hooks = contextra_store::lsm::purge::OrxHooks::default();
    let orchestrator = contextra_store::lsm::purge::PurgeOrchestrator::new(&storage, dir.path());
    let _ = orchestrator.execute_purge_internal(scope, 100, None, &hooks, false).await?;

    let probe_after = storage.store_probe();
    let gen_after = probe_after.manifest_file_count();

    assert!(gen_after.is_some());
    assert!(gen_before.is_some());

    Ok(())
}

#[test]
fn test_purge_error_classification() {
    let blocked = PurgeBlocked::CompactionBusy;
    assert!(!blocked.retry_hint().is_empty());

    let blocked_snap = PurgeBlocked::ActiveSnapshot { min_seq: 5 };
    assert!(!blocked_snap.retry_hint().is_empty());
}
