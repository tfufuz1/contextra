#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{open_with_config, ContextraConfig};
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;
use contextra_types::error::ContextraError;
use serde_json::json;

#[tokio::test]
async fn test_open_and_reopen_data_persistence() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_08;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_recovery seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir =
        std::env::temp_dir().join(format!("e2e_wiring_recovery_reopen_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    // Phase 1: Open, write doc, close
    {
        let db = open_with_config(&tmp_dir, config.clone()).await?;
        db.insert(
            "persistent_doc",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"version": 1})),
        )
        .await?;
        db.close().await?;
    }

    // Phase 2: Reopen, verify doc persisted
    {
        let db = open_with_config(&tmp_dir, config).await?;
        let doc = db
            .get("persistent_doc")
            .await?
            .expect("doc must persist across restart");
        assert_eq!(doc.id, "persistent_doc");
        assert_eq!(
            doc.metadata.as_ref().and_then(|m| m.get("version")),
            Some(&json!(1))
        );
        db.close().await?;
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
async fn test_parallel_open_same_path_single_process_lock_or_error() -> Result<(), ContextraError> {
    let seed: u64 = 20261021_01;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_recovery single_process seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_recovery_lock_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    // 1. Open first instance
    let db1 = open_with_config(&tmp_dir, config.clone()).await?;

    // 2. Attempting to open second instance on same path must fail with defined error (dir lock)
    let db2_res = open_with_config(&tmp_dir, config.clone()).await;
    assert!(
        db2_res.is_err(),
        "Second open on the same directory path while open must return Err"
    );

    // 3. Close db1
    db1.close().await?;

    // 4. Now opening db3 on the same directory path must succeed
    let db3 = open_with_config(&tmp_dir, config).await?;
    assert_eq!(db3.len().await?, 0);
    db3.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
async fn test_drop_facade_during_concurrent_operations() -> Result<(), ContextraError> {
    let seed: u64 = 20261021_02;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_recovery drop_concurrent seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_recovery_drop_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = std::sync::Arc::new(open_with_config(&tmp_dir, config).await?);
    let col = std::sync::Arc::new(db.collection("concurrent_col").await?);

    // Setup initial data
    col.insert("doc_init", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": 0})))
        .await?;

    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
    let mut handles = Vec::new();

    for i in 0..3 {
        let col_clone = col.clone();
        let barrier_clone = barrier.clone();
        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            for j in 0..50 {
                let doc_id = format!("doc_{}_{}", i, j);
                let _ = col_clone
                    .insert(
                        &doc_id,
                        &[0.1, 0.2, 0.3, 0.4],
                        Some(json!({"i": i, "j": j})),
                    )
                    .await;
                let _ = col_clone.get(&doc_id).await;
            }
        }));
    }

    // Task 4 syncs with barrier then drops the primary db reference
    barrier.wait().await;
    drop(db);

    // Wait for worker tasks to complete under watchdog timeout
    let join_all = async {
        for handle in handles {
            let _ = handle.await;
        }
    };

    tokio::time::timeout(std::time::Duration::from_secs(5), join_all)
        .await
        .expect("Concurrent tasks must complete within watchdog timeout after db drop");

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-RECOVERY-01: Purge crash resume and orphan pin recovery on startup in contextra-engine/contextra-checkpoint (J05) not wired in Contextra open/builder path"]
async fn test_purge_crash_resume_and_orphan_pin_recovery() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_09;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_recovery crash seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir =
        std::env::temp_dir().join(format!("e2e_wiring_recovery_crash_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Mid-purge crash resume driver & OrphanRegistry recovery on startup are un-wired
    // in Contextra open/builder path.

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
