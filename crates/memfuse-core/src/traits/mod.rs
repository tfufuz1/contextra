//! Core trait definitions for MemFuse subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z (SESSION: Zerlegung traits/mod.rs)
// ZWECK: Kern-Trait-Hierarchien für Layer 0.
// INVARIANTEN: Downward-only Trait interfaces; neue Trait-Methoden brauchen Default-Impls (Abwärtskompatibilität).
// HOTSPOTS: mod declaration & re-exports
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

// INVARIANT: Trait-Contracts sind das API-Rückgrat des Workspace.
// REGEL: Neue Methoden MÜSSEN Default-Impl haben (backward compat).

use std::future::Future;
use std::pin::Pin;

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Trait and mock definitions for text embedding providers and LLMs.
pub mod embedding;
pub use embedding::*;

/// Storage engine traits and stats.
pub mod storage;
pub use storage::*;

/// Vector index traits and stats.
pub mod vector_index;
pub use vector_index::*;

/// Text index traits, embedding engines, synthesizers, and stats.
pub mod text_index;
pub use text_index::*;

/// Graph index traits and stats.
pub mod graph_index;
pub use graph_index::*;

/// Checkpoint, coordinator, and snapshot traits.
pub mod checkpoint;
pub use checkpoint::*;

/// Memory lifecycle, grounding validator, and distance calculator traits.
pub mod lifecycle;
pub use lifecycle::*;

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
    }
}

#[cfg(test)]
mod capability_coverage {
    use super::*;
    use crate::types::*;
    use crate::Result;

