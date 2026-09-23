use crate::collection::{Collection, StoredDocument, StoredDocumentMeta};
use contextra_core::{DocId, Result, StorageEngine, VectorIndex};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Links two memories together with a specific relation (Zettelkasten A-MEM).
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn link_memories(
        &self,
        from: DocId,
        to: DocId,
        relation: contextra_core::types::domain::LinkRelation,
    ) -> Result<()> {
        if from == to {
            return Err(contextra_core::ContextraError::InvalidInput(
                "Cannot link a document to itself".into(),
            ));
        }

        let from_str = from.inner().to_string();
        let to_str = to.inner().to_string();
        let _guards = self
            .lock_keys_sorted([from_str.as_str(), to_str.as_str()])
            .await;

        // Prevent cycles for ALL relation types: if `to` transitively reaches `from`
        // via the same relation, adding `from -> to` creates a cycle.
        // For Supersedes, cycles cause post-RRF displacement of all docs in the cycle (BL-1).
        // For Associates/DerivedFrom/Elaborates, cycles cause unbounded BFS queue growth
        // in traverse_links (P0 audit fix).
        {
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            visited.insert(to);
            queue.push_back(to);

            let mut steps = 0u32;
            const MAX_BFS_STEPS: u32 = 1000;
            while let Some(curr) = queue.pop_front() {
                steps += 1;
                if steps > MAX_BFS_STEPS {
                    break;
                }
                if curr == from {
                    return Err(contextra_core::ContextraError::InvalidInput(format!(
                        "Cyclic {:?} relation detected: document {:?} transitively reaches {:?}",
                        relation, to, from
                    )));
                }
                let links = self.get_links(curr).await?;
                for link in links {
                    // Only follow edges of the same relation type to detect typed cycles
                    if link.relation == relation && visited.insert(link.target) {
                        queue.push_back(link.target);
                    }
                }
            }
        }

        let tx = self.allocate_tx()?;
        let doc_key = self.namespaced_key(&from.inner().to_le_bytes(), 1);

        if let Some(bytes) = self.storage.get_at_seq(&doc_key, u64::MAX).await? {
            let mut doc_id_str = None;
            let mut updated_links = false;

            // Determine which struct it was saved as
            if let Ok(mut meta) = serde_json::from_slice::<StoredDocumentMeta>(&bytes) {
                doc_id_str = Some(meta.id.clone());
                let meta_obj = meta.metadata.get_or_insert_with(|| serde_json::json!({}));
                if let Some(obj) = meta_obj.as_object_mut() {
                    let mut links: Vec<contextra_core::types::domain::MemoryLink> = obj
                        .get("links")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();

                    // Check if link already exists to avoid duplicates
                    if !links
                        .iter()
                        .any(|l| l.target == to && l.relation == relation)
                    {
                        links.push(contextra_core::types::domain::MemoryLink {
                            target: to,
                            relation,
                            created_at_tx: tx,
                        });

                        let links_val = serde_json::to_value(links).map_err(|e| {
                            contextra_core::ContextraError::Serialization(e.to_string())
                        })?;
                        obj.insert("links".to_string(), links_val);
                        obj.insert("updated_at_tx".to_string(), serde_json::json!(tx.inner()));
                        let updated_bytes = serde_json::to_vec(&meta)?;
                        self.storage.put(tx, &doc_key, &updated_bytes).await?;
                        updated_links = true;
                    }
                }
            } else if let Ok(mut full) = serde_json::from_slice::<StoredDocument>(&bytes) {
                doc_id_str = Some(full.id.clone());
                let full_obj = full.metadata.get_or_insert_with(|| serde_json::json!({}));
                if let Some(obj) = full_obj.as_object_mut() {
                    let mut links: Vec<contextra_core::types::domain::MemoryLink> = obj
                        .get("links")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();

                    if !links
                        .iter()
                        .any(|l| l.target == to && l.relation == relation)
                    {
                        links.push(contextra_core::types::domain::MemoryLink {
                            target: to,
                            relation,
                            created_at_tx: tx,
                        });

                        let links_val = serde_json::to_value(links).map_err(|e| {
                            contextra_core::ContextraError::Serialization(e.to_string())
                        })?;
                        obj.insert("links".to_string(), links_val);
                        obj.insert("updated_at_tx".to_string(), serde_json::json!(tx.inner()));
                        let updated_bytes = serde_json::to_vec(&full)?;
                        self.storage.put(tx, &doc_key, &updated_bytes).await?;
                        updated_links = true;
                    }
                }
            }

            // Also update user_key (key_type=0) if links were updated and string id is known
            if updated_links {
                if let Some(ref id_str) = doc_id_str {
                    let user_key = self.namespaced_key(id_str.as_bytes(), 0);
                    if let Some(user_bytes) = self.storage.get_at_seq(&user_key, u64::MAX).await? {
                        if let Ok(mut full_doc) =
                            serde_json::from_slice::<StoredDocument>(&user_bytes)
                        {
                            let doc_obj = full_doc
                                .metadata
                                .get_or_insert_with(|| serde_json::json!({}));
                            if let Some(obj) = doc_obj.as_object_mut() {
                                let mut links: Vec<contextra_core::types::domain::MemoryLink> = obj
                                    .get("links")
                                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                                    .unwrap_or_default();
                                if !links
                                    .iter()
                                    .any(|l| l.target == to && l.relation == relation)
                                {
                                    links.push(contextra_core::types::domain::MemoryLink {
                                        target: to,
                                        relation,
                                        created_at_tx: tx,
                                    });
                                    let links_val = serde_json::to_value(links).map_err(|e| {
                                        contextra_core::ContextraError::Serialization(e.to_string())
                                    })?;
                                    obj.insert("links".to_string(), links_val);
                                    let new_user_bytes = serde_json::to_vec(&full_doc)?;
                                    self.storage.put(tx, &user_key, &new_user_bytes).await?;
                                }
                            }
                        }
                    }
                }

                // INVARIANT INV-GRAPH-PROV-1: Graph tombstone MUST precede LSM commit
                // for Supersedes links. CSR is rebuilt from LSM on startup — tombstone state is recovered
                // transitively.
                if relation == contextra_core::types::domain::LinkRelation::Supersedes {
                    contextra_graph::cascade_invalidate_edges_for_superseded_doc(
                        &self.graph_index,
                        to,
                        tx.inner(),
                    )
                    .await?;
                }

                // Commit transaction to persist the link updates and edge tombstones atomically
                self.storage.commit(tx).await?;
            }
        }

        Ok(())
    }

    /// Retrieves all links for a given document.
    pub async fn get_links(
        &self,
        doc_id: DocId,
    ) -> Result<Vec<contextra_core::types::domain::MemoryLink>> {
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        if let Some(bytes) = self.storage.get_at_seq(&doc_key, u64::MAX).await? {
            if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&bytes) {
                if let Some(obj) = meta.metadata.as_ref().and_then(|m| m.as_object()) {
                    return Ok(obj
                        .get("links")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default());
                }
            } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&bytes) {
                if let Some(obj) = full.metadata.as_ref().and_then(|m| m.as_object()) {
                    return Ok(obj
                        .get("links")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default());
                }
            }
        }
        Ok(Vec::new())
    }
}
