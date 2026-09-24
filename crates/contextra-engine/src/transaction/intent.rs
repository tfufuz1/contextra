use bytes::Bytes;
use contextra_types::{DocId, TxId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Staged key operation representing (key, optional_value).
pub(super) type StagedKeyOp = (Vec<u8>, Option<Bytes>);

/// Status of a multi-index transaction during the 2-phase commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitIntent {
    /// Transaction is in the "Prepared" state. Stored DocIds assist recovery.
    Pending {
        doc_ids: Arc<Vec<DocId>>,
        #[serde(default)]
        has_text: bool,
        #[serde(default)]
        has_graph: bool,
        #[serde(default)]
        stages_completed: u8,
    },
    /// Transaction is committed across all indices.
    Committed,
    /// Transaction was aborted and compensated.
    Aborted,
    /// Memory consolidation intent (Sleep-Cycle OCC).
    Consolidation {
        source_docs: Vec<(DocId, TxId)>,
        target_id: DocId,
        base_tx: TxId,
    },
    /// Transaction failed during forward recovery or commit.
    Failed { reason: String },
}
