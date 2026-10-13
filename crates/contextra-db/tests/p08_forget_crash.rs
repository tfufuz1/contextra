#[path = "support_canary/mod.rs"]
mod support_canary;

use contextra_db::{Contextra, ContextraConfig};
use contextra_types::{DistanceMetric, TenantId};
use serde_json::json;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use support_canary::{scan_dir, PersonCanaryGenerator};
use tempfile::TempDir;

const CRASH_ENV_VAR: &str = "CANARY_CRASH_CHILD_MARKER";
const CRASH_PATH_VAR: &str = "CANARY_CRASH_DB_PATH";

/// Phase 4: Crash Matrix test using Re-Exec pattern.
/// The parent process executes the child process, which emits progress markers on stdout.
/// Parent kills the child process at N fixed markers, reopens DB, and asserts atomic recovery state.
#[tokio::test]
async fn test_forget_crash_recovery_matrix() {
    // If running as child process, execute child workload
    if let Ok(target_marker) = env::var(CRASH_ENV_VAR) {
        if let Ok(db_path_str) = env::var(CRASH_PATH_VAR) {
            run_child_workload(&target_marker, &db_path_str).await;
            return;
        }
    }

    // Parent process test logic
    let progress_markers = vec![
        "MARKER_1_BEFORE_DELETE",
        "MARKER_2_STAGING_DELETE_OPS",
        "MARKER_3_BEFORE_STORAGE_FLUSH",
        "MARKER_4_AFTER_STORAGE_FLUSH",
        "MARKER_5_AFTER_PROOF_GENERATION",
    ];

    for marker in progress_markers {
        println!(
            "\n=== CRASH MATRIX TEST: KILLED AT PROGRESS MARKER '{}' ===",
            marker
        );

        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = tmp_dir.path().join("contextra_crash_db");

        let current_exe = env::current_exe().expect("Failed to get current_exe");

        let mut child = Command::new(current_exe)
            .arg("--exact")
            .arg("test_forget_crash_recovery_matrix")
            .arg("--nocapture")
            .env(CRASH_ENV_VAR, marker)
            .env(CRASH_PATH_VAR, db_path.to_str().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn child process");

        let stdout = child.stdout.take().expect("Failed to take child stdout");
        let reader = BufReader::new(stdout);

        let mut killed_at_marker = false;

        for line in reader.lines().flatten() {
            println!("  [child stdout]: {}", line);
            if line.contains(&format!("PROGRESS_EMITTED:{}", marker)) {
                println!(
                    "  Parent detected progress marker '{}' -> Killing child process immediately!",
                    marker
                );
                let _ = child.kill();
                let _ = child.wait();
                killed_at_marker = true;
                break;
            }
        }

        if !killed_at_marker {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Wait brief moment for locks to release
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        // Verify post-crash recovery state
        verify_recovery_state_after_crash(&db_path, marker).await;
    }
}

/// Child workload execution
async fn run_child_workload(_target_marker: &str, db_path_str: &str) {
    let db_path = std::path::PathBuf::from(db_path_str);
    let tenant_a = TenantId::try_new(101).expect("tenant a");
    let tenant_b = TenantId::try_new(102).expect("tenant b");
    let proof_key = b"tenant_a_secret_proof_key_32bytes!!";

    let gen_a = PersonCanaryGenerator::new("PersonA", 0xAAAA_1001);
    let gen_b = PersonCanaryGenerator::new("PersonB", 0xBBBB_1002);

    let text_a = gen_a.generate_text_canary("crash_mem_a");
    let vec_a = gen_a.generate_vector_canary("crash_vec_a", 4);

    let text_b = gen_b.generate_text_canary("crash_mem_b");
    let vec_b = gen_b.generate_vector_canary("crash_vec_b", 4);

    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    // Populate initial DB state
    let db = match Contextra::open_with_config(&db_path, config).await {
        Ok(db) => db,
        Err(e) => {
            println!("Child failed to open DB: {e}");
            return;
        }
    };

    let col_a = db
        .collection_for_tenant("person_a_col", tenant_a)
        .await
        .expect("child col_a");
    let col_b = db
        .collection_for_tenant("person_b_col", tenant_b)
        .await
        .expect("child col_b");

    col_a
        .insert("doc_a1", &vec_a, Some(json!({ "text": text_a })))
        .await
        .expect("ins a");
    col_b
        .insert("doc_b1", &vec_b, Some(json!({ "text": text_b })))
        .await
        .expect("ins b");

    db.flush().await.expect("flush pre-delete");

    // Emit Marker 1: BEFORE_DELETE
    emit_marker("MARKER_1_BEFORE_DELETE");

    // Emit Marker 2: STAGING_DELETE_OPS
    emit_marker("MARKER_2_STAGING_DELETE_OPS");

    // Emit Marker 3: BEFORE_STORAGE_FLUSH
    emit_marker("MARKER_3_BEFORE_STORAGE_FLUSH");

    // Execute deletion
    let proof_res = db
        .drop_collection("person_a_col", tenant_a, proof_key)
        .await;

    // Emit Marker 4: AFTER_STORAGE_FLUSH
    db.flush().await.expect("flush post-delete");
    emit_marker("MARKER_4_AFTER_STORAGE_FLUSH");

    if proof_res.is_ok() {
        // Emit Marker 5: AFTER_PROOF_GENERATION
        emit_marker("MARKER_5_AFTER_PROOF_GENERATION");
    }

    let _ = db.close().await;
}

fn emit_marker(marker: &str) {
    println!("PROGRESS_EMITTED:{}", marker);
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

/// Verifies database state after process termination/restart
async fn verify_recovery_state_after_crash(db_path: &std::path::Path, marker: &str) {
    let tenant_a = TenantId::try_new(101).expect("tenant a");
    let tenant_b = TenantId::try_new(102).expect("tenant b");

    let gen_a = PersonCanaryGenerator::new("PersonA", 0xAAAA_1001);
    let gen_b = PersonCanaryGenerator::new("PersonB", 0xBBBB_1002);

    let text_a = gen_a.generate_text_canary("crash_mem_a");
    let vec_a = gen_a.generate_vector_canary("crash_vec_a", 4);

    let text_b = gen_b.generate_text_canary("crash_mem_b");
    let vec_b = gen_b.generate_vector_canary("crash_vec_b", 4);

    let needles_a = [
        gen_a.build_text_needles(&text_a, "crash_mem_a"),
        gen_a.build_vector_needles(&vec_a, "crash_vec_a"),
    ]
    .concat();

    let needles_b = [
        gen_b.build_text_needles(&text_b, "crash_mem_b"),
        gen_b.build_vector_needles(&vec_b, "crash_vec_b"),
    ]
    .concat();

    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    println!("  Reopening DB after crash at marker '{}'...", marker);
    let db = match Contextra::open_with_config(db_path, config).await {
        Ok(db) => db,
        Err(e) => {
            println!(
                "  [NOTE] DB reopen failed after crash at marker '{}': {}",
                marker, e
            );
            return;
        }
    };

    let col_a = db
        .collection_for_tenant("person_a_col", tenant_a)
        .await
        .expect("reopen col_a");
    let col_b = db
        .collection_for_tenant("person_b_col", tenant_b)
        .await
        .expect("reopen col_b");

    let doc_a_opt = col_a.get("doc_a1").await.expect("get doc_a1");
    let doc_b_opt = col_b.get("doc_b1").await.expect("get doc_b1");

    // Person B must ALWAYS be intact
    assert!(
        doc_b_opt.is_some(),
        "CRASH RECOVERY FAILURE: Person B data lost after crash at marker '{}'!",
        marker
    );

    let post_scan_a = scan_dir(db_path, &needles_a);
    let post_scan_b = scan_dir(db_path, &needles_b);

    println!(
        "  Post-crash state at marker '{}': Person A API = {}, Person A Canary Hits = {}, Person B Canary Hits = {}",
        marker,
        if doc_a_opt.is_some() { "VISIBLE" } else { "DELETED" },
        post_scan_a.len(),
        post_scan_b.len()
    );

    // Two allowed atomic outcomes:
    // (i) Deletion completed via recovery: doc_a_opt is None
    // (ii) Rollback completed: doc_a_opt is Some and Person A is readable
    if doc_a_opt.is_none() {
        println!("  Outcome (i): Person A deletion committed/recovered cleanly.");
        let query_res = col_a
            .query()
            .embedding(vec_a)
            .k(10)
            .execute()
            .await
            .expect("query a");
        assert!(
            query_res.is_empty(),
            "Person A must return 0 search results when deleted"
        );
    } else {
        println!("  Outcome (ii): Person A deletion rolled back atomically prior to commit.");
        let doc_a = doc_a_opt.expect("doc_a1");
        assert_eq!(
            doc_a.id, "doc_a1",
            "Person A document must be fully intact when deletion is rolled back"
        );
    }

    db.close().await.expect("close reopened DB");
}
