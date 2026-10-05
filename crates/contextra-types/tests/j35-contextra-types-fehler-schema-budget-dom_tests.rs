//! Integration test suite for J35 contextra-types symbols:
//! - durability_config
//! - orphaned_vector_reference
//! - bytes_len
//! - is_compatible_with_current_build
//! - for_model
//! - try_reserve
//! - with_reserved
//! - try_from_internal_offset
//! - as_metadata_key
//! - with_validity
//! - with_weight

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_types::error::ContextraError;
use contextra_types::schema::{DocIdWidth, ManifestSchemaVersion};
use contextra_types::types::budget::TokenBudget;
use contextra_types::types::domain::{Edge, EntityId, MemoryType, TxId};

#[test]
fn test_j35_durability_config_error_constructor() {
    let err = ContextraError::durability_config("WAL mode mismatch in config");
    assert!(
        matches!(err, ContextraError::DurabilityConfig(ref msg) if msg == "WAL mode mismatch in config")
    );
    assert_eq!(
        err.to_string(),
        "Durability configuration error: WAL mode mismatch in config"
    );
}

#[test]
fn test_j35_orphaned_vector_reference_error_constructor() {
    let err = ContextraError::orphaned_vector_reference("doc_xyz", "vec_idx_99");
    assert!(
        matches!(err, ContextraError::OrphanedVectorReference { ref doc_id, ref index_id } if doc_id == "doc_xyz" && index_id == "vec_idx_99")
    );
    assert_eq!(
        err.to_string(),
        "Orphaned vector reference: index_id=vec_idx_99 -> doc_id=doc_xyz"
    );
}

#[test]
fn test_j35_doc_id_width_bytes_len_and_manifest_schema_version() {
    assert_eq!(DocIdWidth::Bit64.bytes_len(), 8);
    assert_eq!(DocIdWidth::Bit128.bytes_len(), 16);

    let v1 = ManifestSchemaVersion::V1;
    let v2 = ManifestSchemaVersion::V2;
    assert_eq!(v1.doc_id_bytes_len(), 8);
    assert_eq!(v2.doc_id_bytes_len(), 16);
}

#[test]
fn test_j35_is_compatible_with_current_build() {
    let current_version = ManifestSchemaVersion::default();
    assert!(current_version.is_compatible_with_current_build());
    assert!(current_version.validate_build_compatibility().is_ok());

    let other_version = if cfg!(feature = "docid-128") {
        ManifestSchemaVersion::V1
    } else {
        ManifestSchemaVersion::V2
    };
    assert!(!other_version.is_compatible_with_current_build());
    assert!(other_version.validate_build_compatibility().is_err());
}

#[test]
fn test_j35_token_budget_for_model() {
    let b_claude = TokenBudget::for_model("claude-4");
    assert_eq!(b_claude.limit, 200_000);

    let b_gpt = TokenBudget::for_model("gpt-4o");
    assert_eq!(b_gpt.limit, 128_000);
}

#[test]
fn test_j35_token_budget_with_reserved_and_try_reserve() {
    let budget = TokenBudget::for_model("gpt-4o").with_reserved(1_000, 1_400);
    // Effective limit: 128_000 * 0.8 = 102_400. Reserved: 2_400. Available: 100_000.
    assert_eq!(budget.reserved, 2_400);
    assert_eq!(budget.available(), 100_000);

    // Try reserve within budget
    assert!(budget.try_reserve(10_000).is_ok());
    assert_eq!(budget.available(), 90_000);

    // Try reserve exceeding budget
    assert!(budget.try_reserve(100_000).is_err());
}

#[test]
fn test_j35_tx_id_try_from_internal_offset() {
    let offset = 42u64;
    let tx_id = TxId::try_from_internal_offset(offset).expect("Valid internal offset");
    assert_eq!(tx_id.inner(), TxId::INTERNAL_BASE + offset);
    assert!(tx_id.is_valid_origin());

    let overflow_offset = u64::MAX - TxId::INTERNAL_BASE + 1;
    let err = TxId::try_from_internal_offset(overflow_offset);
    assert!(err.is_err());
}

#[test]
fn test_j35_memory_type_as_metadata_key_and_display() {
    let mem = MemoryType::Episodic;
    assert_eq!(mem.as_metadata_key(), "episodic");
    assert_eq!(mem.to_string(), "episodic");

    assert_eq!(MemoryType::Semantic.to_string(), "semantic");
    assert_eq!(MemoryType::Procedural.to_string(), "procedural");
    assert_eq!(MemoryType::Working.to_string(), "working");
}

#[test]
fn test_j35_edge_with_weight_and_with_validity() {
    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);
    let edge = Edge::new(e1, e2, "connected_to")
        .with_weight(0.85)
        .with_validity(Some(TxId::new(100)), Some(TxId::new(200)));

    assert!((edge.weight - 0.85).abs() < f32::EPSILON);
    assert_eq!(edge.tx_valid_from, Some(TxId::new(100)));
    assert_eq!(edge.tx_valid_to, Some(TxId::new(200)));
}
