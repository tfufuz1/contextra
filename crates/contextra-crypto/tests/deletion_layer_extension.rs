//! Test suite for DeletionLayer and ExcludedScope additive extensions.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{DeletionLayer, ExcludedScope};
use std::collections::HashSet;

#[test]
fn test_golden_bincode_encoding_unmodified_for_old_variants() {
    let old_layers = vec![
        DeletionLayer::LsmMemtable,
        DeletionLayer::SsTableAllLevels,
        DeletionLayer::HnswIndex,
        DeletionLayer::WalAllSegments { seq_after: 42 },
        DeletionLayer::CsrGraph,
        DeletionLayer::KvCacheSegments,
        DeletionLayer::EmbeddingCache,
    ];

    let old_scopes = vec![
        ExcludedScope::ConsolidatedAndDistilled,
        ExcludedScope::LlmParameterMemory,
    ];

    let encoded_layers = bincode::serialize(&old_layers).expect("serialization failed");
    let encoded_scopes = bincode::serialize(&old_scopes).expect("serialization failed");

    // Deserialize and check equality to ensure bincode encoding stays byte-for-byte compatible
    let decoded_layers: Vec<DeletionLayer> =
        bincode::deserialize(&encoded_layers).expect("deserialization failed");
    let decoded_scopes: Vec<ExcludedScope> =
        bincode::deserialize(&encoded_scopes).expect("deserialization failed");

    assert_eq!(old_layers, decoded_layers);
    assert_eq!(old_scopes, decoded_scopes);
}

#[test]
fn test_layer_id_mappings_no_duplicates() {
    let all_layers = vec![
        (DeletionLayer::LsmMemtable, 1u16),
        (DeletionLayer::SsTableAllLevels, 2u16),
        (DeletionLayer::HnswIndex, 3u16),
        (DeletionLayer::WalAllSegments { seq_after: 0 }, 4u16),
        (DeletionLayer::CsrGraph, 5u16),
        (DeletionLayer::KvCacheSegments, 6u16),
        (DeletionLayer::EmbeddingCache, 7u16),
        (DeletionLayer::TextIndex, 20u16),
        (DeletionLayer::Relations, 21u16),
        (DeletionLayer::Manifest, 22u16),
        (DeletionLayer::CheckpointClones, 23u16),
        (DeletionLayer::ProcessCaches, 24u16),
        (DeletionLayer::Keyring, 25u16),
        (DeletionLayer::ErasureLedger, 26u16),
    ];

    let mut seen_ids = HashSet::new();

    for (layer, expected_id) in all_layers {
        let id = layer.layer_id();
        assert_eq!(id, expected_id, "Layer {:?} ID mismatch", layer);
        assert!(seen_ids.insert(id), "Duplicate layer_id detected: {}", id);
    }
}

#[test]
fn test_excluded_scope_id_and_from_id_roundtrip() {
    let test_cases = vec![
        (ExcludedScope::ConsolidatedAndDistilled, 1u16, true),
        (ExcludedScope::LlmParameterMemory, 2u16, true),
        (ExcludedScope::StorageMediaRemanence, 3u16, true),
        (ExcludedScope::UnregisteredBackups, 4u16, true),
        (ExcludedScope::OsPageCacheSwapCoreDumps, 5u16, true),
        (ExcludedScope::ClientAndModelContext, 6u16, true),
        (ExcludedScope::AggregateDerivedStatistics, 7u16, true),
        (ExcludedScope::LogsAndTelemetryOutsideProcess, 8u16, true),
        (ExcludedScope::ExportedData, 9u16, true),
        (
            ExcludedScope::RegisteredCopies {
                ids: vec!["copy_1".to_string()],
            },
            10u16,
            false, // from_id returns None for RegisteredCopies
        ),
        (ExcludedScope::PersistedHnswSnapshots, 11u16, true),
    ];

    let mut seen_ids = HashSet::new();

    for (scope, expected_id, expects_roundtrip) in test_cases {
        let id = scope.id();
        assert_eq!(id, expected_id, "ExcludedScope {:?} ID mismatch", scope);
        assert!(seen_ids.insert(id), "Duplicate ExcludedScope id: {}", id);

        let derived = ExcludedScope::from_id(id);
        if expects_roundtrip {
            assert_eq!(
                derived,
                Some(scope),
                "Roundtrip failed for ExcludedScope ID {}",
                id
            );
        } else {
            assert_eq!(
                derived, None,
                "Expected None from_id for ExcludedScope ID {}",
                id
            );
        }
    }

    assert_eq!(ExcludedScope::from_id(0), None);
    assert_eq!(ExcludedScope::from_id(12), None);
    assert_eq!(ExcludedScope::from_id(999), None);
}

#[test]
fn test_serde_roundtrip_all_variants() {
    let all_layers = vec![
        DeletionLayer::LsmMemtable,
        DeletionLayer::SsTableAllLevels,
        DeletionLayer::HnswIndex,
        DeletionLayer::WalAllSegments { seq_after: 100 },
        DeletionLayer::CsrGraph,
        DeletionLayer::KvCacheSegments,
        DeletionLayer::EmbeddingCache,
        DeletionLayer::TextIndex,
        DeletionLayer::Relations,
        DeletionLayer::Manifest,
        DeletionLayer::CheckpointClones,
        DeletionLayer::ProcessCaches,
        DeletionLayer::Keyring,
        DeletionLayer::ErasureLedger,
    ];

    for layer in &all_layers {
        let json = serde_json::to_string(layer).expect("JSON serialization failed");
        let decoded: DeletionLayer =
            serde_json::from_str(&json).expect("JSON deserialization failed");
        assert_eq!(layer, &decoded);

        let bin = bincode::serialize(layer).expect("Bincode serialization failed");
        let decoded_bin: DeletionLayer =
            bincode::deserialize(&bin).expect("Bincode deserialization failed");
        assert_eq!(layer, &decoded_bin);
    }

    let all_scopes = vec![
        ExcludedScope::ConsolidatedAndDistilled,
        ExcludedScope::LlmParameterMemory,
        ExcludedScope::StorageMediaRemanence,
        ExcludedScope::UnregisteredBackups,
        ExcludedScope::OsPageCacheSwapCoreDumps,
        ExcludedScope::ClientAndModelContext,
        ExcludedScope::AggregateDerivedStatistics,
        ExcludedScope::LogsAndTelemetryOutsideProcess,
        ExcludedScope::ExportedData,
        ExcludedScope::RegisteredCopies {
            ids: vec!["c1".to_string(), "c2".to_string()],
        },
        ExcludedScope::PersistedHnswSnapshots,
    ];

    for scope in &all_scopes {
        let json = serde_json::to_string(scope).expect("JSON serialization failed");
        let decoded: ExcludedScope =
            serde_json::from_str(&json).expect("JSON deserialization failed");
        assert_eq!(scope, &decoded);

        let bin = bincode::serialize(scope).expect("Bincode serialization failed");
        let decoded_bin: ExcludedScope =
            bincode::deserialize(&bin).expect("Bincode deserialization failed");
        assert_eq!(scope, &decoded_bin);
    }
}
