// FILE-CONTEXT
// ZWECK: Orchestrierung atomarer 4-Index 2-Phase-Commits und kompensierender Transaktionen.
// INVARIANTEN: [INV-DB-3] Keine verschluckten Fehler bei Rollbacks; Kompensierende Transaktionen bei HNSW/BM25/Graph Ausfällen.
// NICHT-OFFENSICHTLICH: Multi-Attempt LSM-Kompensation mit Split-Brain Tracing-Warnungen bei anhaltenden Fehlern.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

//! # Database Transactions
//!
//! This module provides `DbTransaction`, an orchestrator for atomic multi-index commits
//! between LSM-Tree storage engine (`contextra-store`), HNSW vector index (`contextra-index`),
//! BM25 inverted text index (`contextra-text`), and CSR graph index (`contextra-graph`).
//! It implements a 4-index 2-phase commit protocol and provides compensating transactions for rollbacks.
//!
//! # 2-Phase Commit (2PC) State Machine & Intent Log Format
//!
//! Multi-index transactions guarantee atomic consistency across LSM storage, HNSW vector index,
//! BM25 text index, and CSR graph index via intent logging (`CommitIntent`) and compensating actions.
//!
//! ## State Transitions
//! ```text
//!              +-------------------+
//!              |       Init        |
//!              +---------+---------+
//!                        |
//!                        v
//!            +-----------------------+
//!            |  Pending (stage 0)    |  <-- Write intent marker to storage WAL
//!            +-----------+-----------+
//!                        |
//!       LSM Fail         | LSM Success
//!          +-------------+-------------+
//!          |                           |
//!          v                           v
//!  +---------------+       +-----------------------+
//!  |   Aborted     |       |   Pending (stage 1)   |  <-- LSM payload durable
//!  +---------------+       +-----------+-----------+
//!          ^                           |
//!          |      Index Fail           | Index Success (HNSW + Text + Graph)
//!          +---------------------------+
//!                                      |
//!                                      v
//!                          +-----------------------+
//!                          |       Committed       |
//!                          +-----------------------+
//! ```
//!
//! ## Intent Log Format (`CommitIntent`)
//! All transaction intent markers are JSON-serialized and stored under key namespace `__tx_intent:<tx_id>`
//! (or `__col:<name>:\x03<tx_id>` for non-default collections):
//! - `Pending { doc_ids, has_text, has_graph, stages_completed }`:
//!   - `stages_completed = 0`: Intent logged, staging commenced.
//!   - `stages_completed = 1`: LSM storage committed durably.
//! - `Committed`: All 4 sub-systems committed successfully.
//! - `Aborted`: Transaction aborted and compensated.
//! - `Consolidation`: Memory consolidation intent (Sleep-Cycle OCC).
//! - `Failed { reason }`: Forward or backward recovery encountered terminal error.
//!
//! ## Idempotency & Crash Recovery Invariants
//! - **[INV-DB-3] Strict Error Visibility in Rollbacks**: Compensating actions during
//!   rollback must never silently drop errors. Any failure logs to `tracing::error!` highlighting
//!   potential Split-Brain.
//! - **Idempotency**: All compensating actions (`CompensateLsmAction`, `CompensateHnswAction`,
//!   `CompensateTextAction`, `CompensateGraphAction`) and recovery runs (`recover_pending_intents`)
//!   are fully idempotent; multiple executions yield identical final storage and index state.
//!
//! # Lock Hierarchy & Poison Recovery
//! `DbTransaction` uses fine-grained `std::sync::Mutex` instances for staging index changes.
//! Lock ordering between staged locks is not strict because staged fields are modified sequentially per operation.
//! All Mutex lock acquisitions explicitly handle `PoisonError` via `match` with `p.into_inner()`
//! to ensure fail-safe operation without panics.

mod cleanup;
mod compensating_actions;
mod db_transaction;
mod db_transaction_commit;
mod db_transaction_rollback;
mod intent;

#[cfg(test)]
mod tests;

pub use cleanup::{cleanup_orphaned_consolidation_intents, recover_pending_intents};
pub use compensating_actions::{
    CommitLedger, CompensateHnswAction, CompensateLsmAction, CompensateTextAction,
    CompensatingAction, RollbackStagedAction,
};
pub use db_transaction::DbTransaction;
pub use intent::CommitIntent;
