//! # Asynchroner WalObserver Adapter (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Non-blocking, bounded async wrapper fuer WalObserver
//! // INVARIANTEN: Tokio Task Isolation

use super::types::{CommittedBatch, WalObserver, WriteOrigin};
use contextra_ids::TxId;
use std::sync::Arc;

/// Non-blocking, bounded async adapter wrapper for `WalObserver` implementations.
pub struct AsyncObserverAdapter {
    inner: Arc<dyn WalObserver>,
    sender: tokio::sync::mpsc::Sender<(u64, TxId, WriteOrigin)>,
}

impl AsyncObserverAdapter {
    pub fn new(
        inner: Arc<dyn WalObserver>,
        capacity: usize,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(u64, TxId, WriteOrigin)>(capacity);
        let obs_clone = Arc::clone(&inner);

        let handle = tokio::spawn(async move {
            while let Some((seq_no, tx_id, origin)) = rx.recv().await {
                let empty_batch = CommittedBatch {
                    entries: Vec::new(),
                    origin,
                };
                obs_clone.on_commit(&empty_batch, seq_no, tx_id);
            }
        });

        (Self { inner, sender: tx }, handle)
    }
}

impl WalObserver for AsyncObserverAdapter {
    fn requires_durability(&self) -> bool {
        self.inner.requires_durability()
    }

    fn on_commit(&self, _batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        let _ = self
            .sender
            .try_send((seq_no, tx_id, WriteOrigin::UserWrite));
    }
}
