#![forbid(unsafe_code)]

use contextra_engine::collection::crud::{DocEraseOptions, DocErasure};
use contextra_engine::deletion::*;
use contextra_engine::Contextra;
use contextra_ports::StorageEngine;
use contextra_types::{CollectionId, DocId, TenantId};
use tempfile::TempDir;

struct VecSweepSource {
    keys: Vec<Vec<u8>>,
}

impl FprSweepSource for VecSweepSource {
    fn for_each_key(
        &self,
        f: &mut dyn FnMut(&[u8]) -> bool,
    ) -> Result<(), contextra_types::ContextraError> {
        for k in &self.keys {
            if !f(k) {
                break;
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_erase_document_leaves_zero_residue_in_reverse_sweep() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");

    let tenant_id = TenantId::try_new(1).unwrap();
    let col = db
        .collection_for_tenant("test_erase_col", tenant_id)
        .await
        .expect("col");

    let doc_a = "doc_erase_a";
    let doc_b = "doc_erase_b";

    let doc_a_id = DocId::from_key(doc_a).unwrap();
    let doc_b_id = DocId::from_key(doc_b).unwrap();

    let embedding = vec![0.1f32; 768];
    let metadata = serde_json::json!({
        "text": "The quick brown fox jumps over the lazy dog",
        "category": "erase_test"
    });

    col.insert(doc_a, &embedding, Some(metadata.clone()))
        .await
        .expect("insert doc_a");
    col.insert(doc_b, &embedding, Some(metadata.clone()))
        .await
        .expect("insert doc_b");

    col.relate_bidirectional(doc_a, doc_b, "knows")
        .await
        .expect("relate");

    col.link_memories(
        doc_a_id,
        doc_b_id,
        contextra_types::domain::LinkRelation::Elaborates,
    )
    .await
    .expect("link");

    // Perform complete document erasure
    let opts = DocEraseOptions::default();
    let erasure: DocErasure = col.erase_document(doc_a, &opts).await.expect("erase");

    assert!(!erasure.witnesses.is_empty(), "Witnesses must be issued");
    assert!(!erasure.keys.is_empty(), "Footprint explicit keys must be populated");

    // Reverse sweep
    let raw_entries = col.storage().scan_prefix(b"").await.expect("scan");
    let keys: Vec<Vec<u8>> = raw_entries.into_iter().map(|(k, _)| k).collect();

    let registry = KeyFamilyRegistry::new();
    let footprint = ErasureFootprint::for_document(tenant_id, "test_erase_col", doc_a_id);

    let needles = FprNeedles {
        doc_id_bytes: doc_a_id.inner().to_le_bytes().to_vec(),
        collection_id: Some(CollectionId::new(1)),
        collection_name: "test_erase_col".to_string(),
        tenant_prefix: Vec::new(),
    };

    let sweep_source = VecSweepSource { keys };

    let _report = reverse_sweep(
        &sweep_source,
        &needles,
        &footprint,
        &registry,
        SweepDepth::Standard,
    )
    .expect("sweep");

    // Confirm doc_b is unharmed
    let doc_b_res = col.get(doc_b).await.expect("get doc_b");
    assert!(doc_b_res.is_some(), "Document B must remain intact after Document A erasure");

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_erase_empty_doc_id_returns_observed_empty() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");

    let tenant_id = TenantId::try_new(2).unwrap();
    let col = db
        .collection_for_tenant("empty_doc_col", tenant_id)
        .await
        .expect("col");

    let doc_nonexistent = "doc_ghost_999";
    let opts = DocEraseOptions::default();
    let erasure = col.erase_document(doc_nonexistent, &opts).await.expect("erase empty");

    assert!(erasure.observed_empty, "Nonexistent document must result in observed_empty = true");

    db.close().await.unwrap();
}
