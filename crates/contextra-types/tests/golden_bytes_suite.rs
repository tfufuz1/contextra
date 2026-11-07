#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
//! Golden-Bytes-Test zur Sicherung der Formatstabilität und des Wire-Formats
//! gemäß Baukasten-Vertrag (INV-FORMAT-1).

use contextra_types::{DocId, TenantId, TxId};

#[test]
fn test_doc_id_wire_golden_bytes() {
    let doc_id = DocId::from(0x0123_4567_89AB_CDEF_u64);
    let bytes = bincode::serialize(&doc_id).expect("Serialization must succeed");

    #[cfg(not(feature = "docid-128"))]
    {
        // 64-bit Modus: genau 8 Bytes Little-Endian
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]);
        let deserialized: DocId =
            bincode::deserialize(&bytes).expect("Deserialization must succeed");
        assert_eq!(deserialized, doc_id);
    }

    #[cfg(feature = "docid-128")]
    {
        // 128-bit Modus: genau 16 Bytes Little-Endian
        assert_eq!(bytes.len(), 16);
        assert_eq!(
            bytes,
            vec![
                0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ]
        );
        let deserialized: DocId =
            bincode::deserialize(&bytes).expect("Deserialization must succeed");
        assert_eq!(deserialized, doc_id);
    }
}

#[test]
fn test_tx_id_wire_golden_bytes() {
    let tx_id = TxId::new(42);
    let bytes = bincode::serialize(&tx_id).expect("Serialization must succeed");
    assert_eq!(bytes.len(), 8);
    assert_eq!(bytes, vec![0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let deserialized: TxId = bincode::deserialize(&bytes).expect("Deserialization must succeed");
    assert_eq!(deserialized, tx_id);
}

#[test]
fn test_tenant_id_wire_golden_bytes() {
    let tenant_id = TenantId::try_new(100).expect("Valid tenant");
    let bytes = bincode::serialize(&tenant_id).expect("Serialization must succeed");
    assert_eq!(bytes.len(), 8);
    assert_eq!(bytes, vec![0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let deserialized: TenantId =
        bincode::deserialize(&bytes).expect("Deserialization must succeed");
    assert_eq!(deserialized, tenant_id);
}
