#[path = "support_canary/mod.rs"]
mod support_canary;

use contextra_db::{Contextra, ContextraConfig};
use contextra_types::{DistanceMetric, TenantId};
use serde_json::json;
use support_canary::{scan_dir, PersonCanaryGenerator};
use tempfile::TempDir;

/// Phase 3: Deletion test against real Contextra database with Canary Oracle.
/// Demonstrates canary detection before and after deletion operations.
#[tokio::test]
async fn test_p08_forget_canary_real_database() {
    let tmp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = tmp_dir.path().join("contextra_canary_db");

    let tenant_a = TenantId::try_new(101).expect("Tenant A");
    let tenant_b = TenantId::try_new(102).expect("Tenant B");
    let tenant_c = TenantId::try_new(103).expect("Tenant C");

    let proof_key = b"tenant_a_secret_proof_key_32bytes!!";

    let gen_a = PersonCanaryGenerator::new("PersonA", 0xAAAA_0001);
    let gen_b = PersonCanaryGenerator::new("PersonB", 0xBBBB_0002);
    let gen_c = PersonCanaryGenerator::new("PersonC", 0xCCCC_0003);

    // Canary text strings and vectors
    let text_a1 = gen_a.generate_text_canary("mem_a1");
    let text_a2 = gen_a.generate_text_canary("mem_a2");
    let vec_a = gen_a.generate_vector_canary("vec_a", 4);

    let text_b1 = gen_b.generate_text_canary("mem_b1");
    let vec_b = gen_b.generate_vector_canary("vec_b", 4);

    let text_c1 = gen_c.generate_text_canary("mem_c1");
    let vec_c = gen_c.generate_vector_canary("vec_c", 4);

    let needles_a = [
        gen_a.build_text_needles(&text_a1, "mem_a1"),
        gen_a.build_text_needles(&text_a2, "mem_a2"),
        gen_a.build_vector_needles(&vec_a, "vec_a"),
    ]
    .concat();

    let needles_b = [
        gen_b.build_text_needles(&text_b1, "mem_b1"),
        gen_b.build_vector_needles(&vec_b, "vec_b"),
    ]
    .concat();

    let needles_c = [
        gen_c.build_text_needles(&text_c1, "mem_c1"),
        gen_c.build_vector_needles(&vec_c, "vec_c"),
    ]
    .concat();

    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    // =========================================================================
    // 1. POPULATE DATABASE FOR PERSONS A, B, AND C
    // =========================================================================
    {
        let db = Contextra::open_with_config(&db_path, config.clone())
            .await
            .expect("Failed to open DB for population");

        let col_a = db
            .collection_for_tenant("person_a_col", tenant_a)
            .await
            .expect("Create Person A col");
        let col_b = db
            .collection_for_tenant("person_b_col", tenant_b)
            .await
            .expect("Create Person B col");
        let col_c = db
            .collection_for_tenant("person_c_col", tenant_c)
            .await
            .expect("Create Person C col");

        // Insert Person A documents (multi-modal: text, vector, relations)
        col_a
            .insert(
                "doc_a1",
                &vec_a,
                Some(json!({ "text": text_a1, "owner": "PersonA" })),
            )
            .await
            .expect("Insert doc_a1");

        col_a
            .insert(
                "doc_a2",
                &[0.1, 0.2, 0.3, 0.4],
                Some(json!({ "text": text_a2, "owner": "PersonA" })),
            )
            .await
            .expect("Insert doc_a2");

        col_a
            .relate("doc_a1", "doc_a2", "relates_to")
            .await
            .expect("Relate doc_a1 to doc_a2");

        // Insert Person B
        col_b
            .insert(
                "doc_b1",
                &vec_b,
                Some(json!({ "text": text_b1, "owner": "PersonB" })),
            )
            .await
            .expect("Insert doc_b1");

        // Insert Person C
        col_c
            .insert(
                "doc_c1",
                &vec_c,
                Some(json!({ "text": text_c1, "owner": "PersonC" })),
            )
            .await
            .expect("Insert doc_c1");

        db.flush().await.expect("Flush DB before closing");
        db.close().await.expect("Close DB after population");
    }

    // =========================================================================
    // 2. PRE-DELETION CANARY SCAN
    // =========================================================================
    let pre_hits_a = scan_dir(&db_path, &needles_a);
    let pre_hits_b = scan_dir(&db_path, &needles_b);
    let pre_hits_c = scan_dir(&db_path, &needles_c);

    println!(
        "PRE-DELETION CANARY SCAN RESULTS: Person A = {} hits, Person B = {} hits, Person C = {} hits",
        pre_hits_a.len(),
        pre_hits_b.len(),
        pre_hits_c.len()
    );

    assert!(
        !pre_hits_a.is_empty(),
        "PRE-DELETION CHECK FAILED: Expected at least 1 canary hit for Person A prior to deletion"
    );
    assert!(
        !pre_hits_b.is_empty(),
        "PRE-DELETION CHECK FAILED: Expected at least 1 canary hit for Person B prior to deletion"
    );
    assert!(
        !pre_hits_c.is_empty(),
        "PRE-DELETION CHECK FAILED: Expected at least 1 canary hit for Person C prior to deletion"
    );

    // =========================================================================
    // 3. DELETE PERSON A AND RESTART DATABASE
    // =========================================================================
    let maybe_proof = {
        let db = Contextra::open_with_config(&db_path, config.clone())
            .await
            .expect("Open DB for deletion");

        // Execute drop_collection for Person A
        let proof_res = db
            .drop_collection("person_a_col", tenant_a, proof_key)
            .await;

        let proof = match proof_res {
            Ok(proof) => Some(proof),
            Err(e) => {
                println!("drop_collection note: {e}");
                db.purge_tenant(tenant_a)
                    .await
                    .expect("purge_tenant Person A");
                None
            }
        };

        db.flush().await.expect("Flush after deletion");
        db.close().await.expect("Close DB after deletion");
        proof
    };

    // =========================================================================
    // 4. POST-DELETION CANARY SCAN OVER THE ENTIRE DATABASE DIRECTORY
    // =========================================================================
    let post_hits_a = scan_dir(&db_path, &needles_a);
    let post_hits_b = scan_dir(&db_path, &needles_b);
    let post_hits_c = scan_dir(&db_path, &needles_c);

    println!(
        "POST-DELETION CANARY SCAN RESULTS: Person A = {} hits, Person B = {} hits, Person C = {} hits",
        post_hits_a.len(),
        post_hits_b.len(),
        post_hits_c.len()
    );

    if !post_hits_a.is_empty() {
        println!("Residual Canary Hits found for Person A post-deletion:");
        for hit in &post_hits_a {
            println!("  Residual Hit: {}", hit);
        }
    }

    // =========================================================================
    // 5. VERIFY PERSONS B AND C REMAIN FULLY INTACT AND QUERYABLE
    // =========================================================================
    assert!(
        !post_hits_b.is_empty(),
        "Person B canary hits must remain present post Person A deletion"
    );
    assert!(
        !post_hits_c.is_empty(),
        "Person C canary hits must remain present post Person A deletion"
    );

    {
        let db = Contextra::open_with_config(&db_path, config)
            .await
            .expect("Reopen DB to check Person B and C accessibility");

        let col_b = db
            .collection_for_tenant("person_b_col", tenant_b)
            .await
            .expect("Access col_b");
        let col_c = db
            .collection_for_tenant("person_c_col", tenant_c)
            .await
            .expect("Access col_c");

        let doc_b = col_b
            .get("doc_b1")
            .await
            .expect("Get doc_b1")
            .expect("doc_b1 must exist");
        assert_eq!(doc_b.id, "doc_b1");
        let meta_b = doc_b.metadata.expect("meta b");
        assert_eq!(meta_b["owner"], "PersonB");

        let doc_c = col_c
            .get("doc_c1")
            .await
            .expect("Get doc_c1")
            .expect("doc_c1 must exist");
        assert_eq!(doc_c.id, "doc_c1");

        db.close().await.expect("Close DB after verification");
    }

    // =========================================================================
    // 6. VERIFY Cryptographic DeletionProof if generated
    // =========================================================================
    if let Some(proof) = maybe_proof {
        assert_eq!(proof.tenant_id(), tenant_a);
        let signing_key =
            ed25519_dalek::SigningKey::from_bytes(proof_key[..32].try_into().unwrap());
        let verifying_key = signing_key.verifying_key();
        let is_valid = proof
            .verify(contextra_crypto::deletion_proof::VerificationKey::Ed25519(
                &verifying_key,
            ))
            .expect("Verify proof signature");
        assert!(is_valid, "DeletionProof signature verification failed!");
    }

    // Strict assertion: Scanner must find 0 residual hits for Person A post-deletion
    assert!(
        post_hits_a.is_empty(),
        "POST-DELETION CANARY SCAN FAILED: Found {} residual canary hits for Person A after deletion! Residual hits: {:?}",
        post_hits_a.len(),
        post_hits_a
    );
}
