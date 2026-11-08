//! Mock-Implementierungen für isolierte Tests innerhalb von `contextra-erasure`.

use crate::backend::{ErasureTarget, StorageBackend};
use crate::error::ErasureError;
use crate::types_local::{LayerId, PurgeCx, PurgeOutcome};
use futures_util::future::BoxFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// In-Memory Mock für ein generisches Storage-Backend.
#[derive(Debug)]
pub struct MockStorageBackend {
    layer: LayerId,
    purge_count: Arc<AtomicUsize>,
}

impl MockStorageBackend {
    /// Erzeugt ein neues Mock-Backend für den gegebenen Layer.
    pub fn new(layer: LayerId) -> Self {
        Self {
            layer,
            purge_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Liefert die Anzahl der aufgerufenen Purge-Operationen.
    pub fn purge_count(&self) -> usize {
        self.purge_count.load(Ordering::SeqCst)
    }
}

impl StorageBackend for MockStorageBackend {
    fn purge<'a>(&'a self, _cx: &'a PurgeCx) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        self.purge_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Ok(PurgeOutcome {
                purged_count: 1,
                bytes_reclaimed: 1024,
            })
        })
    }

    fn layer_id(&self) -> LayerId {
        self.layer
    }
}

/// Mock für ein konkretes ErasureTarget.
#[derive(Debug, Clone)]
pub struct MockErasureTarget {
    /// Eindeutige ID des Targets.
    pub id: String,
    /// Art des Löschziels.
    pub kind: &'static str,
}

impl MockErasureTarget {
    /// Erzeugt ein neues MockErasureTarget.
    pub fn new(id: impl Into<String>, kind: &'static str) -> Self {
        Self {
            id: id.into(),
            kind,
        }
    }
}

impl ErasureTarget for MockErasureTarget {
    fn target_id(&self) -> String {
        self.id.clone()
    }

    fn target_type(&self) -> &'static str {
        self.kind
    }
}
