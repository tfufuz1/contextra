// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Real-Scan Integrationstest für Relations- und TextIndex-Zeugen (F6).
// INVARIANTEN: Canary-Erkennung vor Löschung, 0 Treffer nach Löschung, Mutationsprüfung.

#![forbid(unsafe_code)]

use contextra_engine::deletion::relations_witness::{
    issue_relations_witness, scan_and_issue_relations_witness,
};
use contextra_engine::deletion::text_witness::{
    issue_text_index_witness, scan_and_issue_text_index_witness,
};
use contextra_ports::{StorageEngine, TextIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, TxId};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_relations_real_scan_lifecycle_and_mutation() {
    let tmp = TempDir::new().expect("temp dir");
    let db = contextra_engine::Contextra::open(tmp.path())
        .await
        .expect("open db");

    let doc_id = DocId::from_key("canary_doc_rel_1").unwrap();
    let scope_hash = [42u8; 32];
    let as_of_tx = TxId(100);
    let user_id = "canary_doc_rel_1";
    let rel_prefix = b"__rel:";

    // 1. Pre-deletion state: Relation key exists
    let col = db.collection("default").await.expect("col");
    let rel_key = format!("__rel:{}:target_node", user_id);
    col.storage()
        .put(as_of_tx, rel_key.as_bytes(), b"edge_data")
        .await
        .expect("put rel_key");

    // Scan before deletion MUST find canary (hits > 0) and refuse witness issuance
    let pre_scan_res = scan_and_issue_relations_witness(
        col.storage().as_ref(),
        doc_id,
        user_id,
        rel_prefix,
        scope_hash,
        as_of_tx,
    )
    .await;

    assert!(
        pre_scan_res.is_err(),
        "Witness issuance MUST fail when relation edge canary is found before deletion"
    );

    // 2. Perform deletion: remove relation key
    col.storage()
        .delete(as_of_tx, rel_key.as_bytes())
        .await
        .expect("delete rel_key");

    // Scan after deletion MUST find 0 hits and issue valid witness
    let post_scan_witness = scan_and_issue_relations_witness(
        col.storage().as_ref(),
        doc_id,
        user_id,
        rel_prefix,
        scope_hash,
        as_of_tx,
    )
    .await
    .expect("Witness issuance MUST succeed after complete relation edge removal");

    assert_eq!(
        *post_scan_witness.layer(),
        contextra_crypto::deletion_proof::DeletionLayer::Relations
    );

    // 3. Mutation probe: simulated no-op scanner reporting residual hits
    let mutation_fake_scanner = issue_relations_witness(
        doc_id,
        1, // edges_scanned
        scope_hash,
        as_of_tx,
        Some(1), // Injected fake hit
    );

    assert!(
        mutation_fake_scanner.is_err(),
        "Mutation check: faked residual hits MUST cause test assertion failure"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_text_index_real_scan_lifecycle_and_mutation() {
    let tmp = TempDir::new().expect("temp dir");
    let lsm_config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("open storage"));

    let index = InvertedIndex::new_with_language(
        storage,
        "default",
        contextra_text::Language::English,
    );
    let doc_id = DocId::from_key("canary_doc_text_1").unwrap();
    let term_bytes = vec![b"canary_term".to_vec()];
    let scope_hash = [77u8; 32];
    let as_of_tx = TxId(200);

    // 1. Insert text document
    index
        .insert(as_of_tx, doc_id, "canary_term")
        .await
        .expect("insert text");

    // Scan before deletion MUST find canary postings (hits > 0) and refuse witness
    let pre_scan_res =
        scan_and_issue_text_index_witness(&index, doc_id, &term_bytes, scope_hash, as_of_tx).await;

    assert!(
        pre_scan_res.is_err(),
        "Witness issuance MUST fail when text postings exist before deletion"
    );

    // 2. Delete text postings
    index.delete(as_of_tx, doc_id).await.expect("delete text");

    // Scan after deletion MUST find 0 hits and issue valid witness
    let post_scan_witness =
        scan_and_issue_text_index_witness(&index, doc_id, &term_bytes, scope_hash, as_of_tx)
            .await
            .expect("Witness issuance MUST succeed after text posting removal");

    assert_eq!(
        *post_scan_witness.layer(),
        contextra_crypto::deletion_proof::DeletionLayer::TextIndex
    );

    // 3. Mutation probe: simulated no-op scanner returning fake hit
    let mutation_fake_scanner = issue_text_index_witness(
        doc_id,
        &term_bytes,
        scope_hash,
        as_of_tx,
        Some(1), // Injected fake hit
    );

    assert!(
        mutation_fake_scanner.is_err(),
        "Mutation check: faked text hits MUST fail witness issuance"
    );
}

#[test]
fn test_missing_scan_claim_fails_closed() {
    let doc_id = DocId::from_key("doc_no_scan").unwrap();
    let scope_hash = [99u8; 32];
    let as_of_tx = TxId(300);

    // Without a scan claim (None), both witness functions MUST fail closed
    let rel_res = issue_relations_witness(doc_id, 10, scope_hash, as_of_tx, None);
    assert!(
        rel_res.is_err(),
        "Relations witness MUST fail closed when matching_edges_found is None"
    );

    let text_res = issue_text_index_witness(
        doc_id,
        &[b"term".to_vec()],
        scope_hash,
        as_of_tx,
        None,
    );
    assert!(
        text_res.is_err(),
        "TextIndex witness MUST fail closed when term_doc_hits is None"
    );
}
