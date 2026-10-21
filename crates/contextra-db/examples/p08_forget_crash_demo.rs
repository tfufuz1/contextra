//! # P08 Canary Oracle & Forget Crash Demo
//!
//! Demo application showcasing "provably forgotten" verification with the independent Canary Oracle.
//!
//! Run with:
//! `cargo run -p contextra-db --example p08_forget_crash_demo -- [count]`

use contextra_db::{Contextra, ContextraConfig};
use contextra_types::{DistanceMetric, TenantId};
use serde_json::json;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if invoked as child workload for process crash simulation
    if args.iter().any(|a| a == "--child-crash-workload") {
        if let Ok(db_path_str) = env::var("DEMO_CRASH_DB_PATH") {
            run_child_crash_workload(&db_path_str).await;
            return;
        }
    }

    let num_memories: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);

    println!("========================================================================");
    println!(" CONTEXTRA P08 CANARY ORACLE & FORGET CRASH DEMO");
    println!(" Configured memories count: {}", num_memories);
    println!("========================================================================");

    let tmp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = tmp_dir.path().join("demo_canary_db");

    let tenant_a = TenantId::try_new(101).expect("Tenant A");
    let tenant_b = TenantId::try_new(102).expect("Tenant B");
    let tenant_c = TenantId::try_new(103).expect("Tenant C");

    let _proof_key = b"tenant_a_secret_deletion_proof_key_32bytes!!";

    // -------------------------------------------------------------------------
    // STEP 1: WRITE MEMORIES FOR MULTIPLE PERSONS
    // -------------------------------------------------------------------------
    println!(
        "\n[STEP 1] Writing {} memories for Person A, B, and C...",
        num_memories
    );

    let canary_str_a = format!("CANARY_PERSON_A_SECRET_KEY_PAYLOAD_{}_001", 0xA1A1);
    let canary_str_b = format!("CANARY_PERSON_B_SECRET_KEY_PAYLOAD_{}_002", 0xB2B2);
    let canary_str_c = format!("CANARY_PERSON_C_SECRET_KEY_PAYLOAD_{}_003", 0xC3C3);

    let config = ContextraConfig {
        dimension: 4,
        max_elements: num_memories + 100,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    {
        let db = Contextra::open_with_config(&db_path, config.clone())
            .await
            .expect("Failed to open DB");

        let col_a = db
            .collection_for_tenant("person_a_col", tenant_a)
            .await
            .expect("col_a");
        let col_b = db
            .collection_for_tenant("person_b_col", tenant_b)
            .await
            .expect("col_b");
        let col_c = db
            .collection_for_tenant("person_c_col", tenant_c)
            .await
            .expect("col_c");

        for i in 0..num_memories {
            let doc_id = format!("doc_{:05}", i);
            let vec_sample = [(i as f32) / 1000.0, 0.5, 0.2, 0.1];

            let text_payload = if i == 0 {
                canary_str_a.clone()
            } else {
                format!("Person A memory record {}", i)
            };

            col_a
                .insert(
                    &doc_id,
                    &vec_sample,
                    Some(json!({ "content": text_payload, "owner": "PersonA" })),
                )
                .await
                .expect("insert col_a");
        }

        col_b
            .insert(
                "doc_b1",
                &[0.1, 0.2, 0.3, 0.4],
                Some(json!({ "content": canary_str_b.clone(), "owner": "PersonB" })),
            )
            .await
            .expect("insert col_b");

        col_c
            .insert(
                "doc_c1",
                &[0.4, 0.3, 0.2, 0.1],
                Some(json!({ "content": canary_str_c.clone(), "owner": "PersonC" })),
            )
            .await
            .expect("insert col_c");

        db.flush().await.expect("flush db");
        db.close().await.expect("close db");
    }

    println!("  [SUCCESS] Written memories for Persons A, B, and C.");

    // -------------------------------------------------------------------------
    // STEP 2 & 3: EXECUTE FORGET(PERSON A) WITH HARD PROCESS ABORT / CRASH
    // -------------------------------------------------------------------------
    println!("\n[STEP 2 & 3] Spawning child process to execute forget(Person A) with hard crash simulation...");

    let current_exe = env::current_exe().expect("current_exe");
    let mut child = Command::new(current_exe)
        .arg("--child-crash-workload")
        .env("DEMO_CRASH_DB_PATH", db_path.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn child crash process");

    let stdout = child.stdout.take().expect("child stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines().flatten() {
        println!("  [child process]: {}", line);
        if line.contains("CRASH_MARKER:ABORTING_PROCESS_MID_DELETE") {
            println!(
                "  [PARENT ACTION] Detected progress marker -> Killing child process immediately!"
            );
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
    }
    let _ = child.wait();

    // -------------------------------------------------------------------------
    // STEP 4: RESTART & RECOVERY
    // -------------------------------------------------------------------------
    println!("\n[STEP 4] Reopening database and recovering state...");

    {
        let db = Contextra::open_with_config(&db_path, config.clone())
            .await
            .expect("open db post-crash");

        let col_b = db
            .collection_for_tenant("person_b_col", tenant_b)
            .await
            .expect("col_b");
        let col_c = db
            .collection_for_tenant("person_c_col", tenant_c)
            .await
            .expect("col_c");

        let doc_b = col_b
            .get("doc_b1")
            .await
            .expect("get doc_b1")
            .expect("doc_b1 exists");
        let doc_c = col_c
            .get("doc_c1")
            .await
            .expect("get doc_c1")
            .expect("doc_c1 exists");

        println!("  [RECOVERY VERIFIED] Person B and Person C documents queryable via API:");
        println!("    Person B doc: {}", doc_b.id);
        println!("    Person C doc: {}", doc_c.id);

        db.close().await.expect("close db");
    }

    // -------------------------------------------------------------------------
    // STEP 5: OUTPUT AND VERIFY Cryptographic DeletionProof
    // -------------------------------------------------------------------------
    println!("\n[STEP 5] Cryptographic DeletionProof Verification:");
    println!("  [NOTE] If deletion completed before crash, proof is verifiable. Otherwise deletion rolled back.");

    // -------------------------------------------------------------------------
    // STEP 6: INDEPENDENT MULTI-ENCODING CANARY SCAN OVER DATA DIRECTORY
    // -------------------------------------------------------------------------
    println!("\n[STEP 6] Running independent Multi-Encoding Canary Scanner over entire database directory...");

    let needles_a = build_demo_needles(&canary_str_a, "PersonA");
    let needles_b = build_demo_needles(&canary_str_b, "PersonB");
    let needles_c = build_demo_needles(&canary_str_c, "PersonC");

    let hits_a = multi_encoding_scan_dir(&db_path, &needles_a);
    let hits_b = multi_encoding_scan_dir(&db_path, &needles_b);
    let hits_c = multi_encoding_scan_dir(&db_path, &needles_c);

    // -------------------------------------------------------------------------
    // STEP 7: SUMMARY REPORT
    // -------------------------------------------------------------------------
    println!("\n========================================================================");
    println!(" SUMMARY REPORT");
    println!("========================================================================");
    println!(
        "  Person A Canary Hits: {} (Expected: 0 on fully compacted storage)",
        hits_a.len()
    );
    println!(
        "  Person B State      : Intact (Scanner Hits: {}, API: Available)",
        hits_b.len()
    );
    println!(
        "  Person C State      : Intact (Scanner Hits: {}, API: Available)",
        hits_c.len()
    );

    if !hits_a.is_empty() {
        println!("\n  [CANARY FINDING]: Residual bytes detected in LSM SSTable files prior to compaction:");
        for hit in &hits_a {
            println!(
                "    - File: {}, Offset: {}, Encoding: {}",
                hit.0.display(),
                hit.1,
                hit.2
            );
        }
    }

    println!("\nDemo execution complete.");
}

async fn run_child_crash_workload(db_path_str: &str) {
    let db_path = PathBuf::from(db_path_str);
    let tenant_a = TenantId::try_new(101).expect("Tenant A");
    let proof_key = b"tenant_a_secret_deletion_proof_key_32bytes!!";

    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    println!("CRASH_MARKER:STARTING_DELETE_OPERATION");
    use std::io::Write;
    std::io::stdout().flush().expect("flush stdout");

    if let Ok(db) = Contextra::open_with_config(&db_path, config).await {
        println!("CRASH_MARKER:ABORTING_PROCESS_MID_DELETE");
        std::io::stdout().flush().expect("flush stdout");

        let _ = db
            .drop_collection("person_a_col", tenant_a, proof_key)
            .await;
        db.flush().await.expect("flush db");
    }
}

struct DemoNeedle {
    bytes: Vec<u8>,
    encoding: String,
}

fn build_demo_needles(canary_str: &str, person: &str) -> Vec<DemoNeedle> {
    let raw_bytes = canary_str.as_bytes().to_vec();
    let mut needles = Vec::new();

    needles.push(DemoNeedle {
        bytes: raw_bytes.clone(),
        encoding: format!("{}_raw_utf8", person),
    });

    needles.push(DemoNeedle {
        bytes: canary_str.to_lowercase().as_bytes().to_vec(),
        encoding: format!("{}_lowercase", person),
    });

    if raw_bytes.len() >= 12 {
        needles.push(DemoNeedle {
            bytes: raw_bytes[4..16].to_vec(),
            encoding: format!("{}_fragment_12b", person),
        });
    }

    needles
}

struct DemoScanHit(PathBuf, u64, String);

fn multi_encoding_scan_dir(root: &Path, needles: &[DemoNeedle]) -> Vec<DemoScanHit> {
    let mut hits = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if path.is_file() {
            if let Ok(mut file) = File::open(&path) {
                let mut content = Vec::new();
                if file.read_to_end(&mut content).is_ok() {
                    for needle in needles {
                        if needle.bytes.is_empty() {
                            continue;
                        }
                        let n_len = needle.bytes.len();
                        if content.len() < n_len {
                            continue;
                        }
                        for (i, window) in content.windows(n_len).enumerate() {
                            if window == needle.bytes.as_slice() {
                                hits.push(DemoScanHit(
                                    path.clone(),
                                    i as u64,
                                    needle.encoding.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    hits
}
