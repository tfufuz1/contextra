use contextra_crypto::DeletionProof;
use contextra_crypto::deletion_proof::UnverifiedProof;

fn process_verified_proof(_proof: &DeletionProof) {}

fn main() {
    let raw_json = r#"{
        "signature_version": 3,
        "scope": {"Tenant": {"tenant_id": 1}},
        "deleted_keys_hash": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        "deleted_after_tx": 10,
        "signature": [0],
        "covered_layers": [],
        "excluded_scopes": []
    }"#;

    let raw_proof: DeletionProof = serde_json::from_str(raw_json).unwrap();
    let unverified: UnverifiedProof = UnverifiedProof::new(raw_proof);

    // Attempting to pass UnverifiedProof where a verified DeletionProof is required
    process_verified_proof(&unverified);
}
