use super::super::types::EdgePayload;
use super::CsrGraph;
use crate::consistency_enforcement::EdgeAssertion;
use contextra_types::{ContextraError, DocId, EntityId, Result, TxId};
use std::sync::Arc;

impl CsrGraph {
    fn check_consistency_before_insert(
        &self,
        from: EntityId,
        predicate_hash: Option<[u8; 32]>,
        object_repr: Option<Vec<u8>>,
    ) -> Result<()> {
        if let Some(ref enforcer_lock) = self.consistency_enforcer {
            if let (Some(pred_hash), Some(obj)) = (predicate_hash, object_repr.as_ref()) {
                let assertion = EdgeAssertion {
                    subject: from.inner(),
                    predicate_hash: pred_hash,
                    object_repr: obj.clone(),
                };
                let mut enforcer = enforcer_lock.write();
                if let Some(pattern) = enforcer.check_before_insert(&assertion) {
                    if pattern.suppressed {
                        tracing::warn!(
                            suppression_count = pattern.contradiction_count,
                            "ConsistencyEnforcer: edge insertion suppressed by conflict pattern"
                        );
                        return Err(ContextraError::PolicyViolation(
                            "Contradictory edge suppressed by consistency enforcer".to_string(),
                        ));
                    }
                    tracing::warn!(
                        "ConsistencyEnforcer: contradictory edge detected (not yet suppressed)"
                    );
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_edge(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
        predicate_hash: Option<[u8; 32]>,
        object_repr: Option<Vec<u8>>,
    ) -> Result<()> {
        if !weight.is_finite() || weight < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "Invalid edge weight {weight}: weight must be finite and non-negative"
            )));
        }

        self.check_consistency_before_insert(from, predicate_hash, object_repr)?;

        let needs_compact = {
            let mut inner = self.inner_write();
            let from_idx = inner.get_or_create_index(from);
            let to_idx = inner.get_or_create_index(to);
            if let Some(doc_id) = source_doc_id {
                inner
                    .doc_to_edges
                    .entry(doc_id)
                    .or_default()
                    .insert((from, to));
            }
            inner
                .pending_edges
                .entry(from_idx)
                .or_default()
                .push(EdgePayload {
                    target: to_idx,
                    weight,
                    tx_valid_from,
                    tx_valid_to,
                    business_valid_from,
                    business_valid_to,
                    source_doc_id,
                });
            inner.pending_edge_count += 1;
            inner.is_dirty = true;
            inner.add_to_out_weight_sum(from_idx, weight);
            inner.pending_edge_count >= self.config.rebuild_threshold
        };

        if let Some(doc_id) = source_doc_id {
            self.doc_edge_index.record(doc_id, (from, to));
        }

        if needs_compact {
            self.compact_async().await?;
        }

        Ok(())
    }

    pub async fn insert_edge_direct(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
    ) -> Result<()> {
        self.add_edge(from, to, weight, None, None, None, None, None, None, None)
            .await
    }

    pub async fn insert_edge_direct_with_validity(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
    ) -> Result<()> {
        self.add_edge(
            from,
            to,
            weight,
            tx_valid_from,
            tx_valid_to,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_edge_direct_with_bitemporal_validity(
        self: &Arc<Self>,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
    ) -> Result<()> {
        self.add_edge(
            from,
            to,
            weight,
            tx_valid_from,
            tx_valid_to,
            business_valid_from,
            business_valid_to,
            source_doc_id,
            None,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_edge_direct(
        &self,
        from: EntityId,
        to: EntityId,
        weight: f32,
        tx_valid_from: Option<TxId>,
        tx_valid_to: Option<TxId>,
        business_valid_from: Option<i64>,
        business_valid_to: Option<i64>,
        source_doc_id: Option<DocId>,
    ) -> Result<()> {
        if !weight.is_finite() || weight < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "Invalid edge weight {weight}: weight must be finite and non-negative"
            )));
        }
        let mut inner = self.inner_write();
        let from_idx = inner.get_or_create_index(from);
        let to_idx = inner.get_or_create_index(to);
        if let Some(doc_id) = source_doc_id {
            inner
                .doc_to_edges
                .entry(doc_id)
                .or_default()
                .insert((from, to));
        }
        inner
            .pending_edges
            .entry(from_idx)
            .or_default()
            .push(EdgePayload {
                target: to_idx,
                weight,
                tx_valid_from,
                tx_valid_to,
                business_valid_from,
                business_valid_to,
                source_doc_id,
            });
        inner.pending_edge_count += 1;
        inner.is_dirty = true;
        inner.add_to_out_weight_sum(from_idx, weight);
        Ok(())
    }
}
