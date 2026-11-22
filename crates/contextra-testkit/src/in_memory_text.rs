use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum StagedTextOp {
    Insert(String),
    Delete,
}

type TextMap = BTreeMap<DocId, String>;
type StagedTextTxMap = BTreeMap<TxId, BTreeMap<DocId, StagedTextOp>>;

/// An in-memory reference implementation of [`TextIndex`] for testing.
#[derive(Debug, Clone)]
pub struct InMemoryTextIndex {
    data: Arc<RwLock<TextMap>>,
    staged: Arc<RwLock<StagedTextTxMap>>,
    last_tx: Arc<RwLock<TxId>>,
}

impl Default for InMemoryTextIndex {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl InMemoryTextIndex {
    /// Creates a new empty `InMemoryTextIndex`.
    pub fn new() -> Self {
        Self::default()
    }

    fn normalize_term(term: &str) -> String {
        term.to_lowercase()
            .replace('ä', "ae")
            .replace('ö', "oe")
            .replace('ü', "ue")
            .replace('ß', "ss")
    }
}

impl TextIndex for InMemoryTextIndex {
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedTextOp::Insert(text.to_string()));
        Ok(())
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedTextOp::Delete);
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        let ops = {
            let mut staged = self.staged.write();
            staged.remove(&tx)
        };

        if let Some(ops) = ops {
            let mut data = self.data.write();
            for (id, op) in ops {
                match op {
                    StagedTextOp::Insert(t) => {
                        data.insert(id, t);
                    }
                    StagedTextOp::Delete => {
                        data.remove(&id);
                    }
                }
            }
            *self.last_tx.write() = tx;
        }
        Ok(())
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.staged.write().remove(&tx);
        Ok(())
    }

    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        self.staged.write().clear();
        Ok(())
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(*self.last_tx.read())
    }

    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let norm_query = Self::normalize_term(query.trim());
        if norm_query.is_empty() {
            return Ok(Vec::new());
        }

        let query_tokens: Vec<&str> = norm_query.split_whitespace().collect();
        let data = self.data.read();

        let mut matches = Vec::new();
        for (&doc_id, text) in data.iter() {
            let norm_doc = Self::normalize_term(text);
            let mut match_count = 0u32;

            for &q_token in &query_tokens {
                if norm_doc.contains(q_token) {
                    match_count += 1;
                }
            }

            if match_count > 0 {
                let score = match_count as f32 / query_tokens.len().max(1) as f32;
                matches.push(ScoredDocument::new(doc_id, score));
            }
        }

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        matches.truncate(k);
        Ok(matches)
    }

    async fn len(&self) -> usize {
        self.data.read().len()
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        let data = self.data.read();
        let num_documents = data.len();
        let num_tokens: usize = data.values().map(|t| t.split_whitespace().count()).sum();
        let memory_usage_bytes: usize = data.values().map(|t| t.len()).sum();

        Ok(TextIndexStats {
            num_documents,
            num_tokens,
            memory_usage_bytes,
        })
    }
}
