use super::{crud::validate_doc_id, Collection};
use contextra_types::{DocId, Result};
use contextra_ports::{StorageEngine, VectorIndex};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    // AI-TAG[CONCURRENCY][CRITICAL] RESOLVED: AGT-DB-005 — relate() rollback race behoben, siehe ADR-023 (TS:2026-08-28T00:00:00Z)
    /// Creates a directional relationship between two documents in the collection.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()> {
        validate_doc_id(from)?;
        validate_doc_id(to)?;
        let _guards = self.lock_keys_sorted([from, to]).await;
        let db_tx = self.begin_transaction()?;

        let from_id = contextra_types::EntityId::from_key(from)?;
        let to_id = contextra_types::EntityId::from_key(to)?;

        let key_str = format!("{}:{}:{}", from, label, to);
        let key = self.namespaced_key(key_str.as_bytes(), 2);
        let old_val = self.storage.get_at_seq(&key, u64::MAX).await?;

        let val = serde_json::json!({
            "from": from,
            "to": to,
            "label": label,
        });
        let bytes = serde_json::to_vec(&val)?;

        if let Err(e) = self.storage.put(db_tx.tx_id, &key, &bytes).await {
            let tx_id = db_tx.tx_id;
            if let Err(rollback_err) = db_tx.rollback().await {
                tracing::warn!(
                    tx_id = %tx_id,
                    error = %rollback_err,
                    "Konnte Transaktion nach storage.put Fehler in relate() nicht zurückrollen"
                );
            }
            return Err(e);
        }

        let dummy_doc_id = DocId::from_key(from)?;
        db_tx.record_keys_with_old_values(key.clone(), old_val, vec![], None, dummy_doc_id);

        let from_entity = contextra_types::Entity::new(from_id, from, "Node");
        let to_entity = contextra_types::Entity::new(to_id, to, "Node");
        db_tx.stage_graph_entity(from_entity);
        db_tx.stage_graph_entity(to_entity);

        let edge = contextra_types::Edge::new(from_id, to_id, label);
        db_tx.stage_graph_edge(edge);

        match db_tx.commit().await {
            Ok(_) => {
                self.check_and_trigger_community_detection(1);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Creates a bidirectional relationship atomically.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()> {
        self.relate(from, to, label).await?;
        self.relate(to, from, label).await?;
        Ok(())
    }

    /// Creates an n-ary hyperedge relationship connecting two or more participants.
    ///
    /// # Deadlock Prevention & Canonical Lock Ordering
    /// `participants` contain document IDs (`&str`). To prevent lock-ordering deadlocks when concurrent
    /// calls operate on overlapping participant sets in different orderings (e.g. `{A, B, C}` vs `{C, B, A}`),
    /// the resolved `EntityId`s for all participant document IDs are canonically sorted in ascending order
    /// by their `u64` inner value **before** acquiring key locks via `lock_keys_sorted`.
    #[tracing::instrument(level = "trace", skip(self, participants))]
    pub async fn relate_n_ary(
        &self,
        predicate: &str,
        participants: &[(&str, &str)],
        source_doc_id: Option<&str>,
    ) -> Result<contextra_graph::hyperedge::HyperEdgeId> {
        if participants.len() < 2 {
            return Err(contextra_types::ContextraError::InvalidInput(format!(
                "relate_n_ary requires at least 2 participants, found {}",
                participants.len()
            )));
        }

        for &(doc_id_str, _) in participants {
            validate_doc_id(doc_id_str)?;
        }
        if let Some(src_id) = source_doc_id {
            validate_doc_id(src_id)?;
        }

        // 1. Resolve and validate all participant DocIds, EntityIds, and RoleIds
        let mut resolved_participants = Vec::with_capacity(participants.len());
        for &(doc_id_str, role_str) in participants {
            let entity_id = contextra_types::EntityId::from_key(doc_id_str)?;
            let role_doc_id = contextra_types::DocId::from_key(role_str)?;
            let role_id = contextra_graph::hyperedge::RoleId::new(role_doc_id.inner() as u32);
            resolved_participants.push((doc_id_str, role_id, entity_id));
        }

        // 2. Canonical sorting of participant EntityIds before lock acquisition
        let mut lock_keys: Vec<&str> = participants
            .iter()
            .map(|&(doc_id_str, _)| doc_id_str)
            .collect();
        lock_keys.sort_unstable();
        lock_keys.dedup();

        let _guards = self.lock_keys_sorted(lock_keys).await;

        // 3. Begin transaction and prepare HyperEdge
        let db_tx = self.begin_transaction()?;
        let hyperedge_id = contextra_graph::hyperedge::HyperEdgeId::new(db_tx.tx_id.inner());

        let role_bindings: Vec<contextra_graph::hyperedge::RoleBinding> = resolved_participants
            .iter()
            .map(|&(_, role_id, entity_id)| {
                contextra_graph::hyperedge::RoleBinding::new(role_id, entity_id)
            })
            .collect();

        let parsed_src_doc_id = match source_doc_id {
            Some(s) => Some(DocId::from_key(s)?),
            None => None,
        };

        let hyperedge = contextra_graph::hyperedge::HyperEdge::new(
            hyperedge_id,
            contextra_graph::csr::EdgeType::Default,
            role_bindings,
            1.0,
        )
        .with_tx_validity(Some(db_tx.tx_id), None)
        .with_source_doc_id(parsed_src_doc_id);

        hyperedge.validate()?;

        // 4. Persist HyperEdge in LSM storage under __graph:hyperedge:{id}
        let hyperedge_key = format!(
            "{}{}",
            contextra_graph::hyperedge::HYPEREDGE_PREFIX,
            hyperedge_id.inner()
        );
        let key = self.namespaced_key(hyperedge_key.as_bytes(), 2);
        let bytes = hyperedge.serialize()?;

        if let Err(e) = self.storage.put(db_tx.tx_id, &key, &bytes).await {
            let tx_id = db_tx.tx_id;
            if let Err(rollback_err) = db_tx.rollback().await {
                tracing::warn!(
                    tx_id = %tx_id,
                    error = %rollback_err,
                    "Konnte Transaktion nach storage.put Fehler in relate_n_ary() nicht zurückrollen"
                );
            }
            return Err(e);
        }

        // 5. Stage entities and hyperedge in DbTransaction
        for &(doc_id_str, _, entity_id) in &resolved_participants {
            let entity = contextra_types::Entity::new(entity_id, doc_id_str, "Node");
            db_tx.stage_graph_entity(entity);
        }
        db_tx.stage_hyperedge(hyperedge);

        match db_tx.commit().await {
            Ok(_) => {
                self.check_and_trigger_community_detection(1);
                Ok(hyperedge_id)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    async fn test_db(dim: usize) -> (crate::Contextra, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = crate::ContextraConfig {
            dimension: dim,
            max_elements: 10_000,
            distance_metric: contextra_types::DistanceMetric::Cosine,
            ..Default::default()
        };
        let db = crate::Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open db");
        (db, tmp)
    }

    #[tokio::test]
    async fn test_relate_n_ary_basic() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("default").await.expect("collection");

        col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert 1");
        col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert 2");
        col.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
            .await
            .expect("insert 3");

        let participants = vec![
            ("doc-1", "subject"),
            ("doc-2", "object"),
            ("doc-3", "context"),
        ];

        let hyperedge_id = col
            .relate_n_ary("transaction", &participants, Some("doc-1"))
            .await
            .expect("relate_n_ary");

        // Verify graph_index contains the inserted hyperedge
        let retrieved = col.graph_index().get_hyperedge(hyperedge_id);
        assert!(
            retrieved.is_some(),
            "HyperEdge must be retrievable from graph_index"
        );
        let hyperedge = retrieved.expect("retrieved");
        assert_eq!(hyperedge.id, hyperedge_id);
        assert_eq!(hyperedge.participants.len(), 3);

        // Verify hyperedge lookup by participant entity
        let e1 = contextra_types::EntityId::from_key("doc-1").expect("entity_id");
        let hes_e1 = col.graph_index().hyperedges_for_entity(e1);
        assert!(hes_e1.contains(&hyperedge_id));
    }

    #[tokio::test]
    async fn test_relate_n_ary_insufficient_participants() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("default").await.expect("collection");

        col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert 1");

        let single_participant = vec![("doc-1", "subject")];
        let res = col.relate_n_ary("solo", &single_participant, None).await;

        assert!(res.is_err(), "relate_n_ary with < 2 participants must fail");
        let err = res.unwrap_err();
        assert!(
            matches!(err, contextra_types::ContextraError::InvalidInput(ref msg) if msg.contains("at least 2 participants"))
        );
    }
}
