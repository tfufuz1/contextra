use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::ProofBuilder;
use contextra_types::{DocId, TenantId};
use std::fs;

#[test]
fn generate_golden_files() {
    let seed = [0x42u8; 32];
    let signer = ProofKeyPairSigner::from_secret_bytes(seed);
    let scope = contextra_crypto::DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    let proof = ProofBuilder::new(scope, "db-golden-instance-01")
        .with_stage(2)
        .with_storage_scope("lsm+hnsw+kv")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0xAA; 32])
        .with_footprint_hash([0xBB; 32])
        .with_observed_empty(true)
        .with_timestamps(1700000000000, 1)
        .finish(&signer)
        .expect("finish v4 proof");

    let bin_bytes = proof.construct_canonical_payload();
    let json_string = serde_json::to_string_pretty(&proof).expect("to_json");

    fs::create_dir_all("crates/contextra-crypto/tests/golden").expect("create dir");
    fs::write(
        "crates/contextra-crypto/tests/golden/p05_v4_valid.bin",
        &bin_bytes,
    )
    .expect("write bin");
    fs::write(
        "crates/contextra-crypto/tests/golden/p05_v4_valid.json",
        &json_string,
    )
    .expect("write json");

    println!(
        "Successfully generated golden vectors! TLV size: {} bytes",
        bin_bytes.len()
    );
}
