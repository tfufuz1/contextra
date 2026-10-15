#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{open_with_config, ContextraConfig};
use contextra_core::ContextraError;
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;
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
#[ignore = "blockiert durch PROMPT-RECOVERY-01: Purge crash resume and orphan pin recovery on startup are not wired in Contextra open/builder path"]
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
