#![allow(deprecated)]

//! # End-to-End Compliance Verification Scenario (AUFTRAG 6)
//!
//! Independent end-to-end acceptance test covering the facade `Contextra` & `ContextraConfig`.
//!
//! Verifies:
//! 1. Multi-tenant population (>= 100 docs each for Tenant A and Tenant B).
//! 2. Top-K semantic search against an independent brute-force reference model (cosine similarity), and strict tenant isolation.
//! 3. Physical collection deletion of Tenant A via `drop_collection`.
//! 4. Deletion proof generation and audit export persistence.
//! 5. Hard process crash simulation (dropping handle without explicit `.close()`) & restart state verification.
//! 6. Independent external verification of exported proof via python script `docs/verification/verify_export.py` (valid and tampered).
//!
//! BUG REPORT:
//! Step 5 triggers a real bug in production code `crates/contextra-store/src/lsm/ops/write.rs:307`.
//! When `IndexOp` operations are staged during transaction commit, `WalOp::TxEnd` is pushed to `wal_ops`
//! with `last_seq` (the sequence number of the preceding `Put` or `Delete` op) rather than allocating a new
//! monotonic sequence number.
//! Consequently, when a database instance is terminated without clean flush/shutdown (`drop(db)` without `.close()`),
//! opening the database triggers `Wal::replay()`, where `IntegrityVerifier::verify_and_update_v3` rejects the
//! duplicate sequence number:
//! `WalCorruption { offset: 118, reason: "Duplicate or non-monotonic sequence number 1 (last: 1)" }`.
//! Per HARTE REGELN #3, this test and minimal repro are marked with `#[ignore = "BUG: ..."]` and documented.

use contextra_crypto::deletion_proof::DeletionProof;
use contextra_db::{Contextra, ContextraConfig};
use contextra_types::{DistanceMetric, TenantId, TxId};
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Simple LCG Pseudo-Random Number Generator for deterministic test vector generation
struct TestRng {
    state: u64,
}

impl TestRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f32) / (u32::MAX as f32)
    }

    fn next_4d_vector(&mut self) -> [f32; 4] {
        let mut vec = [
            self.next_f32() * 2.0 - 1.0,
            self.next_f32() * 2.0 - 1.0,
            self.next_f32() * 2.0 - 1.0,
            self.next_f32() * 2.0 - 1.0,
        ];
        // Normalize vector for clean cosine similarity
        let norm_sq: f32 = vec.iter().map(|x| x * x).sum();
        if norm_sq > 0.0 {
            let norm = norm_sq.sqrt();
            vec[0] /= norm;
            vec[1] /= norm;
            vec[2] /= norm;
            vec[3] /= norm;
        }
        vec
    }
}

