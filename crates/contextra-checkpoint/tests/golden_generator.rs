#![allow(clippy::expect_used, clippy::unwrap_used)]
use contextra_checkpoint::{CheckpointManifest, CheckpointMeta};
use contextra_types::TxId;
use std::fs;

#[test]
fn generate_golden_files() {
    let meta = CheckpointMeta {
        name: "golden_checkpoint_1".to_string(),
        collection_id: "tenant_alpha_coll".to_string(),
        seq_no: 42,
        tx_id: TxId::new(1001),
        metadata: serde_json::json!({
            "model_version": "v1.0.0",
            "description": "Baseline golden checkpoint"
        }),
        created_at: 1700000000000,
        id_width_bits: contextra_checkpoint::CURRENT_ID_WIDTH_BITS,
    };

    let manifest = CheckpointManifest::new(
        meta.clone(),
        vec!["000001.sst".to_string(), "000002.sst".to_string()],
    )
    .expect("Valid manifest creation");

    let meta_json = serde_json::to_string_pretty(&meta).expect("Serialization of meta");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).expect("Serialization of manifest");

    fs::write("tests/golden/checkpoint_meta_v1.json", meta_json).expect("write meta");
    fs::write("tests/golden/checkpoint_manifest_v1.json", manifest_json)
        .expect("write manifest");
}
