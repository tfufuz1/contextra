use contextra_types::{DocId, DocIdRaw};

#[test]
fn test_docid_width_bytes_constant() {
    #[cfg(feature = "docid-128")]
    {
        assert_eq!(DocId::BYTES, 16);
        assert_eq!(std::mem::size_of::<DocId>(), 16);
        assert_eq!(std::mem::size_of::<DocIdRaw>(), 16);
    }
    #[cfg(not(feature = "docid-128"))]
    {
        assert_eq!(DocId::BYTES, 8);
        assert_eq!(std::mem::size_of::<DocId>(), 8);
        assert_eq!(std::mem::size_of::<DocIdRaw>(), 8);
    }
}

#[test]
fn test_docid_le_bytes_roundtrip() {
    let id_val: u64 = 0x1234_5678_9ABC_DEF0;
    let doc_id = DocId::from(id_val);

    let bytes = doc_id.to_le_bytes();
    assert_eq!(bytes.len(), DocId::BYTES);

    let recovered = DocId::from_le_bytes(bytes);
    assert_eq!(doc_id, recovered);

    #[cfg(not(feature = "docid-128"))]
    {
        assert_eq!(bytes, id_val.to_le_bytes());
    }
    #[cfg(feature = "docid-128")]
    {
        let expected_u128 = u128::from(id_val);
        assert_eq!(bytes, expected_u128.to_le_bytes());
    }
}

#[test]
fn test_docid_raw_and_from_u64_widening() {
    let input: u64 = 42;
    let doc_id = DocId::from(input);

    #[cfg(feature = "docid-128")]
    {
        assert_eq!(doc_id.raw(), 42u128);
        assert_eq!(doc_id.inner(), 42u128);
        assert_eq!(doc_id.as_u64(), 42u64);
    }
    #[cfg(not(feature = "docid-128"))]
    {
        assert_eq!(doc_id.raw(), 42u64);
        assert_eq!(doc_id.inner(), 42u64);
        assert_eq!(doc_id.as_u64(), 42u64);
    }
}

#[test]
fn test_docid_max_min_constants() {
    assert_eq!(DocId::MIN.as_u64(), 0);

    #[cfg(feature = "docid-128")]
    {
        assert_eq!(DocId::MAX.raw(), u128::MAX);
    }
    #[cfg(not(feature = "docid-128"))]
    {
        assert_eq!(DocId::MAX.raw(), u64::MAX);
    }
}

#[test]
fn test_docid_from_key_deterministic() {
    let key = "document-12345";
    let id1 = DocId::from_key(key).unwrap();
    let id2 = DocId::from_key(key).unwrap();

    assert_eq!(id1, id2);
    assert_ne!(id1, DocId::MIN);

    let empty_res = DocId::from_key("");
    assert!(empty_res.is_err());
}
