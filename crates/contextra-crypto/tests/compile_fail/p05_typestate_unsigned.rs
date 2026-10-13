use contextra_crypto::deletion_proof::{DeletionLayer, DeletionScope, LayerCleanupProof};
use contextra_crypto::deletion_proof_typestate::DeletionProofBuilder;
use contextra_types::{DocId, TenantId, TxId};

fn main() {
    let lsm_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();

    let builder = DeletionProofBuilder::new(
        DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(10).unwrap(),
        },
        vec![],
        TxId(100),
        vec![],
        vec![0u8; 32],
    )
    .with_lsm_memtable_cleanup(lsm_proof);

    // Attempting to call finish() before all 7 layers are attested
    let _proof = builder.finish();
}
