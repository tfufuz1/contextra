// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Ableitung des ErasureFootprint für ein Dokument und Fail-Closed Invarianten.

use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, EntityId, Result};

use crate::collection::Collection;
use crate::deletion::footprint::{
    check_nonempty_or_observed_empty, ErasureFootprint, FprRamFootprint, KeyScope, StorageScope,
};

fn minimize_prefixes(mut prefixes: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    prefixes.retain(|p| !p.is_empty());
    prefixes.sort_by_key(|p| p.len());
    let mut min_prefixes: Vec<Vec<u8>> = Vec::new();
    for p in prefixes {
        if !min_prefixes.iter().any(|existing| p.starts_with(existing)) {
            min_prefixes.push(p);
        }
    }
    min_prefixes
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Leitet den vollständigen [`ErasureFootprint`] für ein Dokument aus dem aktuellen Speicherzustand ab.
    ///
    /// # Fail-Closed Invarianten
    /// - Leitet explizite Schlüssel (`user_key`, `doc_key`) und Präfixe (`__docid:`, `__rel:` in beiden Richtungen) ab.
    /// - Liest Terme aus dem gespeicherten Text bzw. Text-Index für den RAM-Footprint ab.
    /// - Gibt `observed_empty = true` zurück, wenn kein Eintrag für das Dokument existiert.
    /// - Führt `check_nonempty_or_observed_empty` durch.
    pub async fn erasure_footprint(&self, doc_id: DocId) -> Result<ErasureFootprint> {
        let tenant_id = self.tenant_id();
        let storage_scope = StorageScope::Tenant(tenant_id);

        let id_str = doc_id.inner().to_string();

        let tx = self.allocate_tx()?;
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);

        let mut actual_user_id = id_str.clone();
        if let Some(doc_val) = self
            .storage
            .get_at_seq_tracked(tx, &doc_key, u64::MAX)
            .await?
        {
            if let Ok(meta) =
                serde_json::from_slice::<crate::collection::StoredDocumentMeta>(&doc_val)
            {
                actual_user_id = meta.id;
            } else if let Ok(full) =
                serde_json::from_slice::<crate::collection::StoredDocument>(&doc_val)
            {
                actual_user_id = full.id;
            }
        }

        let user_key = self.namespaced_key(actual_user_id.as_bytes(), 0);
        let doc_map_key = [b"__docid:", actual_user_id.as_bytes()].concat();

        let rel_outgoing_prefix = self.namespaced_key(format!("{}:", actual_user_id).as_bytes(), 2);
        let rel_legacy_prefix = [b"__rel:", actual_user_id.as_bytes()].concat();

        let mut explicit_keys = vec![user_key.clone(), doc_key.clone(), doc_map_key];
        let raw_prefixes = vec![rel_outgoing_prefix, rel_legacy_prefix];
        let prefixes = minimize_prefixes(raw_prefixes);

        // 2. Derive relation keys (both directions) and graph entities
        let mut ram = FprRamFootprint::default();
        ram.doc_ids.push(doc_id);

        let entity_id = EntityId::from_key(&actual_user_id)?;
        ram.graph_entities.push(entity_id.to_string());

        let mut scanned_count = 0usize;

        if let Some(user_val) = self
            .storage
            .get_at_seq_tracked(tx, &user_key, u64::MAX)
            .await?
        {
            scanned_count += 1;
            if let Ok(stored) =
                serde_json::from_slice::<crate::collection::StoredDocument>(&user_val)
            {
                if let Some(text) = crate::collection::extract_text(&stored.metadata) {
                    for term in text.split_whitespace() {
                        let term_bytes = term.to_lowercase().into_bytes();
                        if !ram.term_bytes.contains(&term_bytes) {
                            ram.term_bytes.push(term_bytes);
                        }
                    }
                }
            }
        }

        if self
            .storage
            .get_at_seq_tracked(tx, &doc_key, u64::MAX)
            .await?
            .is_some()
        {
            scanned_count += 1;
        }

        // Scan relational keys matching document
        let rel_prefix = self.namespaced_key(b"", 2);
        let rel_entries = self.storage.scan_prefix(&rel_prefix).await?;
        for (rel_k, _) in rel_entries {
            if rel_k
                .windows(actual_user_id.len())
                .any(|w| w == actual_user_id.as_bytes())
            {
                scanned_count += 1;
                explicit_keys.push(rel_k.clone());
                ram.relation_keys.push(rel_k);
            }
        }

        let observed_empty = scanned_count == 0;

        // Fail-closed validation check: scanned entries > 0 but explicit_keys is empty is impossible
        check_nonempty_or_observed_empty(scanned_count, explicit_keys.len())?;

        let key_scope = KeyScope {
            explicit: explicit_keys,
            prefixes,
        };

        let ram_hash = ram.compute_hash();

        let mut footprint = ErasureFootprint {
            storage: storage_scope,
            keys: key_scope,
            hash: [0u8; 32],
            ram,
            ram_hash,
            observed_empty,
        };
        footprint.hash = footprint.compute_hash();

        Ok(footprint)
    }
}
