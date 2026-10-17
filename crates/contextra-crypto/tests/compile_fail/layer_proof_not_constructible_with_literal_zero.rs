use contextra_crypto::deletion_proof::{DeletionLayer, LayerCleanupProof};

fn main() {
    let _ = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0);
}
