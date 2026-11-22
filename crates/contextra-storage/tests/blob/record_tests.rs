#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::blob::record::*;

#[test]
fn test_blob_header_encode_decode_roundtrip() {
    let header = BlobHeader::new(42, 1, 3);
    let bytes = header.encode();
    assert_eq!(bytes.len(), BLOB_HEADER_LEN);
    assert_eq!(&bytes[..4], BLOB_MAGIC);

    let decoded = BlobHeader::decode(&bytes).expect("decode header succeeds");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.flags, 0);
    assert_eq!(decoded.blob_no, 42);
    assert_eq!(decoded.generation, 1);
    assert_eq!(decoded.space, 3);
    assert_eq!(decoded.reserved, [0u8; 22]);
}

#[test]
fn test_blob_header_bad_magic() {
    let header = BlobHeader::new(1, 1, 0);
    let mut bytes = header.encode();
    bytes[0] = b'X';
    let err = BlobHeader::decode(&bytes).unwrap_err();
    assert_eq!(err, RecordError::BadMagic);
}

#[test]
fn test_blob_header_checksum_mismatch() {
    let header = BlobHeader::new(1, 1, 0);
    let mut bytes = header.encode();
    bytes[10] ^= 0xFF; // Bitflip in blob_no
    let err = BlobHeader::decode(&bytes).unwrap_err();
    assert!(matches!(err, RecordError::ChecksumMismatch { .. }));
}

#[test]
fn test_blob_footer_encode_decode_roundtrip() {
    let footer = BlobFooter::new(128, 40960);
    let bytes = footer.encode();
    assert_eq!(bytes.len(), BLOB_FOOTER_LEN);
    assert_eq!(&bytes[..4], BLOB_MAGIC);

    let decoded = BlobFooter::decode(&bytes).expect("decode footer succeeds");
    assert_eq!(decoded.record_count, 128);
    assert_eq!(decoded.bytes, 40960);
}

#[test]
fn test_blob_footer_corrupt() {
    let footer = BlobFooter::new(10, 500);
    let mut bytes = footer.encode();
    bytes[5] ^= 1;
    assert!(BlobFooter::decode(&bytes).is_err());
}

#[test]
fn test_blob_ref_encode_decode_roundtrip() {
    let b_ref = BlobRef {
        blob_no: 0x71,
        offset: 4096,
        len: 1000,
        crc32c: 0xDEADBEEF,
        domain: 42,
    };
    let bytes = b_ref.encode();
    assert_eq!(bytes.len(), BLOB_REF_LEN);

    let decoded = BlobRef::decode(&bytes).expect("decode blobref succeeds");
    assert_eq!(decoded, b_ref);
}

#[test]
fn test_blob_record_content_roundtrip() {
    let rec = BlobRecord::new_body(7, 100, b"user:key:123".to_vec(), b"massive_body_payload".to_vec());
    let content = rec.encode_content();

    let decoded = BlobRecord::decode_content(&content).expect("decode content succeeds");
    assert_eq!(decoded.kind, RECORD_KIND_BODY);
    assert_eq!(decoded.domain, 7);
    assert_eq!(decoded.stamp, 100);
    assert_eq!(decoded.key, b"user:key:123");
    assert_eq!(decoded.body, b"massive_body_payload");
}
