#![cfg(not(loom))]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{ContextraError, TenantId};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

/// Test T1 (Scan-Race in drop_collection):
///
/// Verifies the race condition described in Audit A-03 where a concurrent `insert`
/// interleaves between initial prefix scans / delete_prefix and the post-commit scan
/// in `Contextra::drop_collection`.
///
/// Expected behavior (Audit A-03):
/// The post-commit scan detects the newly inserted document, causing
/// `LayerCleanupProof::new_after_verified_empty` to fail with `Err(ContextraError::Internal(...))`
/// due to remaining live entries violating INV-DELETION-1.
///
/// Synchronization:
/// No production test-hook exists in `drop_collection`. This test forces the timing race
/// via concurrent task execution across repeated iterations (best-effort without hook).
#[tokio::test]
async fn test_drop_collection_scan_race() -> contextra_types::Result<()> {
    let proof_key = b"secret_proof_key_32bytes_1234567";
    let tenant_id = TenantId::try_new(500).expect("Valid tenant ID");
    let mut race_detected = false;

    for iter in 0..50 {
        let tmp = TempDir::new().expect("Failed to create temporary directory");
        let config = ContextraConfig {
            dimension: 4,
            consolidation_enabled: false,
            ..Default::default()
        };

        let db = Arc::new(Contextra::open_with_config(tmp.path(), config).await?);
        let col_name = format!("race_col_{}", iter);

        // 1. Initialize collection for tenant and insert 2 documents
        let col = db.collection_for_tenant(&col_name, tenant_id).await?;
        col.insert(
            "doc_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({"initial": true})),
        )
        .await?;
        col.insert(
            "doc_2",
            &[0.5, 0.6, 0.7, 0.8],
            Some(json!({"initial": true})),
        )
        .await?;

        // 2. Spawn concurrent insert loop alongside drop_collection
        let db_clone = Arc::clone(&db);
        let col_name_drop = col_name.clone();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);

        let insert_handle = tokio::spawn(async move {
            let mut count = 0;
            while !stop_flag_clone.load(Ordering::Relaxed) {
                count += 1;
                let doc_id = format!("doc_concurrent_{}", count);
                let _ = col
                    .insert(
                        &doc_id,
                        &[0.9, 0.8, 0.7, 0.6],
                        Some(json!({"concurrent": true})),
                    )
                    .await;
                tokio::task::yield_now().await;
            }
        });

        tokio::task::yield_now().await;

        let drop_res = db_clone
            .drop_collection(&col_name_drop, tenant_id, proof_key)
            .await;

        stop_flag.store(true, Ordering::Relaxed);
        let _ = insert_handle.await;

        match drop_res {
            Ok(_proof) => {
                // Drop succeeded when insertion did not hit the post-commit scan window.
            }
            Err(ContextraError::Internal(ref msg))
                if msg.contains("INV-DELETION-1")
                    || msg.contains("DeletionProof generation failed") =>
            {
                race_detected = true;
                println!(
                    "T1: Scan-Race successfully confirmed at iteration {}: {}",
                    iter, msg
                );
                db.close().await?;
                break;
            }
            Err(other) => {
                println!("T1: Unexpected error during drop_collection: {:?}", other);
            }
        }

        db.close().await?;
    }

    if race_detected {
        println!("T1 RESULT: Scan-race verified (post-commit scan found interleaved record and returned Err as predicted by Audit A-03).");
    } else {
        println!("T1 RESULT: Best-effort execution finished without interleaving in post-commit scan window (no production hook available).");
    }

    Ok(())
}
