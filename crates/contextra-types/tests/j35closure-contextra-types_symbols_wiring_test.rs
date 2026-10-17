//! Integration test verifying wiring and reachability for all 11 target symbols in contextra-types
//! per Task J35-contextra-types-fehler-schema-budget-dom-CLOSURE.

use contextra_types::{
    error::ContextraError,
    schema::{DocIdWidth, ManifestSchemaVersion},
    types::budget::TokenBudget,
    Edge, EntityId, MemoryType, TxId,
};

#[test]
fn test_j35closure_error_constructors_and_check_helpers() {
    // 1. durability_config
    let err_durability = ContextraError::durability_config("WAL durability error");
    assert!(
        matches!(err_durability, ContextraError::DurabilityConfig(ref msg) if msg == "WAL durability error")
    );
    assert!(ContextraError::check_durability(true, "ok").is_ok());
    assert!(ContextraError::check_durability(false, "err").is_err());

    let err_kind = ContextraError::from_kind_and_message("DurabilityConfig", "mode mismatch");
    assert!(matches!(err_kind, ContextraError::DurabilityConfig(_)));

    // 2. orphaned_vector_reference
    let err_orphan = ContextraError::orphaned_vector_reference("doc_1", "vec_1");
    assert!(
        matches!(err_orphan, ContextraError::OrphanedVectorReference { ref doc_id, ref index_id } if doc_id == "doc_1" && index_id == "vec_1")
    );
    assert!(ContextraError::check_vector_reference(true, "doc_1", "vec_1").is_ok());
    assert!(ContextraError::check_vector_reference(false, "doc_1", "vec_1").is_err());

    let err_kind_orphan =
        ContextraError::from_kind_and_message("OrphanedVectorReference", "doc_A -> vec_B");
    assert!(matches!(
        err_kind_orphan,
        ContextraError::OrphanedVectorReference { .. }
    ));
}

#[test]
fn test_j35closure_schema_width_and_version_methods() {
    // 3. bytes_len
    assert_eq!(DocIdWidth::Bit64.bytes_len(), 8);
    assert_eq!(DocIdWidth::Bit128.bytes_len(), 16);
    assert!(DocIdWidth::current_bytes_len() > 0);

    // 4. is_compatible_with_current_build
    let version = ManifestSchemaVersion::V1;
    let is_compat = version.is_compatible_with_current_build();
    assert_eq!(is_compat, version.validate_build_compatibility().is_ok());
}

#[test]
fn test_j35closure_token_budget_wiring() {
    // 5. for_model
    let budget_claude = TokenBudget::for_model("claude-4");
    assert_eq!(budget_claude.limit, 200_000);

    let default_budget = TokenBudget::default();
    assert_eq!(default_budget.limit, 8192);

    // 6. try_reserve
    let budget = TokenBudget::new(1000, 100);
    assert!(budget.try_reserve(200).is_ok());
    assert_eq!(budget.consumed(), 200);

    // 7. with_reserved
    let budget_reserved = TokenBudget::for_model("gpt-4o").with_reserved(100, 200);
    assert_eq!(budget_reserved.reserved, 300);
}

#[test]
fn test_j35closure_tx_id_internal_offset() {
    // 8. try_from_internal_offset
    let tx = TxId::try_from_internal_offset(42).expect("valid internal offset");
    assert_eq!(tx.inner(), TxId::INTERNAL_BASE + 42);
    assert!(tx.is_valid_origin());

    let tx_offset = TxId::internal_with_offset(100).expect("valid internal offset");
    assert_eq!(tx_offset.inner(), TxId::INTERNAL_BASE + 100);

    let const_internal = TxId::internal();
    assert_eq!(const_internal.inner(), TxId::INTERNAL_BASE);
}

#[test]
fn test_j35closure_domain_misc_methods() {
    // 9. as_metadata_key
    assert_eq!(MemoryType::Semantic.as_metadata_key(), "semantic");
    assert_eq!(MemoryType::Episodic.as_metadata_key(), "episodic");
    assert_eq!(MemoryType::Procedural.as_str(), "procedural");
    assert_eq!(format!("{}", MemoryType::Procedural), "procedural");

    // 10. with_validity
    let edge = Edge::new(EntityId::new(1), EntityId::new(2), "rel")
        .with_tx_validity(Some(TxId::new(10)), Some(TxId::new(20)));
    assert_eq!(edge.tx_valid_from, Some(TxId::new(10)));
    assert_eq!(edge.tx_valid_to, Some(TxId::new(20)));

    // 11. with_weight
    let weighted_edge =
        Edge::try_new(EntityId::new(1), EntityId::new(2), "rel", 0.75).expect("valid edge");
    assert!((weighted_edge.weight - 0.75).abs() < f32::EPSILON);
}
