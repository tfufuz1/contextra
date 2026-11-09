//! Static trait conformance and compile-time contract assertions.
//!
//! This module verifies that all port traits define complete, well-formed method
//! signatures that can be called in generic contexts without requiring dynamic dispatch.

#[doc(hidden)]
#[allow(dead_code)]
/// Trait contract verification routines ensuring all trait methods are syntactically
/// and semantically valid in generic contexts.
pub mod check {
    use crate::attention::{AttentionExporter, RequestId};
    use crate::checkpoint::{Checkpoint, CheckpointCoordinator};
    #[allow(deprecated)]
    use crate::embedding::TextGenerator;
    use crate::embedding::{
        ContextSegment, EmbeddingProvider, LlmTextGenerator, LlmTextGeneratorStreaming,
        TextEmbeddingEngine,
    };
    use crate::graph_index::{CommunityResolver, GraphIndex};
    use crate::license::{AuthorizedRing, FeatureRing, LicenseError, LicenseGate};
    use crate::lifecycle::{
        ContextPreparer, GroundingAssessment, GroundingValidator, ResponseGroundingValidator,
    };
    use crate::observability::DriftStatusProvider;
    use crate::reranker::Reranker;
    use crate::scanner::layer::{Purger, ScanTarget};
    use crate::storage::StorageEngine;
    use crate::text_index::{SegmentSynthesizer, TextIndex};
    use crate::vector_index::{HybridSearchProvider, VectorIndex};
    use crate::{
        BoxFuture, BoxStream, ContextChunk, DocId, EntityId, RequestContext, RerankResult, Result,
        TenantId, TokenBudget, TxId, WorkflowState,
    };
    use std::mem::drop;
    use std::ops::Bound;

    /// Verifies that [`Checkpoint`] methods can be invoked in generic contexts.
    pub fn assert_checkpoint_contract<T: Checkpoint>(t: &T, tx: TxId, state: &WorkflowState) {
        let fut: BoxFuture<'_, Result<WorkflowState>> = t.take_snapshot(tx);
        drop(fut);
        let fut: BoxFuture<'_, Result<()>> = t.restore(state);
        drop(fut);
    }

    /// Verifies that [`CheckpointCoordinator`] methods can be invoked in generic contexts.
    pub async fn assert_checkpoint_coordinator_contract<T: CheckpointCoordinator>(
        t: &T,
        tx: TxId,
        metadata: serde_json::Value,
    ) {
        drop(
            t.create_named_checkpoint("name", "coll", 0, tx, metadata)
                .await,
        );
        drop(t.restore_named_checkpoint("name").await);
        drop(t.drop_named_checkpoint("name").await);
        drop(t.list_named_checkpoints().await);
    }

    /// Verifies that [`VectorIndex`] and [`HybridSearchProvider`] methods can be invoked.
    pub async fn assert_vector_index_contract<T: VectorIndex + HybridSearchProvider>(
        t: &T,
        doc: DocId,
    ) {
        drop(t.all_doc_ids());
        drop(t.search_hybrid("query", &[0.0; 4], 10));
        drop(t.search_at(&[0.0; 4], 10, 0).await);
        drop(t.insert(TxId(1), doc, &[0.0; 4]).await);
    }

    /// Verifies that [`AttentionExporter`] methods can be invoked.
    pub fn assert_attention_exporter_contract<T: AttentionExporter>(t: &T, req: RequestId) {
        drop(t.export_attention_weights(req));
    }

    /// Verifies that [`GraphIndex`] and [`CommunityResolver`] methods can be invoked.
    pub fn assert_graph_index_contract<T: GraphIndex + CommunityResolver>(
        t: &T,
        tx: TxId,
        e1: EntityId,
        e2: EntityId,
    ) {
        drop(t.remove_edge(tx, e1, e2));
        drop(t.add_bidirectional(tx, e1, e2, "rel"));
        drop(t.multi_traverse(&[e1, e2], 2));
        drop(t.multi_traverse_at(&[e1, e2], 2, 0));
        drop(t.get_community(e1));
    }

    /// Verifies that [`GroundingValidator`], [`ResponseGroundingValidator`], and [`ContextPreparer`] can be invoked.
    pub fn assert_lifecycle_contract<
        T: GroundingValidator + ResponseGroundingValidator + ContextPreparer,
    >(
        t: &T,
        chunks: &[ContextChunk],
        assessment: &GroundingAssessment,
        budget: &TokenBudget,
    ) {
        drop(t.validate_grounding("resp", chunks));
        drop(t.score_grounding("resp", &["src"]));
        drop(t.prepare_context(chunks.to_vec(), budget, 0.5));
        let _ = assessment.is_grounded;
    }

    /// Verifies that [`EmbeddingProvider`], [`LlmTextGeneratorStreaming`], [`LlmTextGenerator`], and [`TextGenerator`] can be invoked.
    #[allow(deprecated)]
    pub fn assert_embedding_contract<
        T: EmbeddingProvider
            + LlmTextGeneratorStreaming
            + TextEmbeddingEngine
            + LlmTextGenerator
            + TextGenerator,
    >(
        t: &T,
        tenant: TenantId,
        seg: &[ContextSegment],
    ) {
        let _ = t.provider_name();
        let _ = t.embedding_dim();
        let fut: BoxFuture<'_, Result<String>> = t.generate_text("prompt");
        drop(fut);
        let cfg = Default::default();
        let stream: BoxStream<'_, Result<String>> = t.generate_stream("prompt", &cfg);
        drop(stream);
        drop(t.generate_with_context(tenant, seg));
    }

    /// Verifies that [`DriftStatusProvider`] methods can be invoked.
    pub fn assert_observability_contract<T: DriftStatusProvider>(t: &T) {
        let _ = t.overall_drift_status();
    }

