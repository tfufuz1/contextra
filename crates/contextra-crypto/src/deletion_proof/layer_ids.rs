//! Layer ID mappings and utilities according to Contract V1.

use super::types::DeletionLayer;

impl DeletionLayer {
    /// Returns the canonical u16 ID for the deletion layer per Contract V1.
    pub fn layer_id(&self) -> u16 {
        match self {
            Self::LsmMemtable => 1,
            Self::SsTableAllLevels => 2,
            Self::HnswIndex => 3,
            Self::WalAllSegments { .. } => 4,
            Self::CsrGraph => 5,
            Self::KvCacheSegments => 6,
            Self::EmbeddingCache => 7,
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
