//! Checkpoint and Snapshot subsystem trait definitions.

use super::BoxFuture;
use crate::types::*;
use crate::Result;
use std::future::Future;

/// Represents an immutable read snapshot bound to a specific sequence number / version.
///
/// Guarantees that all search signals (vector, text, graph) reading under the same
/// `ReadSnapshot` observe the exact same sequence state (`max_seq`) (REG-10, §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadSnapshot {
    max_seq: u64,
}

impl ReadSnapshot {
    /// Creates a new `ReadSnapshot` bound to the given maximum sequence number.
    pub fn new(max_seq: u64) -> Self {
        Self { max_seq }
    }

    /// Returns the maximum sequence number for this snapshot.
    pub fn max_seq(&self) -> u64 {
        self.max_seq
    }

    /// Returns the sequence number for this snapshot (alias for `max_seq`).
    pub fn seq(&self) -> u64 {
        self.max_seq
    }
}

impl From<u64> for ReadSnapshot {
    fn from(seq: u64) -> Self {
        Self::new(seq)
    }
}

/// Abstract contract for generating consistent checkpoints.
pub trait Checkpoint: Send + Sync + 'static {
    /// Takes a deterministic snapshot of the current state.
    fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>>;

    /// Rolls the state back to the specified checkpoint.
    fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>>;
}

/// Unified Checkpoint Coordinator Trait combining named, TxId+seq_no-scoped, persistent checkpoints.
///
/// # Dyn-Safety Note
/// This trait is intentionally NOT dyn-compatible without specifying the associated type `Meta`
/// (e.g. `Arc<dyn CheckpointCoordinator<Meta = ...>>`) or using a concrete type, due to the
/// associated type `type Meta: Send + Sync`.
///
/// # DECISION-REF
/// ADR-011 — Consolidated Checkpoint Subsystem Architecture (resolving AGT-STORE-002).
pub trait CheckpointCoordinator: Send + Sync + 'static {
    /// Type representing checkpoint metadata.
    type Meta: Send + Sync;

    /// Creates and persists a new named checkpoint.
    fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Restores database state to a named checkpoint.
    fn restore_named_checkpoint(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Deletes a checkpoint by name.
    fn drop_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<()>> + Send;

    /// Lists all active checkpoints.
    fn list_named_checkpoints(&self) -> impl Future<Output = Result<Vec<Self::Meta>>> + Send;
}
