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
//! # Safety & Reliability Invariants
//! - **[INV-DB-3] Strict Error Visibility in Rollbacks**: Compensating transactions during
//!   rollback must never silently drop errors. Discovered during Forensic Audit (HARD-004),
//!   any rollback failure must log explicitly to `tracing::error!` mapping out a potential Split-Brain.
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

pub use cleanup::cleanup_orphaned_consolidation_intents;
pub use compensating_actions::{
    CommitLedger, CompensateHnswAction, CompensateLsmAction, CompensateTextAction,
    CompensatingAction, RollbackStagedAction,
};
pub use db_transaction::DbTransaction;
pub use intent::CommitIntent;