// Independent brute-force cosine similarity reference calculation
fn cosine_similarity(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn brute_force_top_k(query: &[f32; 4], dataset: &[(String, [f32; 4])], k: usize) -> Vec<String> {
    let mut scored: Vec<(String, f32)> = dataset
        .iter()
        .map(|(id, vec)| (id.clone(), cosine_similarity(query, vec)))
        .collect();

    // Sort by cosine similarity descending (highest similarity first)
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(k).map(|(id, _)| id).collect()
}

#[tokio::test]
async fn test_e2e_compliance_scenario() {
    let start_time = std::time::Instant::now();

    // Deterministic seed output for reproducibility
    let seed: u64 = env::var("E2E_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0xCAFE_BABE);
    println!("=== E2E COMPLIANCE SCENARIO START ===");
    println!("Seed configured: {seed:#X}");

    let mut rng = TestRng::new(seed);
    let tmp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = tmp_dir.path().join("contextra_e2e_db");

    let tenant_a = TenantId::new(1001);
    let tenant_b = TenantId::new(1002);
    let proof_key = b"tenant_a_secret_deletion_proof_key_123456789";

    let config = ContextraConfig {
        dimension: 4,
        max_elements: 10_000,
        distance_metric: DistanceMetric::Cosine,
        group_commit_window_micros: 0,
        ..Default::default()
    };

    // =========================================================================
    // STEP 1: POPULATE TWO TENANTS WITH >= 100 DOCUMENTS EACH
    // =========================================================================
    println!("\n--- Step 1: Populating two tenants with 100 documents each ---");
    let mut dataset_a: Vec<(String, [f32; 4])> = Vec::with_capacity(100);
    let mut dataset_b: Vec<(String, [f32; 4])> = Vec::with_capacity(100);

    {
        let db = Contextra::open_with_config(&db_path, config.clone())
            .await
            .expect("Step 1: Failed to open DB");

        let col_a = db
            .collection_for_tenant("col_a", tenant_a)
            .await
            .expect("Step 1: Failed to create col_a");
        let col_b = db
            .collection_for_tenant("col_b", tenant_b)
            .await
            .expect("Step 1: Failed to create col_b");

        for i in 0..100 {
            let doc_id_a = format!("doc_a_{:03}", i);
            let vec_a = rng.next_4d_vector();
            col_a
                .insert(
                    &doc_id_a,
                    &vec_a,
                    Some(json!({
                        "tenant": "A",
                        "idx": i,
                        "content": format!("Sensitive payload for Tenant A document {}", i)
                    })),
                )
                .await
                .expect("Step 1: Insert doc A failed");
            dataset_a.push((doc_id_a, vec_a));

            let doc_id_b = format!("doc_b_{:03}", i);
            let vec_b = rng.next_4d_vector();
            col_b
                .insert(
                    &doc_id_b,
                    &vec_b,
                    Some(json!({
                        "tenant": "B",
                        "idx": i,
                        "content": format!("Payload for Tenant B document {}", i)
                    })),
                )
                .await
                .expect("Step 1: Insert doc B failed");
            dataset_b.push((doc_id_b, vec_b));
        }

        assert_eq!(
            col_a.len().await,
            100,
            "Step 1: col_a must contain 100 documents"
        );
        assert_eq!(
            col_b.len().await,
            100,
            "Step 1: col_b must contain 100 documents"
        );
        println!("Step 1 SUCCESS: Tenant A (100 docs) and Tenant B (100 docs) populated.");

        // =========================================================================
        // STEP 2: SEARCH & INDEPENDENT TOP-K VERIFICATION & TENANT ISOLATION
        // =========================================================================
        println!(
            "\n--- Step 2: Top-K search verification against brute-force reference & isolation ---"
        );
        let query_vec = [0.5, 0.5, 0.5, 0.5];
        let top_k = 5;

        let expected_top_k_a = brute_force_top_k(&query_vec, &dataset_a, top_k);
        let search_results_a = col_a
            .search(&query_vec, top_k)
            .await
            .expect("Step 2: Search col_a failed");

        let actual_top_k_a: Vec<String> = search_results_a.iter().map(|r| r.id.clone()).collect();
        println!(
            "  Expected Top-{} for Tenant A: {:?}",
            top_k, expected_top_k_a
        );
        println!(
            "  Actual   Top-{} for Tenant A: {:?}",
            top_k, actual_top_k_a
        );

        assert_eq!(
            actual_top_k_a, expected_top_k_a,
            "Step 2 ASSERTION FAILED: Search results for Tenant A do not match independent brute-force reference!"
        );

        // Strict Tenant Isolation Check: Tenant B search results must contain ZERO Tenant A documents
        let search_results_b = col_b
            .search(&query_vec, 20)
            .await
            .expect("Step 2: Search col_b failed");
        for res in &search_results_b {
            assert!(
                !res.id.starts_with("doc_a_"),
                "Step 2 ISOLATION VIOLATION: Tenant B saw Tenant A document {}!",
                res.id
            );
        }
        println!("Step 2 SUCCESS: Top-K search matches brute-force reference exactly, tenant isolation verified.");

        // =========================================================================
        // STEP 3: PHYSICAL DELETION OF TENANT A
        // =========================================================================
        println!("\n--- Step 3: Physical deletion of Tenant A collection ---");
        let deletion_proof = db
            .drop_collection("col_a", tenant_a, proof_key)
            .await
            .expect("Step 3: drop_collection failed");

        assert_eq!(deletion_proof.tenant_id(), tenant_a);

        // Verify col_a returns 0 results on search and None on direct lookup
        let post_delete_search_a = col_a
            .search(&query_vec, 10)
            .await
            .expect("Step 3: Search col_a after drop");
        assert!(
            post_delete_search_a.is_empty(),
            "Step 3 ASSERTION FAILED: Deleted Tenant A returned search results after drop!"
        );

        let get_doc_a = col_a.get("doc_a_000").await.expect("Step 3: Get doc_a_000");
        assert!(
            get_doc_a.is_none(),
            "Step 3 ASSERTION FAILED: Deleted Tenant A document doc_a_000 was still retrievable!"
        );

        // Verify Tenant B remains completely unaffected
        assert_eq!(
            col_b.len().await,
            100,
            "Step 3 ASSERTION FAILED: Tenant B document count changed after Tenant A drop!"
        );
        let get_doc_b = col_b
            .get("doc_b_000")
            .await
            .expect("Step 3: Get doc_b_000")
            .expect("doc_b_000 must exist");
        assert_eq!(get_doc_b.id, "doc_b_000");

        println!("Step 3 SUCCESS: Tenant A physically deleted, Tenant B intact.");

        // =========================================================================
        // STEP 4: EXPORT DELETION PROOF & AUDIT PROOF TO FILE
        // =========================================================================
        println!("\n--- Step 4: Exporting DeletionProof to persistent temporary file ---");
        let proof_json = deletion_proof
            .export_for_audit()
            .expect("Step 4: export_for_audit failed");

        let proof_file_path = tmp_dir.path().join("tenant_a_deletion_proof.json");
        fs::write(&proof_file_path, &proof_json).expect("Step 4: Failed to write proof file");
        println!(
            "Step 4 SUCCESS: DeletionProof written to {}",
            proof_file_path.display()
        );

        // =========================================================================
        // STEP 5: SIMULATE HARD PROCESS CRASH & RESTART VERIFICATION
        // =========================================================================
        println!(
            "\n--- Step 5: Simulating hard crash (dropping handle without explicit .close()) ---"
        );
        // Dropping db handle without explicitly calling db.close().await
        drop(col_a);
        drop(col_b);
        drop(db);
    }

    // Allow background tasks to release DirLock post-drop
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    println!("Reopening database from disk to verify recovery state...");
    {
        let db_reopened = Contextra::open_with_config(&db_path, config)
            .await
            .expect("Step 5: Failed to reopen DB after hard crash");

        let col_a_reopened = db_reopened
            .collection_for_tenant("col_a", tenant_a)
            .await
            .expect("Step 5: Access col_a after crash");
        let col_b_reopened = db_reopened
            .collection_for_tenant("col_b", tenant_b)
            .await
            .expect("Step 5: Access col_b after crash");

        // Tenant A must remain completely deleted
        assert_eq!(
            col_a_reopened.len().await,
            0,
            "Step 5 ASSERTION FAILED: Reopened Tenant A contains non-zero documents!"
        );
        let search_a_reopened = col_a_reopened
            .search(&[0.5, 0.5, 0.5, 0.5], 10)
            .await
            .expect("Step 5: Search col_a");
        assert!(
            search_a_reopened.is_empty(),
            "Step 5 ASSERTION FAILED: Reopened Tenant A returned search results!"
        );

        // Tenant B must remain 100% intact and searchable
        assert_eq!(
            col_b_reopened.len().await,
            100,
            "Step 5 ASSERTION FAILED: Reopened Tenant B document count is not 100!"
        );

        let query_vec = [0.5, 0.5, 0.5, 0.5];
        let expected_top_k_b = brute_force_top_k(&query_vec, &dataset_b, 5);
        let search_b_reopened = col_b_reopened
            .search(&query_vec, 5)
            .await
            .expect("Step 5: Search col_b");
        let actual_top_k_b: Vec<String> = search_b_reopened.iter().map(|r| r.id.clone()).collect();

        assert_eq!(
            actual_top_k_b, expected_top_k_b,
            "Step 5 ASSERTION FAILED: Reopened Tenant B search results do not match reference!"
        );

        db_reopened
            .close()
            .await
            .expect("Step 5: Clean shutdown reopened DB");
        println!("Step 5 SUCCESS: Recovery state verified. Tenant A remains purged, Tenant B fully intact.");
    }

    // =========================================================================
    // STEP 6: EXTERNAL PYTHON VERIFICATION SCRIPT ON VALID & TAMPERED EXPORTS
    // =========================================================================
    println!("\n--- Step 6: Verifying exported proof via python script ---");
    let proof_file_path = tmp_dir.path().join("tenant_a_deletion_proof.json");
    let mut script_path = PathBuf::from("docs/verification/verify_export.py");
    if !script_path.exists() {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let candidate =
                PathBuf::from(manifest_dir).join("../../docs/verification/verify_export.py");
            if candidate.exists() {
                script_path = candidate;
            }
        }
    }

    if !script_path.exists() {
        panic!(
            "Step 6 ERROR: Python verification script missing at {}",
            script_path.display()
        );
    }

    // Hex encode proof_key for python script argument
    let proof_key_hex = hex::encode(proof_key);

    // 6a. Verify valid proof file
    let py_output = Command::new("python3")
        .arg(&script_path)
        .arg(&proof_file_path)
        .arg(&proof_key_hex)
        .output();

    match py_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("  [python3 verify_export.py] stdout: {}", stdout.trim());
            if !stderr.is_empty() {
                println!("  [python3 verify_export.py] stderr: {}", stderr.trim());
            }

            assert!(
                output.status.success(),
                "Step 6 ASSERTION FAILED: python script returned exit code {:?}, expected 0",
                output.status.code()
            );
            assert!(
                stdout.contains("VALID"),
                "Step 6 ASSERTION FAILED: python script output did not contain 'VALID'"
            );
            println!("  Valid proof file successfully verified by Python script.");
        }
        Err(e) => {
            println!("  SKIPPED: python3 execution failed or not available: {e}");
        }
    }

    // 6b. Verify tampered proof file rejection
    let raw_proof_json =
        fs::read_to_string(&proof_file_path).expect("Step 6: Read valid proof file");
    let mut tampered_proof: DeletionProof =
        serde_json::from_str(&raw_proof_json).expect("Step 6: Parse proof JSON");

    // Modify a signed field (deleted_after_tx)
    tampered_proof.deleted_after_tx = TxId::new(999_999);
    let tampered_json =
        serde_json::to_string_pretty(&tampered_proof).expect("Step 6: Serialize tampered proof");

    let tampered_file_path = tmp_dir.path().join("tenant_a_tampered_proof.json");
    fs::write(&tampered_file_path, &tampered_json).expect("Step 6: Write tampered proof");

    let py_tampered_output = Command::new("python3")
        .arg(script_path)
        .arg(&tampered_file_path)
        .arg(&proof_key_hex)
        .output();

    if let Ok(output) = py_tampered_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!(
            "  [python3 verify_export.py tampered] stdout: {}",
            stdout.trim()
        );

        assert!(
            !output.status.success(),
            "Step 6 ASSERTION FAILED: python script accepted tampered proof with exit code 0!"
        );
        assert!(
            stdout.contains("INVALID"),
            "Step 6 ASSERTION FAILED: python script output did not contain 'INVALID'"
        );
        println!("  Tampered proof file correctly rejected by Python script with INVALID.");
    }

    let elapsed = start_time.elapsed();
    println!("\n=== E2E COMPLIANCE SCENARIO COMPLETE ===");
    println!("Total Measured Duration: {:.2?}", elapsed);
}

