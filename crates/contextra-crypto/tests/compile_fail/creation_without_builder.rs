// PLACEHOLDER (PRE-A-01 MERGE): Proof creation without using ProofBuilder.
// After A-01 merge: Direct creation methods on DeletionProof are disabled / removed,
// requiring ProofBuilder for creation and causing this code to fail at compile time.

use contextra_crypto::{DeletionProof, DeletionScope};
use contextra_types::{TenantId, TxId};

fn main() {
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(1).unwrap(),
    };
    let proof_key = vec![0u8; 32];
    let _res = DeletionProof::create(
        scope,
        vec![b"k1".to_vec()],
        TxId(10),
        vec![],
        vec![],
        &proof_key,
    );
}