    /// Verifies that [`LicenseGate`] methods can be invoked.
    pub fn assert_license_contract<T: LicenseGate>(t: &T, ring: FeatureRing) {
        let res: std::result::Result<AuthorizedRing, LicenseError> = t.authorize(ring);
        drop(res);
    }

    /// Verifies that [`Reranker`] methods can be invoked.
    pub fn assert_reranker_contract<T: Reranker>(t: &T, results: &[RerankResult]) {
        drop(t.rerank("q", &[]));
        t.record_implicit_feedback(results, 1);
        let _ = t.rerank_deadline_ms();
    }

    /// Verifies that [`SegmentSynthesizer`] methods can be invoked.
    pub fn assert_text_index_contract<T: TextIndex + SegmentSynthesizer>(t: &T) {
        drop(t.synthesize_segment(&["doc1", "doc2"]));
        drop(t.search_at("q", 10, 0));
    }

    /// Verifies that [`StorageEngine`] methods can be invoked.
    pub fn assert_storage_engine_contract<T: StorageEngine>(
        t: &T,
        key: &[u8],
        val: &[u8],
        tx: TxId,
    ) {
        drop(t.get_tracked(tx, key));
        drop(t.get_at_seq_tracked(tx, key, 0));
        drop(t.scan_prefix_tracked(tx, key));
        let _ = t.supports_ssi_tracking();
        drop(t.put_if_absent(tx, key, val));
        drop(t.put_batch(tx, &[(key.to_vec(), val.to_vec())]));
        drop(t.delete_prefix(tx, key));
        drop(t.last_seq_no());
        drop(t.pin_checkpoint(0));
        drop(t.unpin_checkpoint(0));
        drop(t.scan_prefix_bounded(key, 10, None));
        drop(t.scan_prefix_at(key, 0));
        drop(t.scan_bounded(Bound::Included(key), Bound::Excluded(key), 10, None));
    }

    /// Verifies that [`Purger`] methods can be invoked.
    pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget) {
        drop(t.purge(ctx, target));
    }

    /// Verifies that [`crate::vfs::VfsFile`] methods can be invoked in generic contexts.
    pub fn assert_vfs_file_contract<F: crate::vfs::VfsFile>(f: &F, buf: &mut [u8]) {
        drop(f.write_at(buf, 0));
        drop(f.set_len(0));
        drop(f.sync_data());
        drop(f.sync_all());
    }

    /// Verifies that [`crate::gate::ReadGate`] methods can be invoked in generic contexts.
    pub fn assert_read_gate_contract<G: crate::gate::ReadGate>(
        g: &G,
        b: &crate::gate::KeyBound,
        r: &crate::gate::KeyRange,
        k: &crate::gate::KeyBuf,
    ) {
        let _ = g.check_bound(b);
        let _ = g.variant_for_range(r);
        let _ = g.gate_epoch();
        let _ = g.covers(k);
        let raw = [0u8; 78];
        let _ = g.covers_raw(&raw);
        let _ = g.gated_seq_raw(&raw);
    }

    /// Verifies that [`crate::keys::RecordSealer`] and [`crate::keys::FileKeyring`] methods can be invoked.
    pub fn assert_keys_contract<S: crate::keys::RecordSealer, K: crate::keys::FileKeyring>(
        s: &S,
        k: &K,
        file_id: &crate::keys::FileId,
        ctx: &crate::keys::SealCtx,
        key: &crate::gate::KeyBuf,
    ) {
        drop(s.seal(ctx, key, b"plain"));
        drop(s.make_slots_durable());
        drop(k.destroy(*file_id));
    }

    /// Verifies that [`crate::ledger::LedgerSink`] methods can be invoked.
    pub fn assert_ledger_contract<L: crate::ledger::LedgerSink>(l: &L) {
        drop(l.read_all());
        drop(l.prune(crate::ledger::Timestamp(0)));
    }

    /// Verifies that [`crate::erasure::ErasureLayer`] methods can be invoked.
    pub fn assert_erasure_layer_contract<E: crate::erasure::ErasureLayer>(
        e: &E,
        target: &crate::erasure::EraseTarget,
        scope: &crate::erasure::LayerScope,
    ) {
        drop(e.resolve(target));
        drop(e.on_gate(scope));
        let _ = e.key_coverage();
    }

    /// Verifies dynamic object-safety and evaluation conformance of [`crate::policy::EgressPolicy`].
    pub fn assert_egress_policy_contract<T: crate::policy::EgressPolicy>(
        t: &T,
        ctx: &RequestContext,
        payload: &str,
    ) {
        drop(t.evaluate(ctx, payload));
    }

    /// Internal visitor conformance verification for KV deserialization.
    pub fn assert_visitor_methods() {
        use serde::de::Visitor;
        struct TestVisitor;
        impl<'de> Visitor<'de> for TestVisitor {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "test")
            }
            fn visit_bytes<E>(self, _v: &[u8]) -> std::result::Result<Self::Value, E> {
                Ok(())
            }
            fn visit_byte_buf<E>(self, _v: Vec<u8>) -> std::result::Result<Self::Value, E> {
                Ok(())
            }
            fn visit_seq<A>(self, _seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(())
            }
        }
        drop(TestVisitor.visit_bytes::<serde::de::value::Error>(&[]));
        drop(TestVisitor.visit_byte_buf::<serde::de::value::Error>(vec![]));
        let _ = "deserialize_bytes";
    }
}

#[cfg(test)]
mod tests {
    use super::check::*;

    #[test]
    fn test_trait_conformance_compiles() {
        assert_visitor_methods();
    }
}
