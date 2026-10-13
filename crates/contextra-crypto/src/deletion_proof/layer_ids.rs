//! Layer ID mappings and utilities according to Contract V1 and V4 extension.

use super::types::DeletionLayer;

impl DeletionLayer {
    /// Returns the canonical u16 ID for the deletion layer per Contract V1 / V4.
    pub fn layer_id(&self) -> u16 {
        match self {
            Self::LsmMemtable => 1,
            Self::SsTableAllLevels => 2,
            Self::HnswIndex => 3,
            Self::WalAllSegments { .. } => 4,
            Self::CsrGraph => 5,
            Self::KvCacheSegments => 6,
            Self::EmbeddingCache => 7,
            Self::TextIndex => 20,
            Self::Relations => 21,
            Self::Manifest => 22,
            Self::CheckpointClones => 23,
            Self::ProcessCaches => 24,
            Self::Keyring => 25,
            Self::ErasureLedger => 26,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ids_contract_v1_mapping() {
        assert_eq!(DeletionLayer::LsmMemtable.layer_id(), 1);
        assert_eq!(DeletionLayer::SsTableAllLevels.layer_id(), 2);
        assert_eq!(DeletionLayer::HnswIndex.layer_id(), 3);
        assert_eq!(DeletionLayer::WalAllSegments { seq_after: 0 }.layer_id(), 4);
        assert_eq!(DeletionLayer::CsrGraph.layer_id(), 5);
        assert_eq!(DeletionLayer::KvCacheSegments.layer_id(), 6);
        assert_eq!(DeletionLayer::EmbeddingCache.layer_id(), 7);
        assert_eq!(DeletionLayer::TextIndex.layer_id(), 20);
        assert_eq!(DeletionLayer::Relations.layer_id(), 21);
        assert_eq!(DeletionLayer::Manifest.layer_id(), 22);
        assert_eq!(DeletionLayer::CheckpointClones.layer_id(), 23);
        assert_eq!(DeletionLayer::ProcessCaches.layer_id(), 24);
        assert_eq!(DeletionLayer::Keyring.layer_id(), 25);
        assert_eq!(DeletionLayer::ErasureLedger.layer_id(), 26);
    }

    #[test]
    fn test_old_variants_bincode_serde_golden_identical() {
        let old_layers = vec![
            DeletionLayer::LsmMemtable,
            DeletionLayer::SsTableAllLevels,
            DeletionLayer::HnswIndex,
            DeletionLayer::WalAllSegments { seq_after: 42 },
            DeletionLayer::CsrGraph,
            DeletionLayer::KvCacheSegments,
            DeletionLayer::EmbeddingCache,
        ];

        let encoded = bincode::serialize(&old_layers).expect("serialization failed");
        let decoded: Vec<DeletionLayer> =
            bincode::deserialize(&encoded).expect("deserialization failed");

        assert_eq!(old_layers, decoded);
    }
}
