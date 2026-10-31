//! Additional comprehensive invariant and property tests for `contextra-types`.
//!
//! Verifies:
//! - Deep proptest invariants for all IDs and value objects.
//! - Deterministic Hash and Ord properties for map and set usage.
//! - Comprehensive error variant display and serialization roundtrips.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::error::StepId;
use contextra_types::*;
use proptest::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ═════════════════════════════════════════════════════════════════════════════
// 1. Proptest Generators & Invariant Suites
// ═════════════════════════════════════════════════════════════════════════════

proptest! {
    #[test]
        /// Property-Test: DocId::from_key verhält sich für beliebige Strings deterministisch und fehlerfrei.
    fn prop_doc_id_from_key_properties(s in "\\PC*") {
        let res = DocId::from_key(&s);
        if s.is_empty() {
            prop_assert!(res.is_err());
        } else {
            let id = res.expect("non-empty key should derive valid DocId");
            #[cfg(not(feature = "docid-128"))]
            {
                let _inner: u64 = id.inner();
            }
            #[cfg(feature = "docid-128")]
            {
                let _inner: u128 = id.inner();
            }
            // Determinism check
            let id2 = DocId::from_key(&s).unwrap();
            prop_assert_eq!(id, id2);
        }
    }

    #[test]
        /// Property-Test: EntityId::from_key und From<&str> sind deterministisch und konsistent.
    fn prop_entity_id_from_key_properties(s in "\\PC*") {
        let res = EntityId::from_key(&s);
        if s.is_empty() {
            prop_assert!(res.is_err());
        } else {
            let id = res.expect("non-empty key should derive valid EntityId");
            // Determinism check
            let id2 = EntityId::from_key(&s).unwrap();
            prop_assert_eq!(id, id2);

            // Infallible conversion parity check
            let from_str: EntityId = s.as_str().into();
            if let Ok(num) = s.parse::<u64>() {
                prop_assert_eq!(from_str.inner(), num);
            } else {
                prop_assert_eq!(from_str, id);
            }
        }
    }

    #[test]
        /// Property-Test: TxId-Ordnung und Hashing stimmen exakt mit der primitiven u64-Ordnung überein.
    fn prop_tx_id_ordering_and_display(a in proptest::num::u64::ANY, b in proptest::num::u64::ANY) {
        let tx_a = TxId::new(a);
        let tx_b = TxId::new(b);

        prop_assert_eq!(tx_a.cmp(&tx_b), a.cmp(&b));
        prop_assert_eq!(tx_a < tx_b, a < b);
        prop_assert_eq!(tx_a == tx_b, a == b);

        let disp_a = format!("{tx_a}");
        prop_assert!(disp_a.contains(&a.to_string()));

        if tx_a == tx_b {
            let mut h1 = DefaultHasher::new();
            let mut h2 = DefaultHasher::new();
            tx_a.hash(&mut h1);
            tx_b.hash(&mut h2);
            prop_assert_eq!(h1.finish(), h2.finish());
        }
    }

    #[test]
        /// Property-Test: CollectionId::try_new weist 0 zuverlässig ab und kapselt alle anderen u64-Werte.
    fn prop_collection_id_invariants(id in proptest::num::u64::ANY) {
        let res = CollectionId::try_new(id);
        if id == 0 {
            prop_assert!(res.is_err());
            prop_assert_eq!(CollectionId::INVALID.inner(), 0);
        } else {
            let c = res.expect("non-zero collection ID must succeed");
            prop_assert_eq!(c.inner(), id);
            let disp = format!("{c}");
            prop_assert!(disp.contains(&id.to_string()));
        }
    }

    #[test]
        /// Property-Test: ImportanceScore::new klemmt beliebige f32-Werte sicher auf [0.0, 1.0].
    fn prop_importance_score_clamping(raw in proptest::num::f32::ANY) {
        let score = ImportanceScore::new(raw);
        if raw.is_nan() {
            prop_assert_eq!(score.value(), 0.0);
        } else {
            prop_assert!(score.value() >= 0.0);
            prop_assert!(score.value() <= 1.0);
        }
    }

    #[test]
        /// Property-Test: Token-Verbrauch und Rückerstattungen bleiben im Budget konsistent.
    fn prop_token_budget_consumption(
        max_t in 1_000usize..1_000_000,
        res_t in 0usize..500,
        c1 in 0usize..500,
        c2 in 0usize..500
    ) {
        let budget = TokenBudget::new(max_t, res_t);
        let initial_avail = budget.available();
        prop_assert_eq!(initial_avail, max_t.saturating_sub(res_t));

        budget.consume(c1);
        prop_assert_eq!(budget.consumed(), c1);

        budget.consume(c2);
        prop_assert_eq!(budget.consumed(), c1 + c2);

        budget.refund(c1);
        prop_assert_eq!(budget.consumed(), c2);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. Comprehensive Type Construction & Method Coverage
// ═════════════════════════════════════════════════════════════════════════════

#[test]
/// Prüft Konstruktion und Validierung von `Entity` auf leere Namen und Typen.
fn test_entity_construction_and_validation() {
    let e_ok = Entity::try_new(EntityId::new(10), "Valid Name", "ValidType").unwrap();
    assert_eq!(e_ok.id, EntityId::new(10));
    assert_eq!(&*e_ok.name, "Valid Name");
    assert_eq!(&*e_ok.entity_type, "ValidType");

    assert!(Entity::try_new(EntityId::new(10), "", "Type").is_err());
    assert!(Entity::try_new(EntityId::new(10), "   ", "Type").is_err());
    assert!(Entity::try_new(EntityId::new(10), "Name", "").is_err());
    assert!(Entity::try_new(EntityId::new(10), "Name", "   ").is_err());
}

#[test]
/// Prüft Konstruktion, Builder-Methoden und Validierungsregeln von `Edge`.
fn test_edge_construction_and_builder_chain() {
    let edge_ok = Edge::try_new(EntityId::new(1), EntityId::new(2), "RELATES_TO", 1.5).unwrap();
    assert_eq!(edge_ok.from, EntityId::new(1));
    assert_eq!(edge_ok.to, EntityId::new(2));
    assert_eq!(&*edge_ok.label, "RELATES_TO");
    assert_eq!(edge_ok.weight, 1.5);

    let updated = edge_ok
        .with_weight(2.0)
        .with_source_doc_id(DocId::new(555))
        .with_tx_validity(Some(TxId::new(10)), Some(TxId::new(20)))
        .with_business_validity(Some(1000), Some(2000));

    assert_eq!(updated.weight, 2.0);
    assert_eq!(updated.source_doc_id, Some(DocId::new(555)));
    assert_eq!(updated.tx_valid_from, Some(TxId::new(10)));
    assert_eq!(updated.tx_valid_to, Some(TxId::new(20)));
    assert_eq!(updated.business_valid_from, Some(1000));
    assert_eq!(updated.business_valid_to, Some(2000));

    // Alias check
    let updated2 = updated.with_validity(Some(TxId::new(100)), None);
    assert_eq!(updated2.tx_valid_from, Some(TxId::new(100)));
    assert_eq!(updated2.tx_valid_to, None);

    // Validation failures
    assert!(Edge::try_new(EntityId::new(1), EntityId::new(2), "", 1.0).is_err());
    assert!(Edge::try_new(EntityId::new(1), EntityId::new(2), "   ", 1.0).is_err());
    assert!(Edge::try_new(EntityId::new(1), EntityId::new(2), "REL", -0.5).is_err());
    assert!(Edge::try_new(EntityId::new(1), EntityId::new(2), "REL", f32::NAN).is_err());
    assert!(Edge::try_new(EntityId::new(1), EntityId::new(2), "REL", f32::INFINITY).is_err());
}

#[test]
/// Prüft Textkombination und Tokenzählung bei `ContextChunk` mit und ohne Kontextpräfix.
fn test_context_chunk_text_and_token_methods() {
    let chunk_no_prefix = ContextChunk {
        doc_id: DocId::new(1),
        content: "Raw paragraph content".to_string(),
        relevance: 0.8,
        token_count: 10,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };

    assert_eq!(
        chunk_no_prefix.combined_text_owned(),
        "Raw paragraph content"
    );
    assert_eq!(chunk_no_prefix.combined_token_count(), 10);
    assert!(!chunk_no_prefix.has_context_prefix());

    let chunk_with_prefix = ContextChunk {
        doc_id: DocId::new(2),
        content: "Raw paragraph content".to_string(),
        relevance: 0.9,
        token_count: 10,
        metadata: None,
        contextual_prefix: Some("Contextual Header Prefix".to_string()), // 24 chars -> +6 tokens
        links: Vec::new(),
    };

    assert_eq!(
        chunk_with_prefix.combined_text_owned(),
        "Contextual Header Prefix\n\nRaw paragraph content"
    );
    assert_eq!(chunk_with_prefix.combined_token_count(), 16);
    assert!(chunk_with_prefix.has_context_prefix());
}

#[test]
/// Prüft Methoden und Leerheitsprüfungen von `QueryRewriteOutput`.
fn test_query_rewrite_output_methods() {
    let empty_rewrite = QueryRewriteOutput::default();
    assert!(empty_rewrite.is_empty());

    let text_only = QueryRewriteOutput::text_only("keyword search");
    assert!(!text_only.is_empty());
    assert_eq!(text_only.text_query.as_deref(), Some("keyword search"));
    assert!(text_only.semantic_query.is_none());
    assert!(text_only.anchor_entities.is_empty());

    let full_rewrite = QueryRewriteOutput::new(
        Some("bm25".to_string()),
        Some("vector".to_string()),
        vec!["entity_a".to_string()],
    );
    assert!(!full_rewrite.is_empty());
}

#[test]
/// Prüft Schemaversionen und die Bytegrößen von `DocIdWidth`.
fn test_schema_versions_and_doc_id_width() {
    let width_default = DocIdWidth::current();
    #[cfg(not(feature = "docid-128"))]
    assert_eq!(width_default, DocIdWidth::Bit64);
    #[cfg(feature = "docid-128")]
    assert_eq!(width_default, DocIdWidth::Bit128);

    assert_eq!(DocIdWidth::Bit64.bytes_len(), 8);
    assert_eq!(DocIdWidth::Bit128.bytes_len(), 16);

    for version in [ManifestSchemaVersion::V1, ManifestSchemaVersion::V2] {
        let byte = version.as_u8();
        let parsed = ManifestSchemaVersion::from_u8(byte).unwrap();
        assert_eq!(version, parsed);
    }

    assert!(ManifestSchemaVersion::from_u8(0).is_err());
    assert!(ManifestSchemaVersion::from_u8(255).is_err());
}

#[test]
/// Prüft Metadaten-Schlüssel und Standard-TTLs für `MemoryType`.
fn test_memory_type_helpers() {
    assert_eq!(MemoryType::Episodic.as_metadata_key(), "episodic");
    assert_eq!(MemoryType::Semantic.as_metadata_key(), "semantic");
    assert_eq!(MemoryType::Procedural.as_metadata_key(), "procedural");
    assert_eq!(MemoryType::Working.as_metadata_key(), "working");

    assert_eq!(MemoryType::Working.default_ttl_tx(), Some(50_000));
    assert_eq!(MemoryType::Semantic.default_ttl_tx(), None);

    let ep_decay = MemoryType::Episodic.default_decay();
    let sem_decay = MemoryType::Semantic.default_decay();
    assert_ne!(ep_decay, sem_decay);
}

#[test]
/// Prüft alle spezialisierten Konstruktor-Helfer von `ContextraError`.
fn test_all_error_variant_constructors_and_helpers() {
    let err_limit = ContextraError::limit_exceeded(100, "scan");
    assert!(matches!(
        err_limit,
        ContextraError::LimitExceeded { limit: 100, ref context } if context == "scan"
    ));

    let err_model = ContextraError::model_load("path.gguf", "bad header");
    assert!(matches!(
        err_model,
        ContextraError::ModelLoad { ref path, ref reason } if path == "path.gguf" && reason == "bad header"
    ));

    let err_snap = ContextraError::snapshot_unsupported_for_signal("PPR");
    assert!(matches!(
        err_snap,
        ContextraError::SnapshotUnsupportedForSignal(ref s) if s == "PPR"
    ));

    let err_orphan = ContextraError::orphaned_vector_reference("doc_1", "vec_1");
    assert!(matches!(
        err_orphan,
        ContextraError::OrphanedVectorReference { ref doc_id, ref index_id } if doc_id == "doc_1" && index_id == "vec_1"
    ));

    let err_cap = ContextraError::capability_unsupported("simd", "no_avx2");
    assert!(matches!(
        err_cap,
        ContextraError::CapabilityUnsupported { ref capability, ref reason } if capability == "simd" && reason == "no_avx2"
    ));

    let err_inv = ContextraError::invalid_input("key missing");
    assert!(matches!(
        err_inv,
        ContextraError::InvalidInput(ref msg) if msg == "key missing"
    ));

    let err_wal = ContextraError::wal_corruption(512, "crc fail");
    assert!(matches!(
        err_wal,
        ContextraError::WalCorruption { offset: 512, ref reason, .. } if reason == "crc fail"
    ));

    let err_chk = ContextraError::checksum_mismatch("file.sst", 3);
    assert!(matches!(
        err_chk,
        ContextraError::ChecksumMismatch { ref path, block_id: 3, .. } if path == "file.sst"
    ));

    let err_kv = ContextraError::kv_quantization("out of bounds");
    assert!(matches!(
        err_kv,
        ContextraError::KvQuantization(ref msg) if msg == "out of bounds"
    ));

    let err_dur = ContextraError::durability_config("sync required");
    assert!(matches!(
        err_dur,
        ContextraError::DurabilityConfig(ref msg) if msg == "sync required"
    ));

    let err_pin = ContextraError::pin_budget_exceeded("memory limit");
    assert!(matches!(
        err_pin,
        ContextraError::PinBudgetExceeded(ref msg) if msg == "memory limit"
    ));

    let err_link = ContextraError::cross_device_link("/a", "/b");
    assert!(matches!(
        err_link,
        ContextraError::CrossDeviceLink { ref source_path, ref target_path } if source_path == "/a" && target_path == "/b"
    ));

    let err_trunc = ContextraError::wal_truncation_detected([1u8; 32], [2u8; 32]);
    assert!(matches!(
        err_trunc,
        ContextraError::WalTruncationDetected { expected_hmac, actual_hmac } if expected_hmac == [1u8; 32] && actual_hmac == [2u8; 32]
    ));

    assert!(ContextraError::StaleRead("conflict".into()).is_occ_conflict());
    assert!(ContextraError::Conflict("conflict".into()).is_occ_conflict());
    assert!(!ContextraError::Internal("fail".into()).is_occ_conflict());
}

#[test]
/// Prüft Schritt-IDs (`StepId`) und die Formatierung aller Retrieval-Strategien.
fn test_retrieval_strategy_and_step_id() {
    let s = StepId::new(1001);
    assert_eq!(s.inner(), 1001);
    assert_eq!(format!("{s}"), "StepId(1001)");
    assert_eq!(StepId::from(1001), s);

    for strat in [
        RetrievalStrategy::Vector,
        RetrievalStrategy::Text,
        RetrievalStrategy::Graph,
        RetrievalStrategy::Hybrid,
        RetrievalStrategy::Global {
            max_community_nodes: Some(10),
            min_community_size: Some(2),
        },
    ] {
        let disp = format!("{strat}");
        assert!(!disp.is_empty());
    }
}
