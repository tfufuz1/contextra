//! Composition root wiring entrypoint for `contextra-mvcc`.

use std::sync::Arc;
use std::time::Duration;

use contextra_types::ContextraError;

use crate::seq_log::SequenceLog;
use crate::snapshot::SnapshotRegistry;
use crate::ssi::{SequenceLogSsiValidator, SsiCommitGate};
use crate::tx_buffer::TxBuffer;

/// Dependencies required to construct [`MvccWiring`].
#[derive(Debug, Clone, Default)]
pub struct MvccWiringDeps {
    /// Maximum tracked keys limit for SSI validation.
    pub max_tracked_keys: Option<usize>,
    /// Maximum snapshot pin duration before diagnostic alerts.
    pub max_pin_duration: Option<Duration>,
    /// Optional shared sequence log.
    pub sequence_log: Option<Arc<parking_lot::RwLock<SequenceLog>>>,
    /// Optional shared snapshot registry.
    pub snapshot_registry: Option<Arc<SnapshotRegistry>>,
}

/// Entrypoint wiring structure for `contextra-mvcc` components.
#[derive(Debug)]
pub struct MvccWiring {
    /// Shared sequence log.
    pub sequence_log: Arc<parking_lot::RwLock<SequenceLog>>,
    /// Shared snapshot registry.
    pub snapshot_registry: Arc<SnapshotRegistry>,
    /// SSI commit validator.
    pub ssi_validator: Arc<SequenceLogSsiValidator>,
    /// SSI commit gate.
    pub ssi_commit_gate: SsiCommitGate,
    /// Transaction staging buffer.
    pub tx_buffer: TxBuffer<Vec<u8>>,
}

impl MvccWiring {
    /// Constructs and wires all `contextra-mvcc` instances according to provided dependencies.
    pub fn build(deps: MvccWiringDeps) -> Result<Self, ContextraError> {
        let sequence_log = deps
            .sequence_log
            .unwrap_or_else(|| Arc::new(parking_lot::RwLock::new(SequenceLog::new())));
        let snapshot_registry = deps
            .snapshot_registry
            .unwrap_or_else(|| Arc::new(SnapshotRegistry::new()));

        let mut builder = SequenceLogSsiValidator::builder()
            .with_sequence_log(sequence_log.clone())
            .with_snapshot_registry(snapshot_registry.clone());

        if let Some(limit) = deps.max_tracked_keys {
            builder = builder.with_max_tracked_keys(limit);
        }
        if let Some(duration) = deps.max_pin_duration {
            builder = builder.with_max_pin_duration(duration);
        }

        let validator = Arc::new(builder.build());
        let ssi_commit_gate = SsiCommitGate::new(validator.clone());
        let tx_buffer = TxBuffer::new();

        Ok(Self {
            sequence_log,
            snapshot_registry,
            ssi_validator: validator,
            ssi_commit_gate,
            tx_buffer,
        })
    }
}
