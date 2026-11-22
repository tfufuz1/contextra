// FILE-CONTEXT
// ZWECK: End-to-End Facade Wiring Matrix Integration Tests (§20.2 / Regel R1).
// INVARIANTEN: Greift ausschließlich über die öffentliche Facade (contextra::open, ContextraBuilder) zu.

use contextra::performance_profile::PerformanceProfile;
use contextra::{builder, open, open_with_config, ContextraConfig, ContextraError};
use contextra_store::lsm::DurabilityMode;
use tempfile::TempDir;

#[tokio::test]
async fn flush_preserves_pinned_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let db = builder(16).with_storage_path(tmp.path()).build().await?;

    let coll = db.collection("default").await?;

    // Record baseline snapshot sequence prior to inserts
    let snap_seq = coll.snapshot_seq().await?;

    coll.insert("doc1", &[0.1; 16], None).await?;

    // Pinned snapshot at snap_seq before insertion must return None
    let snap_doc = coll.get_at_snapshot("doc1", snap_seq).await?;
    assert!(
        snap_doc.is_none(),
        "Snapshot at seq {} must not see doc1 inserted after it",
        snap_seq
    );

    // Fresh read observes inserted document
    let current_doc = coll.get("doc1").await?;
    assert!(current_doc.is_some(), "Fresh read must find inserted doc1");

    // Flush and verify snapshot consistency at snap_seq remains None
    db.flush().await?;
    let snap_doc_after_flush = coll.get_at_snapshot("doc1", snap_seq).await?;
    assert!(
        snap_doc_after_flush.is_none(),
        "Snapshot at seq {} must remain None after flush",
        snap_seq
    );

    Ok(())
}

#[tokio::test]
async fn second_open_same_directory_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let _db1 = open(tmp.path()).await?;

    // Second open on the same directory in the same process must be rejected with lock failure
    let db2_res = open(tmp.path()).await;
    assert!(
        db2_res.is_err(),
        "Second open on locked directory must be rejected"
    );

    Ok(())
}

#[tokio::test]
async fn open_without_gate_never_grants_sovereign() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;

    // Requesting Compliance performance profile (which requires Sovereign ring) without attaching a valid signed license gate must fail
    let res = builder(16)
        .with_storage_path(tmp.path())
        .with_performance_profile(PerformanceProfile::Compliance)
        .build()
        .await;

    assert!(
        res.is_err(),
        "Compliance/Sovereign feature ring without signed license must be rejected"
    );
    if let Err(err) = res {
        assert!(
            matches!(err, ContextraError::PolicyViolation(_)),
            "Expected PolicyViolation error, got: {:?}",
            err
        );
    }

    Ok(())
}

#[tokio::test]
async fn profile_durability_reaches_lsm_config() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;

    // BareMetal performance profile sets MemoryOnly durability mode
    let db = builder(16)
        .with_storage_path(tmp.path())
        .with_performance_profile(PerformanceProfile::BareMetal)
        .build()
        .await?;

    let config = db.config();
    assert_eq!(
        config.durability_mode,
        DurabilityMode::MemoryOnly,
        "BareMetal profile must set MemoryOnly durability in LSM config"
    );

    Ok(())
}

#[tokio::test]
async fn confirmed_commit_survives_drop_reopen() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;

    {
        let db = builder(16).with_storage_path(tmp.path()).build().await?;
        let coll = db.collection("persistent").await?;
        coll.insert("key1", &[0.5; 16], None).await?;
        // Explicitly close db instance to release DirLock
        db.close().await?;
    }

    // Reopen with matching dimension config (16) and verify data persisted across instance lifecycle
    let config = ContextraConfig {
        dimension: 16,
        ..Default::default()
    };
    let db_reopened = open_with_config(tmp.path(), config).await?;
    let coll_reopened = db_reopened.collection("persistent").await?;
    assert!(
        coll_reopened.get("key1").await?.is_some(),
        "Committed write key1 must survive drop and reopen"
    );

    Ok(())
}
