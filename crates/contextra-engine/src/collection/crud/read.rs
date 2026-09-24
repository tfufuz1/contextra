use super::internal::validate_doc_id;
use crate::collection::{Collection, StoredDocument, StoredDocumentMeta};
use contextra_types::{DocId, Result, TxId};
use contextra_ports::{StorageEngine, VectorIndex};

/// Harte Obergrenze für scan()/scan_prefix()-Ergebnisse, falls kein explizites `limit`
/// übergeben wird. Verhindert unbeabsichtigten Vollscan bei generischen Präfixen.
pub const DEFAULT_SCAN_LIMIT: usize = 10_000;

/// Harte Obergrenze für den maximal erlaubten `limit`-Parameter in scan() und scan_prefix().
/// Verhindert unbegrenzte Materialisierung im Speicher selbst wenn explizit ein riesiges Limit angefragt wird.
pub const HARD_SCAN_CEILING: usize = 100_000;

/// Alias for backwards compatibility with earlier MAX_SCAN_RESULTS references.
pub const MAX_SCAN_RESULTS: usize = DEFAULT_SCAN_LIMIT;

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    // AI-TAG[CONVENTION-DRIFT][MAJOR] RESOLVED: AGT-DB-001 — snapshot_seq() now propagates storage errors (TS:2026-08-25T00:00:00Z)
    // instead of silently mapping them to u64::MAX (ID: AGT-DB-001).
    // Consistent with every other error-propagation path in this file.
    pub async fn snapshot_seq(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    /// Retrieves a document by its user-provided string ID.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get(&self, id: &str) -> Result<Option<crate::Document>> {
        self.get_at_snapshot(id, u64::MAX).await
    }

    /// Retrieves a document at a specific snapshot point.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<crate::Document>> {
        validate_doc_id(id)?;
        let key = self.namespaced_key(id.as_bytes(), 0);
        if let Some(data) = self.storage.get_at_seq(&key, seq_no).await? {
            if let Ok(stored) = serde_json::from_slice::<StoredDocument>(&data) {
                return Ok(Some(crate::Document {
                    id: stored.id,
                    metadata: stored.metadata,
                }));
            } else if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&data) {
                return Ok(Some(crate::Document {
                    id: id.to_string(),
                    metadata: Some(val),
                }));
            }
        }
        Ok(None)
    }

    /// Returns the latest transaction ID for a document (via `updated_at_tx` or `created_at_tx`).
    pub async fn get_doc_tx(&self, doc_id: DocId) -> Result<Option<TxId>> {
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        if let Some(bytes) = self.storage.get(&doc_key).await? {
            if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&bytes) {
                if let Some(ref m) = meta.metadata {
                    if let Some(tx_val) = m.get("updated_at_tx") {
                        if let Some(u) = tx_val.as_u64() {
                            return Ok(Some(TxId::new(u)));
                        }
                    }
                    if let Some(imp) = m.get("importance") {
                        if let Some(tx_val) = imp.get("created_at_tx") {
                            if let Ok(tx) = serde_json::from_value::<TxId>(tx_val.clone()) {
                                return Ok(Some(tx));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Scans documents in the collection that match a given key prefix.
    /// Caps maximum results to `MAX_SCAN_RESULTS_DEFAULT` to protect against unbounded memory growth.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn scan_prefix(
        &self,
        prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, serde_json::Value)>> {
        let effective_limit = match limit {
            None => DEFAULT_SCAN_LIMIT,
            Some(n) if n > MAX_SCAN_RESULTS => {
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "requested limit {n} exceeds MAX_SCAN_RESULTS ({MAX_SCAN_RESULTS}); use cursor-based pagination via repeated calls instead"
                )));
            }
            Some(n) => n,
        };

        let real_prefix = if prefix.starts_with("__rel:") {
            self.namespaced_key(
                prefix.strip_prefix("__rel:").unwrap_or(prefix).as_bytes(),
                2,
            )
        } else {
            self.namespaced_key(prefix.as_bytes(), 0)
        };

        let mut results = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;
        const BATCH_SIZE: usize = 1000;

        loop {
            let (batch, next_cursor) = self
                .storage
                .scan_prefix_bounded(&real_prefix, BATCH_SIZE, cursor.as_deref())
                .await?;

            if batch.is_empty() {
                break;
            }

            for (k, v) in batch {
                let key_str = String::from_utf8_lossy(&k).to_string();
                let user_key = if self.name == "default" {
                    key_str
                } else {
                    let prefix_len = self.prefix.len() + 1;
                    if key_str.len() >= prefix_len {
                        key_str[prefix_len..].to_string()
                    } else {
                        key_str
                    }
                };

                if let Ok(val) = serde_json::from_slice(&v) {
                    if results.len() >= effective_limit {
                        return Err(contextra_types::ContextraError::LimitExceeded {
                            limit: effective_limit,
                            context: format!(
                                "scan_prefix(prefix={})",
                                String::from_utf8_lossy(&real_prefix)
                            ),
                        });
                    }
                    results.push((user_key, val));
                }
            }

            if let Some(next) = next_cursor {
                cursor = Some(next);
            } else {
                break;
            }
        }

        if results.len() == DEFAULT_SCAN_LIMIT {
            tracing::warn!(
                collection = %self.name,
                prefix = %prefix,
                limit = DEFAULT_SCAN_LIMIT,
                "scan_prefix returned exactly DEFAULT_SCAN_LIMIT entries; potential truncation — use cursor-based pagination via scan_prefix_bounded instead"
            );
        }

        Ok(results)
    }

    /// Performs a range scan of documents in the collection.
    #[tracing::instrument(level = "trace", skip(self, start, end))]
    pub async fn scan(
        &self,
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
        limit: Option<usize>,
    ) -> Result<Vec<(String, serde_json::Value)>> {
        let effective_limit = match limit {
            None => DEFAULT_SCAN_LIMIT,
            Some(n) if n > MAX_SCAN_RESULTS => {
                return Err(contextra_types::ContextraError::invalid_input(format!(
                    "requested limit {n} exceeds MAX_SCAN_RESULTS ({MAX_SCAN_RESULTS}); use cursor-based pagination via repeated calls instead"
                )));
            }
            Some(n) => n,
        };

        use std::ops::Bound;

        let start_ns = match start {
            Bound::Included(b) => Bound::Included(self.namespaced_key(b, 0)),
            Bound::Excluded(b) => Bound::Excluded(self.namespaced_key(b, 0)),
            Bound::Unbounded => {
                if self.name == "default" {
                    Bound::Unbounded
                } else {
                    let mut b = self.prefix.clone();
                    b.push(0);
                    Bound::Included(b)
                }
            }
        };

        let end_ns = match end {
            Bound::Included(b) => Bound::Included(self.namespaced_key(b, 0)),
            Bound::Excluded(b) => Bound::Excluded(self.namespaced_key(b, 0)),
            Bound::Unbounded => {
                if self.name == "default" {
                    Bound::Unbounded
                } else {
                    let mut b = self.prefix.clone();
                    b.push(1);
                    Bound::Excluded(b)
                }
            }
        };

        let start_bytes = match &start_ns {
            Bound::Included(v) => Bound::Included(v.as_slice()),
            Bound::Excluded(v) => Bound::Excluded(v.as_slice()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end_bytes = match &end_ns {
            Bound::Included(v) => Bound::Included(v.as_slice()),
            Bound::Excluded(v) => Bound::Excluded(v.as_slice()),
            Bound::Unbounded => Bound::Unbounded,
        };

        let mut results = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;
        const BATCH_SIZE: usize = 1000;

        loop {
            let (batch, next_cursor) = self
                .storage
                .scan_bounded(start_bytes, end_bytes, BATCH_SIZE, cursor.as_deref())
                .await?;

            if batch.is_empty() {
                break;
            }

            for (k, v) in batch {
                let key_str = String::from_utf8_lossy(&k).to_string();
                let user_key = if self.name == "default" {
                    key_str
                } else {
                    let prefix_len = self.prefix.len() + 1;
                    if key_str.len() >= prefix_len {
                        key_str[prefix_len..].to_string()
                    } else {
                        key_str
                    }
                };

                if let Ok(val) = serde_json::from_slice(&v) {
                    if results.len() >= effective_limit {
                        return Err(contextra_types::ContextraError::LimitExceeded {
                            limit: effective_limit,
                            context: "scan()".to_string(),
                        });
                    }
                    results.push((user_key, val));
                }
            }

            if let Some(next) = next_cursor {
                cursor = Some(next);
            } else {
                break;
            }
        }

        Ok(results)
    }
}
