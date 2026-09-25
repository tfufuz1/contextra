#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_types::{CollectionId, DocId, TenantId, TxId};
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof,
};

#[derive(Arbitrary, Debug)]
pub struct VerifyExternalFuzzInput {
    /// Arbitrary raw bytes to attempt bincode deserialization into DeletionProof
    pub raw_proof_bytes: Vec<u8>,
    /// Arbitrary raw public key / HMAC key byte material (0..N bytes)
    pub public_key_bytes: Vec<u8>,
    /// Fields to construct synthetic DeletionProof instances
    pub signature_version: u8,
    pub scope_type: u8,
    pub doc_id: u64,
    pub tenant_id: u64,
    pub collection_id: u64,
    pub deleted_keys_hash: [u8; 32],
    pub deleted_after_tx: u64,
    pub signature: Vec<u8>,
    pub layers_mask: u8,
    pub excluded_mask: u8,
    pub wal_chain_receipt: Option<[u8; 32]>,
}

fuzz_target!(|input: VerifyExternalFuzzInput| {
    // Strategy 1: Fuzz bincode deserialization + verify_external with arbitrary key bytes
    if let Ok(deserialized_proof) = bincode::deserialize::<DeletionProof>(&input.raw_proof_bytes) {
        let _ = deserialized_proof.verify_external(&input.public_key_bytes);
    }

    // Strategy 2: Construct synthetic DeletionProof with arbitrary/corrupt fields
    let tenant_id = match TenantId::try_new(input.tenant_id.max(1)) {
        Ok(t) => t,
        Err(_) => return,
    };

    let scope = match input.scope_type % 3 {
        0 => DeletionScope::Document {
            doc_id: DocId::new(input.doc_id),
            tenant_id,
        },
        1 => DeletionScope::Collection {
            collection_id: CollectionId::new(input.collection_id),
            tenant_id,
        },
        _ => DeletionScope::Tenant { tenant_id },
    };

    let mut covered_layers = Vec::new();
    if input.layers_mask & 0x01 != 0 {
        covered_layers.push(DeletionLayer::LsmMemtable);
    }
    if input.layers_mask & 0x02 != 0 {
        covered_layers.push(DeletionLayer::SsTableAllLevels);
    }
    if input.layers_mask & 0x04 != 0 {
        covered_layers.push(DeletionLayer::HnswIndex);
    }
    if input.layers_mask & 0x08 != 0 {
        covered_layers.push(DeletionLayer::WalAllSegments { seq_after: 42 });
    }

    let mut excluded_scopes = Vec::new();
    if input.excluded_mask & 0x01 != 0 {
        excluded_scopes.push(ExcludedScope::ConsolidatedAndDistilled);
    }
    if input.excluded_mask & 0x02 != 0 {
        excluded_scopes.push(ExcludedScope::LlmParameterMemory);
    }

    let synthetic_proof = DeletionProof {
        signature_version: input.signature_version,
        scope,
        deleted_keys_hash: input.deleted_keys_hash,
        deleted_after_tx: TxId::new(input.deleted_after_tx),
        signature: input.signature,
        covered_layers,
        excluded_scopes,
        wal_chain_receipt: input.wal_chain_receipt,
        integrity_warning: None,
    };

    let _ = synthetic_proof.verify_external(&input.public_key_bytes);

    // Strategy 3: Construct valid v3 (Ed25519) and v2 (HMAC) proofs, then verify against arbitrary/mutated key bytes
    let keypair = DeletionProofKeyPair::generate();
    let layer_proof = match LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0) {
        Ok(p) => p,
        Err(_) => return,
    };

    if let Ok(valid_v3) = DeletionProof::create_v3(
        DeletionScope::Document {
            doc_id: DocId::new(100),
            tenant_id,
        },
        vec![b"fuzz_key".to_vec()],
        TxId::new(1),
        vec![layer_proof.clone()],
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    ) {
        // Verification with valid 32-byte key
        let _ = valid_v3.verify_external(&keypair.verifying_key_bytes());
        // Verification with arbitrary public key bytes (short, long, invalid Curve25519 points)
        let _ = valid_v3.verify_external(&input.public_key_bytes);
    }

    if let Ok(valid_v2) = DeletionProof::create(
        DeletionScope::Document {
            doc_id: DocId::new(101),
            tenant_id,
        },
        vec![b"fuzz_key_2".to_vec()],
        TxId::new(2),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        &input.public_key_bytes,
    ) {
        let _ = valid_v2.verify_external(&input.public_key_bytes);
    }
});
