#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_types::{DocId, TenantId, TxId};
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof,
};

#[derive(Arbitrary, Debug)]
pub struct TamperMutation {
    pub offset: usize,
    pub bit_mask: u8,
}

#[derive(Arbitrary, Debug)]
pub struct DeletionProofTamperInput {
    pub proof_key: Vec<u8>,
    pub mutations: Vec<TamperMutation>,
    pub raw_bytes: Vec<u8>,
    pub arbitrary_version: u8,
    pub random_signature_64: [u8; 64],
}

fuzz_target!(|input: DeletionProofTamperInput| {
    let proof_key = if input.proof_key.is_empty() {
        b"fuzz_default_proof_key_32bytes!!".to_vec()
    } else {
        input.proof_key
    };

    // 1. Fuzz arbitrary raw bytes deserialization directly
    if let Ok(proof) = bincode::deserialize::<DeletionProof>(&input.raw_bytes) {
        let _ = proof.verify(&proof_key);
        let _ = proof.export_for_audit();
    }

    // 2. Construct valid proof, apply mutations, and verify
    let tenant_id = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let scope = DeletionScope::Document {
        doc_id: DocId::new(500),
        tenant_id,
    };
    let layer_proof = match Ok(LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)) {
        Ok(p) => p,
        Err(_) => return,
    };

    let valid_proof = match DeletionProof::create(
        scope,
        vec![b"key_a".to_vec(), b"key_b".to_vec()],
        TxId::new(42),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        &proof_key,
    ) {
        Ok(p) => p,
        Err(_) => return,
    };

    if let Ok(mut serialized) = bincode::serialize(&valid_proof) {
        if !serialized.is_empty() {
            for m in input.mutations.iter().take(10) {
                let idx = m.offset % serialized.len();
                serialized[idx] ^= m.bit_mask;
            }

            match bincode::deserialize::<DeletionProof>(&serialized) {
                Ok(tampered_proof) => {
                    // Tampered proof must never panic during verify
                    let is_valid = tampered_proof.verify(&proof_key).unwrap_or(false);
                    let _ = is_valid;
                }
                Err(_) => {
                    // Deserialization failure is expected for corrupted bytes
                }
            }
        }
    }

    // --- ADDITIONAL FUZZ STRATEGIES (AUFGABE D) ---

    // 1. Empty proof_key verification must never panic
    let _ = valid_proof.verify(&[]);

    // 2. signature_version manipulation with arbitrary u8 (incl. > 3)
    let mut version_manipulated = valid_proof.clone();
    version_manipulated.signature_version = input.arbitrary_version;
    let _ = version_manipulated.verify(&proof_key);

    let keypair = DeletionProofKeyPair::generate();
    let _ = version_manipulated.verify(&keypair.verifying_key);

    // 3. Random 64-byte signature candidate against v3 proof path
    let v3_layer_proof = match Ok(LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)) {
        Ok(p) => p,
        Err(_) => return,
    };
    if let Ok(mut v3_proof) = DeletionProof::create_v3(
        DeletionScope::Document {
            doc_id: DocId::new(501),
            tenant_id,
        },
        vec![b"key_x".to_vec()],
        TxId::new(43),
        vec![v3_layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    ) {
        v3_proof.signature = input.random_signature_64.to_vec();
        let is_valid = v3_proof.verify(&keypair.verifying_key).unwrap_or(false);
        let _ = is_valid;
    }

    // 4. V5 Proof issue & verify fuzzing
    use contextra_testkit::mint_witness_for_tests;
    use contextra_ports::scanner::ScanTechnique;
    use contextra_ports::SystemClock;
    use contextra_types::{RequestContext, SagaRecord};

    let clock = SystemClock::new();
    let ctx = RequestContext::new(tenant_id, TxId::new(100));
    let saga = SagaRecord::new([0xBB; 32]);
    let witnesses = vec![
        mint_witness_for_tests("LsmMemtable", 10, 0, ScanTechnique::Structural),
        mint_witness_for_tests("SsTableAllLevels", 50, 0, ScanTechnique::Structural),
        mint_witness_for_tests("WalAllSegments", 100, 0, ScanTechnique::Structural),
    ];

    if let Ok(mut v5_proof) = DeletionProof::issue(
        &ctx,
        &saga,
        &witnesses,
        &[ExcludedScope::LlmParameterMemory],
        contextra_crypto::deletion_proof::CustodyLevel::Local,
        keypair.signing_key(),
        &clock,
    ) {
        let _ = v5_proof.verify(&keypair.verifying_key);
        let _ = v5_proof.verify_external(&keypair.verifying_key);
        let _ = v5_proof.export_for_audit();

        v5_proof.signature = input.random_signature_64.to_vec();
        let _ = v5_proof.verify(&keypair.verifying_key);
        let _ = v5_proof.verify_external(&keypair.verifying_key);
    }
});
