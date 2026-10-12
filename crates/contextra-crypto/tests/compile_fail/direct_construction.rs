// PLACEHOLDER (PRE-A-01 MERGE): Direct struct instantiation of DeletionProof.
// After A-01 merge: DeletionProof struct fields / constructors are private / restricted,
// causing direct instantiation to fail at compile time.

use contextra_crypto::{DeletionProof, DeletionScope};
use contextra_types::{TenantId, TxId};

fn main() {
    let _proof = DeletionProof {
        signature_version: 3,
        scope: DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        },
        deleted_keys_hash: [0u8; 32],
        deleted_after_tx: TxId(100),
        timestamp: 1700000000,
        signature: vec![0u8; 64],
        covered_layers: vec![],
        uncovered_layers: vec![],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        integrity_warning: None,
    };
}
