use contextra_graph::hyperedge::{HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;

#[test]
fn test_golden_hyperedge_id_serde_bincode_compatibility() {
    let id = HyperEdgeId::new(0xDEAD_BEEF_1234_5678);

    // Test bincode layout (u64 little-endian = 8 bytes)
    let bincode_bytes = bincode::serialize(&id).expect("bincode serialization failed");
    assert_eq!(bincode_bytes.len(), 8);
    assert_eq!(
        bincode_bytes,
        0xDEAD_BEEF_1234_5678u64.to_le_bytes().to_vec()
    );

    let deserialized: HyperEdgeId =
        bincode::deserialize(&bincode_bytes).expect("bincode deserialization failed");
    assert_eq!(deserialized, id);

    // Test JSON layout
    let json_str = serde_json::to_string(&id).expect("json serialization failed");
    assert_eq!(json_str, "16045690981402826360");
    let deserialized_json: HyperEdgeId =
        serde_json::from_str(&json_str).expect("json deserialization failed");
    assert_eq!(deserialized_json, id);
}

#[test]
fn test_golden_role_id_serde_bincode_compatibility() {
    let role = RoleId::new(42);

    // Test bincode layout (u32 little-endian = 4 bytes)
    let bincode_bytes = bincode::serialize(&role).expect("bincode serialization failed");
    assert_eq!(bincode_bytes.len(), 4);
    assert_eq!(bincode_bytes, 42u32.to_le_bytes().to_vec());

    let deserialized: RoleId =
        bincode::deserialize(&bincode_bytes).expect("bincode deserialization failed");
    assert_eq!(deserialized, role);

    // Test JSON layout
    let json_str = serde_json::to_string(&role).expect("json serialization failed");
    assert_eq!(json_str, "42");
    let deserialized_json: RoleId =
        serde_json::from_str(&json_str).expect("json deserialization failed");
    assert_eq!(deserialized_json, role);
}

#[test]
fn test_golden_role_binding_serde_bincode_compatibility() {
    let binding = RoleBinding::new(RoleId::new(10), EntityId::new(200));

    // Test bincode serialization
    let bincode_bytes = bincode::serialize(&binding).expect("bincode serialization failed");
    let deserialized: RoleBinding =
        bincode::deserialize(&bincode_bytes).expect("bincode deserialization failed");
    assert_eq!(deserialized, binding);

    // Test JSON serialization
    let json_str = serde_json::to_string(&binding).expect("json serialization failed");
    let deserialized_json: RoleBinding =
        serde_json::from_str(&json_str).expect("json deserialization failed");
    assert_eq!(deserialized_json, binding);
}
