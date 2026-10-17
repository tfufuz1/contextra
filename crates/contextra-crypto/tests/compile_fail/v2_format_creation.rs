// PLACEHOLDER (PRE-A-01 MERGE): Proof creation using legacy v2 format functions.
// After A-01 merge: Creation of version 2 proofs (create_v2 / create_v2_with_wal_receipt) is disabled / removed,
// causing this code to fail at compile time.

use contextra_crypto::{DeletionProof, DeletionScope};
use contextra_types::{TenantId, TxId};

fn main() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let proof_key = vec![0u8; 32];
    let _res = DeletionProof::create_v2(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        &proof_key,
    );
}
