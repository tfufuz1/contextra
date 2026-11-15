#![allow(deprecated, clippy::io_other_error)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Generator test for Golden Data (FND.1)

use acta_types::{
    CollectionRef, ActaError, DocId, DocKey, EntityId, HnswDeletionError, OperatorRef,
    SubjectRef, TenantId, TenantRef, TxId,
};
use std::fs;

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
#[ignore]
fn generate_golden_ids_data() {
    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../acta-ids/tests/golden");
    let suffix = if cfg!(feature = "docid-128") {
        "_128"
    } else {
        "_64"
    };
    fs::create_dir_all(out_dir).unwrap();

    let long_key = "X".repeat(4096);
    let sample_keys = vec![
        "",
        "a",
        "Hello, World!",
        "Mandant_Übersicht_42_🚀",
        long_key.as_str(),
    ];

    let mut from_key_results = Vec::new();
    for key in sample_keys {
        let doc_res = DocId::from_key(key);
        let doc_hex = match doc_res {
            Ok(id) => format!("OK:{:x}", id.inner()),
            Err(e) => format!("ERR:{}", e),
        };
        let ent_res = EntityId::from_key(key);
        let ent_hex = match ent_res {
            Ok(id) => format!("OK:{:x}", id.inner()),
            Err(e) => format!("ERR:{}", e),
        };
        from_key_results.push(serde_json::json!({
            "key_len": key.len(),
            "key_preview": if key.len() > 30 { &key[..30] } else { key },
            "doc_id": doc_hex,
            "entity_id": ent_hex,
        }));
    }
    fs::write(
        format!("{}/from_key_vectors{}.json", out_dir, suffix),
        serde_json::to_string_pretty(&from_key_results).unwrap(),
    )
    .unwrap();

    let doc_id = DocId::from(0x0123_4567_89AB_CDEF_u64);
    let entity_id = EntityId::from(0xFEDC_BA98_7654_3210_u64);
    let tx_id = TxId::new(42);
    let tenant_id = TenantId::try_new(100).unwrap();
    let col_ref = CollectionRef::try_new(b"coll_test_1".to_vec()).unwrap();
    let doc_key = DocKey::try_new(b"key_test_1".to_vec()).unwrap();
    let op_ref = OperatorRef::try_new(b"op_test_1".to_vec()).unwrap();
    let sub_ref = SubjectRef::try_new(b"sub_test_1".to_vec()).unwrap();
    let ten_ref = TenantRef::try_new(b"ten_test_1".to_vec()).unwrap();

    let serde_bundle = serde_json::json!({
        "doc_id": {
            "json": serde_json::to_string(&doc_id).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&doc_id).unwrap()),
        },
        "entity_id": {
            "json": serde_json::to_string(&entity_id).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&entity_id).unwrap()),
        },
        "tx_id": {
            "json": serde_json::to_string(&tx_id).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&tx_id).unwrap()),
        },
        "tenant_id": {
            "json": serde_json::to_string(&tenant_id).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&tenant_id).unwrap()),
        },
        "collection_ref": {
            "json": serde_json::to_string(&col_ref).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&col_ref).unwrap()),
        },
        "doc_key": {
            "json": serde_json::to_string(&doc_key).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&doc_key).unwrap()),
        },
        "operator_ref": {
            "json": serde_json::to_string(&op_ref).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&op_ref).unwrap()),
        },
        "subject_ref": {
            "json": serde_json::to_string(&sub_ref).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&sub_ref).unwrap()),
        },
        "tenant_ref": {
            "json": serde_json::to_string(&ten_ref).unwrap(),
            "bincode_hex": encode_hex(&bincode::serialize(&ten_ref).unwrap()),
        },
    });
    fs::write(
        format!("{}/serde_ids_refs{}.json", out_dir, suffix),
        serde_json::to_string_pretty(&serde_bundle).unwrap(),
    )
    .unwrap();

    let dummy_io = std::io::Error::new(std::io::ErrorKind::Other, "io failure");
    let dummy_json: serde_json::Error = serde_json::from_str::<String>("invalid json").unwrap_err();
    let dummy_bincode: bincode::Error = bincode::deserialize::<String>(&[0xFF]).unwrap_err();

    let errors = vec![
        ("Internal", ActaError::Internal("test msg".into())),
        (
            "InvalidInput",
            ActaError::InvalidInput("test msg".into()),
        ),
        ("NotFound", ActaError::NotFound("test msg".into())),
        (
            "PolicyViolation",
            ActaError::PolicyViolation("test msg".into()),
        ),
        ("Storage", ActaError::Storage("test msg".into())),
        ("Io", ActaError::Io(dummy_io)),
        (
            "WalCorruption",
            ActaError::wal_corruption(1024, "bad crc"),
        ),
        (
            "ChecksumMismatch",
            ActaError::checksum_mismatch("data.sst", 4),
        ),
        (
            "Transaction",
            ActaError::Transaction("test msg".into()),
        ),
        (
            "TransactionTimeout",
            ActaError::TransactionTimeout {
                tx_id: 1,
                elapsed_ms: 5000,
            },
        ),
        ("Conflict", ActaError::Conflict("test msg".into())),
        (
            "InvalidSequenceNumber",
            ActaError::InvalidSequenceNumber(123),
        ),
        ("Index", ActaError::Index("test msg".into())),
        (
            "EmbeddingDimensionMismatch",
            ActaError::EmbeddingDimensionMismatch {
                expected: 128,
                got: 256,
            },
        ),
        (
            "HnswConnectivityDegraded",
            ActaError::HnswConnectivityDegraded { deleted_ratio: 0.5 },
        ),
        ("Text", ActaError::Text("test msg".into())),
        (
            "MemoryBudgetExceeded",
            ActaError::MemoryBudgetExceeded {
                used_mb: 512,
                limit_mb: 256,
            },
        ),
        ("Sandbox", ActaError::Sandbox("test msg".into())),
        (
            "MemoryLimitExceeded",
            ActaError::MemoryLimitExceeded("limit exceeded".into()),
        ),
        (
            "SandboxTimeout",
            ActaError::SandboxTimeout("timeout".into()),
        ),
        (
            "Timeout",
            ActaError::Timeout {
                operation: "query".into(),
                timeout_ms: 1000,
            },
        ),
        (
            "Serialization",
            ActaError::Serialization("test msg".into()),
        ),
        ("Json", ActaError::Json(dummy_json)),
        ("Crypto", ActaError::Crypto("test msg".into())),
        ("CheckpointNotFound", ActaError::CheckpointNotFound),
        ("Cluster", ActaError::Cluster("test msg".into())),
        ("ParseError", ActaError::ParseError("test msg".into())),
        ("Bincode", ActaError::Bincode(dummy_bincode)),
        (
            "CapabilityUnsupported",
            ActaError::CapabilityUnsupported {
                capability: "simd".into(),
                reason: "no avx".into(),
            },
        ),
        (
            "SnapshotUnsupportedForSignal",
            ActaError::SnapshotUnsupportedForSignal("vec".into()),
        ),
        ("StaleRead", ActaError::StaleRead("test msg".into())),
        (
            "LimitExceeded",
            ActaError::LimitExceeded {
                limit: 100,
                context: "scan".into(),
            },
        ),
        (
            "ModelLoad",
            ActaError::ModelLoad {
                path: "model.bin".into(),
                reason: "corrupt".into(),
            },
        ),
        (
            "OrphanedVectorReference",
            ActaError::OrphanedVectorReference {
                doc_id: "doc1".into(),
                index_id: "idx1".into(),
            },
        ),
        ("CommitTimeout", ActaError::CommitTimeout { tx_id: 99 }),
        (
            "KvDeleteModeConfig",
            ActaError::KvDeleteModeConfig("invalid mode"),
        ),
        (
            "KvQuantization",
            ActaError::KvQuantization("overflow".into()),
        ),
        (
            "DurabilityConfig",
            ActaError::DurabilityConfig("bad durability".into()),
        ),
        ("Plugin", ActaError::Plugin("plugin failure".into())),
        (
            "PinBudgetExceeded",
            ActaError::PinBudgetExceeded("max pins".into()),
        ),
        (
            "GraphRepairFailed",
            ActaError::GraphRepairFailed(HnswDeletionError::NodeNotFound(DocId::from(1u64))),
        ),
        (
            "WalTruncationDetected",
            ActaError::WalTruncationDetected {
                expected_hmac: [1u8; 32],
                actual_hmac: [2u8; 32],
            },
        ),
        (
            "CrossDeviceLink",
            ActaError::CrossDeviceLink {
                source_path: "/a".into(),
                target_path: "/b".into(),
            },
        ),
    ];

    let mut err_map = std::collections::BTreeMap::new();
    for (name, err) in errors {
        err_map.insert(name.to_string(), err.to_string());
    }

    fs::write(
        format!("{}/acta_error_display.json", out_dir),
        serde_json::to_string_pretty(&err_map).unwrap(),
    )
    .unwrap();

    println!("Golden files for acta-ids successfully generated.");
}
