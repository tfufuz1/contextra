use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::ContextraError;
use proptest::prelude::*;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_write_skew_prevention_via_engine_api() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Arc::new(Contextra::open_with_config(tmp_dir.path(), config).await?);
    let col = db.collection("default").await?;

    // Initial state: Both Doctor 1 and Doctor 2 are on call
    let emb = vec![1.0, 0.0, 0.0, 0.0];
    col.insert("doctor_1", &emb, Some(json!({ "status": "on_call" })))
        .await?;
    col.insert("doctor_2", &emb, Some(json!({ "status": "on_call" })))
        .await?;

    // Transaction 1: Doctor 1 reads both doctors (check if at least 2 are on call) then updates doctor_1
    let tx1 = col.begin_transaction()?;
    let doc1_tx1 = col.get_tracked(tx1.tx_id, "doctor_1").await?;
    let doc2_tx1 = col.get_tracked(tx1.tx_id, "doctor_2").await?;
    assert!(doc1_tx1.is_some() && doc2_tx1.is_some());

    // Transaction 2: Doctor 2 reads both doctors (check if at least 2 are on call) then updates doctor_2
    let tx2 = col.begin_transaction()?;
    let doc1_tx2 = col.get_tracked(tx2.tx_id, "doctor_1").await?;
    let doc2_tx2 = col.get_tracked(tx2.tx_id, "doctor_2").await?;
    assert!(doc1_tx2.is_some() && doc2_tx2.is_some());

    // Tx1 updates doctor_1
    col.update_op(
        &tx1,
        "doctor_1",
        &emb,
        Some(json!({ "status": "off_call" })),
    )
    .await?;

    // Tx2 updates doctor_2
    col.update_op(
        &tx2,
        "doctor_2",
        &emb,
        Some(json!({ "status": "off_call" })),
    )
    .await?;

    // Commit tx1
    tx1.commit().await?;

    // Commit tx2 - MUST fail with ContextraError::Conflict/Transaction due to SSI
    let tx2_res = tx2.commit().await;

    assert!(
        tx2_res.is_err(),
        "Tx2 MUST fail commit due to SSI conflict on overlapping read/write sets"
    );

    match tx2_res {
        Err(ContextraError::Conflict(msg)) | Err(ContextraError::Transaction(msg))
            if msg.contains("Conflict") || msg.contains("Serializable isolation violation") =>
        {
            tracing::info!("Tx2 correctly aborted with Conflict: {}", msg);
        }
        Err(other) => panic!(
            "Expected ContextraError::Conflict or Conflict Transaction error, got {:?}",
            other
        ),
        Ok(_) => panic!("Tx2 commit succeeded when it should have failed with Conflict"),
    }

    Ok(())
}

#[tokio::test]
async fn test_engine_storage_supports_ssi_tracking() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;

    use contextra_ports::StorageEngine;
    assert!(
        col.storage().supports_ssi_tracking(),
        "Production storage in Contextra collection MUST support SSI tracking"
    );

    Ok(())
}