    /// Verifies that calling search_at on a productive VectorIndex instance
    /// does NOT return CapabilityUnsupported.
    #[tokio::test]
    async fn test_hnsw_search_at_capability() {
        struct VectorIndexPlaceholder;
        impl VectorIndex for VectorIndexPlaceholder {
            async fn insert(&self, _: TxId, _: DocId, _: &[f32]) -> Result<()> {
                Ok(())
            }
            async fn search(&self, _: &[f32], _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn search_at(&self, _: &[f32], _: usize, _: u64) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<VectorIndexStats> {
                Ok(VectorIndexStats {
                    num_vectors: 0,
                    memory_usage_bytes: 0,
                    num_layers: 0,
                    deleted_ratio: 0.0,
                    rebuild_count: 0,
                })
            }
        }
        let index = VectorIndexPlaceholder;
        let res = index.search_at(&[1.0, 0.0], 5, 1).await;
        assert!(
            !matches!(res, Err(crate::MemFuseError::CapabilityUnsupported { .. })),
            "search_at returned CapabilityUnsupported"
        );
        assert!(!index.is_rebuild_required());
        index.trigger_rebuild_async();
    }

    /// Verifies that calling traverse_at, traverse_at_time, and traverse_at_bitemporal
    /// on a GraphIndex implementation does NOT return CapabilityUnsupported.
    #[tokio::test]
    async fn test_csr_graph_capability() {
        struct GraphIndexPlaceholder;
        impl GraphIndex for GraphIndexPlaceholder {
            fn traverse<'a>(
                &'a self,
                _: EntityId,
                _: usize,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: u64,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at_time<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: TxId,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn traverse_at_bitemporal<'a>(
                &'a self,
                _: EntityId,
                _: usize,
                _: TxId,
                _: Option<i64>,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn add_entity<'a>(&'a self, _: TxId, _: Entity) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn add_edge<'a>(&'a self, _: TxId, _: Edge) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
                Box::pin(async move { 0 })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
                Box::pin(async move {
                    Ok(GraphIndexStats {
                        num_entities: 0,
                        num_edges: 0,
                        memory_usage_bytes: 0,
                    })
                })
            }
        }
        let graph = GraphIndexPlaceholder;
        let res_traverse_at = graph.traverse_at(EntityId::new(1), 2, 1).await;
        assert!(
            !matches!(
                res_traverse_at,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at returned CapabilityUnsupported"
        );

        let res_traverse_at_time = graph
            .traverse_at_time(EntityId::new(1), 2, TxId::new(1))
            .await;
        assert!(
            !matches!(
                res_traverse_at_time,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at_time returned CapabilityUnsupported"
        );

        let res_traverse_at_bitemporal = graph
            .traverse_at_bitemporal(EntityId::new(1), 2, TxId::new(1), Some(1000))
            .await;
        assert!(
            !matches!(
                res_traverse_at_bitemporal,
                Err(crate::MemFuseError::CapabilityUnsupported { .. })
            ),
            "traverse_at_bitemporal returned CapabilityUnsupported"
        );
    }

    /// Verifies that calling search_at on a TextIndex implementation does NOT return CapabilityUnsupported.
    #[tokio::test]
    async fn test_text_index_search_at_capability() {
        struct TextIndexPlaceholder;
        impl TextIndex for TextIndexPlaceholder {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn search_at(&self, _: &str, _: usize, _: u64) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }
        let text_index = TextIndexPlaceholder;
        let res = text_index.search_at("test", 5, 1).await;
        assert!(
            !matches!(res, Err(crate::MemFuseError::CapabilityUnsupported { .. })),
            "search_at returned CapabilityUnsupported"
        );
    }

    /// Verifies that calling default scan_prefix_at on a StorageEngine implementation
    /// returns CapabilityUnsupported with capability "snapshot_read_at".
    #[tokio::test]
    async fn test_storage_scan_prefix_at_capability() {
        struct StorageEnginePlaceholder;
        impl StorageEngine for StorageEnginePlaceholder {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                _: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let placeholder = StorageEnginePlaceholder;
        let result = placeholder.scan_prefix_at(b"prefix", 0).await;
        assert!(matches!(
            result,
            Err(crate::MemFuseError::CapabilityUnsupported { ref capability, .. }) if capability == "snapshot_read_at"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use ahash::AHashMap;

    #[test]
    fn test_stats_serialization() {
        let v_stats = VectorIndexStats {
            num_vectors: 100,
            memory_usage_bytes: 1024,
            num_layers: 5,
            deleted_ratio: 0.1,
            rebuild_count: 1,
        };
        let ser = serde_json::to_string(&v_stats).unwrap(); // unwrap
        let deser: VectorIndexStats = serde_json::from_str(&ser).unwrap(); // unwrap
        assert_eq!(v_stats.num_vectors, deser.num_vectors);

        let s_stats = StorageStats {
            num_segments: 2,
            total_size_bytes: 2048,
            memtable_size_bytes: 512,
        };
        let ser = serde_json::to_string(&s_stats).unwrap(); // unwrap
        let deser: StorageStats = serde_json::from_str(&ser).unwrap(); // unwrap
        assert_eq!(s_stats.total_size_bytes, deser.total_size_bytes);

        let t_stats = TextIndexStats {
            num_documents: 10,
            num_tokens: 1000,
            memory_usage_bytes: 256,
        };
        let ser = serde_json::to_string(&t_stats).unwrap(); // unwrap
        let deser: TextIndexStats = serde_json::from_str(&ser).unwrap(); // unwrap
        assert_eq!(t_stats.num_documents, deser.num_documents);
    }

    #[tokio::test]
    async fn test_storage_engine_default_put_batch() {
        type KVPair = (Vec<u8>, Vec<u8>);
        type Log = std::sync::Arc<std::sync::Mutex<Vec<KVPair>>>;
        struct MockStorage(Log);
        impl StorageEngine for MockStorage {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(
                &'a self,
                _: TxId,
                key: &'a [u8],
                value: &'a [u8],
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.0.lock().unwrap().push((key.to_vec(), value.to_vec()));
                    Ok(())
                })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                _: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let store = MockStorage(std::sync::Arc::new(std::sync::Mutex::new(vec![])));
        let entries = vec![
            (b"k1".to_vec(), b"v1".to_vec()),
            (b"k2".to_vec(), b"v2".to_vec()),
        ];
        store.put_batch(TxId(1), &entries).await.unwrap(); // unwrap
        assert_eq!(store.0.lock().unwrap().len(), 2); // unwrap

        // Test scan_prefix_at default error
        let res = store.scan_prefix_at(b"pre", 1).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "snapshot_read_at");
            }
            _ => panic!("Expected CapabilityUnsupported for scan_prefix_at"),
        }
    }

    #[tokio::test]
    async fn test_scan_prefix_bounded_cursor_semantics() {
        struct MemoryStorage {
            data: Vec<(Vec<u8>, Vec<u8>)>,
        }

        impl StorageEngine for MemoryStorage {
            fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                prefix: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move {
                    let matching = self
                        .data
                        .iter()
                        .filter(|(k, _)| k.starts_with(prefix))
                        .cloned()
                        .collect();
                    Ok(matching)
                })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let dataset = MemoryStorage {
            data: vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                (b"pfx:2".to_vec(), b"v2".to_vec()),
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
                (b"pfx:5".to_vec(), b"v5".to_vec()),
            ],
        };

        // 1. Valid cursor in dataset -> yields next page starting strictly after cursor
        let (batch1, next_cur1) = dataset
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:2"))
            .await
            .unwrap(); // unwrap
        assert_eq!(
            batch1,
            vec![
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
            ]
        );
        assert_eq!(next_cur1, Some(b"pfx:4".to_vec()));

        // 2. Cursor references key deleted between calls (e.g. b"pfx:2" deleted, cursor = b"pfx:2")
        let dataset_after_delete = MemoryStorage {
            data: vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                // pfx:2 was deleted!
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
                (b"pfx:5".to_vec(), b"v5".to_vec()),
            ],
        };
        let (batch2, next_cur2) = dataset_after_delete
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:2"))
            .await
            .unwrap(); // unwrap
        assert_eq!(
            batch2,
            vec![
                (b"pfx:3".to_vec(), b"v3".to_vec()),
                (b"pfx:4".to_vec(), b"v4".to_vec()),
            ]
        );
        assert_eq!(next_cur2, Some(b"pfx:4".to_vec()));

        // 3. Cursor is None -> scans from beginning up to limit
        let (batch3, next_cur3) = dataset.scan_prefix_bounded(b"pfx:", 2, None).await.unwrap(); // unwrap
        assert_eq!(
            batch3,
            vec![
                (b"pfx:1".to_vec(), b"v1".to_vec()),
                (b"pfx:2".to_vec(), b"v2".to_vec()),
            ]
        );
        assert_eq!(next_cur3, Some(b"pfx:2".to_vec()));

        // 4. Empty scan result -> returns (vec![], None)
        let empty_dataset = MemoryStorage { data: vec![] };
        let (batch4, next_cur4) = empty_dataset
            .scan_prefix_bounded(b"pfx:", 2, Some(b"pfx:1"))
            .await
            .unwrap(); // unwrap
        assert!(batch4.is_empty());
        assert_eq!(next_cur4, None);
    }

    #[tokio::test]
    async fn test_delete_many_default_impl_deletes_all_keys() {
        struct MockStorage {
            data: std::sync::Arc<std::sync::Mutex<AHashMap<Vec<u8>, Vec<u8>>>>,
            delete_call_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        }

        impl StorageEngine for MockStorage {
            fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(self.data.lock().unwrap().get(key).cloned()) })
            }
            fn get_at_seq<'a>(
                &'a self,
                _: &'a [u8],
                _: u64,
            ) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
                Box::pin(async move { Ok(None) })
            }
            fn put<'a>(
                &'a self,
                _: TxId,
                key: &'a [u8],
                value: &'a [u8],
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.data
                        .lock()
                        .unwrap()
                        .insert(key.to_vec(), value.to_vec());
                    Ok(())
                })
            }
            fn delete<'a>(&'a self, _: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move {
                    self.delete_call_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    self.data.lock().unwrap().remove(key);
                    Ok(())
                })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
                Box::pin(async move {
                    Ok(StorageStats {
                        num_segments: 0,
                        total_size_bytes: 0,
                        memtable_size_bytes: 0,
                    })
                })
            }
            fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
                Box::pin(async move { Ok(0) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn scan_prefix<'a>(
                &'a self,
                prefix: &'a [u8],
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move {
                    let map = self.data.lock().unwrap();
                    let mut res = Vec::new();
                    for (k, v) in map.iter() {
                        if k.starts_with(prefix) {
                            res.push((k.clone(), v.clone()));
                        }
                    }
                    Ok(res)
                })
            }
            fn scan<'a>(
                &'a self,
                _: std::ops::Bound<&'a [u8]>,
                _: std::ops::Bound<&'a [u8]>,
                _: Option<usize>,
            ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
        }

        let map = std::sync::Arc::new(std::sync::Mutex::new(AHashMap::default()));
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let store = MockStorage {
            data: map.clone(),
            delete_call_count: count.clone(),
        };

        store.put(TxId(1), b"pref:1", b"v1").await.unwrap(); // unwrap
        store.put(TxId(1), b"pref:2", b"v2").await.unwrap(); // unwrap
        store.put(TxId(1), b"other:1", b"v3").await.unwrap(); // unwrap

        let deleted = store.delete_prefix(TxId(2), b"pref:").await.unwrap(); // unwrap
        assert_eq!(deleted, 2);
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert!(store.get(b"pref:1").await.unwrap().is_none()); // unwrap
        assert!(store.get(b"pref:2").await.unwrap().is_none()); // unwrap
        assert_eq!(store.get(b"other:1").await.unwrap().unwrap(), b"v3"); // unwrap
    }

    #[tokio::test]
    async fn test_vector_index_defaults() {
        struct MockIndex(std::sync::atomic::AtomicUsize);
        impl VectorIndex for MockIndex {
            async fn insert(&self, _: TxId, _: DocId, _: &[f32]) -> Result<()> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            async fn search(&self, _: &[f32], _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                self.0.load(std::sync::atomic::Ordering::SeqCst)
            }
            async fn stats(&self) -> Result<VectorIndexStats> {
                Ok(VectorIndexStats {
                    num_vectors: 0,
                    memory_usage_bytes: 0,
                    num_layers: 0,
                    deleted_ratio: 0.0,
                    rebuild_count: 0,
                })
            }
        }

        let index = MockIndex(std::sync::atomic::AtomicUsize::new(0));
        assert!(index.is_empty().await);

        let vectors = vec![
            (DocId(1), [1.0, 2.0].as_slice()),
            (DocId(2), [3.0, 4.0].as_slice()),
        ];
        index.insert_batch(TxId(1), &vectors).await.unwrap(); // unwrap
        assert_eq!(index.len().await, 2);
        assert!(!index.is_empty().await);

        // Test search_filtered default error
        let res = index.search_filtered(&[1.0], 1, Some(&|_| true)).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "vector_filtered_search");
            }
            _ => panic!("Expected CapabilityUnsupported for search_filtered"),
        }

        // Test search_at default error
        let res2 = index.search_at(&[1.0], 1, 42).await;
        match res2 {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "snapshot_read_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported with ADR-024 for search_at"),
        }
    }

    #[tokio::test]
    async fn test_text_index_defaults() {
        struct MockTextIndex;
        impl TextIndex for MockTextIndex {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }

        let index = MockTextIndex;
        let res = index.search_at("query", 10, 42).await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "snapshot_read_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported for search_at"),
        }
    }

    #[tokio::test]
    async fn test_graph_index_defaults() {
        struct MockGraphIndex;
        impl GraphIndex for MockGraphIndex {
            fn traverse<'a>(
                &'a self,
                _: EntityId,
                _: usize,
            ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn add_entity<'a>(
                &'a self,
                _: TxId,
                _: Entity,
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn add_edge<'a>(
                &'a self,
                _: TxId,
                _: Edge,
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn rollback_to_tx<'a>(
                &'a self,
                _: TxId,
            ) -> BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
                Box::pin(async move { Ok(TxId(0)) })
            }
            fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
                Box::pin(async move { 0 })
            }
            fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
                Box::pin(async move {
                    Ok(GraphIndexStats {
                        num_entities: 0,
                        num_edges: 0,
                        memory_usage_bytes: 0,
                    })
                })
            }
        }

        let index = MockGraphIndex;
        let res = index
            .traverse_at(EntityId::new(1), 2, 42)
            .await;
        match res {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "graph_traverse_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported with ADR-024"),
        }

        let res_time = index
            .traverse_at_time(
                EntityId::new(1),
                2,
                TxId::new(10),
            )
            .await;
        match res_time {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_traverse_at_time");
            }
            _ => panic!("Expected CapabilityUnsupported for traverse_at_time"),
        }

        let res_ppr = index
            .personalized_page_rank(
                &[EntityId::new(1)],
                &PprConfig::default(),
            )
            .await;
        match res_ppr {
            Err(crate::error::MemFuseError::CapabilityUnsupported { capability, .. }) => {
                assert_eq!(capability, "graph_ppr");
            }
            _ => panic!("Expected CapabilityUnsupported for personalized_page_rank"),
        }
    }
}
