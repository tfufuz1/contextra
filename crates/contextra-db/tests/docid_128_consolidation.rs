//! # 128-Bit-DocId-Konsolidierung
//!
//! Überprüft die konsistente Handhabung von 128-Bit-Dokumenten-IDs in Konsolidierungsphasen.
//!
use contextra_db::memory_consolidation::compute_community_hash;
#[cfg(feature = "volatile-vault")]
use contextra_db::volatile_vault::{SignalModality, VaultChunk};
use contextra_types::DocId;
#[cfg(feature = "volatile-vault")]
use contextra_types::TxId;

// ANCHOR[contextra-db:TEST-DOCID_128_CONSOLIDATION-01-TEST_VAULT_CHUNK_DOCID_TYPE]
// Testet die Funktionalität von `test_vault_chunk_docid_type`.
#[cfg(feature = "volatile-vault")]
#[test]
fn test_vault_chunk_docid_type() {
    if let Ok(doc_id) = DocId::from_key("vault_test_doc_key") {
        let chunk = VaultChunk::new(
            doc_id,
            b"sensitive payload".to_vec(),
            SignalModality::TextInput,
            TxId::new(1),
        );

        assert_eq!(chunk.id, doc_id);
        assert_eq!(chunk.content, b"sensitive payload");
    }
}

// ANCHOR[contextra-db:TEST-DOCID_128_CONSOLIDATION-02-TEST_COMPUTE_COMMUNITY_HASH_DETERMINISTIC]
// Testet die Funktionalität von `test_compute_community_hash_deterministic`.
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
