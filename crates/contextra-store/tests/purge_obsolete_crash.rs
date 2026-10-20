// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Absturztest zwischen Manifest-Tausch und P5 (Entsorgung). Prüft, dass die alte SSTable nach Neustart über das Journal entsorgt wird.

use contextra_durable_fs::PendingDeleteJournal;
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::orchestrator::OrxHooks;
use contextra_store::lsm::purge::resume::resume_pending;
use contextra_store::lsm::LsmStorage;
use contextra_types::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_obsolete_crash_between_manifest_and_p5() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let obsolete_path = {
        let storage = LsmStorage::open(config.clone()).await?;

        let key1 = b"t:1:k1".to_vec();
        let val1 = b"v1".to_vec();

        storage.put(TxId(1), &key1, &val1).await?;
        storage.commit(TxId(1)).await?;
        StorageEngine::flush(&storage).await?;

        // Perform purge_keys directly (which writes to journal before manifest swap)
        let scope = PurgeScope::Keys(vec![key1]);
        let report = storage.purge_keys(scope, 10, None).await?;

        assert_eq!(report.sstables_replaced, 1);
        let old_file = report.obsolete_files[0].clone();

        // SIMULATE CRASH: Process crashes right here before P5 dispose_obsolete_files is called!
        // The old file still exists on disk, but is recorded in pending_delete.journal and unreferenced by Manifest.
        assert!(old_file.exists(), "Old file must exist before P5 disposal");

        old_file
    };

    // Confirm that pending_delete.journal contains the obsolete file
    let journal_path = dir.path().join("pending_delete.journal");
    assert!(journal_path.exists(), "Journal must exist after purge_keys");
    let journal = PendingDeleteJournal::open(&journal_path)?;
    assert!(
        journal.pending().contains(&obsolete_path),
        "Obsolete file must be recorded in journal"
    );

    // RESTART: Run resume_pending on the store directory upon startup
    let hooks = OrxHooks::default();
    let res = resume_pending(dir.path(), &hooks, None).await?;

    assert!(res.is_some(), "resume_pending must detect pending journal work");

    // VERIFY: The obsolete file is now durably removed from disk
    assert!(
        !obsolete_path.exists(),
        "Obsolete file must be deleted post-restart via journal cleanup"
    );

    Ok(())
}
