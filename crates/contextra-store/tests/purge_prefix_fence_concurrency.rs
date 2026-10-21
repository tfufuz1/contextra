use contextra_ports::{StorageEngine, TxId};
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::{OrxHooks, PurgeOrchestrator};
use contextra_store::lsm::LsmStorage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct DummySpaceProbe;
impl FreeSpaceProbe for DummySpaceProbe {
    fn available_bytes(&self) -> Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(1024 * 1024 * 1024))
    }
}

#[tokio::test]
async fn test_purge_prefix_fence_allows_concurrent_unrelated_writes_and_blocks_scoped_writes(
) -> std::io::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(
        LsmStorage::open(config)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?,
    );

    // Initial setup write
    storage
        .put(
            TxId::try_new(1)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?,
            b"scope_a:key1",
            b"val1",
        )
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    storage
        .put(
            TxId::try_new(2)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?,
            b"scope_b:key1",
            b"val1",
        )
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let in_compaction = Arc::new(AtomicBool::new(false));
    let write_b_succeeded = Arc::new(AtomicBool::new(false));
    let write_a_rejected = Arc::new(AtomicBool::new(false));

    let in_comp_clone = Arc::clone(&in_compaction);
    let storage_clone = Arc::clone(&storage);
    let write_b_clone = Arc::clone(&write_b_succeeded);
    let write_a_clone = Arc::clone(&write_a_rejected);

    let mut hooks = OrxHooks::default();
    hooks.flush_memtable = Some(Box::new(move || {
        in_comp_clone.store(true, Ordering::SeqCst);

        // Attempt write on Scope B during purge of Scope A -> MUST succeed
        let st_b = Arc::clone(&storage_clone);
        let wb = Arc::clone(&write_b_clone);
        let st_a = Arc::clone(&storage_clone);
        let wa = Arc::clone(&write_a_clone);

        tokio::spawn(async move {
            let res_b = st_b
                .put(TxId::try_new(10).unwrap(), b"scope_b:key2", b"val2")
                .await;
            if res_b.is_ok() {
                wb.store(true, Ordering::SeqCst);
            }

            // Attempt write on Scope A during purge of Scope A -> MUST be rejected by fence
            let res_a = st_a
                .put(TxId::try_new(11).unwrap(), b"scope_a:key2", b"val2")
                .await;
            if res_a.is_err() || !st_a.erasure_fence().check_write_allowed(b"scope_a:key2") {
                wa.store(true, Ordering::SeqCst);
            }
        });

        Ok(())
    }));

    let orchestrator = PurgeOrchestrator::new(&storage);
    let scope_a = PurgeScope::Prefixes(vec![b"scope_a:".to_vec()]);
    let probe = DummySpaceProbe;

    let outcome = orchestrator
        .execute_purge_outcome(scope_a, 100, Some(&probe), &hooks, false)
        .await;

    assert!(outcome.is_success(), "Purge execution must succeed");

    // Wait briefly for spawned task if needed
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(
        write_b_succeeded.load(Ordering::SeqCst),
        "Write to Scope B must succeed during purge of Scope A"
    );
    assert!(
        write_a_rejected.load(Ordering::SeqCst),
        "Write to Scope A must be rejected during purge of Scope A"
    );

    Ok(())
}
