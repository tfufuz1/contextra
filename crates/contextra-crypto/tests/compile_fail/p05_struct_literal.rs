use contextra_crypto::DeletionProof;

fn main() {
    // Attempting to construct DeletionProof via direct struct literal from an external module/crate
    let _proof = DeletionProof {
        signature_version: 4,
        scope: contextra_crypto::DeletionScope::Tenant {
            tenant_id: contextra_types::TenantId::try_new(1).unwrap(),
        },
        deleted_keys_hash: [0u8; 32],
        deleted_after_tx: contextra_types::TxId(1),
        timestamp: 0,
        signature: vec![],
        covered_layers: vec![],
        uncovered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        integrity_warning: None,
        _private: (),
    };
}
