#![cfg(not(loom))]

use contextra_engine::collection::{
    StoredDocument, STORED_DOCUMENT_MAGIC, STORED_DOCUMENT_VERSION_V2,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn roundtrip_v2_binary_finite_floats(
        id in "[a-zA-Z0-9_\\-]{1,64}",
        embedding in prop::collection::vec(-1000.0f32..1000.0f32, 1..64),
        meta_str in prop::option::of("[a-zA-Z0-9_]{1,32}")
    ) {
        let metadata = meta_str.map(|s| serde_json::json!({ "tag": s }));
        let doc = StoredDocument {
            id,
            embedding,
            metadata,
        };

        let encoded = doc.encode_v2().expect("encode_v2 must succeed for finite floats");

        // Verify binary prefix header
        prop_assert_eq!(&encoded[0..4], STORED_DOCUMENT_MAGIC);
        prop_assert_eq!(encoded[4], STORED_DOCUMENT_VERSION_V2);

        // Round-trip decode via from_slice
        let decoded = StoredDocument::from_slice(&encoded).expect("from_slice must decode valid v2 payload");
        prop_assert_eq!(&doc, &decoded);

        // Versioned decode check
        let (decoded_ver, ver) = StoredDocument::from_slice_versioned(&encoded).expect("from_slice_versioned must succeed");
        prop_assert_eq!(ver, 2);
        prop_assert_eq!(&doc, &decoded_ver);
    }
}

#[test]
fn reject_invalid_version_and_malformed() {
    // 1. Invalid version byte (0x03 instead of 0x02)
    let mut bad_ver = Vec::new();
    bad_ver.extend_from_slice(STORED_DOCUMENT_MAGIC);
    bad_ver.push(0x03);
    bad_ver.extend_from_slice(&1u32.to_be_bytes()); // ID len
    bad_ver.extend_from_slice(b"a");
    bad_ver.extend_from_slice(&0u32.to_be_bytes()); // vec count
    bad_ver.push(0); // meta flag

    let res = StoredDocument::from_slice(&bad_ver);
    assert!(
        res.is_err(),
        "from_slice must reject unsupported version byte 0x03"
    );

    // 2. Truncated payloads
    for len in 0..5 {
        let truncated = &STORED_DOCUMENT_MAGIC[..len.min(4)];
        let res = StoredDocument::from_slice(truncated);
        assert!(
            res.is_err(),
            "Truncated header of length {len} must return Err"
        );
    }

    // 3. Trailing junk bytes
    let valid_doc = StoredDocument {
        id: "test".to_string(),
        embedding: vec![1.0, 2.0],
        metadata: None,
    };
    let mut with_trailing = valid_doc.encode_v2().unwrap();
    with_trailing.extend_from_slice(b"EXTRA_JUNK");

    let res = StoredDocument::from_slice(&with_trailing);
    assert!(
        res.is_err(),
        "from_slice must reject payload with trailing bytes"
    );
}

#[test]
fn reject_non_finite_floats() {
    let doc_nan = StoredDocument {
        id: "doc_nan".to_string(),
        embedding: vec![f32::NAN, 1.0],
        metadata: None,
    };
    assert!(
        doc_nan.encode_v2().is_err(),
        "encode_v2 must reject NaN"
    );

    let doc_inf = StoredDocument {
        id: "doc_inf".to_string(),
        embedding: vec![f32::INFINITY, 0.0],
        metadata: None,
    };
    assert!(
        doc_inf.encode_v2().is_err(),
        "encode_v2 must reject Infinity"
    );

    // Manually constructed v2 binary payload containing NaN f32 bits
    let mut raw_v2 = Vec::new();
    raw_v2.extend_from_slice(STORED_DOCUMENT_MAGIC);
    raw_v2.push(STORED_DOCUMENT_VERSION_V2);
    raw_v2.extend_from_slice(&4u32.to_be_bytes()); // id len
    raw_v2.extend_from_slice(b"test");
    raw_v2.extend_from_slice(&1u32.to_be_bytes()); // embedding count 1
    raw_v2.extend_from_slice(&f32::NAN.to_le_bytes()); // NaN float
    raw_v2.push(0); // meta flag = None

    let res = StoredDocument::from_slice(&raw_v2);
    assert!(
        res.is_err(),
        "from_slice must reject decoded NaN float in v2 binary payload"
    );
}

#[test]
fn decode_v1_json_fallback() {
    let v1_json = r#"{"id":"v1_doc","embedding":[0.1,0.2,0.3],"metadata":{"key":"val"}}"#;
    let (decoded, ver) =
        StoredDocument::from_slice_versioned(v1_json.as_bytes()).expect("v1 JSON must decode");

    assert_eq!(ver, 1, "Format version for JSON must be 1");
    assert_eq!(decoded.id, "v1_doc");
    assert_eq!(decoded.embedding, vec![0.1, 0.2, 0.3]);
    assert_eq!(
        decoded.metadata,
        Some(serde_json::json!({"key": "val"}))
    );
}