fn apply_op(val: i64, delta: i64) -> i64 {
    val.saturating_add(delta)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    #[test]
    fn prop_concurrent_engine_transactions_serializable(
        init1 in 0i64..100,
        init2 in 0i64..100,
        read1_k1 in prop::bool::ANY,
        read1_k2 in prop::bool::ANY,
        write1_k1 in prop::bool::ANY,
        delta1 in 1i64..10,
        read2_k1 in prop::bool::ANY,
        read2_k2 in prop::bool::ANY,
        write2_k1 in prop::bool::ANY,
        delta2 in 1i64..10,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let tmp_dir = TempDir::new().unwrap();
            let config = ContextraConfig {
                dimension: 4,
                consolidation_enabled: false,
                ..Default::default()
            };
            let db = Contextra::open_with_config(tmp_dir.path(), config).await.unwrap();
            let col = db.collection("default").await.unwrap();
            let emb = vec![1.0, 0.0, 0.0, 0.0];

            col.insert("k1", &emb, Some(json!({ "val": init1 }))).await.unwrap();
            col.insert("k2", &emb, Some(json!({ "val": init2 }))).await.unwrap();

            let tx1 = col.begin_transaction().unwrap();
            let tx2 = col.begin_transaction().unwrap();

            // Tx1 reads
            let v1_k1 = init1;
            let v1_k2 = init2;
            if read1_k1 || (!read1_k1 && !read1_k2) {
                col.get_tracked(tx1.tx_id, "k1").await.unwrap();
            }
            if read1_k2 {
                col.get_tracked(tx1.tx_id, "k2").await.unwrap();
            }

            // Tx2 reads
            if read2_k1 {
                col.get_tracked(tx2.tx_id, "k1").await.unwrap();
            }
            if read2_k2 || (!read2_k1 && !read2_k2) {
                col.get_tracked(tx2.tx_id, "k2").await.unwrap();
            }

            // Tx1 writes
            let target1 = if write1_k1 { "k1" } else { "k2" };
            let base1 = if write1_k1 { v1_k1 } else { v1_k2 };
            let new_val1 = apply_op(base1, delta1);
            col.update_op(&tx1, target1, &emb, Some(json!({ "val": new_val1 }))).await.unwrap();

            // Tx2 writes
            let target2 = if write2_k1 { "k1" } else { "k2" };
            let base2 = if write2_k1 { init1 } else { init2 };
            let new_val2 = apply_op(base2, delta2);
            col.update_op(&tx2, target2, &emb, Some(json!({ "val": new_val2 }))).await.unwrap();

            let res1 = tx1.commit().await;
            let res2 = tx2.commit().await;

            // Fetch final state
            let final_k1 = col.get("k1").await.unwrap().unwrap().metadata.unwrap()["val"].as_i64().unwrap();
            let final_k2 = col.get("k2").await.unwrap().unwrap().metadata.unwrap()["val"].as_i64().unwrap();

            // Calculate expected serial outcomes:
            // Outcome A: Tx1 then Tx2
            let mut s_a_k1 = init1;
            let mut s_a_k2 = init2;
            if write1_k1 { s_a_k1 = apply_op(init1, delta1); }
            else { s_a_k2 = apply_op(init2, delta1); }

            let s_a_base2 = if write2_k1 { s_a_k1 } else { s_a_k2 };
            if write2_k1 { s_a_k1 = apply_op(s_a_base2, delta2); }
            else { s_a_k2 = apply_op(s_a_base2, delta2); }

            // Outcome B: Tx2 then Tx1
            let mut s_b_k1 = init1;
            let mut s_b_k2 = init2;
            if write2_k1 { s_b_k1 = apply_op(init1, delta2); }
            else { s_b_k2 = apply_op(init2, delta2); }

            let s_b_base1 = if write1_k1 { s_b_k1 } else { s_b_k2 };
            if write1_k1 { s_b_k1 = apply_op(s_b_base1, delta1); }
            else { s_b_k2 = apply_op(s_b_base1, delta1); }

            // Outcome C: Only Tx1 committed
            let mut s_c_k1 = init1;
            let mut s_c_k2 = init2;
            if write1_k1 { s_c_k1 = apply_op(init1, delta1); }
            else { s_c_k2 = apply_op(init2, delta1); }

            // Outcome D: Only Tx2 committed
            let mut s_d_k1 = init1;
            let mut s_d_k2 = init2;
            if write2_k1 { s_d_k1 = apply_op(init1, delta2); }
            else { s_d_k2 = apply_op(init2, delta2); }

            // Outcome E: Neither committed
            let s_e_k1 = init1;
            let s_e_k2 = init2;

            let matches_serial =
                (res1.is_ok() && res2.is_ok() && ((final_k1 == s_a_k1 && final_k2 == s_a_k2) || (final_k1 == s_b_k1 && final_k2 == s_b_k2))) ||
                (res1.is_ok() && res2.is_err() && final_k1 == s_c_k1 && final_k2 == s_c_k2) ||
                (res1.is_err() && res2.is_ok() && final_k1 == s_d_k1 && final_k2 == s_d_k2) ||
                (res1.is_err() && res2.is_err() && final_k1 == s_e_k1 && final_k2 == s_e_k2);

            assert!(
                matches_serial,
                "Final state ({final_k1}, {final_k2}) with res1={:?}, res2={:?} must match one of valid serial schedules (A=({s_a_k1},{s_a_k2}), B=({s_b_k1},{s_b_k2}), C=({s_c_k1},{s_c_k2}), D=({s_d_k1},{s_d_k2}))",
                res1, res2
            );
        });
    }
}