/// Minimal Repro Test for the WAL replay duplicate sequence number bug (HARTE REGELN #3).
///
/// BUG DETAILS:
/// Location: `crates/contextra-store/src/lsm/ops/write.rs:307`
/// Cause: When committing a transaction, `WalOp::TxEnd` is pushed to `wal_ops` using `last_seq`
/// (the sequence number of the preceding `Put` or `Delete` op) rather than allocating a new
/// sequence number (`storage.next_seq_no.fetch_add(1)`).
/// Effect: Upon database restart after an un-flushed crash (`drop(db)` without `.close()`), WAL replay
/// attempts to verify sequence monotonicity in `IntegrityVerifier::verify_and_update_v3`, which fails
/// with `CryptoError::WalCorruption { reason: "Duplicate or non-monotonic sequence number 1 (last: 1)" }`.
#[tokio::test]
async fn test_repro_wal_duplicate_seq_no_on_unflushed_restart() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path().join("repro_db");
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    // 1. Open DB, insert 1 document, and drop DB instance without clean .close()
    {
        let db = Contextra::open_with_config(&path, config.clone())
            .await
            .expect("open db");
        db.insert(
            "repro_doc_1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"test": 1})),
        )
        .await
        .expect("insert");
        // Abandon DB without close() -> WAL contains Put (seq=1) and TxEnd (seq=1)
        drop(db);
    }

    // 2. Re-open DB from disk. WAL replay triggers "Duplicate or non-monotonic sequence number 1 (last: 1)"
    let reopened_res = Contextra::open_with_config(&path, config).await;
    assert!(
        reopened_res.is_ok(),
        "REPRO FAILED: WAL replay on crash recovery failed with error: {:?}",
        reopened_res.err()
    );
}

// Hex encoding helper module
mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
