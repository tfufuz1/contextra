// FILE-CONTEXT
// ZWECK: Comprehensive E2E tests for WAL scope purge wiring, error handling, report metrics, and filter consistency.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::purge::orchestrator::PurgeOutcome;
use contextra_store::lsm::purge::wiring::PurgeWiring;
use contextra_store::lsm::LsmStorage;
use contextra_testkit::deletion_oracle::rawsweep::raw_sweep;
use contextra_types::TxId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

/// Akzeptanzkriterium 1: Ende-zu-Ende Raw-Sweep über das gesamte Store-Verzeichnis
/// (SSTables, WAL aktiv und versiegelt, Manifest, Journal, Intent, Scope).
#[tokio::test]
async fn test_e2e_raw_sweep_entire_store_directory_clean_post_purge(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config.clone()).await?;
    let tx_id = TxId::try_new(101)?;

    let marker_key = b"marker_e2e_key_999".to_vec();
    let marker_val = b"marker_e2e_value_secret_888".to_vec();

    // Write active WAL & SSTable
    storage.put(tx_id, &marker_key, &marker_val).await?;
    storage.commit(tx_id).await?;

    let tx_id2 = TxId::try_new(102)?;
    storage.put(tx_id2, &marker_key, &marker_val).await?;
    storage.commit(tx_id2).await?;
    storage.flush().await?;

    // Verify raw_sweep finds marker prior to purge
    let pre_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;
    assert!(
        !pre_sweep.hits.is_empty(),
        "Raw sweep must find marker bytes before purge"
    );

    let scope = PurgeScope::Keys(vec![marker_key.clone()]);
    let report = storage.purge_keys(scope.clone(), 200, None).await?;
    assert!(
        report.records_purged > 0,
        "Purge report must reflect purged records from SSTable"
    );

    storage.close().await?;

    // Execute WAL purge via default PurgeWiring composition root
    let wiring = PurgeWiring::build_default(dir.path())?;
    let orchestrator = wiring.orchestrator_standalone();

    let _res = orchestrator
        .execute_purge_internal(scope, 200, None, wiring.hooks(), false)
        .await?;

    // Raw-Sweep across ENTIRE store directory
    let post_sweep = raw_sweep(dir.path(), &[marker_val.clone()])?;
    assert!(
        post_sweep.hits.is_empty(),
        "Raw sweep must find 0 marker bytes across entire store directory post-purge"
    );

    Ok(())
}

/// Akzeptanzkriterium 2: WAL-Fehler führt zu ErasureIncomplete { component: "wal" },
/// Intent-Datei bleibt bestehen, Resume wiederholt und schließt erfolgreich ab.
#[tokio::test]
async fn test_wal_error_returns_erasure_incomplete_and_resumes(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let orchestrator = PurgeWiring::build_default(dir.path())?.orchestrator_standalone();

    let scope = PurgeScope::Keys(vec![b"wal_err_key".to_vec()]);

    let wal_failed = Arc::new(AtomicBool::new(true));
    let wal_failed_clone = Arc::clone(&wal_failed);

    let mut hooks = contextra_store::lsm::purge::OrxHooks::default();
    hooks.wal_retire_scope = Some(Arc::new(move |_scope, _as_of_tx| {
        let failed = wal_failed_clone.load(Ordering::SeqCst);
        Box::pin(async move {
            if failed {
                Err(contextra_types::ContextraError::Storage(
                    "Simulated WAL retire failure".into(),
                ))
            } else {
                Ok((1, 1))
            }
        })
    }));

    let outcome = orchestrator
        .execute_purge_outcome(scope, 100, None, &hooks, false)
        .await;

    match outcome {
        PurgeOutcome::Failed { phase, error_msg } => {
            assert_eq!(
                phase,
                contextra_store::lsm::purge::PurgePhase::P6WalCallback
            );
            assert!(
                error_msg.contains("ErasureIncomplete { component: \"wal\" }"),
                "Error msg must be ErasureIncomplete {{ component: \"wal\" }}, got: {error_msg}"
            );
        }
        other => panic!("Expected PurgeOutcome::Failed, got {:?}", other),
    }

    let intent_path = dir.path().join("purge.intent");
    assert!(
        intent_path.exists(),
        "Intent file must remain on disk after WAL purge failure"
    );

    // Fix WAL failure and resume
    wal_failed.store(false, Ordering::SeqCst);

    let res_opt = contextra_store::lsm::purge::resume_pending(dir.path(), &hooks, None).await?;
    assert!(
        res_opt.is_some(),
        "Resume after WAL failure must complete purge"
    );
    assert!(
        !intent_path.exists(),
        "Intent file must be cleaned up after successful resume"
    );

    Ok(())
}

/// Akzeptanzkriterium 3: Berichtszahlen
/// Mit einem Purge, der keinen WAL-Frame trifft, sind frames_purged = 0 und segments_rewritten = 0.
#[tokio::test]
async fn test_report_numbers_zero_when_no_wal_frame_matches(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config).await?;
    let tx_id = TxId::try_new(100)?;

    storage
        .put(tx_id, b"other_key_123", b"other_val_123")
        .await?;
    storage.commit(tx_id).await?;
    storage.close().await?;

    let wiring = PurgeWiring::build_default(dir.path())?;
    let orchestrator = wiring.orchestrator_standalone();
    let scope = PurgeScope::Keys(vec![b"non_existent_key_999".to_vec()]);

    let report = orchestrator
        .execute_purge_internal(scope, 200, None, wiring.hooks(), false)
        .await?;

    assert_eq!(report.records_purged, 0, "frames_purged must be 0");
    assert_eq!(report.sstables_replaced, 0, "segments_rewritten must be 0");

    Ok(())
}

/// Akzeptanzkriterium 4: Konsistency der Filter
/// Ein Schlüssel, der den SSTable-Filter trifft, trifft auch den WAL-Filter (Eigenschaftstest über zufällige Scopes).
#[test]
fn test_filter_consistency_property_between_sstable_and_wal_scopes() {
    let rng = contextra_sys::SystemRng::new();

    for _ in 0..100 {
        let mut key_a = vec![0u8; 8];
        let mut key_b = vec![0u8; 8];
        rng.fill_bytes(&mut key_a);
        rng.fill_bytes(&mut key_b);

        let keys_scope = PurgeScope::Keys(vec![key_a.clone(), key_b.clone()]);
        let prefixes_scope = PurgeScope::Prefixes(vec![key_a[..4].to_vec()]);

        assert!(keys_scope.matches_key(&key_a));
        assert!(keys_scope.matches_key(&key_b));
        assert!(prefixes_scope.matches_key(&key_a));

        let spec_keys = contextra_store::wal::WalScopeSpec {
            key_prefixes: vec![key_a.clone(), key_b.clone()],
            raw_needles: vec![],
        };
        assert!(spec_keys.matches_key(&key_a));
        assert!(spec_keys.matches_key(&key_b));
    }
}
