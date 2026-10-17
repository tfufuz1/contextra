#![cfg(not(loom))]

use contextra_engine::collection::{StoredDocument, STORED_DOCUMENT_MAGIC, STORED_DOCUMENT_VERSION_V2};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_stored_document_v2_roundtrip(
        id in "[a-zA-Z0-9_-]{1,64}",
        embedding in proptest::collection::vec(
            any::<f32>().prop_filter("finite float", |f| f.is_finite()),
            0..64
        ),
        has_meta in any::<bool>(),
        meta_num in any::<i64>(),
    ) {
        let metadata = if has_meta {
            Some(serde_json::json!({ "num": meta_num, "tag": "proptest" }))
        } else {
            None
        };

        let doc = StoredDocument {
            id: id.clone(),
            embedding: embedding.clone(),
            metadata: metadata.clone(),
        };

        let encoded = doc.encode_v2().expect("encode_v2 must succeed for finite floats");
        let decoded = StoredDocument::from_slice(&encoded).expect("from_slice must decode valid v2");

        prop_assert_eq!(decoded.id, id);
        prop_assert_eq!(decoded.embedding, embedding);
        prop_assert_eq!(decoded.metadata, metadata);
    }
}

#[test]
fn test_reject_unsupported_version_returns_error() {
    let mut doc_bytes = StoredDocument {
        id: "test".to_string(),
        embedding: vec![1.0, 2.0],
        metadata: None,
    }
    .encode_v2()
    .expect("encode");

    // Change version byte from 2 to 99
    doc_bytes[4] = 99;

    let res = StoredDocument::from_slice(&doc_bytes);
    assert!(res.is_err(), "Must reject unsupported version byte 99");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unsupported StoredDocument binary version"),
        "Expected unsupported version error, got: {err_msg}"
    );
}

#[test]
fn test_reject_truncated_header_returns_error() {
    for len in 0..5 {
        let truncated = &STORED_DOCUMENT_MAGIC[..len.min(4)];
        let res = StoredDocument::from_slice(truncated);
        assert!(res.is_err(), "Truncated header of length {len} must return Err");
    }
}

#[test]
fn test_reject_truncated_body_returns_error() {
    let doc_bytes = StoredDocument {
        id: "trunc_test".to_string(),
        embedding: vec![1.0, 2.0, 3.0, 4.0],
        metadata: Some(serde_json::json!({"key": "value"})),
    }
    .encode_v2()
    .expect("encode");

    for len in 5..doc_bytes.len() - 1 {
        let res = StoredDocument::from_slice(&doc_bytes[..len]);
        assert!(
            res.is_err(),
            "Truncated payload of length {len}/{} must return Err",
            doc_bytes.len()
        );
    }
}

#[test]
fn test_reject_trailing_bytes_returns_error() {
    let mut doc_bytes = StoredDocument {
        id: "trailing_test".to_string(),
        embedding: vec![1.0],
        metadata: None,
    }
    .encode_v2()
    .expect("encode");

    doc_bytes.push(0xFF);

    let res = StoredDocument::from_slice(&doc_bytes);
    assert!(res.is_err(), "Payload with trailing bytes must return Err");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Trailing bytes"),
        "Expected trailing bytes error, got: {err_msg}"
    );
}

#[test]
fn test_reject_non_finite_floats_returns_error() {
    // Non-finite float during encoding
    let doc_nan = StoredDocument {
        id: "nan_doc".to_string(),
        embedding: vec![1.0, f32::NAN],
        metadata: None,
    };
    assert!(doc_nan.encode_v2().is_err(), "encode_v2 must reject NaN");

    let doc_inf = StoredDocument {
        id: "inf_doc".to_string(),
        embedding: vec![f32::INFINITY, 0.0],
        metadata: None,
    };
    assert!(doc_inf.encode_v2().is_err(), "encode_v2 must reject Infinity");

    // Manually constructed v2 binary payload containing NaN f32 bits
    let mut raw_v2 = Vec::new();
    raw_v2.extend_from_slice(&STORED_DOCUMENT_MAGIC);
    raw_v2.push(STORED_DOCUMENT_VERSION_V2);
    // id_len = 3 ("nan")
    raw_v2.extend_from_slice(&3u32.to_be_bytes());
    raw_v2.extend_from_slice(b"nan");
    // emb_len = 1
    raw_v2.extend_from_slice(&1u32.to_be_bytes());
    // f32 NaN LE bytes
    raw_v2.extend_from_slice(&f32::NAN.to_le_bytes());
    // meta_len = 0
    raw_v2.extend_from_slice(&0u32.to_be_bytes());

    let res = StoredDocument::from_slice(&raw_v2);
    assert!(res.is_err(), "from_slice must reject decoded NaN float in v2 binary payload");
}

#[test]
fn test_reject_invalid_utf8_id_returns_error() {
    let mut raw_v2 = Vec::new();
    raw_v2.extend_from_slice(&STORED_DOCUMENT_MAGIC);
    raw_v2.push(STORED_DOCUMENT_VERSION_V2);
    // id_len = 4
    raw_v2.extend_from_slice(&4u32.to_be_bytes());
    // invalid UTF-8 bytes
    raw_v2.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]);
    // emb_len = 0
    raw_v2.extend_from_slice(&0u32.to_be_bytes());
    // meta_len = 0
    raw_v2.extend_from_slice(&0u32.to_be_bytes());

    let res = StoredDocument::from_slice(&raw_v2);
    assert!(res.is_err(), "from_slice must reject invalid UTF-8 in id");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid UTF-8 in StoredDocument v2 id"),
        "Expected invalid UTF-8 error, got: {err_msg}"
    );
}

#[test]
fn test_reject_malformed_json_v1_returns_error() {
    let bad_json = b"{ \"id\": \"bad\", \"embedding\": [1.0, 2.0], invalid }";
    let res = StoredDocument::from_slice(bad_json);
    assert!(res.is_err(), "Malformed JSON v1 must return Err");
}
