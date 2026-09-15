//! Abstract contracts for snapshot and checkpoint management.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: Checkpoint, CheckpointCoordinator & Snapshot Trait-Definitionen für Layer 0.
// INVARIANTEN: Downward-only Trait interfaces; dyn-safety constraints in CheckpointCoordinator.

use super::BoxFuture;
use crate::types::{TxId, WorkflowState};
use crate::Result;
use std::future::Future;

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

/// Represents a point-in-time view of the database.
pub trait Snapshot: Send + Sync {
    /// Returns the sequence number for this snapshot.
    fn seq_no(&self) -> u64;
}
