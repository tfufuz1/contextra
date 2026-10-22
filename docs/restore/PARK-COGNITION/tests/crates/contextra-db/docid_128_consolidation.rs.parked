use contextra_db::memory_consolidation::compute_community_hash;
use contextra_types::DocId;

#[test]
fn test_compute_community_hash_deterministic() {
    if let (Ok(doc_id_1), Ok(doc_id_2)) = (
        DocId::from_key("community_doc_1"),
        DocId::from_key("community_doc_2"),
    ) {
        let hash_1 = compute_community_hash(&[doc_id_1, doc_id_2]);
        let hash_2 = compute_community_hash(&[doc_id_2, doc_id_1]);

        assert_eq!(
            hash_1, hash_2,
            "Community hash must be order-independent due to internal sorting"
        );
    }
}
