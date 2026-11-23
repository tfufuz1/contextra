//! Redaktionstests (SP-PORTS §5, Canary-Prinzip): Debug-Ausgaben enthalten keine Kennungs- oder Schlüsselbytes.
use contextra_storage_ports::error::ErrorClass;
use contextra_storage_ports::{ClusterId, Error, Key32, Redacted};

#[test]
fn cluster_id_debug_is_redacted() {
    let id = ClusterId::from_wire([0xAB; 16]);
    assert_eq!(format!("{id:?}"), "ClusterId(<redacted>)");
}

#[test]
fn key_debug_is_redacted() {
    let k = Key32::from_array([0x5C; 32]);
    assert_eq!(format!("{k:?}"), "Key32(<redacted>)");
}

#[test]
fn redacted_wrapper_hides_payload() {
    assert_eq!(format!("{:?}", Redacted(vec![1u8, 2, 3])), "<redacted>");
}

#[test]
fn error_classes_and_hints_are_static_text() {
    let e = Error::Corrupt { place: "Block" };
    assert_eq!(e.class(), ErrorClass::Corrupt);
    assert!(!e.hint().is_empty());
    assert_eq!(Error::CasLost.class(), ErrorClass::Conflict);
}
