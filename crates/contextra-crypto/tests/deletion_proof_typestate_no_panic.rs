//! Test suite verifying zero-panic guarantees and proof semantics in DeletionProofBuilder.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
    VerificationKey,
};
use contextra_crypto::deletion_proof_typestate::DeletionProofBuilder;
use contextra_crypto::error::CryptoError;
use contextra_types::{DocId, TenantId, TxId};
use std::panic::catch_unwind;

fn test_proof_key() -> Vec<u8> {
    vec![0x42u8; 32]
}

#[test]
fn test_all_layers_mismatch_return_error_without_panic() {
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(10).unwrap(),
    };

    let lsm_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();
    let sst_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap();
    let hnsw_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap();
    let wal_proof = LayerCleanupProof::new_after_verified_empty(
        DeletionLayer::WalAllSegments { seq_after: 10 },
        0,
    )
    .unwrap();
    let csr_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::CsrGraph, 0).unwrap();
    let kv_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0).unwrap();
    let emb_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::EmbeddingCache, 0).unwrap();

    // 1. LSM Memtable mismatch (passing sst_proof)
    let res1 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(sst_proof.clone())
    });
    match res1 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("LsmMemtable"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on LSM layer mismatch"),
        Err(_) => panic!("LSM layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 2. SSTable mismatch (passing hnsw_proof)
    let res2 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(hnsw_proof.clone()))
    });
    match res2 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("SsTableAllLevels"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on SST layer mismatch"),
        Err(_) => panic!("SST layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 3. HNSW mismatch (passing wal_proof)
    let res3 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(sst_proof.clone()))
            .and_then(|b| b.with_hnsw_index_cleanup(wal_proof.clone()))
    });
    match res3 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("HnswIndex"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on HNSW layer mismatch"),
        Err(_) => panic!("HNSW layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 4. WAL mismatch (passing csr_proof)
    let res4 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(sst_proof.clone()))
            .and_then(|b| b.with_hnsw_index_cleanup(hnsw_proof.clone()))
            .and_then(|b| b.with_wal_all_segments_cleanup(csr_proof.clone()))
    });
    match res4 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("WalAllSegments"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on WAL layer mismatch"),
        Err(_) => panic!("WAL layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 5. CSR Graph mismatch (passing kv_proof)
    let res5 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(sst_proof.clone()))
            .and_then(|b| b.with_hnsw_index_cleanup(hnsw_proof.clone()))
            .and_then(|b| b.with_wal_all_segments_cleanup(wal_proof.clone()))
            .and_then(|b| b.with_csr_graph_cleanup(kv_proof.clone()))
    });
    match res5 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("CsrGraph"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on CSR layer mismatch"),
        Err(_) => panic!("CSR layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 6. KV Cache mismatch (passing emb_proof)
    let res6 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(sst_proof.clone()))
            .and_then(|b| b.with_hnsw_index_cleanup(hnsw_proof.clone()))
            .and_then(|b| b.with_wal_all_segments_cleanup(wal_proof.clone()))
            .and_then(|b| b.with_csr_graph_cleanup(csr_proof.clone()))
            .and_then(|b| b.with_kv_cache_segments_cleanup(emb_proof.clone()))
    });
    match res6 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("KvCacheSegments"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on KV layer mismatch"),
        Err(_) => panic!("KV layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }

    // 7. Embedding Cache mismatch (passing lsm_proof)
    let res7 = catch_unwind(|| {
        DeletionProofBuilder::new(scope.clone(), vec![], TxId(100), vec![], test_proof_key())
            .with_lsm_memtable_cleanup(lsm_proof.clone())
            .and_then(|b| b.with_sstable_all_levels_cleanup(sst_proof.clone()))
            .and_then(|b| b.with_hnsw_index_cleanup(hnsw_proof.clone()))
            .and_then(|b| b.with_wal_all_segments_cleanup(wal_proof.clone()))
            .and_then(|b| b.with_csr_graph_cleanup(csr_proof.clone()))
            .and_then(|b| b.with_kv_cache_segments_cleanup(kv_proof.clone()))
            .and_then(|b| b.with_embedding_cache_cleanup(lsm_proof.clone()))
    });
    match res7 {
        Ok(Err(CryptoError::InvalidInput(msg))) => {
            assert!(msg.contains("EmbeddingCache"), "unexpected msg: {msg}");
        }
        Ok(Ok(_)) => panic!("expected Err on Embedding layer mismatch"),
        Err(_) => panic!("Embedding layer mismatch panicked"),
        Ok(Err(other)) => panic!("expected InvalidInput, got {:?}", other),
    }
}

#[test]
fn test_golden_roundtrip_semantics_unchanged() -> contextra_types::Result<()> {
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let deleted_keys = vec![b"key1".to_vec(), b"key2".to_vec()];
    let tx_id = TxId(100);
    let excluded = vec![ExcludedScope::LlmParameterMemory];
    let receipt = Some([0xAAu8; 32]);
    let key = test_proof_key();

    let lsm_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();
    let sst_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap();
    let hnsw_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap();
    let wal_proof = LayerCleanupProof::new_after_verified_empty(
        DeletionLayer::WalAllSegments { seq_after: 50 },
        0,
    )
    .unwrap();
    let csr_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::CsrGraph, 0).unwrap();
    let kv_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0).unwrap();
    let emb_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::EmbeddingCache, 0).unwrap();

    let graph_repair = vec![contextra_crypto::deletion_proof::GraphRepairAttestation {
        doc_id: DocId(42),
        verified_no_ghost_pointers: true,
        attested_at: 0,
    }];

    let built_proof =
        DeletionProofBuilder::new(scope, deleted_keys, tx_id, excluded, key.clone())
            .with_wal_chain_receipt(receipt)
            .with_graph_repair(graph_repair)
            .with_lsm_memtable_cleanup(lsm_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_sstable_all_levels_cleanup(sst_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_hnsw_index_cleanup(hnsw_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_wal_all_segments_cleanup(wal_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_csr_graph_cleanup(csr_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_kv_cache_segments_cleanup(kv_proof)
            .map_err(contextra_types::ContextraError::from)?
            .with_embedding_cache_cleanup(emb_proof)
            .map_err(contextra_types::ContextraError::from)?
            .finish()?;

    assert_eq!(built_proof.signature_version, 4);

    let sk = ed25519_dalek::SigningKey::from_bytes(key[..32].try_into().unwrap());
    let vk = sk.verifying_key();
    assert!(built_proof.verify(VerificationKey::Ed25519(&vk))?);

    // JSON serialization roundtrip check
    let serialized = serde_json::to_vec(&built_proof).expect("serde_json serialization failed");
    let deserialized: DeletionProof =
        serde_json::from_slice(&serialized).expect("serde_json deserialization failed");

    assert_eq!(built_proof, deserialized);
    assert!(deserialized.verify(VerificationKey::Ed25519(&vk))?);

    Ok(())
}
